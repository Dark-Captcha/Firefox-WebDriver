//! Element search and observation methods.

use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex as ParkingMutex;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tracing::debug;

use crate::browser::Element;
use crate::browser::selector::By;
use crate::error::{Error, Result};
use crate::identifiers::{ElementId, SubscriptionId};
use crate::protocol::event::ParsedEvent;
use crate::protocol::{Command, ElementCommand, Event};

use super::Tab;

// ============================================================================
// Constants
// ============================================================================

/// Default timeout for wait_for_element (30 seconds).
const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

// ============================================================================
// Tab - Element Search
// ============================================================================

impl Tab {
    /// Finds a single element using a locator strategy.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use firefox_webdriver::By;
    ///
    /// // CSS selector
    /// let btn = tab.find_element(By::Css("#submit")).await?;
    ///
    /// // By ID
    /// let form = tab.find_element(By::Id("login-form")).await?;
    ///
    /// // By text content
    /// let link = tab.find_element(By::Text("Click here")).await?;
    ///
    /// // By XPath
    /// let btn = tab.find_element(By::XPath("//button[@type='submit']")).await?;
    /// ```
    pub async fn find_element(&self, by: By) -> Result<Element> {
        let command = Command::Element(ElementCommand::Find {
            strategy: by.strategy().to_string(),
            value: by.value().to_string(),
            parent_id: None,
        });

        let response = self.send_command(command).await?;

        let element_id = response
            .result
            .as_ref()
            .and_then(|v| v.get("elementId"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                Error::element_not_found(
                    format!("{}:{}", by.strategy(), by.value()),
                    self.inner.tab_id,
                    self.inner.frame_id,
                )
            })?;

        Ok(Element::new(
            ElementId::new(element_id),
            self.inner.tab_id,
            self.inner.frame_id,
            self.inner.session_id,
            self.inner.window.clone(),
        ))
    }

