//! Calls into the classic scripts loaded in the static `<head>` (`assets/ktv_sync.js`, `assets/ktv_keys.js`)
//! through `Reflect`, never `eval`, so the CSP needs no `'unsafe-eval'`. Messages from JS arrive on a channel.

use futures_channel::mpsc::{unbounded, UnboundedReceiver};

/// A value passed to a JS method.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JsArg {
    Num(f64),
    Bool(bool),
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use js_sys::{Array, Function, Reflect};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    use super::JsArg;

    type Sender = Closure<dyn FnMut(JsValue)>;

    thread_local! {
        /// Latest message callback per core; replacing one drops the old closure (the core was rebound first).
        static SENDERS: RefCell<HashMap<&'static str, Sender>> = RefCell::new(HashMap::new());
    }

    fn global(name: &str) -> Option<JsValue> {
        let window: JsValue = web_sys::window()?.into();
        let value = Reflect::get(&window, &name.into()).ok()?;
        (!value.is_undefined() && !value.is_null()).then_some(value)
    }

    pub fn call(object: &str, method: &str, args: &[JsArg]) -> Option<JsValue> {
        let target = global(object)?;
        let function: Function = Reflect::get(&target, &method.into()).ok()?.dyn_into().ok()?;
        let args: Array = args
            .iter()
            .map(|arg| match *arg {
                JsArg::Num(n) => JsValue::from_f64(n),
                JsArg::Bool(b) => JsValue::from_bool(b),
            })
            .collect();
        function.apply(&target, &args).ok()
    }

    pub fn install(core: &'static str, send: Box<dyn FnMut(String)>) {
        let mut send = send;
        let closure = Closure::<dyn FnMut(JsValue)>::new(move |msg: JsValue| {
            if let Some(text) = msg.as_string() {
                send(text);
            }
        });
        let (Some(target), Some(window)) = (global(core), web_sys::window()) else { return };
        let Ok(install) = Reflect::get(&target, &"install".into()).and_then(|f| f.dyn_into::<Function>()) else { return };
        if install.call2(&target, &window, closure.as_ref()).is_ok() {
            SENDERS.with(|s| s.borrow_mut().insert(core, closure));
        }
    }

    pub fn get_f64(value: &JsValue, key: &str) -> Option<f64> {
        Reflect::get(value, &key.into()).ok()?.as_f64()
    }
}

/// Call `window[object][method](...args)`; `None` if it is not loaded (or on the host).
pub fn call(object: &str, method: &str, args: &[JsArg]) -> Option<JsValueOut> {
    #[cfg(target_arch = "wasm32")]
    {
        web::call(object, method, args).map(JsValueOut)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (object, method, args);
        None
    }
}

/// Opaque JS return value (wasm only); read fields with [`JsValueOut::f64`].
pub struct JsValueOut(#[cfg(target_arch = "wasm32")] wasm_bindgen::JsValue);

impl JsValueOut {
    pub fn f64(&self, key: &str) -> Option<f64> {
        #[cfg(target_arch = "wasm32")]
        {
            web::get_f64(&self.0, key)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = key;
            None
        }
    }
}

/// `window[core].install(window, send)`: wires the core once per page (a second call only rebinds `send`) and
/// returns its messages. On the host the channel is closed at once.
pub fn install(core: &'static str) -> UnboundedReceiver<String> {
    let (tx, rx) = unbounded::<String>();
    #[cfg(target_arch = "wasm32")]
    web::install(core, Box::new(move |msg| {
        let _ = tx.unbounded_send(msg);
    }));
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (core, tx);
    }
    rx
}
