# Menemen

[![Crates.io Version](https://img.shields.io/crates/v/menemen?logo=rust)](https://crates.io/crates/menemen)
[![Documentation](https://docs.rs/menemen/badge.svg)](https://docs.rs/menemen)

Menemen is a Turkish food and also simple streaming http/https blocking/async client.

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("http://postman-echo.com/get", RequestTypes::GET)?;
    let mut response = request.send()?;
    let response_text = response.text().await?;
    
    println!("Text: {response_text}");
    Ok(())
}
```

## Examples

You can find examples [here](https://github.com/behemehal/Menemen/tree/main/examples)
