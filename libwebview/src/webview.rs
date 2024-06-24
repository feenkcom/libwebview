use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use raw_window_handle_extensions::VeryRawWindowHandle;
use string_box::StringBox;
use value_box::{ReturnBoxerResult, ValueBox, ValueBoxIntoRaw, ValueBoxPointer};
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::raw_window_handle::{RawWindowHandle, WindowHandle};
use wry::{Rect, WebView, WebViewAttributes, WebViewBuilder};

fn build(
    attributes: *mut ValueBox<WebViewAttributes>,
    raw_window_handle: *mut VeryRawWindowHandle,
) -> value_box::Result<WebView> {
    let raw_window_handle = unsafe { VeryRawWindowHandle::from_ptr(raw_window_handle) }
        .map_err(|error| anyhow!(error))?
        .clone();

    let raw_window_handle = TryInto::<RawWindowHandle>::try_into(raw_window_handle.clone())
        .map_err(|error| anyhow!(error))?;

    let window_handle = unsafe { WindowHandle::borrow_raw(raw_window_handle) };

    let mut builder = WebViewBuilder::new_as_child(&window_handle);

    attributes.take_value().map(|mut attributes| {
        attributes.devtools = true;
        builder.attrs = attributes
    })?;

    let webview = builder.build().map_err(|error| anyhow!(error))?;
    Ok(webview)
}

#[derive(Clone, Debug)]
pub struct ScriptToEvaluate(Arc<ScriptToEvaluateData>);

impl ScriptToEvaluate {
    pub fn signal_semaphore(&self) {
        unsafe { (self.0.semaphore_signaller)(self.0.semaphore_index) };
    }
}

#[derive(Debug)]
struct ScriptToEvaluateData {
    script: String,
    result: Mutex<ScriptToEvaluateResult>,
    semaphore_index: usize,
    semaphore_signaller: unsafe extern "C" fn(usize),
}

#[derive(Debug)]
struct ScriptToEvaluateResult {
    value: String,
    state: ScriptEvaluationState,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ScriptEvaluationState {
    Pending,
    Finished,
    Errored,
    Released,
}

#[no_mangle]
pub extern "C" fn webview_build(
    attributes: *mut ValueBox<WebViewAttributes>,
    window_handle: *mut VeryRawWindowHandle,
) -> *mut ValueBox<WebView> {
    build(attributes, window_handle)
        .map(|webview| ValueBox::new(webview))
        .into_raw()
}

#[no_mangle]
pub extern "C" fn webview_script_to_evaluate_new(
    script: *mut ValueBox<StringBox>,
    semaphore_index: usize,
    semaphore_signaller: unsafe extern "C" fn(usize),
) -> *mut ValueBox<ScriptToEvaluate> {
    script
        .with_ref_ok(|script| {
            ValueBox::new(ScriptToEvaluate(Arc::new(ScriptToEvaluateData {
                script: script.to_string(),
                result: Mutex::new(ScriptToEvaluateResult {
                    value: "".to_string(),
                    state: ScriptEvaluationState::Pending,
                }),
                semaphore_index,
                semaphore_signaller,
            })))
        })
        .into_raw()
}

#[no_mangle]
pub extern "C" fn webview_script_to_evaluate_get_result(
    script: *mut ValueBox<ScriptToEvaluate>,
    result: *mut ValueBox<StringBox>,
) -> ScriptEvaluationState {
    script
        .with_ref(|script| {
            result.with_mut_ok(|result| {
                let lock = script.0.result.lock().unwrap();
                result.set_string(lock.value.clone());
                lock.state
            })
        })
        .or_log(ScriptEvaluationState::Released)
}

#[no_mangle]
pub extern "C" fn webview_script_to_evaluate_release(script: *mut ValueBox<ScriptToEvaluate>) {
    script.release();
}

#[no_mangle]
pub extern "C" fn webview_evaluate_script_with_result(
    webview: *mut ValueBox<WebView>,
    script: *mut ValueBox<ScriptToEvaluate>,
) {
    webview
        .with_ref(|webview| {
            script.with_ref(|script| {
                let script_clone = script.clone();
                webview
                    .evaluate_script_with_callback(script.0.script.as_str(), move |value| {
                        *script_clone.0.result.lock().unwrap() = ScriptToEvaluateResult {
                            value,
                            state: ScriptEvaluationState::Finished,
                        };
                        script_clone.signal_semaphore();
                    })
                    .inspect_err(|error| {
                        *script.0.result.lock().unwrap() = ScriptToEvaluateResult {
                            value: error.to_string(),
                            state: ScriptEvaluationState::Errored,
                        };
                    })
                    .map_err(|error| anyhow!(error).into())
            })
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_set_visible(webview: *mut ValueBox<WebView>, is_visible: bool) {
    webview
        .with_ref_ok(|webview| webview.set_visible(is_visible))
        .log();
}

#[no_mangle]
pub extern "C" fn webview_set_bounds(
    webview: *mut ValueBox<WebView>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) {
    webview
        .with_ref_ok(|webview| {
            webview.set_bounds(Rect {
                position: LogicalPosition::new(x, y).into(),
                size: LogicalSize::new(width, height).into(),
            })
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_release(webview: *mut ValueBox<WebView>) {
    webview.release();
}
