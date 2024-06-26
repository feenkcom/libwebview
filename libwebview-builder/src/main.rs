use std::error::Error;

use libwebview_builder::latest_libwebview;
use shared_library_builder::build_standalone;

fn main() -> Result<(), Box<dyn Error>> {
    build_standalone(|_| Ok(Box::new(latest_libwebview())))
}
