# Menemen [Work In Progress]

[![Crates.io Version](https://img.shields.io/crates/v/menemen?logo=rust)](https://crates.io/crates/menemen)
[![Documentation](https://docs.rs/menemen/badge.svg)](https://docs.rs/menemen)

Menemen is a Turkish food also simple, streaming http client. Https is supported but not working like expected.

```rust
use std::io::{Write, Read};
use menemen::request::{Request, RequestTypes};

fn main() {
    let mut request = Request::new("http://postman-echo.com/get", RequestTypes::GET).unwrap();
    let mut response = request.send().unwrap();
    let mut text_buffer = Vec::new();
    response.stream.read_to_end(&mut text_buffer);
    println!("Text: {}", String::from_utf8_lossy(&text_buffer));
}
```

## Examples

You can find examples [here](./examples)
