#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, deny(warnings))]
#![doc(html_root_url = "https://docs.rs/menemen/0.2.0-alpha")]
//!# Menemen
///
///Menemen is a Turkish food also simple, streaming http client.
///
///## Usage
///
///```
///use menemen::{Menemen, MenemenError};
///
///
///```
/// This module contains error enums
pub mod error;
/// This module contains request utilities
pub mod request;
/// This module contains response structs and utilities enums
pub mod response;
/// This module contains ssl tcp stream
pub mod transport;
/// This module contains url utilities
pub mod url;
