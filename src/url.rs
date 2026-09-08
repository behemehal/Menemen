use crate::error::RequestError;

/// QueryParam
#[derive(Clone, Debug, PartialEq)]
pub struct QueryParam {
    /// The name of the query parameter
    pub name: String,
    /// Value of the query parameter
    pub value: String,
}

/// URL struct
#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    /// Is url uses https
    pub is_https: bool,
    /// Host name
    pub host: String,
    /// Query parameters ([`QueryParam`]) in [`Vec`]
    pub query_params: Vec<QueryParam>,
    /// Port number
    pub port: u16,
    /// Paths
    pub paths: Vec<String>,
}

impl Url {
    /// Builds a URL from a string
    /// * `url_string` - The URL string
    /// ## Returns
    /// [`Url`] if the URL was successfully parsed else [`RequestError::MalformedUrl`]
    /// ## Example
    /// ```rust
    /// use menemen::url::Url;
    /// let url = Url::build_from_string("https://behemehal.org/test?qtest=123".to_string()).unwrap();
    ///
    /// assert_eq!(url.is_https, true);
    /// assert_eq!(url.host, "behemehal.org".to_string());
    /// assert_eq!(url.query_params.len(), 1);
    /// assert_eq!(url.query_params[0].name, "qtest".to_string());
    /// assert_eq!(url.query_params[0].value, "123".to_string());
    /// assert_eq!(url.port, 443);
    /// assert_eq!(url.paths.len(), 1);
    /// assert_eq!(url.paths[0], "test".to_string());
    /// ```
    pub fn build_from_string(url: String) -> Result<Url, RequestError> {
        // A fragment is client-side only and must never reach the server.
        let url = match url.split_once('#') {
            Some((before_fragment, _)) => before_fragment.to_string(),
            None => url,
        };

        // Split the query off before host and path parsing. Otherwise a URL
        // with a query but no path ("http://example.com?q=1") folds the query
        // into the hostname.
        let (authority_and_path, query_string) = match url.split_once('?') {
            Some((before_query, query)) => (before_query.to_string(), Some(query.to_string())),
            None => (url.clone(), None),
        };

        let protocol = authority_and_path
            .split("://")
            .collect::<Vec<&str>>()
            .first()
            .ok_or(RequestError::MalformedUrl)?
            .to_string();

        let mut new_url =
            authority_and_path.replace(&format!("{}://", protocol.as_str()), "");

        let (host, port) = {
            let _host = if new_url.contains("/") {
                new_url.split("/").collect::<Vec<&str>>()[0].to_string()
            } else {
                new_url.clone()
            };
            let host = if _host.contains(":") {
                _host.split(":").collect::<Vec<&str>>()[0].to_string()
            } else {
                _host.clone()
            };

            let _port = if _host.contains(":") {
                _host.split(":").collect::<Vec<&str>>()[1].to_string()
            } else if protocol == "https" {
                "443".to_string()
            } else {
                "80".to_string()
            };
            let port = _port
                .parse::<u16>()
                .map_err(|_| RequestError::MalformedUrl)?;
            (host, port)
        };

        new_url = format!(
            "/{}",
            new_url.split("/").collect::<Vec<&str>>()[1..].join("/")
        );

        let paths = if new_url.contains("/") && new_url != "/" {
            new_url.split("/").collect::<Vec<&str>>()[1..]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let query_params = match query_string {
            Some(query) if !query.is_empty() => query
                .split("&")
                .filter(|pair| !pair.is_empty())
                .map(|pair| match pair.split_once('=') {
                    Some((name, value)) => QueryParam {
                        name: name.to_string(),
                        value: value.to_string(),
                    },
                    None => QueryParam {
                        name: pair.to_string(),
                        value: String::new(),
                    },
                })
                .collect::<Vec<QueryParam>>(),
            _ => Vec::new(),
        };

        Ok(Url {
            is_https: protocol == "https",
            host,
            port,
            paths,
            query_params,
        })
    }

    /// Join url parameters according to the url scheme
    /// ## Returns
    /// String of joined parameters
    ///
    /// ## Example
    /// ```
    /// use menemen::url::Url;
    /// let url = Url::build_from_string("https://behemehal.org/test?first=test&second=test".to_string()).unwrap();
    /// let joiner_query_params = url.join_query_params();
    /// assert_eq!(joiner_query_params, "first=test&second=test".to_string());
    /// ```
    pub fn join_query_params(&self) -> String {
        self.query_params
            .clone()
            .into_iter()
            .map(|x| format!("{}={}", x.name, x.value))
            .collect::<Vec<String>>()
            .join("&")
    }
}
