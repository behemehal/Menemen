fn main() {
    match menemen::url::Url::build_from_string("https://behemehal.net/test?qtest=123".to_string()) {
        Ok(url) => {
            println!("{:#?}", url);
        }
        Err(_) => {
            println!("Failed to parse url");
        }
    }
}