    /// Finds all elements using a locator strategy.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use firefox_webdriver::By;
    ///
    /// let buttons = tab.find_elements(By::Tag("button")).await?;
    /// let links = tab.find_elements(By::PartialText("Read")).await?;
    /// ```
    pub async fn find_elements(&self, by: By) -> Result<Vec<Element>> {
        let command = Command::Element(ElementCommand::FindAll {
            strategy: by.strategy().to_string(),
            value: by.value().to_string(),
            parent_id: None,
        });

        let response = self.send_command(command).await?;

        let elements = response
            .result
            .as_ref()
            .and_then(|v| v.get("elementIds"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|id| {
                        Element::new(
                            ElementId::new(id),
                            self.inner.tab_id,
                            self.inner.frame_id,
                            self.inner.session_id,
                            self.inner.window.clone(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(elements)
    }
}

// ============================================================================
// Tab - Element Observation
// ============================================================================

impl Tab {
    /// Waits for an element using a locator strategy.
    ///
    /// Uses MutationObserver (no polling). Times out after 30 seconds.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use firefox_webdriver::By;
    ///
    /// let btn = tab.wait_for_element(By::Id("submit")).await?;
    /// let link = tab.wait_for_element(By::Css("a.login")).await?;
    /// let el = tab.wait_for_element(By::XPath("//button")).await?;
    /// ```
    pub async fn wait_for_element(&self, by: By) -> Result<Element> {
        self.wait_for_element_timeout(by, DEFAULT_WAIT_TIMEOUT)
            .await
    }

    /// Waits for an element using a locator strategy with custom timeout.
    pub async fn wait_for_element_timeout(
        &self,
        by: By,
        timeout_duration: Duration,
    ) -> Result<Element> {
        debug!(
            tab_id = %self.inner.tab_id,
            strategy = by.strategy(),
            value = by.value(),
            timeout_ms = timeout_duration.as_millis(),
            "Waiting for element"
        );

        let window = self.get_window()?;

        let (tx, rx) = oneshot::channel::<Result<Element>>();
        let tx = Arc::new(ParkingMutex::new(Some(tx)));
        let expected_strategy = by.strategy().to_string();
        let expected_value = by.value().to_string();
        let tab_id = self.inner.tab_id;
        let frame_id = self.inner.frame_id;
        let session_id = self.inner.session_id;
        let window_clone = self.inner.window.clone();
        let tx_clone = Arc::clone(&tx);

        window.inner.pool.set_event_handler(
            window.inner.session_id,
            Box::new(move |event: Event| {
                if event.method.as_str() != "element.added" {
                    return None;
                }

                let parsed = event.parse();
                if let ParsedEvent::ElementAdded {
                    strategy,
                    value,
                    element_id,
                    ..
                } = parsed
                    && strategy == expected_strategy
                    && value == expected_value
                {
                    let element = Element::new(
                        ElementId::new(&element_id),
                        tab_id,
                        frame_id,
                        session_id,
                        window_clone.clone(),
                    );

                    if let Some(tx) = tx_clone.lock().take() {
                        let _ = tx.send(Ok(element));
                    }
                }

                None
            }),
        );

        let command = Command::Element(ElementCommand::Subscribe {
            strategy: by.strategy().to_string(),
            value: by.value().to_string(),
            one_shot: true,
            timeout: Some(timeout_duration.as_millis() as u64),
        });
        let response = self.send_command(command).await?;

        // Check if element already exists
        if let Some(element_id) = response
            .result
            .as_ref()
            .and_then(|v| v.get("elementId"))
            .and_then(|v| v.as_str())
        {
            window
                .inner
                .pool
                .clear_event_handler(window.inner.session_id);

            return Ok(Element::new(
                ElementId::new(element_id),
                self.inner.tab_id,
                self.inner.frame_id,
                self.inner.session_id,
                self.inner.window.clone(),
            ));
        }

        let result = timeout(timeout_duration, rx).await;

        window
            .inner
            .pool
            .clear_event_handler(window.inner.session_id);

        match result {
            Ok(Ok(element)) => element,
            Ok(Err(_)) => Err(Error::protocol("Channel closed unexpectedly")),
            Err(_) => Err(Error::Timeout {
                operation: format!("wait_for({}:{})", by.strategy(), by.value()),
                timeout_ms: timeout_duration.as_millis() as u64,
            }),
        }
    }

    /// Registers a callback for when elements matching the selector appear.
    ///
    /// # Returns
    ///
    /// Subscription ID for later unsubscription.
    pub async fn on_element_added<F>(&self, by: By, callback: F) -> Result<SubscriptionId>
    where
        F: Fn(Element) + Send + Sync + 'static,
    {
        debug!(
            tab_id = %self.inner.tab_id,
            strategy = by.strategy(),
            value = by.value(),
            "Subscribing to element.added"
        );

        let window = self.get_window()?;

        let expected_strategy = by.strategy().to_string();
        let expected_value = by.value().to_string();
        let tab_id = self.inner.tab_id;
        let frame_id = self.inner.frame_id;
        let session_id = self.inner.session_id;
        let window_clone = self.inner.window.clone();
        let callback = Arc::new(callback);

        window.inner.pool.set_event_handler(
            window.inner.session_id,
            Box::new(move |event: Event| {
                if event.method.as_str() != "element.added" {
                    return None;
                }

                let parsed = event.parse();
                if let ParsedEvent::ElementAdded {
                    strategy,
                    value,
                    element_id,
                    ..
                } = parsed
                    && strategy == expected_strategy
                    && value == expected_value
                {
                    let element = Element::new(
                        ElementId::new(&element_id),
                        tab_id,
                        frame_id,
                        session_id,
                        window_clone.clone(),
                    );
                    callback(element);
                }

                None
            }),
        );

        let command = Command::Element(ElementCommand::Subscribe {
            strategy: by.strategy().to_string(),
            value: by.value().to_string(),
            one_shot: false,
            timeout: None,
        });

        let response = self.send_command(command).await?;

        let subscription_id = response
            .result
            .as_ref()
            .and_then(|v| v.get("subscriptionId"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::protocol("No subscriptionId in response"))?;

        Ok(SubscriptionId::new(subscription_id))
    }

    /// Registers a callback for when a specific element is removed.
    pub async fn on_element_removed<F>(&self, element_id: &ElementId, callback: F) -> Result<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        debug!(tab_id = %self.inner.tab_id, %element_id, "Watching for element removal");

        let window = self.get_window()?;

        let element_id_clone = element_id.as_str().to_string();
        let callback = Arc::new(callback);

        window.inner.pool.set_event_handler(
            window.inner.session_id,
            Box::new(move |event: Event| {
                if event.method.as_str() != "element.removed" {
                    return None;
                }

                let parsed = event.parse();
                if let ParsedEvent::ElementRemoved {
                    element_id: removed_id,
                    ..
                } = parsed
                    && removed_id == element_id_clone
                {
                    callback();
                }

                None
            }),
        );

        let command = Command::Element(ElementCommand::WatchRemoval {
            element_id: element_id.clone(),
        });

        self.send_command(command).await?;
        Ok(())
    }

    /// Unsubscribes from element observation.
    pub async fn unsubscribe(&self, subscription_id: &SubscriptionId) -> Result<()> {
        let command = Command::Element(ElementCommand::Unsubscribe {
            subscription_id: subscription_id.as_str().to_string(),
        });

        self.send_command(command).await?;

        if let Some(window) = &self.inner.window {
            window
                .inner
                .pool
                .clear_event_handler(window.inner.session_id);
        }

        Ok(())
    }
}
