use anyhow::{Context, Error};

/// QueryParam
#[derive(Clone, Debug)]
pub struct QueryParam {
    /// The name of the query parameter
    pub name: String,
    /// Value of the query parameter
    pub value: String,
}

/// URL struct
#[derive(Debug, Clone)]
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
    /// [`Url`] if the URL was successfully parsed else [`Error`]
    /// ## Example
    /// ```rust
    /// use menemen::url::Url;
    /// let url = Url::build_from_string("https://behemehal.net/test?qtest=123".to_string()).unwrap();
    /// 
    /// assert_eq!(url.is_https, true);
    /// assert_eq!(url.host, "behemehal.net".to_string());
    /// assert_eq!(url.query_params.len(), 1);
    /// assert_eq!(url.query_params[0].name, "qtest".to_string());
    /// assert_eq!(url.query_params[0].value, "123".to_string());
    /// assert_eq!(url.port, 443);
    /// assert_eq!(url.paths.len(), 1);
    /// assert_eq!(url.paths[0], "test".to_string());
    /// ```
    pub fn build_from_string(url: String) -> Result<Url, Error> {
        let protocol = url
            .split("://")
            .collect::<Vec<&str>>()
            .first()
            .with_context(|| "Failed to parse protocol")?
            .to_string();
        let url_without_protocol = url
            .split("://")
            .collect::<Vec<&str>>()
            .last()
            .with_context(|| "Failed to parse host")?
            .to_string();

        let host = if url_without_protocol.contains("/") {
            url_without_protocol.split("/").collect::<Vec<&str>>()[0].to_string()
        } else if url_without_protocol.contains(":") {
            url_without_protocol.split(":").collect::<Vec<&str>>()[0].to_string()
        } else {
            url_without_protocol.clone()
        };

        let port = if url_without_protocol.contains(":") {
            url_without_protocol.split(":").collect::<Vec<&str>>()[1]
                .parse::<u16>()
                .with_context(|| "Failed to parse port")?
        } else if protocol == "https" {
            443
        } else {
            80
        };

        let paths = if url_without_protocol.contains("/") {
            url_without_protocol.split("/").collect::<Vec<&str>>()[1..]
                .iter()
                .map(|s| {
                    if s.contains("?") {
                        s.to_string().split("?").collect::<Vec<&str>>()[0].to_string()
                    } else {
                        s.to_string()
                    }
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let query_params = if paths.len() == 0 {
            vec![]
        } else if url_without_protocol.split("/").collect::<Vec<&str>>()[1..]
            .last()
            .with_context(|| "Failed to parse url")?
            .contains("?")
        {
            url_without_protocol.split("/").collect::<Vec<&str>>()[1..]
                .last()
                .with_context(|| "Failed to parse url")?
                .split("?")
                .collect::<Vec<&str>>()
                .last()
                .with_context(|| "Failed to parse url")?
                .split("&")
                .map(|x| {
                    let param = x.split("=").collect::<Vec<&str>>();
                    QueryParam {
                        name: param[0].to_string(),
                        value: if param.len() == 1 {
                            String::new()
                        } else {
                            param[1].to_string()
                        },
                    }
                })
                .collect::<Vec<QueryParam>>()
        } else {
            Vec::new()
        };

        Ok(Url {
            is_https: protocol == "https",
            host: host.to_string(),
            port,
            paths,
            query_params: query_params,
        })
    }

    /// Join url parameters according to the url scheme
    /// ## Returns
    /// String of joined parameters
    /// 
    /// ## Example
    /// ```
    /// use menemen::url::Url;
    /// let url = Url::build_from_string("https://behemehal.com/test?first=test&second=test".to_string()).unwrap();
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
