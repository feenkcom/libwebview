use string_box::StringBox;
use value_box::{ReturnBoxerResult, ValueBox, ValueBoxPointer};
use wry::dpi::{LogicalPosition, Position, Size};
use wry::{dpi, Rect, WebViewAttributes};

use crate::events_handler::{EventsHandler, WebViewId};

#[no_mangle]
pub extern "C" fn webview_attributes_default() -> *mut ValueBox<WebViewAttributes> {
    let mut attributes = WebViewAttributes::default();
    attributes.focused = false;
    ValueBox::new(attributes).into_raw()
}

#[no_mangle]
pub extern "C" fn webview_attributes_set_url(
    attributes: *mut ValueBox<WebViewAttributes>,
    url: *mut ValueBox<StringBox>,
) {
    attributes
        .with_mut(|attributes| {
            url.with_ref_ok(|url| {
                if url.len() > 0 {
                    attributes.url = Some(url.to_string());
                } else {
                    attributes.url = None;
                }
            })
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_attributes_set_html(
    attributes: *mut ValueBox<WebViewAttributes>,
    html: *mut ValueBox<StringBox>,
) {
    attributes
        .with_mut(|attributes| {
            html.with_ref_ok(|html| {
                attributes.html = Some(html.to_string());
            })
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_attributes_set_events_handler(
    attributes: *mut ValueBox<WebViewAttributes>,
    events_handler: *mut ValueBox<EventsHandler>,
    webview_id: WebViewId
) {
    attributes
        .with_mut(|attributes| {
            events_handler.with_clone_ok(|event_handler| {
                let handler_for_ipc = event_handler.clone();
                attributes.ipc_handler = Some(Box::new(move |request| {
                    handler_for_ipc.enqueue_request(webview_id, request);
                }));
                let handler_for_navigation = event_handler.clone();
                attributes.navigation_handler = Some(Box::new(move |url| {
                    handler_for_navigation.enqueue_navigation(webview_id,url);
                    true
                }));
                let handler_for_loading = event_handler.clone();
                attributes.on_page_load_handler = Some(Box::new(move |event, url| {
                    handler_for_loading.enqueue_page_load(webview_id, event, url);
                }))
            })
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_attributes_set_position(
    attributes: *mut ValueBox<WebViewAttributes>,
    x: f64,
    y: f64,
) {
    let new_position: Position = LogicalPosition::new(x, y).into();

    attributes
        .with_mut_ok(|attributes| {
            attributes.bounds = attributes
                .bounds
                .map(|mut bounds| {
                    bounds.position = new_position;
                    bounds
                })
                .or_else(|| {
                    Some(Rect {
                        position: new_position,
                        size: dpi::LogicalSize::new(200, 200).into(),
                    })
                })
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_attributes_add_initial_script(
    attributes: *mut ValueBox<WebViewAttributes>,
    script: *mut ValueBox<StringBox>,
) {
    script
        .with_ref(|script| {
            attributes.with_mut_ok(|attributes| {
                attributes.initialization_scripts.push(script.to_string())
            })
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_attributes_set_size(
    attributes: *mut ValueBox<WebViewAttributes>,
    width: f64,
    height: f64,
) {
    let new_size: Size = dpi::LogicalSize::new(width, height).into();

    attributes
        .with_mut_ok(|attributes| {
            attributes.bounds = attributes
                .bounds
                .map(|mut bounds| {
                    bounds.size = new_size;
                    bounds
                })
                .or_else(|| {
                    Some(Rect {
                        position: LogicalPosition::new(0.0, 0.0).into(),
                        size: new_size,
                    })
                })
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_attributes_release(attributes: *mut ValueBox<WebViewAttributes>) {
    attributes.release();
}
