use serde_json::Value;
use std::{borrow::Cow, error::Error, io::ErrorKind};

mod microsoft;
pub mod value;

pub use microsoft::*;
pub use value::*;
/// Short hand Result
pub type PileResult<T = ()> = Result<T, ErrPile>;

/// Encapsulates all the possible Error that might be encountered
#[derive(Debug, thiserror::Error)]
pub enum ErrPile {
    #[error("Error connecting/ storing to DB")]
    DB(
        #[source]
        #[from]
        sqlx::Error,
    ),

    #[error("An error occurred with SSH")]
    Ssh(
        #[source]
        #[from]
        russh::Error,
    ),

    #[error("An error occurred with sftp connection")]
    Sftp(
        #[source]
        #[from]
        russh_sftp::client::error::Error,
    ),

    #[error("Invalid username or password was provided. Please try again")]
    Auth,

    #[error("User does not have permission to perform this action.")]
    Permission,

    #[error("This action can't be performed as it is being currently used elsewhere")]
    InUse,

    #[error("The resource is not ready yet, please try again later")]
    NotReady,

    #[error("An error occurred while getting data using Microsoft Graph")]
    Graph(
        #[source]
        #[from]
        Box<graph_rs_sdk::GraphFailure>,
    ),

    #[error("Graph Error Message")]
    GraphErrMSg(
        #[source]
        #[from]
        Box<graph_rs_sdk::error::ErrorMessage>,
    ),

    #[error("Error parsing Json Data (Serde)")]
    Json(
        #[source]
        #[from]
        serde_json::Error,
    ),

    #[error("Request responded with an error")]
    MS(MSResponseError),

    #[error("An error occurred while parsing the PDF text (PDF_Extract)")]
    ExtractPdf(
        #[source]
        #[from]
        pdfium_render::prelude::PdfiumError,
    ),

    #[error("Error opening zip archive")]
    Zip(
        #[source]
        #[from]
        zip::result::ZipError,
    ),

    #[error("Error decoding from base64 content bytes")]
    Decode(
        #[source]
        #[from]
        base64::DecodeError,
    ),

    #[error("A thread panicked while executing a task")]
    Thread(
        #[source]
        #[from]
        tokio::task::JoinError,
    ),

    #[error("An error occurred while performing an operation on a Image")]
    Image(
        #[source]
        #[from]
        image::ImageError,
    ),

    #[error("A TimeFrame error occurred")]
    Timeframe(
        #[source]
        #[from]
        timeframe::TimeErr,
    ),

    #[error("IO Err: {0}")]
    IO(
        #[source]
        #[from]
        std::io::Error,
    ),

    #[cfg(feature = "python")]
    #[error("An error occurred on Python Side: {0}")]
    Python(
        #[from]
        #[source]
        pyo3::PyErr,
    ),

    #[error("An error occurred while parsing the URL")]
    Url(
        #[source]
        #[from]
        url::ParseError,
    ),

    #[error("An error occurred while sending request")]
    Req(
        #[source]
        #[from]
        reqwest::Error,
    ),

    #[error("An error occurred while converting Http Header to string")]
    ReqToStr(
        #[source]
        #[from]
        reqwest::header::ToStrError,
    ),

    #[error("Document Intelligence Services returned with an error")]
    AZ(
        #[source]
        #[from]
        Box<AZError>,
    ),

    #[error("{0}")]
    FromValue(
        #[source]
        #[from]
        SerdeValue,
    ),

    #[error("{0}")]
    Custom(String),
}

impl ErrPile {
    pub fn custom<'a, I>(msg: I) -> Self
    where
        I: Into<Cow<'a, str>>,
    {
        let s = msg.into().into_owned();
        Self::Custom(s)
    }

    /// the error is related to invalid credentials
    pub fn is_encrypted(&self) -> bool {
        matches!(self, Self::Auth)
    }

    /// get the string version of the source error
    pub fn source_str(&self) -> String {
        if let Self::FromValue(val) = &self {
            return val.extract_error_from_json();
        }

        self.source()
            .map(|e| e.to_string())
            .unwrap_or_else(|| self.to_string())
    }

