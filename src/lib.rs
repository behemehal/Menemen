#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, deny(warnings))]
#![doc(html_root_url = "https://docs.rs/menemen/2.0.0")]

//!# Menemen
//!Menemen is a Turkish food and also simple streaming http/https blocking/async client.
//!
//!## Usage
//!
//!```
//! use std::io::{Write, Read};
//! use menemen::request::{Request, RequestTypes};
//! use serde::{Deserialize, Serialize};
//!
//! fn main() {
//!     let mut request_stream = Request::new("http://postman-echo.com/get", RequestTypes::GET).unwrap();
//!     let mut response_stream = request_stream.send().unwrap();
//!
//!     // Stream response
//!     let mut text_buffer = Vec::new();
//!     response_stream.stream.read_to_end(&mut text_buffer).expect("read failed");
//!     println!("Text: {}", String::from_utf8_lossy(&text_buffer));
//!
//!     // or read response as text
//!     let mut request_text = Request::new("http://postman-echo.com/get", RequestTypes::GET).unwrap();
//!     let mut response_text = request_text.send().unwrap();
//!     let response_string = response_text.text().unwrap();
//!
//!     println!("Response: {}", response_string);
//!
//!     // or read response as json
//!     let mut request_json = Request::new("http://postman-echo.com/get", RequestTypes::GET).unwrap();
//!     let mut response_json_raw = request_json.send().unwrap();
//!
//!     #[derive(Serialize, Deserialize, Debug)]
//!     struct Args {}
//!
//!     #[derive(Serialize, Deserialize, Debug)]
//!     struct Root {
//!         pub args: Args
//!     }
//!     let response_json = response_json_raw.json::<Root>().unwrap();
//!     println!("Response: {:?}", response_json.args);
//! }
//! ```
//! You can find more examples [here](https://github.com/behemehal/Menemen/tree/main/examples)

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

macro_rules! cfg_not_imports {
    ($feature:literal, $($item:item)*) => {
        $(
            #[cfg(not(feature = $feature))]
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

cfg_not_imports! {
    "async",
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
