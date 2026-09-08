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

    #[test]
    fn test_query_params_without_explicit_value() {
        let url = menemen::url::Url::build_from_string(
            "https://example.com/test?flag&empty=&key=value".to_string(),
        )
        .unwrap();

        assert_eq!(
            url.query_params,
            vec![
                QueryParam {
                    name: "flag".to_string(),
                    value: "".to_string(),
                },
                QueryParam {
                    name: "empty".to_string(),
                    value: "".to_string(),
                },
                QueryParam {
                    name: "key".to_string(),
                    value: "value".to_string(),
                },
            ]
        );
    }

    #[test]
    fn test_join_query_params() {
        let url = menemen::url::Url::build_from_string(
            "https://example.com/test?first=1&second=2".to_string(),
        )
        .unwrap();

        assert_eq!(url.join_query_params(), "first=1&second=2");
    }

    #[test]
    fn explicit_port_overrides_scheme_default() {
        let url = menemen::url::Url::build_from_string("https://example.com:8443/x".to_string())
            .unwrap();
        assert_eq!(url.is_https, true);
        assert_eq!(url.port, 8443);
        assert_eq!(url.host, "example.com".to_string());
    }

    #[test]
    fn host_only_url_defaults_to_port_80() {
        let url = menemen::url::Url::build_from_string("http://example.com".to_string()).unwrap();
        assert_eq!(url.port, 80);
        assert_eq!(url.is_https, false);
        assert_eq!(url.paths.len(), 0);
    }

    #[test]
    fn trailing_slash_yields_no_paths() {
        let url = menemen::url::Url::build_from_string("http://example.com/".to_string()).unwrap();
        assert_eq!(url.host, "example.com".to_string());
        assert_eq!(url.paths.len(), 0);
    }

    #[test]
    fn out_of_range_port_is_rejected() {
        // 70000 does not fit in a u16.
        let result = menemen::url::Url::build_from_string("http://example.com:70000/".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn non_numeric_port_is_rejected() {
        let result = menemen::url::Url::build_from_string("http://example.com:abc/".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn ip_literal_host_is_preserved() {
        let url =
            menemen::url::Url::build_from_string("http://127.0.0.1:3000/health".to_string())
                .unwrap();
        assert_eq!(url.host, "127.0.0.1".to_string());
        assert_eq!(url.port, 3000);
        assert_eq!(url.paths, vec!["health".to_string()]);
    }

    /// Regression: a query string on a host-only URL used to be folded into the
    /// hostname and then dropped, so the request went out without it.
    #[test]
    fn query_on_host_only_url_is_kept() {
        let url =
            menemen::url::Url::build_from_string("https://example.com?q1=123&q2=456".to_string())
                .unwrap();

        assert_eq!(url.is_https, true);
        assert_eq!(url.host, "example.com".to_string());
        assert_eq!(url.port, 443);
        assert_eq!(url.paths.len(), 0);
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
        assert_eq!(url.join_query_params(), "q1=123&q2=456".to_string());
    }
}
