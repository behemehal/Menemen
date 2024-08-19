//#![deny(missing_docs)]
//#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, deny(warnings))]
#![doc(html_root_url = "https://docs.rs/menemen/1.0.3")]

//!# Menemen
//!Menemen is a Turkish food and also simple streaming http/https client.
//!
//!## Usage
//!
//!```
//! use std::io::{Write, Read};
//! use menemen::request::{Request, RequestTypes};
//!
//! fn main() {
//!     let mut request = Request::new("http://postman-echo.com/get", RequestTypes::GET).unwrap();
//!     let mut response = request.send().unwrap();
//!
//!     // Stream response
//!     let mut text_buffer = Vec::new();
//!     response.stream.read_to_end(&mut text_buffer).expect("TODO: panic message");
//!     println!("Text: {}", String::from_utf8_lossy(&text_buffer));
//!     //or
//!     let text = response.text().unwrap();
//!     println!("Text: {}", text);
//!
//!     //or
//!     let mut response = request.send().unwrap();
//!     let response_string = response.text().unwrap();
//!
//!     println!("Response: {}", response_string);
//!
//!     //or
//!     struct Root {
//!         pub args: Args
//!     }
//!
//!     let mut response = request.send().unwrap();
//!     let response_json = response.json::<Root>().unwrap();
//!     println!("Response: {:?}", response_json.args);
//! }
//! ```
//! You can find more examples [here](https://github.com/behemehal/Menemen/tree/main/examples)

/// Various error types for Menemen
pub mod error;
/// Http FormData utilities
pub mod form_data;
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
    pub use async_lib::multipart_form;
}

cfg_imports! {
    "blocking",
    pub mod blocking;
    /// This module contains client utilities
    pub use blocking::client;
    /// This module contains body utilities
    pub use blocking::body;
    /// This module is abstracts the transport layer (ssl | tcp)
    pub use blocking::transport;
    /// Http Multipart form utilities
    pub use blocking::multipart_form;
}
