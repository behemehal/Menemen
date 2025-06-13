use std::{fs::File, io::Cursor};
use menemen::prelude::*;

/// This example demonstrates how to use the Into trait to convert different types into a Body.

fn main() {
    let string_body = "This is a test string.";

    // Using Into for &str
    let body_from_str: Body = string_body.into();

    // Using Into for String
    let string = String::from("Another test string.");
    let body_from_string: Body = string.into();

    // Using Into for Vec<u8>
    let data = vec![1, 2, 3, 4];
    let body_from_vec: Body = data.into();

    // Using Into for something that implements Read
    let cursor = Cursor::new(vec![5, 6, 7, 8]);
    let body_from_reader: Body = cursor.into();

    // Using Into for File
    let file = File::open("./testData/file.txt").unwrap();
    let body_from_file: Body = file.into();

    println!("Body from &str: {:?}", body_from_str);
    println!("Body from String: {:?}", body_from_string);
    println!("Body from Vec<u8>: {:?}", body_from_vec);
    println!("Body from Cursor<Vec<u8>>: {:?}", body_from_reader);
    println!("Body from File: {:?}", body_from_file);
}
