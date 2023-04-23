#[cfg(test)]
mod tests {
    use menemen::url::QueryParam;

    #[test]
    fn test_url_parsing() {
        // Test URLs with http schema
        let url = menemen::url::Url::build_from_string(
            "http://example.com/path/to/resource?foo=bar&baz=qux".to_string(),
        )
        .unwrap();
        assert_eq!(url.is_https, false);
        assert_eq!(url.host, "example.com".to_string());
        assert_eq!(url.port, 80);
        assert_eq!(
            url.paths,
            vec!["path".to_string(), "to".to_string(), "resource".to_string()]
        );
        assert_eq!(
            url.query_params,
            vec![
                QueryParam {
                    name: "foo".to_string(),
                    value: "bar".to_string()
                },
                QueryParam {
                    name: "baz".to_string(),
                    value: "qux".to_string()
                }
            ]
        );

        // Test URLs with https schema
        let url = menemen::url::Url::build_from_string(
            "https://example.com/test?q1=123&q2=456".to_string(),
        )
        .unwrap();
        assert_eq!(url.is_https, true);
        assert_eq!(url.host, "example.com".to_string());
        assert_eq!(url.port, 443);
        assert_eq!(url.paths, vec!["test".to_string()]);
        assert_eq!(
            url.query_params,
            vec![
                QueryParam {
                    name: "q1".to_string(),
                    value: "123".to_string()
                },
                QueryParam {
                    name: "q2".to_string(),
                    value: "456".to_string()
                }
            ]
        );

        // Test URLs with custom port number
        let url = menemen::url::Url::build_from_string(
            "http://example.com:8080/foo/bar?baz=qux".to_string(),
        )
        .unwrap();
        assert_eq!(url.is_https, false);
        assert_eq!(url.host, "example.com".to_string());
        assert_eq!(url.port, 8080);
        assert_eq!(url.paths, vec!["foo".to_string(), "bar".to_string()]);
        assert_eq!(
            url.query_params,
            vec![QueryParam {
                name: "baz".to_string(),
                value: "qux".to_string()
            }]
        );

        // Test URLs with multiple paths
        let url = menemen::url::Url::build_from_string(
            "https://example.com/path/to/my/resource?param1=value1&param2=value2".to_string(),
        )
        .unwrap();
        assert_eq!(url.is_https, true);
        assert_eq!(url.host, "example.com".to_string());
        assert_eq!(url.port, 443);
        assert_eq!(
            url.paths,
            vec![
                "path".to_string(),
                "to".to_string(),
                "my".to_string(),
                "resource".to_string()
            ]
        );
        assert_eq!(
            url.query_params,
            vec![
                QueryParam {
                    name: "param1".to_string(),
                    value: "value1".to_string()
                },
                QueryParam {
                    name: "param2".to_string(),
                    value: "value2".to_string()
                }
            ]
        );

        // Test URLs with no path and no query parameters
        let url = menemen::url::Url::build_from_string("https://example.com".to_string()).unwrap();
        assert_eq!(url.is_https, true);
        assert_eq!(url.host, "example.com".to_string());
        assert_eq!(url.port, 443);
        assert_eq!(url.paths.len(), 0);
        assert_eq!(url.query_params, vec![]);
    }
}
