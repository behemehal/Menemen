#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, deny(warnings))]
#![doc(html_root_url = "https://docs.rs/menemen/0.2.2-alpha")]

//!# Menemen
//!Menemen is a Turkish food and also simple streaming http client.
//!
//!## Usage
//!
//! ```
//! use std::io::{Write, Read};
//! use menemen::request::{Request, RequestTypes};
//!
//! fn main() {
//!    let mut request = Request::new("http://postman-echo.com/get", RequestTypes::GET).unwrap();
//!    let mut response = request.send().unwrap();
//!    let mut text_buffer = Vec::new();
//!    response.stream.read_to_end(&mut text_buffer);
//!    println!("Text: {}", String::from_utf8_lossy(&text_buffer));
//! }
//! ```
//! You can find more examples [here](https://github.com/behemehal/Menemen/tree/main/examples)

/// Various error types for Menemen
pub mod error;
/// Request module and http utilities 
pub mod request;
/// This module contains response structs and utilities enums
pub mod response;
/// This module contains ssl tcp stream
pub mod transport;
/// This module contains url utilities
pub mod url;
