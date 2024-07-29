use std::collections::VecDeque;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use string_box::StringBox;

use value_box::{ReturnBoxerResult, ValueBox, ValueBoxPointer};
use wry::http::Request;
use wry::PageLoadEvent;

#[derive(Clone)]
pub struct EventsHandler(Arc<EventsHandlerData>);

struct EventsHandlerData {
    events: Mutex<VecDeque<WebViewEvent>>,
    semaphore_index: usize,
    semaphore_signaller: unsafe extern "C" fn(usize),
}

impl EventsHandler {
    pub fn enqueue_request(&self, request: Request<String>) {
        self.enqueue_event(WebViewEvent::Request(WebViewRequestEvent(request)));
    }

    pub fn enqueue_navigation(&self, url: String) {
        self.enqueue_event(WebViewEvent::Navigation(WebViewNavigationEvent(url)));
    }

    pub fn enqueue_page_load(&self, event: PageLoadEvent, url: String) {
        self.enqueue_event(WebViewEvent::PageLoad(WebViewPageLoadEvent(event, url)));
    }

    fn enqueue_event(&self, event: WebViewEvent) {
        let mut lock = self.0.events.lock().unwrap();
        lock.push_back(event);
        unsafe { (self.0.semaphore_signaller)(self.0.semaphore_index) };
    }

    pub fn pop_event(&self) -> Option<WebViewEvent> {
        let mut lock = self.0.events.lock().unwrap();
        lock.pop_front()
    }
}

pub enum WebViewEvent {
    Request(WebViewRequestEvent),
    Navigation(WebViewNavigationEvent),
    PageLoad(WebViewPageLoadEvent),
}

pub struct WebViewRequestEvent(Request<String>);
pub struct WebViewNavigationEvent(String);
pub struct WebViewPageLoadEvent(PageLoadEvent, String);

impl WebViewEvent {
    pub fn get_type(&self) -> WebViewEventType {
        match self {
            Self::Request(_) => WebViewEventType::Request,
            Self::Navigation(_) => WebViewEventType::Navigation,
            Self::PageLoad(_) => WebViewEventType::PageLoad,
        }
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum WebViewEventType {
    Unknown,
    Request,
    Navigation,
    PageLoad,
}

#[no_mangle]
pub extern "C" fn webview_events_handler_new(
    semaphore_index: usize,
    semaphore_signaller: unsafe extern "C" fn(usize),
) -> *mut ValueBox<EventsHandler> {
    ValueBox::new(EventsHandler(Arc::new(EventsHandlerData {
        events: Default::default(),
        semaphore_index,
        semaphore_signaller,
    })))
    .into_raw()
}

#[no_mangle]
pub extern "C" fn webview_events_handler_pop_event(
    handler: *mut ValueBox<EventsHandler>,
) -> *mut ValueBox<WebViewEvent> {
    handler
        .with_ref_ok(|handler| {
            handler
                .pop_event()
                .map(|event| ValueBox::new(event).into_raw())
                .unwrap_or(std::ptr::null_mut())
        })
        .or_log(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn webview_events_handler_release(handler: *mut ValueBox<EventsHandler>) {
    handler.release();
}

#[no_mangle]
pub extern "C" fn webview_event_get_type(event: *mut ValueBox<WebViewEvent>) -> WebViewEventType {
    event
        .with_ref_ok(|event| event.get_type())
        .or_log(WebViewEventType::Unknown)
}

#[no_mangle]
pub extern "C" fn webview_event_into_inner(event: *mut ValueBox<WebViewEvent>) -> *mut c_void {
    event
        .take_value()
        .map(|value| match value {
            WebViewEvent::Request(event) => ValueBox::new(event).into_raw() as *mut c_void,
            WebViewEvent::Navigation(event) => ValueBox::new(event).into_raw() as *mut c_void,
            WebViewEvent::PageLoad(event) => ValueBox::new(event).into_raw() as *mut c_void,
        })
        .or_log(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn webview_navigation_event_get_url(
    event: *mut ValueBox<WebViewNavigationEvent>,
    url: *mut ValueBox<StringBox>,
) {
    event
        .with_ref(|event| url.with_mut_ok(|url| url.set_string(event.0.clone())))
        .log();
}

#[no_mangle]
pub extern "C" fn webview_request_event_get_content(
    event: *mut ValueBox<WebViewRequestEvent>,
    content: *mut ValueBox<StringBox>,
) {
    event
        .with_ref(|event| content.with_mut_ok(|content| content.set_string(event.0.body().clone())))
        .log();
}

#[no_mangle]
pub extern "C" fn webview_page_load_event_get_url(
    event: *mut ValueBox<WebViewPageLoadEvent>,
    url: *mut ValueBox<StringBox>,
) {
    event
        .with_ref(|event| url.with_mut_ok(|url| url.set_string(event.1.clone())))
        .log();
}

#[no_mangle]
pub extern "C" fn webview_page_load_event_is_started(
    event: *mut ValueBox<WebViewPageLoadEvent>,
) -> u8 {
    event
        .with_ref_ok(|event| match event.0 {
            PageLoadEvent::Started => 1,
            PageLoadEvent::Finished => 2,
        })
        .or_log(0)
}

#[no_mangle]
pub extern "C" fn webview_event_release(event: *mut ValueBox<WebViewEvent>) {
    event.release();
}

#[no_mangle]
pub extern "C" fn webview_navigation_event_release(event: *mut ValueBox<WebViewNavigationEvent>) {
    event.release();
}

#[no_mangle]
pub extern "C" fn webview_page_load_event_release(event: *mut ValueBox<WebViewPageLoadEvent>) {
    event.release();
}

#[no_mangle]
pub extern "C" fn webview_request_event_release(event: *mut ValueBox<WebViewRequestEvent>) {
    event.release();
}
