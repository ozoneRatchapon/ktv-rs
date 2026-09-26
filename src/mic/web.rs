use js_sys::{Array, Float32Array, Object, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, Blob, BlobPropertyBag, DomException, MediaStream,
    MediaStreamConstraints, MediaStreamTrack, MessageEvent, Url,
};

use super::types::{MicError, FRAME_SIZE, HOP_SIZE};

const WORKLET_JS: &str = include_str!("../../assets/mic_worklet.js");
/// Must match `registerProcessor` in `assets/mic_worklet.js`.
const PROCESSOR_NAME: &str = "ktv-mic-frames";

/// Live mic session. Dropping it releases the device (browser mic indicator turns off).
/// Each resource is owned by a guard as soon as it exists, so a start that is cancelled
/// or fails half-way (component unmounted, permission race) never leaves the mic open.
pub struct Mic {
    // Field order is drop order: stop delivering frames, then the graph, then the device.
    _frames: FrameListener,
    _ctx: ContextGuard,
    _stream: StreamGuard,
}

impl Mic {
    /// Ask for the mic and start delivering `FRAME_SIZE`-sample mono frames every `HOP_SIZE` samples.
    /// `make_handler` receives the device sample rate and returns the per-frame callback.
    pub async fn start<F, H>(make_handler: F) -> Result<Self, MicError>
    where
        F: FnOnce(f32) -> H,
        H: FnMut(&[f32]) + 'static,
    {
        let window = web_sys::window().ok_or(MicError::Unsupported)?;
        let devices = window.navigator().media_devices().map_err(|_| MicError::Unsupported)?;

        let constraints = MediaStreamConstraints::new();
        constraints.set_audio(&audio_constraints());
        let request = devices.get_user_media_with_constraints(&constraints).map_err(to_mic_error)?;
        let stream = StreamGuard(JsFuture::from(request).await.map_err(to_mic_error)?.unchecked_into());

        let ctx = ContextGuard(AudioContext::new().map_err(to_mic_error)?);
        load_worklet(&ctx.0).await?;

        let options = AudioWorkletNodeOptions::new();
        options.set_number_of_inputs(1);
        options.set_number_of_outputs(0); // analysis only: never routed to the speakers
        options.set_channel_count(1);
        options.set_processor_options(Some(&processor_options()));
        let node = AudioWorkletNode::new_with_options(&ctx.0, PROCESSOR_NAME, &options).map_err(to_mic_error)?;
        let port = node.port().map_err(to_mic_error)?;

        let mut on_frame = make_handler(ctx.0.sample_rate());
        let mut buf = vec![0.0f32; FRAME_SIZE as usize];
        let closure = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            if let Ok(frame) = event.data().dyn_into::<Float32Array>() {
                buf.resize(frame.length() as usize, 0.0);
                frame.copy_to(&mut buf);
                on_frame(&buf);
            }
        });
        port.set_onmessage(Some(closure.as_ref().unchecked_ref()));
        let frames = FrameListener { port, node, _closure: closure };

        let source = ctx.0.create_media_stream_source(&stream.0).map_err(to_mic_error)?;
        source.connect_with_audio_node(&frames.node).map_err(to_mic_error)?;
        // Created after an await, so the context may start suspended under autoplay rules
        JsFuture::from(ctx.0.resume().map_err(to_mic_error)?).await.map_err(to_mic_error)?;

        Ok(Self { _frames: frames, _ctx: ctx, _stream: stream })
    }
}

/// Echo cancellation stays on: it removes the karaoke backing track (played by this tab)
/// from the mic signal, which would otherwise dominate the pitch. Noise suppression and AGC
/// are off because they smear sustained notes and pump the level.
fn audio_constraints() -> JsValue {
    let audio = Object::new();
    for (key, on) in [("echoCancellation", true), ("noiseSuppression", false), ("autoGainControl", false)] {
        let _ = Reflect::set(&audio, &key.into(), &on.into());
    }
    let _ = Reflect::set(&audio, &"channelCount".into(), &1.into());
    audio.into()
}

fn processor_options() -> Object {
    let opts = Object::new();
    let _ = Reflect::set(&opts, &"frame_size".into(), &FRAME_SIZE.into());
    let _ = Reflect::set(&opts, &"hop_size".into(), &HOP_SIZE.into());
    opts
}

/// Worklet source is compiled in and served from a Blob URL (no extra asset or fetch).
async fn load_worklet(ctx: &AudioContext) -> Result<(), MicError> {
    let parts = Array::of1(&WORKLET_JS.into());
    let props = BlobPropertyBag::new();
    props.set_type("text/javascript");
    let blob = Blob::new_with_str_sequence_and_options(&parts, &props).map_err(to_mic_error)?;
    let url = Url::create_object_url_with_blob(&blob).map_err(to_mic_error)?;
    let loaded = match ctx.audio_worklet() {
        Ok(worklet) => match worklet.add_module(&url) {
            Ok(promise) => JsFuture::from(promise).await.map(|_| ()).map_err(to_mic_error),
            Err(err) => Err(to_mic_error(err)),
        },
        Err(_) => Err(MicError::Unsupported), // insecure context: `audioWorklet` is undefined
    };
    let _ = Url::revoke_object_url(&url);
    loaded
}

fn to_mic_error(err: JsValue) -> MicError {
    match err.dyn_ref::<DomException>() {
        Some(dom) => MicError::from_dom_name(&dom.name(), &dom.message()),
        None => MicError::Failed(err.as_string().unwrap_or_else(|| format!("{err:?}"))),
    }
}

struct StreamGuard(MediaStream);

impl Drop for StreamGuard {
    fn drop(&mut self) {
        for track in self.0.get_tracks().iter() {
            if let Ok(track) = track.dyn_into::<MediaStreamTrack>() {
                track.stop();
            }
        }
    }
}

struct ContextGuard(AudioContext);

impl Drop for ContextGuard {
    fn drop(&mut self) {
        let _ = self.0.close();
    }
}

struct FrameListener {
    port: web_sys::MessagePort,
    node: AudioWorkletNode,
    _closure: Closure<dyn FnMut(MessageEvent)>,
}

impl Drop for FrameListener {
    fn drop(&mut self) {
        // Detach before the closure is freed: a queued message must not call into freed memory
        self.port.set_onmessage(None);
        self.node.disconnect().ok();
        self.port.close();
    }
}
