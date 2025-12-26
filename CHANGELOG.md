# Changelog

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3] - 2025-12-26

### Added

#### Screenshot API
- `Tab::screenshot()` - Builder for capturing screenshots
- `ScreenshotBuilder::png()` - PNG format (default)
- `ScreenshotBuilder::jpeg(quality)` - JPEG format with quality (0-100)
- `ScreenshotBuilder::full_page()` - Capture full scrollable page
- `ScreenshotBuilder::capture()` - Returns base64 string
- `ScreenshotBuilder::capture_bytes()` - Returns raw bytes
- `ScreenshotBuilder::save(path)` - Saves to file
- `Tab::capture_screenshot()` - Quick PNG capture
- `Tab::save_screenshot(path)` - Quick save (format from extension)
- `ImageFormat` enum (Png, Jpeg)

#### Scroll API
- `Tab::scroll_by(x, y)` - Scroll by offset
- `Tab::scroll_to(x, y)` - Scroll to position
- `Tab::scroll_to_top()` - Scroll to top
- `Tab::scroll_to_bottom()` - Scroll to bottom
- `Tab::get_scroll_position()` - Get current scroll position
- `Tab::get_page_size()` - Get page dimensions
- `Tab::get_viewport_size()` - Get viewport dimensions

#### Element Convenience Methods
- `Element::double_click()` - Double-click element
- `Element::context_click()` - Right-click element
- `Element::hover()` - Hover over element
- `Element::scroll_into_view()` - Scroll element into view (smooth)
- `Element::scroll_into_view_instant()` - Scroll element into view (instant)
- `Element::get_bounding_rect()` - Get element position/size

#### Checkbox/Radio Methods
- `Element::is_checked()` - Check if checked
- `Element::check()` - Check the element
- `Element::uncheck()` - Uncheck the element
- `Element::toggle()` - Toggle checked state
- `Element::set_checked(bool)` - Set checked state

#### Select/Dropdown Methods
- `Element::select_by_text(text)` - Select option by visible text
- `Element::select_by_value(value)` - Select option by value attribute
- `Element::select_by_index(index)` - Select option by index
- `Element::get_selected_value()` - Get selected option's value
- `Element::get_selected_index()` - Get selected option's index
- `Element::get_selected_text()` - Get selected option's text
- `Element::is_multiple()` - Check if multi-select

#### Navigation
- `Tab::get_page_source()` - Get page HTML source

### Changed

- **BREAKING**: Refactored `src/browser/tab.rs` into `src/browser/tab/` module folder
  - `tab/core.rs` - Tab struct and accessors
  - `tab/navigation.rs` - URL navigation, history
  - `tab/frames.rs` - Frame switching
  - `tab/script.rs` - JavaScript execution
  - `tab/elements.rs` - Element search and observation
  - `tab/network.rs` - Request interception, blocking
  - `tab/storage.rs` - Cookies, localStorage, sessionStorage
  - `tab/proxy.rs` - Tab-level proxy
  - `tab/screenshot.rs` - Screenshot capture
  - `tab/scroll.rs` - Scroll control

## [0.1.2] - 2025-12-26

### Fixed

- Fixed Connection drop causing premature shutdown when cloned connections were dropped
- Fixed ARCHITECTURE.md version mismatch (was 2.0.0, now matches Cargo.toml)
- Fixed Section 1.4 reference in `src/identifiers.rs` (was incorrectly 1.7)
- Added missing `SessionNotFound` error to ARCHITECTURE.md §6.2
- Added missing `responseStarted`, `responseCompleted` network events to documentation
- Updated extension filenames to match current version
- Added Firefox download link for Linux users in README.md

### Removed

- Removed `PendingServer` (legacy, replaced by `ConnectionPool`)

## [0.1.1] - 2025-12-24

### Changed

#### Architecture

- **BREAKING**: `Driver::builder().build()` is now async (returns `impl Future<Output = Result<Driver>>`)
- Refactored to single-port multiplexed WebSocket architecture
- `Driver` now owns a `ConnectionPool` that binds once at creation
- All Firefox windows connect to the same WebSocket port
- Messages routed by `SessionId` instead of per-window connections
- `Window` no longer owns `Connection`, holds `Arc<ConnectionPool>` + `SessionId`

#### Transport

- Added `ConnectionPool` for managing multiple WebSocket connections
- `ConnectionPool::new()` binds WebSocket server on creation
- `ConnectionPool::wait_for_session()` waits for specific session to connect
- `ConnectionPool::send()` routes requests by `SessionId`
- `Connection` now implements `Clone`

#### Identifiers

- Added `SessionId::from_u32()` for parsing session IDs from READY messages

#### Errors

- Added `Error::SessionNotFound` for when session ID doesn't exist in pool

## [0.1.0] - 2025-12-23

### Added

#### Core

- `Driver` - Factory for creating browser Windows
- `DriverBuilder` - Configuration builder with `binary()`, `extension()`, `extension_base64()`
- `Window` - Browser window owning Firefox process, WebSocket, profile
- `WindowBuilder` - Configuration with `headless()`, `window_size()`, `profile()`
- `Tab` - Browser tab with frame context
- `Element` - DOM element reference (UUID-based)

#### Navigation

