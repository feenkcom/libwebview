use shared_library_builder::{GitLocation, LibraryLocation, RustLibrary};

pub fn libwebview(version: Option<impl Into<String>>) -> RustLibrary {
    RustLibrary::new(
        "WebView",
        LibraryLocation::Git(GitLocation::github("feenkcom", "libwebview").tag_or_latest(version)),
    )
    .package("libwebview")
}

pub fn latest_libwebview() -> RustLibrary {
    let version: Option<String> = None;
    libwebview(version)
}
