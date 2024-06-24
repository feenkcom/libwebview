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
