use string_box::StringBox;
use value_box::{ReturnBoxerResult, ValueBox, ValueBoxPointer};
use wry::dpi::{LogicalPosition, Position, Size};
use wry::{dpi, Rect, WebViewAttributes};

#[no_mangle]
pub extern "C" fn webview_attributes_default() -> *mut ValueBox<WebViewAttributes> {
    ValueBox::new(WebViewAttributes::default()).into_raw()
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
