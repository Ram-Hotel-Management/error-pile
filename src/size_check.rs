#[test]
fn size_check() {
    use std::mem::size_of;
    println!("Size analysis of ErrPile variants:");
    println!(
        "std::mem::size_of::<ErrPile>() = {}",
        size_of::<crate::ErrPile>()
    );
    println!();

    // Check sizes of underlying error types
    println!("Underlying error type sizes:");
    println!("sqlx::Error = {}", size_of::<sqlx::Error>());
    println!("russh::Error = {}", size_of::<russh::Error>());
    println!(
        "russh_sftp::client::error::Error = {}",
        size_of::<russh_sftp::client::error::Error>()
    );
    println!(
        "graph_rs_sdk::GraphFailure = {}",
        size_of::<graph_rs_sdk::GraphFailure>()
    );
    println!(
        "Boxed graph_rs_sdk::GraphFailure = {}",
        size_of::<Box<graph_rs_sdk::GraphFailure>>()
    );
    println!(
        "graph_rs_sdk::error::ErrorMessage = {}",
        size_of::<graph_rs_sdk::error::ErrorMessage>()
    );
    println!(
        "Boxed graph_rs_sdk::error::ErrorMessage = {}",
        size_of::<Box<graph_rs_sdk::error::ErrorMessage>>()
    );
    println!("serde_json::Error = {}", size_of::<serde_json::Error>());
    println!(
        "crate::MSResponseError = {}",
        size_of::<crate::MSResponseError>()
    );
    println!(
        "pdfium_render::prelude::PdfiumError = {}",
        size_of::<pdfium_render::prelude::PdfiumError>()
    );
    println!(
        "zip::result::ZipError = {}",
        size_of::<zip::result::ZipError>()
    );
    println!("base64::DecodeError = {}", size_of::<base64::DecodeError>());
    println!(
        "tokio::task::JoinError = {}",
        size_of::<tokio::task::JoinError>()
    );
    println!("image::ImageError = {}", size_of::<image::ImageError>());
    println!(
        "Box<image::ImageError> = {}",
        size_of::<Box<image::ImageError>>()
    );
    println!("timeframe::TimeErr = {}", size_of::<timeframe::TimeErr>());
    println!("std::io::Error = {}", size_of::<std::io::Error>());
    #[cfg(feature = "python")]
    println!("pyo3::PyErr = {}", size_of::<pyo3::PyErr>());
    println!("url::ParseError = {}", size_of::<url::ParseError>());
    println!("reqwest::Error = {}", size_of::<reqwest::Error>());
    println!(
        "reqwest::header::ToStrError = {}",
        size_of::<reqwest::header::ToStrError>()
    );
    println!("crate::AZError = {}", size_of::<crate::AZError>());
    println!(
        "Boxed crate::AZError = {}",
        size_of::<Box<crate::AZError>>()
    );
    println!("String = {}", size_of::<String>());

    println!("Value = {}", size_of::<serde_json::Value>());
}
