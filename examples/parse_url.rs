fn main() {
    match menemen::url::Url::build_from_string("https://github.com/test?/Test?ctest".to_string()) {
        Ok(url) => {
            println!("{:#?}", url);
        }
        Err(_) => {
            println!("Failed to parse url");
        }
    }
}
