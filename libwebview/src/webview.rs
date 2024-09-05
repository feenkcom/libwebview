use crate::events_handler::{EventsHandler, WebViewId};
use anyhow::anyhow;
use raw_window_handle_extensions::VeryRawWindowHandle;
use string_box::StringBox;
use value_box::{ReturnBoxerResult, ValueBox, ValueBoxIntoRaw, ValueBoxPointer};
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::raw_window_handle::{RawWindowHandle, WindowHandle};
use wry::{Rect, WebView, WebViewAttributes, WebViewBuilder, WebViewExtWindows};

use crate::script::ScriptToEvaluate;

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

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    )))]
    let fixed = {
        use gtk::prelude::*;

        let fixed = gtk::Fixed::new();
        fixed.show_all();
        fixed
    };

    let create_webview_builder = || {
        #[cfg(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        ))]
        return WebViewBuilder::new_as_child(&window_handle);

        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        )))]
        {
            use wry::WebViewBuilderExtUnix;
            WebViewBuilder::new_gtk(&fixed)
        }
    };

    let mut builder = create_webview_builder();

    attributes.take_value().map(|mut attributes| {
        attributes.devtools = true;
        builder.attrs = attributes
    })?;

    let webview = builder.build().map_err(|error| anyhow!(error))?;
    Ok(webview)
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
pub extern "C" fn webview_evaluate_script_with_result(
    webview: *mut ValueBox<WebView>,
    script: *mut ValueBox<ScriptToEvaluate>,
) {
    webview
        .with_ref(|webview| {
            script.with_ref(|script| {
                let script_clone = script.clone();
                webview
                    .evaluate_script_with_callback(script.script(), move |value| {
                        script_clone.set_value(value);
                    })
                    .inspect_err(|error| {
                        script.set_error(error.to_string());
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
pub extern "C" fn webview_set_event_handler(
    webview: *mut ValueBox<WebView>,
    events_handler: *mut ValueBox<EventsHandler>,
    webview_id: WebViewId,
) {
    #[cfg(target_os = "windows")]
    {
        webview
            .with_ref(|webview| {
                events_handler.with_ref_ok(|events_handler| {
                    use webview2_com::FocusChangedEventHandler;
                    use windows::Win32::System::WinRT::EventRegistrationToken;
                    use wry::WebViewExtWindows;

                    let mut token = EventRegistrationToken::default();

                    let got_focus_handler = events_handler.clone();
                    let got_focus_callback = FocusChangedEventHandler::create(Box::new(|_, _| {
                        got_focus_handler.enqueue_got_focus(webview_id);
                        Ok(())
                    }));

                    let lost_focus_handler = events_handler.clone();
                    let lost_focus_callback = FocusChangedEventHandler::create(Box::new(|_, _| {
                        lost_focus_handler.enqueue_lost_focus(webview_id);
                        Ok(())
                    }));

                    unsafe {
                        webview
                            .controller()
                            .add_GotFocus(&got_focus_callback, &mut token)
                            .expect("Set GotFocus handler");
                    };

                    unsafe {
                        webview
                            .controller()
                            .add_LostFocus(&lost_focus_callback, &mut token)
                            .expect("Set LostFocus handler");
                    }
                })
            })
            .log();
    }
}

#[no_mangle]
pub extern "C" fn webview_toggle_visible(webview: *mut ValueBox<WebView>) {
    webview
        .with_ref_ok(|webview| {
            let _ = webview.set_visible(false);
            let _ = webview.set_visible(true);
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_load_url(webview: *mut ValueBox<WebView>, url: *mut ValueBox<StringBox>) {
    webview
        .with_ref(|webview| {
            url.with_ref_ok(|url| {
                let _ = webview.load_url(url.as_str());
            })
        })
        .log();
}

#[no_mangle]
pub extern "C" fn webview_release(webview: *mut ValueBox<WebView>) {
    webview.release();
}
