use std::sync::{Arc, Mutex};
use string_box::StringBox;
use value_box::{ReturnBoxerResult, ValueBox, ValueBoxIntoRaw, ValueBoxPointer};

#[derive(Clone, Debug)]
pub struct ScriptToEvaluate(Arc<ScriptToEvaluateData>);

impl ScriptToEvaluate {
    pub fn signal_semaphore(&self) {
        unsafe { (self.0.semaphore_signaller)(self.0.semaphore_index) };
    }

    pub fn script(&self) -> &str {
        self.0.script.as_str()
    }

    pub fn set_value(&self, value: String) {
        *self.0.result.lock().unwrap() = ScriptToEvaluateResult {
            value,
            state: ScriptEvaluationState::Finished,
        };
        self.signal_semaphore();
    }

    pub fn set_error(&self, error: String) {
        *self.0.result.lock().unwrap() = ScriptToEvaluateResult {
            value: error,
            state: ScriptEvaluationState::Errored,
        };
        self.signal_semaphore();
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
