//! Small browser actions called from event handlers (web-sys directly, no `eval`). No-ops on the host.

/// Copy text to the clipboard (best effort: needs a secure context and a user gesture).
pub fn copy_text(text: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let _ = window.navigator().clipboard().write_text(text);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = text;
}

/// Enter fullscreen on the first element matching `selector`, or leave fullscreen. Call from a click handler.
pub fn toggle_fullscreen(selector: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        match document.fullscreen_element() {
            Some(_) => document.exit_fullscreen(),
            None => {
                if let Ok(Some(element)) = document.query_selector(selector) {
                    let _ = element.request_fullscreen();
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = selector;
}

/// Whether the page is in fullscreen, sent on every enter / leave (the button, or Esc). Call once per page.
/// Never sends on the host.
pub fn watch_fullscreen() -> futures_channel::mpsc::UnboundedReceiver<bool> {
    let (tx, rx) = futures_channel::mpsc::unbounded();
    #[cfg(target_arch = "wasm32")]
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        use wasm_bindgen::{closure::Closure, JsCast};
        let doc = document.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            let _ = tx.unbounded_send(doc.fullscreen_element().is_some());
        });
        let _ = document.add_event_listener_with_callback("fullscreenchange", closure.as_ref().unchecked_ref());
        closure.forget(); // listens for the page's life
    }
    #[cfg(not(target_arch = "wasm32"))]
    drop(tx);
    rx
}

/// GET a same-origin file as text (`None` on a network error or a non-2xx status). Always `None` on the host.
pub async fn fetch_text(url: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        let response: web_sys::Response = JsFuture::from(web_sys::window()?.fetch_with_str(url)).await.ok()?.dyn_into().ok()?;
        if !response.ok() {
            return None;
        }
        JsFuture::from(response.text().ok()?).await.ok()?.as_string()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = url;
        None
    }
}

/// Resolve after `ms` milliseconds (a `setTimeout`). Never resolves on the host, so a polling loop there just waits.
pub async fn sleep_ms(ms: i32) {
    #[cfg(target_arch = "wasm32")]
    {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            if let Some(window) = web_sys::window() {
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
            }
        });
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ms;
        std::future::pending::<()>().await;
    }
}
