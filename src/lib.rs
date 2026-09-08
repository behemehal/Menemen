#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, deny(warnings))]
#![doc(html_root_url = "https://docs.rs/menemen/2.0.0")]

//!# Menemen
//!Menemen is a Turkish food and also simple streaming http/https blocking/async client.
//!
//!## Transport modes
//!
//!The `blocking` client is the standard mode and is enabled by default. The
//!`async` client is opt-in and currently replaces it rather than sitting
//!alongside it, so it needs `default-features = false`:
//!
//!```toml
//!menemen = { version = "2", default-features = false, features = ["async", "https", "json"] }
//!```
//!
//!## Usage
//!
//!```
//! # #[cfg(feature = "blocking")]
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use std::io::Read;
//! use menemen::request::{Request, RequestTypes};
//!
//! // Stream the response body
//! let mut request_stream = Request::new("http://postman-echo.com/get", RequestTypes::GET)?;
//! let mut response_stream = request_stream.send()?;
//!
//! let mut text_buffer = Vec::new();
//! response_stream.read_to_end(&mut text_buffer)?;
//! println!("Text: {}", String::from_utf8_lossy(&text_buffer));
//!
//! // or read the whole body as text
//! let mut request_text = Request::new("http://postman-echo.com/get", RequestTypes::GET)?;
//! let mut response_text = request_text.send()?;
//! println!("Response: {}", response_text.text()?);
//! # Ok(())
//! # }
//! ```
//!
//!Parsing a JSON body (requires the `json` feature):
//!
//!```
//! # #[cfg(all(feature = "blocking", feature = "json"))]
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use menemen::request::{Request, RequestTypes};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct Args {}
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct Root {
//!     pub args: Args,
//! }
//!
//! let mut request_json = Request::new("http://postman-echo.com/get", RequestTypes::GET)?;
//! let mut response_json_raw = request_json.send()?;
//! let response_json = response_json_raw.json::<Root>()?;
//!
//! println!("Response: {:?}", response_json.args);
//! # Ok(())
//! # }
//! ```
//! You can find more examples [here](https://github.com/behemehal/Menemen/tree/main/examples)

#[cfg(all(feature = "async", feature = "blocking"))]
compile_error!(
    "Menemen's 'async' and 'blocking' features are mutually exclusive. 'blocking' is the \
     default mode; to use the async client, set default-features = false and enable 'async'."
);

#[cfg(not(any(feature = "async", feature = "blocking")))]
compile_error!(
    "Menemen requires either the 'blocking' or 'async' feature to be enabled. \
     Please enable one in your Cargo.toml."
);

/// Various error types for Menemen
pub mod error;
/// Http FormData utilities
pub mod form_data;
/// A prelude for glob import
pub mod prelude;
/// Http Request utilities
pub mod request;
/// This module contains response structs and utilities
pub mod response;
/// This module contains url utilities
pub mod url;

macro_rules! cfg_imports {
    ($feature:literal, $($item:item)*) => {
        $(
            #[cfg(feature = $feature)]
            $item
        )*
    };
}

cfg_imports! {
    "async",
    mod async_lib;
    /// This module contains client utilities
    pub use async_lib::client;
    /// This module contains body utilities
    pub use async_lib::body;
    /// This module is abstracts the transport layer (ssl | tcp)
    pub use async_lib::transport;
    /// Http Multipart form utilities
    #[cfg(feature = "multipart")]
    pub use async_lib::multipart_form;
}

cfg_imports! {
    "blocking",
    /// Blocking implementation modules.
    pub mod blocking;
    /// This module contains client utilities
    pub use blocking::client;
    /// This module contains body utilities
    pub use blocking::body;
    /// This module is abstracts the transport layer (ssl | tcp)
    pub use blocking::transport;
    /// Http Multipart form utilities
    #[cfg(feature = "multipart")]
    pub use blocking::multipart_form;
}
