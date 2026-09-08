# Menemen CLI

A curl-like HTTP client built on the Menemen blocking client.

## Build and Run

The CLI is behind the `cli` feature. Since `blocking` is the default mode, the
plain form works:

```bash
cargo run --features cli -- --help
```

To install it as a standalone binary:

```bash
cargo install --path . --features cli
```

The CLI is blocking-only. It cannot be built together with the `async` feature —
see [Transport modes](README.md#feature-matrix).

## Usage

```
menemen [OPTIONS] [URL] [COMMAND]
```

### Commands

| Command | Purpose |
|---|---|
| `request` | Make an HTTP request (this is the default, so it can be omitted) |
| `completions <SHELL>` | Print a completion script for `bash`, `zsh`, `fish`, `powershell` or `elvish` |
| `version` | Print the version |
| `help` | Print help |

### Options

| Flag | Argument | Purpose |
|---|---|---|
| `-X`, `--method` | `METHOD` | HTTP method: `GET`, `POST`, `PUT`, `DELETE`, `HEAD`, `PATCH`, `OPTIONS` (default `GET`) |
| `-H`, `--header` | `HEADER` | Add a header as `Name: value`. Repeatable |
| `--json` | `JSON` | Send a JSON body and set `Content-Type: application/json` |
| `-d`, `--data` | `DATA` | Send a raw body |
| `-F`, `--form` | `KEY=VALUE` | Send a URL-encoded form field. Repeatable |
| `--multipart` | `NAME=@PATH` | Multipart field. `NAME=@PATH` attaches a file, `NAME=VALUE` sends text. Repeatable |
| `-i`, `--include` | | Print response status and headers before the body |
| `-v`, `--verbose` | | Print request and response metadata to stderr |
| `-o`, `--output` | `FILE` | Write the body to a file instead of stdout |
| `--pretty-json` | | Pretty-print the body when the response is `application/json` |
| `--timeout` | `MS` | Request timeout in milliseconds (default `5000`) |
| `--no-follow` | | Return the redirect response instead of following it |
| `--color` | `WHEN` | `auto` (default), `always` or `never`. `auto` colorizes only when stdout is a terminal and `NO_COLOR` is unset |
| `-h`, `--help` | | Print help |
| `-V`, `--version` | | Print version |

`--json`, `--data`, `--form` and `--multipart` are mutually exclusive.

## Examples

Simple GET:

```bash
menemen http://example.com
```

Show response headers as well as the body:

```bash
menemen -i http://example.com
```

Send headers:

```bash
menemen -H "Authorization: Bearer token" -H "Accept: application/json" https://api.example.com
```

POST JSON, pretty-printed response:

```bash
menemen -X POST --json '{"key":"value"}' --pretty-json https://postman-echo.com/post
```

POST a URL-encoded form:

```bash
menemen -X POST -F key=value -F other=thing https://postman-echo.com/post
```

Save the body to a file:

```bash
menemen -o page.html http://example.com
```

Verbose mode — request/response metadata goes to stderr, so the body can still
be piped cleanly:

```bash
menemen -v http://example.com > body.html
```

Raise the timeout for a slow endpoint:

```bash
menemen --timeout 30000 http://slow.example.com
```

Upload a file and a text field together:

```bash
menemen -X POST --multipart "doc=@./testData/file.txt" --multipart "note=hello" https://postman-echo.com/post
```

Inspect a redirect without following it:

```bash
menemen -i --no-follow "http://postman-echo.com/redirect-to?url=http%3A%2F%2Fexample.com"
```

Install completions (bash):

```bash
menemen completions bash > /etc/bash_completion.d/menemen
```

PowerShell:

```powershell
menemen completions powershell | Out-String | Invoke-Expression
```

## Notes

- Malformed `-H` and `-F` values are reported on stderr and skipped rather than
  aborting the request.
- Redirects are followed automatically for `302`, `303`, `307` and `308`, up to a
  redirect limit. Use `--no-follow` to disable this.
- `--pretty-json` only reformats when the response `Content-Type` contains
  `application/json`; otherwise the body is passed through unchanged.
