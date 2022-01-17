/// List of request errors
#[derive(Clone, Debug)]
pub enum RequestErrors {
    /// Cannot set headers after they sent
    CantSetHeadersAfterRequestSent,
    /// Cannot resolve given url
    CantResolveUrl,
    /// Connection timed out
    ConnectionTimeout,
    /// Given url is not correct
    MalformedUrl,
    /// Request already sent
    AlreadySent,
    /// Connection error occured with string
    ConnectionError(String),
}
