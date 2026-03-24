<image src="./image.png" width="150" height="150" alt="Menemen Logo">

# Menemen

[![Crates.io Version](https://img.shields.io/crates/v/menemen?logo=rust)](https://crates.io/crates/menemen)
[![Documentation](https://docs.rs/menemen/badge.svg)](https://docs.rs/menemen)

Menemen is a lightweight streaming HTTP client for Rust with both blocking and async modes.

Menemen also includes an optional curl-like CLI binary.
See [CLI.md](CLI.md) for CLI usage and examples.

It is designed around:

- simple request creation
- streaming response reads
- feature-flag-driven transport capabilities
- optional JSON, multipart, and gzip support

## Why Menemen

- Unified API style for blocking and async workflows
- Stream-friendly response type (`Read` in blocking mode, `AsyncRead` in async mode)
- Lightweight feature flags so consumers only enable what they need

## Feature Matrix

| Feature | Purpose |
|---|---|
| `blocking` (default) | Enables blocking client implementation |
| `async` | Enables tokio-based async client implementation |
| `https` | Enables TLS support |
| `json` (default) | Adds `response.json::<T>()` |
| `multipart` (default) | Adds multipart form support |
| `gzip` | Enables gzip decompression in `response.text()` |
| `danger-transport-no-tls` | Allows HTTPS URLs without TLS handshake (not recommended) |

## Installation

Blocking client (default features):

```toml
[dependencies]
menemen = "2"
```

Async + TLS + JSON example:

```toml
[dependencies]
menemen = { version = "2", default-features = false, features = ["async", "https", "json", "multipart"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

Blocking request:

```rust
use menemen::request::{Request, RequestTypes};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("http://postman-echo.com/get", RequestTypes::GET)?;
    let mut response = request.send()?;

    println!("Status: {}", response.response_info.status_code);
    println!("Body: {}", response.text()?);
    Ok(())
}
```

Async request:

```rust
use menemen::request::{Request, RequestTypes};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("http://postman-echo.com/get", RequestTypes::GET)?;
    let mut response = request.send().await?;

    println!("Status: {}", response.response_info.status_code);
    println!("Body: {}", response.text().await?);
    Ok(())
}
```

## JSON Parsing

```rust
use menemen::request::{Request, RequestTypes};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct EchoResponse {
    url: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("http://postman-echo.com/get", RequestTypes::GET)?;
    let mut response = request.send()?;

    let parsed: EchoResponse = response.json()?;
    println!("Echo URL: {}", parsed.url);
    Ok(())
}
```

## Multipart Upload

Multipart examples are available in the examples directory and demonstrate file + text uploads.

Run:

```bash
cargo run --example multipart_form_data
```

## Long Polling (WebSocket Alternative)

This repository includes a long-polling client/server demo:

- `examples/long_polling.rs`
- `examples/long_poll_server.js`

Run server:

```bash
node examples/long_poll_server.js
```

Run Rust long-poll client:

```bash
cargo run --example long_polling
```

Override endpoint:

PowerShell:

```powershell
$env:LONG_POLL_URL="http://localhost:3000/long-poll"
cargo run --example long_polling
```

## Examples

You can find all examples in this repository:

https://github.com/behemehal/Menemen/tree/main/examples

## CLI

Enable the CLI feature and run:

```bash
cargo run --features cli -- --help
```

Full CLI documentation is available in [CLI.md](CLI.md).

## Development

Build:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Build with all features:

```bash
cargo build --all-features
```

## Notes

- Redirects are followed automatically for `302`, `303`, `307`, `308` with a redirect limit.
- Timeout configuration is supported via `request.set_timeout(ms)`.
- For large payload scenarios, prefer stream processing over collecting entire bodies in memory.

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("http://postman-echo.com/get", RequestTypes::GET)?;
    let mut response = request.send().await?;
    let response_text = response.text().await?;
    
    println!("Text: {response_text}");
    Ok(())
}
```
