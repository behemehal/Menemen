#[derive(Clone, Debug)]
pub enum RequestErrors {
    CantSetHeadersAfterRequestSent,
    CantResolveUrl,
    ConnectionTimeout,
    MalformedUrl,
    AlreadySent,
    ConnectionError(String),
}
