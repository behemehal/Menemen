fn main() {
    /*     //If https enabled but async not enabled
    #[cfg(all(feature = "https-blocking", feature = "async"))]
    compile_error!("Cannot enable 'https-blocking' feature with 'async' feature, use 'https' feature instead.");

    #[cfg(all(feature = "https-blocking", feature = "https"))]
    compile_error!("Cannot enable both 'https-blocking' and 'https' features"); */

    #[cfg(not(any(feature = "async", feature = "blocking")))]
    compile_error!(
        "Menemen requires either the 'async' or 'blocking' feature to be enabled. \
     Please enable one in your Cargo.toml."
    );
}
