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

/// Say `text` in `lang` (a BCP 47 tag) with an on-device voice, queued after anything still being said.
/// Returns `false` without speaking when the device has no local voice for the language: Chrome's network voices
/// ("Google US English") send the text to a server. No-op (`false`) on the host.
pub fn speak(text: &str, lang: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        let Some(synth) = web_sys::window().and_then(|w| w.speech_synthesis().ok()) else {
            return false;
        };
        let primary = |tag: &str| tag.split(['-', '_']).next().unwrap_or_default().to_ascii_lowercase();
        let voice = synth
            .get_voices()
            .iter()
            .filter_map(|v| v.dyn_into::<web_sys::SpeechSynthesisVoice>().ok())
            .find(|v| v.local_service() && primary(&v.lang()) == primary(lang));
        let (Some(voice), Ok(utterance)) = (voice, web_sys::SpeechSynthesisUtterance::new_with_text(text)) else {
            return false;
        };
        utterance.set_voice(Some(&voice));
        utterance.set_lang(lang);
        synth.speak(&utterance);
        true
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (text, lang);
        false
    }
}

/// Ask for the speech voices early: Chrome loads them on the first request and answers an empty list until then.
pub fn load_voices() {
    #[cfg(target_arch = "wasm32")]
    if let Some(synth) = web_sys::window().and_then(|w| w.speech_synthesis().ok()) {
        let _ = synth.get_voices();
    }
}

/// Stop speaking and drop anything queued (the MC was turned off).
pub fn stop_speaking() {
    #[cfg(target_arch = "wasm32")]
    if let Some(synth) = web_sys::window().and_then(|w| w.speech_synthesis().ok()) {
        synth.cancel();
    }
}

/// Whether the MC is speaking right now. The mic ignores those moments: speech played by the OS may not pass
/// through this tab's echo cancellation, and the MC's voice must not be scored as singing. `false` on the host.
pub fn is_speaking() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window().and_then(|w| w.speech_synthesis().ok()).is_some_and(|synth| synth.speaking())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

/// This page's origin (`https://host`), for links a phone opens. `None` on the host.
pub fn page_origin() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()?.location().origin().ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// GET a same-origin file as text (`None` on a network error or a non-2xx status). Always `None` on the host.
pub async fn fetch_text(url: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        response_text(web_sys::window()?.fetch_with_str(url)).await
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = url;
        None
    }
}

/// POST a JSON body (a JSON-RPC call) and read the answer as text; gives up after `timeout_ms`.
/// `None` on a network error, a timeout or a non-2xx status (an RPC rate limit is HTTP 429). Always `None` on the host.
pub async fn post_json(url: &str, body: &str, timeout_ms: u32) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let init = web_sys::RequestInit::new();
        init.set_method("POST");
        init.set_body(&wasm_bindgen::JsValue::from_str(body));
        let headers = web_sys::Headers::new().ok()?;
        headers.set("content-type", "application/json").ok()?;
        init.set_headers(&headers);
        init.set_signal(Some(&web_sys::AbortSignal::timeout_with_u32(timeout_ms)));
        response_text(web_sys::window()?.fetch_with_str_and_init(url, &init)).await
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (url, body, timeout_ms);
        None
    }
}

#[cfg(target_arch = "wasm32")]
async fn response_text(request: js_sys::Promise) -> Option<String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    let response: web_sys::Response = JsFuture::from(request).await.ok()?.dyn_into().ok()?;
    if !response.ok() {
        return None;
    }
    JsFuture::from(response.text().ok()?).await.ok()?.as_string()
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

#[cfg(target_arch = "wasm32")]
thread_local! {
    /// One audio context for count-in clicks, made on the first tap (browsers start audio only from a gesture).
    static CLICKS: std::cell::RefCell<Option<web_sys::AudioContext>> = const { std::cell::RefCell::new(None) };
}

/// Get the click sound ready. Call from a click handler: the first call makes the audio context.
pub fn prepare_clicks() {
    #[cfg(target_arch = "wasm32")]
    CLICKS.with(|cell| {
        let mut ctx = cell.borrow_mut();
        if ctx.is_none() {
            *ctx = web_sys::AudioContext::new().ok();
        }
        if let Some(ctx) = ctx.as_ref() {
            let _ = ctx.resume();
        }
    });
}

/// Short clicks `delays` seconds from now on the audio clock (sample-accurate); the first is higher, like a
/// metronome's downbeat. Negative delays are skipped. Needs [`prepare_clicks`] first; no-op on the host.
pub fn clicks_in(delays: &[f64]) {
    #[cfg(target_arch = "wasm32")]
    CLICKS.with(|cell| {
        let Some(ctx) = cell.borrow().clone() else { return };
        let now = ctx.current_time();
        for (i, &delay) in delays.iter().enumerate().filter(|(_, d)| **d >= 0.0) {
            let at = now + delay;
            let (Ok(osc), Ok(gain)) = (ctx.create_oscillator(), ctx.create_gain()) else { return };
            osc.frequency().set_value(if i == 0 { 1500.0 } else { 1000.0 });
            let level = gain.gain();
            let _ = level.set_value_at_time(0.0, at);
            let _ = level.linear_ramp_to_value_at_time(0.6, at + 0.002);
            let _ = level.exponential_ramp_to_value_at_time(0.001, at + 0.06);
            if osc.connect_with_audio_node(&gain).is_ok() && gain.connect_with_audio_node(&ctx.destination()).is_ok() {
                let _ = osc.start_with_when(at);
                let _ = osc.stop_with_when(at + 0.07);
            }
        }
    });
    #[cfg(not(target_arch = "wasm32"))]
    let _ = delays;
}
