/// Blocking HTTP client implementation.
pub mod client;
/// Blocking request body utilities.
pub mod body;
/// Blocking transport abstraction over TCP and optional TLS.
pub mod transport;
/// Blocking multipart form-data support.
#[cfg(feature = "multipart")]
pub mod multipart_form;