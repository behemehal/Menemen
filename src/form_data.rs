use core::fmt;
use core::fmt::Debug;
use std::{
    collections::HashMap,
    fmt::Formatter,
    io::{Cursor, Read},
    ops::{Index, IndexMut},
};

/// URL-encoded form data container.
///
/// This type stores key/value pairs and can be converted into a
/// `application/x-www-form-urlencoded` payload.
pub struct FormData {
    /// Ordered key/value pairs in the form body.
    pub form_data: Vec<(String, String)>,
    /// Payload snapshot the `Read` impl streams from, encoded on first read.
    /// Encoding per call would be quadratic in the payload size, and without
    /// any cursor the impl never reports EOF at all.
    read_buffer: Option<Cursor<Vec<u8>>>,
}

const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// Percent-encodes one key or value for an `application/x-www-form-urlencoded`
/// payload: unreserved characters pass through, a space becomes `+`, and every
/// other byte becomes an uppercase `%XX` escape.
fn encode_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());

    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*byte as char)
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push('%');
                encoded.push(HEX_DIGITS[(byte >> 4) as usize] as char);
                encoded.push(HEX_DIGITS[(byte & 0x0f) as usize] as char);
            }
        }
    }

    encoded
}

/// Reverses [`encode_component`]. Invalid escapes are kept verbatim rather than
/// dropped, so parsing never loses data.
fn decode_component(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let high = (bytes[index + 1] as char).to_digit(16);
                let low = (bytes[index + 2] as char).to_digit(16);
                match (high, low) {
                    (Some(high), Some(low)) => {
                        decoded.push((high * 16 + low) as u8);
                        index += 3;
                    }
                    _ => {
                        decoded.push(bytes[index]);
                        index += 1;
                    }
                }
            }
            other => {
                decoded.push(other);
                index += 1;
            }
        }
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

impl FormData {
    /// Creates an empty form-data collection.
    pub fn new() -> Self {
        Self {
            form_data: Vec::new(),
            read_buffer: None,
        }
    }

    /// Builds a form-data collection from a vector of pairs.
    pub fn from_map(map: Vec<(String, String)>) -> Self {
        Self {
            form_data: map,
            read_buffer: None,
        }
    }

    /// Parses a URL-encoded form-data string, percent-decoding keys and values.
    ///
    /// Invalid pairs (for example missing keys) are skipped.
    pub fn from_str(data: &str) -> Self {
        let mut form_data = Vec::new();
        if data.is_empty() {
            return Self {
                form_data,
                read_buffer: None,
            };
        }

        for pair in data.split('&') {
            if pair.is_empty() {
                continue;
            }

            let mut split = pair.splitn(2, '=');
            let key = split.next().unwrap_or("");
            if key.is_empty() {
                continue;
            }
            let value = split.next().unwrap_or("");
            form_data.push((decode_component(key), decode_component(value)));
        }
        Self {
            form_data,
            read_buffer: None,
        }
    }

    /// Appends a new key/value pair.
    pub fn add(&mut self, key: &str, value: &str) {
        self.read_buffer = None;
        self.form_data.push((key.to_string(), value.to_string()));
    }

    /// Updates the first matching key or appends it if not present.
    pub fn set(&mut self, key: &str, value: &str) {
        self.read_buffer = None;
        let mut found = false;
        for pair in self.form_data.iter_mut() {
            if pair.0 == key {
                pair.1 = value.to_string();
                found = true;
                break;
            }
        }
        if !found {
            self.add(key, value);
        }
    }

    /// Removes all pairs matching `key`.
    pub fn remove(&mut self, key: &str) {
        self.read_buffer = None;
        self.form_data.retain(|pair| pair.0 != key);
    }

    /// Returns the first value for `key`, if present.
    pub fn get(&self, key: &str) -> Option<&String> {
        for pair in self.form_data.iter() {
            if pair.0 == key {
                return Some(&pair.1);
            }
        }
        None
    }

    pub(crate) fn build(&self) -> String {
        self.form_data
            .iter()
            .map(|(key, value)| {
                format!("{}={}", encode_component(key), encode_component(value))
            })
            .collect::<Vec<String>>()
            .join("&")
    }
}

impl Index<usize> for FormData {
    type Output = (String, String);

    fn index(&self, index: usize) -> &Self::Output {
        &self.form_data[index]
    }
}

impl IndexMut<usize> for FormData {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.form_data[index]
    }
}

impl From<Vec<(String, String)>> for FormData {
    fn from(map: Vec<(String, String)>) -> Self {
        Self {
            form_data: map,
            read_buffer: None,
        }
    }
}

impl Into<Vec<(String, String)>> for FormData {
    fn into(self) -> Vec<(String, String)> {
        self.form_data
    }
}

impl From<Vec<(&str, &str)>> for FormData {
    fn from(map: Vec<(&str, &str)>) -> Self {
        Self {
            form_data: map
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            read_buffer: None,
        }
    }
}

impl From<HashMap<String, String>> for FormData {
    fn from(map: HashMap<String, String>) -> Self {
        Self {
            form_data: map.into_iter().collect(),
            read_buffer: None,
        }
    }
}

impl Into<HashMap<String, String>> for FormData {
    fn into(self) -> HashMap<String, String> {
        self.form_data.into_iter().collect()
    }
}

impl Debug for FormData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        format!("{}", self.build()).fmt(f)
    }
}

impl Read for FormData {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        if self.read_buffer.is_none() {
            let payload = self.build().into_bytes();
            self.read_buffer = Some(Cursor::new(payload));
        }

        match self.read_buffer.as_mut() {
            Some(cursor) => cursor.read(buf),
            None => Ok(0),
        }
    }
}