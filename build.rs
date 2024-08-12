fn main() {
    //If https-async enabled but async not enabled
    #[cfg(all(feature = "https-async", not(feature = "async")))]
    compile_error!("Cannot enable 'https-async' feature without enabling 'async' feature.");

    //If https-async enabled but async not enabled
    #[cfg(all(feature = "https-async", not(feature = "async")))]
    compile_error!("Cannot enable 'https-async' feature without enabling 'async' feature.");

    #[cfg(all(feature = "https-async", not(feature = "https")))]
    compile_error!("Cannot enable 'https-async' feature without enabling 'https' feature.");
}
