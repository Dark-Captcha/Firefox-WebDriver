# Firefox WebDriver Protocol Specification

> **Version:** 0.1.2 | **Status:** Active | **Updated:** 2025-12-26

Bidirectional browser automation protocol for Firefox. Extension-based, event-driven, undetectable.

---

## Table of Contents

| Section                          | Topic           | Description                        |
| -------------------------------- | --------------- | ---------------------------------- |
| [1](#1-introduction)             | Introduction    | Purpose, architecture, terminology |
| [2](#2-protocol)                 | Protocol        | Message format, correlation        |
| [3](#3-transport)                | Transport       | WebSocket lifecycle, timeouts      |
| [4](#4-modules)                  | Modules         | Commands and events by domain      |
| [5](#5-events)                   | Events          | Event model, subscriptions         |
| [6](#6-errors)                   | Errors          | Error codes, handling              |
| [7](#7-implementation)           | Implementation  | Rust and Extension structure       |
| [A](#appendix-a-quick-reference) | Quick Reference | Command and event tables           |

---

## 1. Introduction

### 1.1. Purpose

This specification defines a bidirectional protocol for Firefox browser automation:

| Goal          | Approach                                               |
| ------------- | ------------------------------------------------------ |
| Undetectable  | No `navigator.webdriver`, no globals, extension-based  |
| Event-driven  | MutationObserver, no polling                           |
| CSP bypass    | `browser.scripting.executeScript` with `world: "MAIN"` |
| Bidirectional | Commands down, events up via WebSocket                 |

### 1.2. Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      LOCAL END (Rust)                           │
├─────────────────────────────────────────────────────────────────┤
│  Driver                                                         │
│    └── ConnectionPool (single WebSocket server, shared)         │
│    └── Window (owns Firefox process + profile)                  │
│          └── Tab (frame context)                                │
│                └── Element (UUID reference)                     │
├─────────────────────────────────────────────────────────────────┤
│  Transport                                                      │
│    └── ConnectionPool (localhost:PORT, multiplexed)             │
│    └── Connection per session (keyed by SessionId)              │
│    └── Request/Response correlation (by UUID)                   │
│    └── Event handler callbacks                                  │
└───────────────────────────────┬─────────────────────────────────┘
                                │ WebSocket (all windows share port)
┌───────────────────────────────▼─────────────────────────────────┐
│                     REMOTE END (Extension)                      │
├─────────────────────────────────────────────────────────────────┤
│  Background Script                                              │
│    └── Session (WebSocket client)                               │
│    └── Registry (command dispatch)                              │
│    └── Modules (browsingContext, element, script, etc.)         │
├─────────────────────────────────────────────────────────────────┤
│  Content Script (per frame)                                     │
│    └── Element Store: Map<UUID, Element>                        │
│    └── MutationObserver (DOM events)                            │
│    └── Bridge (WEBDRIVER_INIT forwarding)                       │
└─────────────────────────────────────────────────────────────────┘
```

### 1.3. Terminology

| Term              | Definition                                                  |
| ----------------- | ----------------------------------------------------------- |
| Local End         | Rust driver that sends commands, receives events            |
| Remote End        | Firefox extension that executes commands, emits events      |
| Command           | Request from local → remote, expects response               |
| Event             | Notification from remote → local                            |
| EventReply        | Response to event requiring decision (network interception) |
| Module            | Collection of related commands (e.g., `browsingContext`)    |
| Element Reference | UUID mapping to DOM element in content script Map           |

### 1.4. ID System

| ID               | Type         | Source       | Purpose                         |
| ---------------- | ------------ | ------------ | ------------------------------- |
| `SessionId`      | `NonZeroU32` | Rust counter | Window identification           |
| `TabId`          | `NonZeroU32` | Firefox      | Tab identification              |
| `FrameId`        | `u64`        | Firefox      | Frame identification (0 = main) |
| `RequestId`      | UUID v4      | Rust         | Request/response correlation    |
| `ElementId`      | UUID v4      | Extension    | DOM element reference           |
| `ScriptId`       | UUID v4      | Extension    | Preload script reference        |
| `SubscriptionId` | UUID v4      | Extension    | Element observation             |
| `InterceptId`    | UUID v4      | Extension    | Network interception            |

### 1.5. Design Principles

#### 1.5.1. Reference-Based Element Storage

```
Content Script:
  elementStore: Map<UUID, Element>

  FIND { selector } → uuid
    element = document.querySelector(selector)
    elementStore.set(uuid, element)
    return uuid

  ACTION { uuid, method, args }
    element = elementStore.get(uuid)
    element[method](...args)  // Dynamic property access (CSP-safe)
```

**Why:** `element[methodName]()` is bracket notation, not `eval()`. CSP only blocks `eval()`.

#### 1.5.2. Event-Driven Architecture

| Traditional (Polling)     | This Protocol (Events)                     |
| ------------------------- | ------------------------------------------ |
| Check element every 100ms | `element.added` fires when element appears |
| Poll for navigation       | `browsingContext.load` fires on load       |
| Poll network idle         | Network events fire on activity            |

#### 1.5.3. Process Isolation

Each `Window` owns:

- One Firefox process
- Reference to shared ConnectionPool (single WebSocket port)
- One profile directory
- Independent state (keyed by SessionId)

---

## 2. Protocol

### 2.1. Message Types

| Type       | Direction      | Purpose              |
| ---------- | -------------- | -------------------- |
| Command    | Local → Remote | Request operation    |
| Response   | Remote → Local | Command result       |
| Event      | Remote → Local | Browser notification |
| EventReply | Local → Remote | Event decision       |

### 2.2. Command Format

```json
{
  "id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "method": "browsingContext.navigate",
  "tabId": 1,
  "frameId": 0,
  "params": {
    "url": "https://example.com"
  }
}
```

| Field     | Type    | Required | Description                |
| --------- | ------- | -------- | -------------------------- |
| `id`      | UUID v4 | Yes      | Correlation ID             |
| `method`  | string  | Yes      | `module.methodName` format |
| `tabId`   | number  | Yes      | Target tab                 |
| `frameId` | number  | Yes      | Target frame (0 = main)    |
| `params`  | object  | No       | Command parameters         |

### 2.3. Response Format

**Success:**

```json
{
  "id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "type": "success",
  "result": { "title": "Example" }
}
```

**Error:**

```json
{
  "id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "type": "error",
  "error": "no such element",
  "message": "Element not found: #missing"
}
```

### 2.4. Event Format

```json
{
  "id": "event-uuid",
  "type": "event",
  "method": "element.added",
  "params": {
    "selector": "#login",
    "elementId": "elem-uuid",
    "subscriptionId": "sub-uuid",
    "tabId": 1,
    "frameId": 0
  }
}
```

### 2.5. EventReply Format

```json
{
  "id": "event-uuid",
  "replyTo": "network.beforeRequestSent",
  "result": { "action": "block" }
}
```

### 2.6. READY Handshake

Extension sends on connect (nil UUID):

```json
{
  "id": "00000000-0000-0000-0000-000000000000",
  "type": "success",
  "result": { "sessionId": 1, "tabId": 1 }
}
```

---

## 3. Transport

### 3.1. Connection Model

```
Driver (Rust)                    Extension (Background)
     │                                    │
     │  1. Driver::build() binds          │
     │     ConnectionPool to localhost:0  │
     │                                    │
     │  2. spawn_window() launches        │
     │     Firefox with data URI          │
     │                                    │
     │  3. Data URI posts WEBDRIVER_INIT  │
     │  4. Content script forwards        │
     │                                    │
     │◄─────── WebSocket Connect ─────────│
     │         (to shared port)           │
     │                                    │
     │◄─────── READY (SessionId) ─────────│
     │         Pool routes by SessionId   │
     │                                    │
     │◄────── Commands/Events ───────────►│
     │         Routed by SessionId        │
```

All Firefox windows connect to the same WebSocket port. Messages are
routed by `SessionId` which is included in the READY handshake.

### 3.2. Data URI Initialization

```html
data:text/html,
<script>
  window.postMessage(
    {
      type: "WEBDRIVER_INIT",
      wsUrl: "ws://127.0.0.1:PORT",
      sessionId: 1,
    },
    "*"
  );
</script>
```

Content script validates localhost URL, forwards to background.

### 3.3. Timeouts

| Timeout     | Duration | Purpose                        |
| ----------- | -------- | ------------------------------ |
| Connection  | 30s      | Wait for extension connect     |
| Command     | 30s      | Wait for response              |
| Event Reply | 30s      | Wait for interception decision |
| Shutdown    | 5s       | Graceful close                 |

### 3.4. Error Recovery

| Error               | Cause                    | Recovery                     |
| ------------------- | ------------------------ | ---------------------------- |
| `ConnectionTimeout` | Extension didn't connect | Return error                 |
| `RequestTimeout`    | No response in 30s       | Remove pending, return error |
| `ConnectionClosed`  | WebSocket closed         | Fail all pending requests    |

---

## 4. Modules

### 4.1. session Module

| Command             | Description                  |
| ------------------- | ---------------------------- |
| `session.status`    | Get connection status        |
| `session.stealLogs` | Get and clear extension logs |

### 4.2. browsingContext Module

| Command                                | Description           |
| -------------------------------------- | --------------------- |
| `browsingContext.navigate`             | Navigate to URL       |
| `browsingContext.reload`               | Reload page           |
| `browsingContext.goBack`               | Navigate back         |
| `browsingContext.goForward`            | Navigate forward      |
| `browsingContext.getTitle`             | Get page title        |
| `browsingContext.getUrl`               | Get current URL       |
| `browsingContext.newTab`               | Create new tab        |
| `browsingContext.closeTab`             | Close tab             |
| `browsingContext.focusTab`             | Focus tab             |
| `browsingContext.focusWindow`          | Focus window          |
| `browsingContext.switchToFrame`        | Switch by element     |
| `browsingContext.switchToFrameByIndex` | Switch by index       |
| `browsingContext.switchToFrameByUrl`   | Switch by URL pattern |
| `browsingContext.switchToParentFrame`  | Switch to parent      |
| `browsingContext.getFrameCount`        | Get child frame count |
| `browsingContext.getAllFrames`         | Get all frames info   |

**Events:**

| Event                               | Description        |
| ----------------------------------- | ------------------ |
| `browsingContext.load`              | Page load complete |
| `browsingContext.domContentLoaded`  | DOM ready          |
| `browsingContext.navigationStarted` | Navigation began   |
| `browsingContext.navigationFailed`  | Navigation failed  |

### 4.3. element Module

| Command                    | Description                   |
| -------------------------- | ----------------------------- |
| `element.find`             | Find single element           |
| `element.findAll`          | Find all elements             |
| `element.getProperty`      | Get `element[name]`           |
| `element.setProperty`      | Set `element[name] = value`   |
| `element.callMethod`       | Call `element[name](...args)` |
| `element.subscribe`        | Watch for element appearance  |
| `element.unsubscribe`      | Stop watching                 |
| `element.watchRemoval`     | Watch element removal         |
| `element.unwatchRemoval`   | Stop watching removal         |
| `element.watchAttribute`   | Watch attribute changes       |
| `element.unwatchAttribute` | Stop watching attributes      |

**Events:**

| Event                      | Description       | Requires Reply |
| -------------------------- | ----------------- | -------------- |
| `element.added`            | Element appeared  | No             |
| `element.removed`          | Element removed   | No             |
| `element.attributeChanged` | Attribute changed | No             |

### 4.4. script Module

| Command                      | Description           |
| ---------------------------- | --------------------- |
| `script.evaluate`            | Execute sync script   |
| `script.evaluateAsync`       | Execute async script  |
| `script.addPreloadScript`    | Add preload script    |
| `script.removePreloadScript` | Remove preload script |

**CSP Bypass:** Uses `browser.scripting.executeScript` with `world: "MAIN"`.

### 4.5. input Module

| Command            | Description                        |
| ------------------ | ---------------------------------- |
| `input.typeKey`    | Type single key with modifiers     |
| `input.typeText`   | Type string character by character |
| `input.mouseClick` | Click at element/coordinates       |
| `input.mouseMove`  | Move to element/coordinates        |
| `input.mouseDown`  | Press mouse button                 |
| `input.mouseUp`    | Release mouse button               |

**Event Sequence (typeKey):**

```
keydown → value update → input → keypress → keyup
```

**Event Sequence (mouseClick):**

```
mousemove → mousedown → mouseup → click
```

### 4.6. network Module

| Command                   | Description            |
| ------------------------- | ---------------------- |
| `network.addIntercept`    | Enable interception    |
| `network.removeIntercept` | Disable interception   |
| `network.setBlockRules`   | Set URL block patterns |
| `network.clearBlockRules` | Clear block patterns   |

**Intercept Options:**

| Option                    | Description                  |
| ------------------------- | ---------------------------- |
| `interceptRequests`       | Intercept before send        |
| `interceptRequestHeaders` | Intercept request headers    |
| `interceptRequestBody`    | Log request body (read-only) |
| `interceptResponses`      | Intercept response headers   |
| `interceptResponseBody`   | Intercept response body      |

**Events:**

| Event                       | Description            | Requires Reply |
| --------------------------- | ---------------------- | -------------- |
| `network.beforeRequestSent` | Request about to send  | Yes            |
| `network.requestHeaders`    | Request headers        | Yes            |
| `network.requestBody`       | Request body (logging) | No             |
| `network.responseStarted`   | Response headers recv  | No             |
| `network.responseHeaders`   | Response headers       | Yes            |
| `network.responseBody`      | Response body          | Yes            |
| `network.responseCompleted` | Response completed     | No             |

**EventReply Actions:**

| Action                                            | Description    |
| ------------------------------------------------- | -------------- |
| `{ "action": "allow" }`                           | Continue       |
| `{ "action": "block" }`                           | Cancel         |
| `{ "action": "redirect", "url": "..." }`          | Redirect       |
| `{ "action": "modifyHeaders", "headers": {...} }` | Modify headers |
| `{ "action": "modifyBody", "body": "..." }`       | Modify body    |

### 4.7. proxy Module

| Command                  | Description            |
| ------------------------ | ---------------------- |
| `proxy.setWindowProxy`   | Set proxy for all tabs |
| `proxy.clearWindowProxy` | Clear window proxy     |
| `proxy.setTabProxy`      | Set proxy for tab      |
| `proxy.clearTabProxy`    | Clear tab proxy        |

**Proxy Types:** `http`, `https`, `socks4`, `socks5`, `direct`

**Scope:** Tab proxy overrides window proxy.

### 4.8. storage Module

| Command                 | Description        |
| ----------------------- | ------------------ |
| `storage.getCookie`     | Get cookie by name |
| `storage.setCookie`     | Set cookie         |
| `storage.deleteCookie`  | Delete cookie      |
| `storage.getAllCookies` | Get all cookies    |

---

## 5. Events

### 5.1. Event Model

Events are push-based notifications. Some require replies (network interception).

### 5.2. Subscription

Subscribe via `element.subscribe`:

```json
{
  "method": "element.subscribe",
  "params": {
    "selector": "#login-form",
    "oneShot": true
  }
}
```

**Response:**

```json
{
  "subscriptionId": "sub-uuid",
  "elementId": "elem-uuid" // If element already exists
}
```

### 5.3. Event-Driven Waiting

```rust
// Rust API
let element = tab.wait_for_element("#login").await?;

// Internally:
// 1. Send element.subscribe
// 2. Wait for element.added event
// 3. Return element
```

---

## 6. Errors

### 6.1. Error Codes

| Code                | Description                |
| ------------------- | -------------------------- |
| `unknown command`   | Method not recognized      |
| `invalid argument`  | Invalid parameter          |
| `no such element`   | Element not found          |
| `stale element`     | Element removed from DOM   |
| `no such frame`     | Frame not found            |
| `no such tab`       | Tab not found              |
| `no such intercept` | Intercept ID not found     |
| `no such script`    | Script ID not found        |
| `script error`      | JavaScript execution error |
| `timeout`           | Operation timed out        |
| `connection closed` | WebSocket closed           |
| `session not found` | Session ID not in pool     |
| `unknown error`     | Unexpected error           |

### 6.2. Rust Error Types

```rust
pub enum Error {
    Config { message: String },
    Profile { message: String },
    FirefoxNotFound { path: PathBuf },
    ProcessLaunchFailed { message: String },
    Connection { message: String },
    ConnectionTimeout { timeout_ms: u64 },
    ConnectionClosed,
    UnknownCommand { command: String },
    InvalidArgument { message: String },
    Protocol { message: String },
    ElementNotFound { selector: String, tab_id: TabId, frame_id: FrameId },
    StaleElement { element_id: ElementId },
    FrameNotFound { frame_id: FrameId },
    TabNotFound { tab_id: TabId },
    ScriptError { message: String },
    Timeout { operation: String, timeout_ms: u64 },
    RequestTimeout { request_id: RequestId, timeout_ms: u64 },
    InterceptNotFound { intercept_id: String },
    SessionNotFound { session_id: SessionId },
    Io(IoError),
    Json(serde_json::Error),
    WebSocket(WsError),
    ChannelClosed(RecvError),
}
```

---

## 7. Implementation

### 7.1. Rust Structure

```
src/
├── lib.rs              # Public exports
├── error.rs            # Error types
├── identifiers.rs      # ID newtypes
├── driver/
│   ├── mod.rs          # Module exports
│   ├── core.rs         # Driver factory (owns ConnectionPool)
│   ├── builder.rs      # DriverBuilder (async build)
│   ├── options.rs      # FirefoxOptions
│   ├── profile/
│   │   ├── mod.rs      # Profile management
│   │   ├── extensions.rs   # ExtensionSource
│   │   └── preferences.rs  # Firefox prefs
│   └── assets.rs       # Data URI generation
├── browser/
│   ├── mod.rs          # Module exports
│   ├── window.rs       # Window + WindowBuilder (holds pool ref)
│   ├── tab.rs          # Tab (navigation, frames, network)
│   ├── element.rs      # Element (properties, input)
│   ├── network.rs      # Interception types
│   └── proxy.rs        # ProxyConfig
├── protocol/
│   ├── mod.rs          # Module exports
│   ├── command.rs      # Command enums by module
│   ├── request.rs      # Request, Response
│   └── event.rs        # Event, EventReply, ParsedEvent
└── transport/
    ├── mod.rs          # Module exports
    ├── pool.rs         # ConnectionPool (multiplexed connections)
    └── connection.rs   # Connection, event loop
```

### 7.2. Extension Structure

```
extension/src/
├── background/
│   ├── index.ts        # Entry point, message routing
│   └── modules/
│       ├── session/        # session.status, session.stealLogs
│       ├── browsing-context/   # Navigation, tabs, frames
│       ├── element/        # Find, properties, observer
│       ├── script/         # Evaluate, preload
│       ├── input/          # Keyboard, mouse
│       ├── network/        # Interception, blocking
│       ├── proxy/          # Per-tab/window proxy
│       └── storage/        # Cookies
├── content/
│   ├── index.ts        # Entry point
│   ├── bridge.ts       # WEBDRIVER_INIT forwarding
│   ├── elements.ts     # Element store, input handlers
│   ├── observer.ts     # MutationObserver
│   ├── messaging.ts    # Message hub
│   └── logger.ts       # Content logging
├── core/
│   ├── index.ts        # Core exports
│   ├── registry.ts     # Handler registry
│   ├── session.ts      # WebSocket session
│   ├── logger.ts       # Background logging
│   └── utils.ts        # Utilities
├── types/
│   ├── index.ts        # Type exports
│   ├── protocol.ts     # Request, Response, Event
│   └── identifiers.ts  # ID types
└── popup/
    └── index.ts        # Debug UI
```

### 7.3. Handler Registration

Extension uses registry-based dispatch:

```typescript
// modules/element/index.ts
import { registry } from "../../../core/registry.js";

registry.register("element.find", handleFind);
registry.register("element.findAll", handleFindAll);
registry.register("element.getProperty", handleGetProperty);
// ...
```

### 7.4. Handler Count

| Module          | Handlers |
| --------------- | -------- |
| session         | 2        |
| browsingContext | 16       |
| element         | 11       |
| script          | 4        |
| input           | 6        |
| network         | 4        |
| proxy           | 4        |
| storage         | 4        |
| **Total**       | **51**   |

---

## Appendix A: Quick Reference

### A.1. All Commands

| Module          | Command                                                                                                                                                                                                                                   |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| session         | `status`, `stealLogs`                                                                                                                                                                                                                     |
| browsingContext | `navigate`, `reload`, `goBack`, `goForward`, `getTitle`, `getUrl`, `newTab`, `closeTab`, `focusTab`, `focusWindow`, `switchToFrame`, `switchToFrameByIndex`, `switchToFrameByUrl`, `switchToParentFrame`, `getFrameCount`, `getAllFrames` |
| element         | `find`, `findAll`, `getProperty`, `setProperty`, `callMethod`, `subscribe`, `unsubscribe`, `watchRemoval`, `unwatchRemoval`, `watchAttribute`, `unwatchAttribute`                                                                         |
| script          | `evaluate`, `evaluateAsync`, `addPreloadScript`, `removePreloadScript`                                                                                                                                                                    |
| input           | `typeKey`, `typeText`, `mouseClick`, `mouseMove`, `mouseDown`, `mouseUp`                                                                                                                                                                  |
| network         | `addIntercept`, `removeIntercept`, `setBlockRules`, `clearBlockRules`                                                                                                                                                                     |
| proxy           | `setWindowProxy`, `clearWindowProxy`, `setTabProxy`, `clearTabProxy`                                                                                                                                                                      |
| storage         | `getCookie`, `setCookie`, `deleteCookie`, `getAllCookies`                                                                                                                                                                                 |

### A.2. All Events

| Module          | Event                                                               | Requires Reply |
| --------------- | ------------------------------------------------------------------- | -------------- |
| browsingContext | `load`, `domContentLoaded`, `navigationStarted`, `navigationFailed` | No             |
| element         | `added`, `removed`, `attributeChanged`                              | No             |
| network         | `beforeRequestSent`                                                 | Yes            |
| network         | `requestHeaders`                                                    | Yes            |
| network         | `requestBody`                                                       | No             |
| network         | `responseStarted`                                                   | No             |
| network         | `responseHeaders`                                                   | Yes            |
| network         | `responseBody`                                                      | Yes            |
| network         | `responseCompleted`                                                 | No             |

### A.3. Rust API Quick Reference

```rust
// Driver
let driver = Driver::builder()
    .binary("/path/to/firefox")
    .extension("./extension")
    .build().await?;

// Window
let window = driver.window().headless().spawn().await?;
window.set_proxy(ProxyConfig::socks5("host", 1080)).await?;

// Tab
let tab = window.tab();
tab.goto("https://example.com").await?;
tab.set_block_rules(&["*ads*"]).await?;

// Element
let el = tab.find_element("#submit").await?;
el.click().await?;
el.type_text("hello").await?;
let text = el.get_text().await?;

// Wait
let el = tab.wait_for_element("#loaded").await?;

// Interception
let id = tab.intercept_request(|req| {
    if req.url.contains("ads") {
        RequestAction::block()
    } else {
        RequestAction::allow()
    }
}).await?;
```

---

_End of specification._