    /// checks if this error is not ready error
    pub fn is_not_ready(&self) -> bool {
        matches!(self, Self::NotReady)
    }

    /// checks if this error is transcient error
    /// meaning can this error automatically fixed, if the program tries
    /// again. This will be useful when using retry functions with backoff
    /// feature.
    pub fn is_transient(&self) -> bool {
        if let Self::Req(req) = &self {
            if let Some(status) = req.status() {
                return matches!(status.as_u16(), 408 | 429 | 500 | 502 | 503 | 504);
            }
        }

        if let Self::IO(io) = &self {
            return Self::is_io_transient(io.kind());
        }

        if let Self::DB(db) = self {
            return match &db {
                sqlx::Error::Io(err) if Self::is_io_transient(err.kind()) => true,
                sqlx::Error::Database(_) => true, // Database errors can be transient
                sqlx::Error::PoolTimedOut => true,
                sqlx::Error::PoolClosed => true,
                _ => false,
            };
        }

        if let Self::NotReady = self {
            return true; // Not ready errors are transient
        }

        false
    }

    fn is_io_transient(kind: std::io::ErrorKind) -> bool {
        matches!(
            kind,
            ErrorKind::WouldBlock
                | ErrorKind::TimedOut
                | ErrorKind::Interrupted
                | ErrorKind::ConnectionReset
                | ErrorKind::ConnectionAborted
                | ErrorKind::NotConnected
                | ErrorKind::ConnectionRefused
                | ErrorKind::AddrInUse
                | ErrorKind::Deadlock
                | ErrorKind::HostUnreachable
                | ErrorKind::NetworkDown
                | ErrorKind::NetworkUnreachable
                | ErrorKind::ResourceBusy
        )
    }

    /// Handle all types of HTTP errors comprehensively
    async fn handle_error_response(response: reqwest::Response) -> ErrPile {
        let status = response.status();
        let status_code = status.as_u16();

        // Categorize the error type
        let error_category = match status_code {
            // 1xx - Informational (shouldn't be errors, but handle just in case)
            100..=199 => "Informational",
            // 3xx - Redirection errors
            300..=399 => "Redirection",
            // 4xx - Client errors
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            408 => "Request Timeout",
            409 => "Conflict",
            410 => "Gone",
            413 => "Payload Too Large",
            414 => "URI Too Long",
            415 => "Unsupported Media Type",
            422 => "Unprocessable Entity",
            429 => "Too Many Requests",
            430..=499 => "Client Error",
            // 5xx - Server errors
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            507 => "Insufficient Storage",
            508..=599 => "Server Error",
            _ => "Unknown Error",
        };

        // Try to get response body
        match response.json::<Value>().await {
            Ok(body) => {
                // First try to parse as structured AZError
                if let Ok(az_error) = serde_json::from_value::<AZError>(body.clone()) {
                    return ErrPile::AZ(Box::new(az_error));
                }

                SerdeValue(body).into()
            }
            Err(e) => ErrPile::Custom(format!(
                "{error_category} ({status_code}): Failed to read error response: {e}"
            )),
        }
    }
}

impl From<serde_json::Value> for ErrPile {
    fn from(value: serde_json::Value) -> Self {
        ErrPile::FromValue(SerdeValue(value))
    }
}

impl<T> From<ErrPile> for PileResult<T> {
    fn from(value: ErrPile) -> Self {
        Err(value)
    }
}

impl From<&str> for ErrPile {
    fn from(value: &str) -> Self {
        ErrPile::custom(value)
    }
}

pub trait ReqwestPileResExt {
    /// converts the reponse into appropriate ErrPile
    /// this will also take care of Azure Document Intelligence errors
    /// based on the response
    #[allow(async_fn_in_trait)]
    async fn to_pile_result<T>(self) -> PileResult<T>
    where
        T: for<'de> serde::Deserialize<'de>;
}

impl ReqwestPileResExt for reqwest::Response {
    async fn to_pile_result<T>(self) -> PileResult<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let status = self.status();
        if status.is_success() {
            return Ok(self.json::<T>().await?);
        }

        Err(ErrPile::handle_error_response(self).await)
    }
}
