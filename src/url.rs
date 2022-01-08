use anyhow::{Context, Error};

#[derive(Clone, Debug)]
pub struct QueryParam {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]

pub struct Url {
    pub is_https: bool,
    pub host: String,
    pub query_params: Vec<QueryParam>,
    pub port: u16,
    pub paths: Vec<String>,
}

impl Url {
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

    pub fn join_query_params(&self) -> String {
        self.query_params
            .clone()
            .into_iter()
            .map(|x| format!("{}={}", x.name, x.value))
            .collect::<Vec<String>>()
            .join("&")
    }
}
