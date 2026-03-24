use core::fmt;
use core::fmt::Debug;
use std::{
    collections::HashMap, fmt::Formatter, io::Read, ops::{Index, IndexMut}
};

/// URL-encoded form data container.
///
/// This type stores key/value pairs and can be converted into a
/// `application/x-www-form-urlencoded` payload.
pub struct FormData {
    /// Ordered key/value pairs in the form body.
    pub form_data: Vec<(String, String)>,
}

impl FormData {
    /// Creates an empty form-data collection.
    pub fn new() -> Self {
        Self {
            form_data: Vec::new(),
        }
    }

    /// Builds a form-data collection from a vector of pairs.
    pub fn from_map(map: Vec<(String, String)>) -> Self {
        Self { form_data: map }
    }

    /// Parses a URL-encoded form-data string.
    ///
    /// Invalid pairs (for example missing keys) are skipped.
    pub fn from_str(data: &str) -> Self {
        let mut form_data = Vec::new();
        if data.is_empty() {
            return Self { form_data };
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
            form_data.push((key.to_string(), value.to_string()));
        }
        Self { form_data }
    }

    /// Appends a new key/value pair.
    pub fn add(&mut self, key: &str, value: &str) {
        self.form_data.push((key.to_string(), value.to_string()));
    }

    /// Updates the first matching key or appends it if not present.
    pub fn set(&mut self, key: &str, value: &str) {
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
            .map(|(key, value)| format!("{}={}", key, value))
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
        Self { form_data: map }
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
        }
    }
}

impl From<HashMap<String, String>> for FormData {
    fn from(map: HashMap<String, String>) -> Self {
        Self {
            form_data: map.into_iter().collect(),
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
        self.build().as_bytes().read(buf)
    }
}