# Changelog

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Released]

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

[Unreleased]: https://github.com/user/firefox-webdriver/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/user/firefox-webdriver/releases/tag/v0.1.0
