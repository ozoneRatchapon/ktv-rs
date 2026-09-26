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
