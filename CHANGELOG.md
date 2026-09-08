# Changelog

## v1.0.3
- Implement first version

## v2.0.0
- Implement async runtime
- Separate blocking and async client
- Add multipart & form data support
- Add built-in compression support
- Add chunked response support
- Make `blocking` the default transport mode; `async` is now opt-in via
  `default-features = false`
- Select the blocking client from the `blocking` feature itself instead of the
  absence of `async`, and report a conflict between the two at compile time
- Expose concrete `RequestError` values instead of `anyhow::Error` in the public
  API (`Request::new`, `Header::parse`, `Url::build_from_string`,
  `ResponseInfo::parse_response_info`), with new `InvalidHeader` and
  `InvalidResponse` variants
- Fix gzip decoding in async mode
- Fix an empty chunked body decoding as an error instead of an empty body,
  and stop the blocking chunked reader spinning on a truncated body
- Generate multipart boundaries from an alphanumeric alphabet so they stay
  valid per RFC 2046
- Fix `blocking::MultipartFormData::add_file` being declared `async`, which made
  it uncallable without an executor
- Drop the `anyhow`, `bufstream` and `futures` dependencies
- Percent-encode form keys and values, and decode them in `FormData::from_str`;
  an unescaped `&` or `=` in a value previously split one field into several
- Fix `Read for FormData` never reporting EOF: it rebuilt the payload on every
  call and restarted from the beginning, so `read_to_end` looped forever and
  grew until it exhausted memory. The payload is now encoded once and streamed
- Strip the URL fragment, which was being sent to the server as part of the path
- Keep the query string of a host-only URL such as `http://example.com?q=1`,
  which was previously folded into the hostname and then dropped
- Stop sending `Content-Type` on bodyless requests, and stop defaulting it to an
  `Accept`-style value; send `Accept: */*` instead
- Keep `From<FormData> for Body` as a form body so the client labels it
  `application/x-www-form-urlencoded` rather than falling back to the default
- Add the `HEAD`, `PATCH` and `OPTIONS` methods, and treat a HEAD response as
  bodyless so reads return EOF instead of consuming the next response
- Add `Request::set_follow_redirects` to opt out of automatic redirects
- CLI: implement shell completions, `--multipart`, `--no-follow` and
  `--color`, which were previously accepted but inert
- Add decoder, error and boundary test suites; the test suite now passes in
  async mode as well as blocking
- Add wire-level tests that assert the exact request bytes, plus an
  `examples/methods.rs` demonstrating HEAD, PATCH and OPTIONS
- Add `CLI.md` and a bundled chunked demo server for `examples/chunked.rs`