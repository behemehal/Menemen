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
- Fix `blocking::MultipartFormData::add_file` being declared `async`, which made
  it uncallable without an executor
- Drop the `anyhow`, `bufstream`, `futures` and `clap_complete` dependencies
- Add `CLI.md`