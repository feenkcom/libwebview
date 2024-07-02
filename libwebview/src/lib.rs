mod attributes;
mod webview;

pub use value_box_ffi::*;

#[no_mangle]
pub extern "C" fn webview_test() -> bool {
    true
}

#[no_mangle]
pub extern "C" fn webview_init_logger() {
    env_logger::init();
}

#[no_mangle]
pub extern "C" fn webview_init_gtk() {
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    )))]
    {
        use value_box::ReturnBoxerResult;
        gtk::init()
            .map_err(|error| anyhow::anyhow!(error).into())
            .log();
    }
}
