use std::any::type_name;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use string_box::StringBox;

use value_box::{ReturnBoxerResult, ValueBox, ValueBoxPointer};
use wry::http::Request;
use wry::PageLoadEvent;

#[derive(Clone)]
pub struct EventsHandler(Arc<EventsHandlerData>);

pub type WebViewId = u64;

struct EventsHandlerData {
    events: Mutex<VecDeque<WebViewEvent>>,
    semaphore_index: usize,
    semaphore_signaller: unsafe extern "C" fn(usize),
}

impl EventsHandler {
    pub fn enqueue_request(&self, webview_id: WebViewId, request: Request<String>) {
        self.enqueue_event(WebViewEvent::Request(WebViewRequestEvent {
            webview_id,
            request,
        }));
    }

    pub fn enqueue_navigation(&self, webview_id: WebViewId, url: String) {
        self.enqueue_event(WebViewEvent::Navigation(WebViewNavigationEvent {
            webview_id,
            url,
        }));
    }

    pub fn enqueue_page_load(&self, webview_id: WebViewId, page_event: PageLoadEvent, url: String) {
        self.enqueue_event(WebViewEvent::PageLoad(WebViewPageLoadEvent {
            webview_id,
            page_event,
            url,
        }));
    }

    pub fn enqueue_got_focus(&self, webview_id: WebViewId) {
        self.enqueue_event(WebViewEvent::GotFocus(WebViewGotFocusEvent { webview_id }));
    }

    pub fn enqueue_lost_focus(&self, webview_id: WebViewId) {
        self.enqueue_event(WebViewEvent::LostFocus(WebViewLostFocusEvent {
            webview_id,
        }));
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

#[derive(Debug)]
pub enum WebViewEvent {
    Request(WebViewRequestEvent),
    Navigation(WebViewNavigationEvent),
    PageLoad(WebViewPageLoadEvent),
    GotFocus(WebViewGotFocusEvent),
    LostFocus(WebViewLostFocusEvent),
}

pub struct WebViewRequestEvent {
    webview_id: u64,
    request: Request<String>,
}

impl Debug for WebViewRequestEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("webview_id", &self.webview_id)
            .field("request", self.request.body())
            .finish()
    }
}

pub struct WebViewNavigationEvent {
    webview_id: u64,
    url: String,
}

impl Debug for WebViewNavigationEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("webview_id", &self.webview_id)
            .field("url", &self.url)
            .finish()
    }
}

pub struct WebViewPageLoadEvent {
    webview_id: u64,
    page_event: PageLoadEvent,
    url: String,
}

impl Debug for WebViewPageLoadEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("webview_id", &self.webview_id)
            .field(
                "page_event",
                match &self.page_event {
                    PageLoadEvent::Started => &"Started",
                    PageLoadEvent::Finished => &"Finished",
                },
            )
            .field("url", &self.url)
            .finish()
    }
}

#[derive(Debug)]
pub struct WebViewGotFocusEvent {
    webview_id: u64,
}

#[derive(Debug)]
pub struct WebViewLostFocusEvent {
    webview_id: u64,
}

impl WebViewEvent {
    pub fn get_type(&self) -> WebViewEventType {
        match self {
            Self::Request(_) => WebViewEventType::Request,
            Self::Navigation(_) => WebViewEventType::Navigation,
            Self::PageLoad(_) => WebViewEventType::PageLoad,
            Self::GotFocus(_) => WebViewEventType::GotFocus,
            Self::LostFocus(_) => WebViewEventType::LostFocus,
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
    GotFocus,
    LostFocus,
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
            WebViewEvent::GotFocus(event) => ValueBox::new(event).into_raw() as *mut c_void,
            WebViewEvent::LostFocus(event) => ValueBox::new(event).into_raw() as *mut c_void,
        })
        .or_log(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn webview_navigation_event_get_url(
    event: *mut ValueBox<WebViewNavigationEvent>,
    url: *mut ValueBox<StringBox>,
) {
    event
        .with_ref(|event| url.with_mut_ok(|url| url.set_string(event.url.clone())))
        .log();
}

#[no_mangle]
pub extern "C" fn webview_navigation_event_get_id(
    event: *mut ValueBox<WebViewNavigationEvent>,
) -> WebViewId {
    event.with_ref_ok(|event| event.webview_id).or_log(0)
}

#[no_mangle]
pub extern "C" fn webview_request_event_get_content(
    event: *mut ValueBox<WebViewRequestEvent>,
    content: *mut ValueBox<StringBox>,
) {
    event
        .with_ref(|event| {
            content.with_mut_ok(|content| content.set_string(event.request.body().clone()))
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_request_event_get_id(
    event: *mut ValueBox<WebViewRequestEvent>,
) -> WebViewId {
    event.with_ref_ok(|event| event.webview_id).or_log(0)
}

#[no_mangle]
pub extern "C" fn webview_page_load_event_get_url(
    event: *mut ValueBox<WebViewPageLoadEvent>,
    url: *mut ValueBox<StringBox>,
) {
    event
        .with_ref(|event| url.with_mut_ok(|url| url.set_string(event.url.clone())))
        .log();
}

#[no_mangle]
pub extern "C" fn webview_page_load_event_get_id(
    event: *mut ValueBox<WebViewPageLoadEvent>,
) -> WebViewId {
    event.with_ref_ok(|event| event.webview_id).or_log(0)
}

#[no_mangle]
pub extern "C" fn webview_page_load_event_is_started(
    event: *mut ValueBox<WebViewPageLoadEvent>,
) -> u8 {
    event
        .with_ref_ok(|event| match event.page_event {
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