- `Tab::goto()` - Navigate to URL
- `Tab::reload()` - Reload page
- `Tab::back()` - Navigate back
- `Tab::forward()` - Navigate forward
- `Tab::get_title()` - Get page title
- `Tab::get_url()` - Get current URL
- `Tab::load_html()` - Load HTML directly

#### Element Search

- `Tab::find_element()` - Find single element by CSS selector
- `Tab::find_elements()` - Find all matching elements
- `Tab::wait_for_element()` - Wait for element (MutationObserver)
- `Tab::wait_for_element_timeout()` - Wait with custom timeout
- `Element::find_element()` - Nested search
- `Element::find_elements()` - Nested search all

#### Element Interaction

- `Element::click()` - Click element
- `Element::focus()` - Focus element
- `Element::blur()` - Blur element
- `Element::clear()` - Clear value
- `Element::type_text()` - Type text with keyboard events
- `Element::set_value()` - Set value directly
- `Element::get_text()` - Get text content
- `Element::get_value()` - Get input value
- `Element::get_attribute()` - Get attribute
- `Element::is_displayed()` - Check visibility
- `Element::is_enabled()` - Check enabled state

#### Mouse Input

- `Element::mouse_click()` - Click with mouse events
- `Element::mouse_move()` - Move mouse to element
- `Element::mouse_down()` - Press mouse button
- `Element::mouse_up()` - Release mouse button

#### Keyboard Input

- `Element::type_key()` - Type single key with modifiers
- `Element::type_char()` - Type single character

#### Frame Switching

- `Tab::switch_to_frame()` - Switch by element
- `Tab::switch_to_frame_by_index()` - Switch by index
- `Tab::switch_to_frame_by_url()` - Switch by URL pattern
- `Tab::switch_to_parent_frame()` - Switch to parent
- `Tab::switch_to_main_frame()` - Switch to main frame
- `Tab::get_frame_count()` - Get child frame count
- `Tab::get_all_frames()` - Get all frame info

#### Script Execution

- `Tab::execute_script()` - Execute synchronous JavaScript
- `Tab::execute_async_script()` - Execute async JavaScript

#### Network

- `Tab::set_block_rules()` - Block URLs by pattern
- `Tab::clear_block_rules()` - Clear block rules
- `Tab::intercept_request()` - Intercept requests
- `Tab::intercept_request_headers()` - Intercept request headers
- `Tab::intercept_request_body()` - Intercept request body
- `Tab::intercept_response()` - Intercept response headers
- `Tab::intercept_response_body()` - Intercept response body
- `Tab::stop_intercept()` - Stop interception
- `RequestAction` - Allow, Block, Redirect
- `HeadersAction` - Allow, ModifyHeaders
- `BodyAction` - Allow, ModifyBody

#### Proxy

- `Window::set_proxy()` - Set window-level proxy
- `Window::clear_proxy()` - Clear window proxy
- `Tab::set_proxy()` - Set tab-level proxy
- `Tab::clear_proxy()` - Clear tab proxy
- `ProxyConfig` - HTTP, HTTPS, SOCKS4, SOCKS5, Direct
- `ProxyConfig::with_credentials()` - Add authentication
- `ProxyConfig::with_proxy_dns()` - Enable DNS proxying

#### Storage

- `Tab::get_cookie()` - Get cookie by name
- `Tab::set_cookie()` - Set cookie
- `Tab::delete_cookie()` - Delete cookie
- `Tab::get_all_cookies()` - Get all cookies
- `Tab::local_storage_get()` - Get localStorage value
- `Tab::local_storage_set()` - Set localStorage value
- `Tab::local_storage_delete()` - Delete localStorage key
- `Tab::local_storage_clear()` - Clear localStorage
- `Tab::session_storage_get()` - Get sessionStorage value
- `Tab::session_storage_set()` - Set sessionStorage value
- `Tab::session_storage_delete()` - Delete sessionStorage key
- `Tab::session_storage_clear()` - Clear sessionStorage

#### Element Observation

- `Tab::on_element_added()` - Callback when element appears
- `Tab::on_element_removed()` - Callback when element removed
- `Tab::unsubscribe()` - Stop observation

#### Tab Management

- `Window::tab()` - Get initial tab
- `Window::new_tab()` - Create new tab
- `Window::tab_count()` - Get tab count
- `Tab::focus()` - Focus tab
- `Tab::close()` - Close tab

#### Error Handling

- `Error` enum with 20+ variants
- `Error::is_timeout()` - Check timeout errors
- `Error::is_element_error()` - Check element errors
- `Error::is_connection_error()` - Check connection errors
- `Error::is_recoverable()` - Check if retryable

#### Identifiers

- `SessionId` - Session identifier
- `TabId` - Tab identifier
- `FrameId` - Frame identifier
- `ElementId` - Element identifier
- `InterceptId` - Network intercept identifier
- `SubscriptionId` - Element subscription identifier

### Architecture

- WebSocket-based communication
- WebExtension for browser control
- MutationObserver for element waiting (no polling)
- UUID-based element references (undetectable)
- Per-window process isolation

[Unreleased]: https://github.com/Dark-Captcha/Firefox-WebDriver/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/Dark-Captcha/Firefox-WebDriver/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/Dark-Captcha/Firefox-WebDriver/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Dark-Captcha/Firefox-WebDriver/releases/tag/v0.1.0
