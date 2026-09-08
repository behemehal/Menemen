<div align="center">

<img src="./image.png" width="150" height="150" alt="Menemen Logo">

# Menemen

**A lightweight streaming HTTP client for Rust — blocking by default, async when you want it.**

[![Crates.io Version](https://img.shields.io/crates/v/menemen?logo=rust)](https://crates.io/crates/menemen)
[![Documentation](https://docs.rs/menemen/badge.svg)](https://docs.rs/menemen)
[![License](https://img.shields.io/crates/l/menemen)](./LICENSE)

</div>

---

Menemen is a Turkish breakfast dish, and also a small HTTP/HTTPS client built
around three ideas:

- **Streaming first** — a response *is* a `Read` (or `AsyncRead`), so you can
  process a huge download without holding it in memory.
- **Blocking by default** — no runtime, no executor, no `async` keyword unless
  you ask for one.
- **Pay for what you use** — TLS, JSON, multipart and gzip are all opt-in
  feature flags.

It also ships an optional curl-like CLI. See **[CLI.md](CLI.md)**.

## Contents

- [Installation](#installation)
- [Feature Matrix](#feature-matrix)
- [Quick Start](#quick-start)
- [Streaming](#streaming)
- [JSON](#json)
- [Forms and Multipart](#forms-and-multipart)
- [Long Polling](#long-polling)
- [Examples](#examples)
- [CLI](#cli)
- [Development](#development)
- [Notes](#notes)

## Installation

```toml
[dependencies]
menemen = "2"
```

That gives you the blocking client with JSON and multipart support. Add `https`
for TLS:

```toml
menemen = { version = "2", features = ["https"] }
```

### Async

`async` *replaces* the blocking client rather than sitting beside it, so it
needs `default-features = false`:

```toml
[dependencies]
menemen = { version = "2", default-features = false, features = ["async", "https", "json", "multipart"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Enabling both `blocking` and `async` is a compile error with a message telling
you what to do about it.

## Feature Matrix

| Feature | Default | Purpose |
|---|:---:|---|
| `blocking` | yes | Blocking client. The standard mode |
| `json` | yes | Adds `response.json::<T>()` |
| `multipart` | yes | Multipart form-data bodies |
| `async` | | Tokio-based async client. Requires `default-features = false` |
| `https` | | TLS support |
| `gzip` | | gzip decompression in `response.text()` |
| `cli` | | Builds the `menemen` command-line binary |
| `danger-transport-no-tls` | | Allow `https://` URLs without a TLS handshake. Not recommended |

## Quick Start

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

The async version is the same code with `.await` in two places:

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

## Streaming

`Response` implements `Read` in blocking mode and `AsyncRead` in async mode, and
decodes `Transfer-Encoding: chunked` transparently. Prefer this over `text()`
for anything large:

```rust
use menemen::request::{Request, RequestTypes};
use std::fs::File;
use std::io::copy;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("http://example.com/big.iso", RequestTypes::GET)?;
    let mut response = request.send()?;

    // Streams straight to disk; memory use stays flat.
    let mut file = File::create("big.iso")?;
    copy(&mut response, &mut file)?;
    Ok(())
}
```

## JSON

Requires the `json` feature (on by default).

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

## Forms and Multipart

URL-encoded form:

```rust
use menemen::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut form = FormData::new();
    form.add("key", "value");

    let mut request = Request::new("http://postman-echo.com/post", RequestTypes::POST)?;
    request.append_body(form.into());

    println!("{}", request.send()?.text()?);
    Ok(())
}
```

Multipart upload — the `Content-Type` header and boundary are filled in for you:

```rust
use menemen::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut multipart = MultipartFormData::new();
    multipart.add_file("document", "./testData/file.txt")?;
    multipart.add_string("note", "hello".to_string());

    let mut request = Request::new("http://postman-echo.com/post", RequestTypes::POST)?;
    request.append_body(multipart.into());

    println!("{}", request.send()?.text()?);
    Ok(())
}
```

```bash
cargo run --example multipart_form_data
```

## Long Polling

A WebSocket alternative when you only need server-to-client updates. The repo
ships both halves of a demo:

```bash
# terminal 1
node examples/long_poll_server.js

# terminal 2
cargo run --example long_polling
```

Point it somewhere else with `LONG_POLL_URL`:

```powershell
$env:LONG_POLL_URL="http://localhost:3000/long-poll"
cargo run --example long_polling
```

## Examples

Every example runs in both blocking and async mode.

| Example | What it shows |
|---|---|
| `echo` | Smallest possible GET |
| `post` | POST with a body |
| `body` | Body construction options |
| `parse_url` | URL parser in isolation |
| `form_data` | URL-encoded forms |
| `multipart_form_data` | File + text upload |
| `json` | Typed JSON deserialization (needs `async`) |
| `gzip` | gzip-compressed responses |
| `chunked` | `Transfer-Encoding: chunked` decoding |
| `download_file` | Streaming download with a progress bar |
| `download_file_async` | Same, async (needs `async`) |
| `long_polling` | Long-poll loop with backoff |

```bash
cargo run --example echo
```

Two examples expect a local demo server, and report that clearly instead of
hanging if it is not running:

```bash
node examples/chunked_server.js     # then: cargo run --example chunked
node examples/long_poll_server.js   # then: cargo run --example long_polling
```

Full listing: <https://github.com/behemehal/Menemen/tree/main/examples>

## CLI

```bash
cargo run --features cli -- --help
```

Install it as a standalone binary:

```bash
cargo install --path . --features cli
```

```bash
menemen -i https://example.com
menemen -X POST --json '{"key":"value"}' --pretty-json https://postman-echo.com/post
menemen -X POST --multipart "doc=@./testData/file.txt" https://postman-echo.com/post
```

Full documentation: **[CLI.md](CLI.md)**.

## Development

```bash
cargo build
cargo test
```

Because `blocking` and `async` are mutually exclusive there is no single
all-features build. Test both modes:

```bash
cargo test
cargo test --no-default-features --features async,https,json,multipart,gzip
```

The chunked-decoder tests spin up an in-process TCP server, so the suite needs
no network access.

## Notes

- Redirects are followed automatically for `302`, `303`, `307` and `308`, up to
  a redirect limit. Disable with `request.set_follow_redirects(false)`.
- Timeouts are set per request with `request.set_timeout(ms)`.
- For large payloads prefer streaming (see [Streaming](#streaming)) over
  collecting the whole body with `text()`.
- Fallible operations return [`RequestError`](https://docs.rs/menemen/latest/menemen/error/enum.RequestError.html),
  which implements `std::error::Error`.

## License

GPL-2.0. See [LICENSE](./LICENSE).
