use crate::events_handler::{EventsHandler, WebViewId};
use anyhow::anyhow;
use std::borrow::Cow;
use std::str::FromStr;
use string_box::StringBox;
use value_box::{ReturnBoxerResult, ValueBox, ValueBoxPointer};
use wry::dpi::{LogicalPosition, Position, Size};
use wry::http::{HeaderMap, HeaderName, HeaderValue, Response, StatusCode};
use wry::{dpi, Rect, WebViewAttributes};

#[no_mangle]
pub extern "C" fn webview_attributes_default() -> *mut ValueBox<WebViewAttributes<'static>> {
    let mut attributes = WebViewAttributes::default();
    attributes.focused = false;
    ValueBox::new(attributes).into_raw()
}

#[no_mangle]
pub extern "C" fn webview_attributes_set_url(
    attributes: *mut ValueBox<WebViewAttributes<'static>>,
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
    attributes: *mut ValueBox<WebViewAttributes<'static>>,
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
pub extern "C" fn webview_attributes_add_header(
    attributes: *mut ValueBox<WebViewAttributes<'static>>,
    header_name: *mut ValueBox<StringBox>,
    header_value: *mut ValueBox<StringBox>,
) -> bool {
    attributes
        .with_mut(|attributes| {
            header_name.with_ref(|name| {
                header_value.with_ref(|value| {
                    HeaderName::from_str(name.as_str())
                        .map_err(|error| anyhow!(error).into())
                        .and_then(|header_name| {
                            HeaderValue::from_str(value.as_str())
                                .map_err(|error| anyhow!(error).into())
                                .map(|header_value| {
                                    if attributes.headers.is_none() {
                                        attributes.headers = Some(HeaderMap::new());
                                    }
                                    if let Some(headers) = attributes.headers.as_mut() {
                                        headers.insert(header_name, header_value);
                                    };
                                    true
                                })
                        })
                })
            })
        })
        .or_log(false)
}

#[no_mangle]
pub extern "C" fn webview_attributes_add_custom_protocol(
    attributes: *mut ValueBox<WebViewAttributes<'static>>,
    protocol_name: *mut ValueBox<StringBox>,
    content: *mut ValueBox<StringBox>,
) {
    attributes
        .with_mut(|attributes| {
            protocol_name.with_ref(|protocol_name| {
                content.with_ref_ok(|content| {
                    let content = Vec::from(content.as_bytes());
                    attributes.custom_protocols.insert(
                        protocol_name.to_string(),
                        Box::new(move |_webview_id, _request, responder| {
                            responder.respond(
                                Response::builder()
                                    .status(StatusCode::OK)
                                    .body(Cow::Owned(content.clone()))
                                    .unwrap(),
                            )
                        }),
                    );
                })
            })
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_attributes_set_events_handler(
    attributes: *mut ValueBox<WebViewAttributes<'static>>,
    events_handler: *mut ValueBox<EventsHandler>,
    webview_id: WebViewId,
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
                    handler_for_navigation.enqueue_navigation(webview_id, url);
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
    attributes: *mut ValueBox<WebViewAttributes<'static>>,
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
    attributes: *mut ValueBox<WebViewAttributes<'static>>,
    script: *mut ValueBox<StringBox>,
) {
    script
        .with_ref(|script| {
            attributes.with_mut_ok(|attributes| {
                attributes
                    .initialization_scripts
                    .push((script.to_string(), true))
            })
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_attributes_set_size(
    attributes: *mut ValueBox<WebViewAttributes<'static>>,
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
pub extern "C" fn webview_attributes_release(
    attributes: *mut ValueBox<WebViewAttributes<'static>>,
) {
    attributes.release();
}
