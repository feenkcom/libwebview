use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use string_box::StringBox;

use value_box::{ReturnBoxerResult, ValueBox, ValueBoxPointer};
use wry::http::Request;

#[derive(Clone)]
pub struct IpcHandler(Arc<IpcHandlerData>);

struct IpcHandlerData {
    messages: Mutex<VecDeque<Request<String>>>,
    semaphore_index: usize,
    semaphore_signaller: unsafe extern "C" fn(usize),
}

impl IpcHandler {
    pub fn enqueue(&self, request: Request<String>) {
        let mut lock = self.0.messages.lock().unwrap();
        lock.push_back(request);
        unsafe { (self.0.semaphore_signaller)(self.0.semaphore_index) };
    }

    pub fn pop(&self) -> Option<Request<String>> {
        let mut lock = self.0.messages.lock().unwrap();
        lock.pop_front()
    }
}

#[no_mangle]
pub extern "C" fn webview_ipc_handler_new(
    semaphore_index: usize,
    semaphore_signaller: unsafe extern "C" fn(usize),
) -> *mut ValueBox<IpcHandler> {
    ValueBox::new(IpcHandler(Arc::new(IpcHandlerData {
        messages: Default::default(),
        semaphore_index,
        semaphore_signaller,
    })))
    .into_raw()
}

#[no_mangle]
pub extern "C" fn webview_ipc_handler_pop(
    handler: *mut ValueBox<IpcHandler>,
) -> *mut ValueBox<Request<String>> {
    handler
        .with_ref_ok(|handler| {
            handler
                .pop()
                .map(|request| ValueBox::new(request).into_raw())
                .unwrap_or(std::ptr::null_mut())
        })
        .or_log(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn webview_ipc_handler_release(handler: *mut ValueBox<IpcHandler>) {
    handler.release();
}

#[no_mangle]
pub extern "C" fn webview_ipc_handler_request_get_body(
    request: *mut ValueBox<Request<String>>,
    body: *mut ValueBox<StringBox>,
) {
    request
        .with_ref(|request| body.with_mut_ok(|body| body.set_string(request.body().to_string())))
        .log();
}

#[no_mangle]
pub extern "C" fn webview_ipc_handler_request_release(request: *mut ValueBox<Request<String>>) {
    request.release();
}
