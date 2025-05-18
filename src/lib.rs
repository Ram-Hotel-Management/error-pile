use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{borrow::Cow, error::Error, fmt};

/// Accomdate the use for mapping to correct response
/// from Microsoft Graph response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MSResponseErrorInner {
    pub code: String,
    pub inner_error: Value,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct MSResponseError {
    pub error: MSResponseErrorInner,
}

#[derive(Debug, Deserialize)]
pub struct MSResponse<T> {
    value: Option<T>,
    error: Option<MSResponseError>,
}

impl<T: std::fmt::Debug> From<MSResponse<T>> for PileResult<T> {
    fn from(value: MSResponse<T>) -> Self {
        if let Some(err) = value.error {
            return Err(ErrPile::MS(err));
        }

        if let Some(val) = value.value {
            return Ok(val);
        }

        Err(ErrPile::Custom(format!(
            "Could not parse Ok variant or the Err variant | Response: {:?}",
            value
        )))
    }
}

/////////////////////////////// AZURE Document Intelligence Errors ////////////////////////
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AZError {
    pub error: AZErrorDetails,
}

impl fmt::Display for AZError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl std::error::Error for AZError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error as &dyn Error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AZErrorDetails {
    pub code: String,
    pub message: String,
    pub target: Option<String>,
    pub details: Option<Vec<AZError>>,
    pub innererror: Option<AZErrorInner>,
}

impl fmt::Display for AZErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.code, self.message,)
    }
}

impl std::error::Error for AZErrorDetails {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.innererror.as_ref().map(|e| e as &dyn Error)
    }
}

type BoxAZErrorInner = Box<AZErrorInner>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AZErrorInner {
    pub code: Option<String>,
    pub message: Option<String>,
    pub innererror: Option<BoxAZErrorInner>,
}

impl fmt::Display for AZErrorInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {}",
            self.code.as_ref().map_or("", |v| v),
            self.message.as_ref().map_or("", |v| v)
        )
    }
}

impl std::error::Error for AZErrorInner {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.innererror.as_deref().map(|e| e as &dyn Error)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AZWarnning {
    /** One of a server-defined set of warning codes. */
    code: String,
    /** A human-readable representation of the warning. */
    message: String,
    /** The target of the error. */
    target: Option<String>,
}

/////////////////////////////// END OF AZURE Document Intelligence Errors ////////////////////////

/// Short hand Result
pub type PileResult<T> = Result<T, ErrPile>;

/// Encapsulates all the possible Error that might be encountered
#[derive(Debug, thiserror::Error)]
pub enum ErrPile {
    #[error("Error connecting/ storing to DB")]
    DB(
        #[source]
        #[from]
        DbErr,
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

    #[error("An invalid username or password was provided. Please try again")]
    Auth,

    #[error("An error occurred while getting data using Microsoft Graph")]
    Graph(
        #[source]
        #[from]
        graph_rs_sdk::GraphFailure,
    ),

    #[error("Graph Error Message")]
    GraphErrMSg(
        #[source]
        #[from]
        graph_rs_sdk::error::ErrorMessage,
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

    #[error("An ocr client/ server returned an error")]
    Ocr(
        #[source]
        #[from]
        ocr_client::OcrErrs,
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

    #[error("An error occurred while parsing the URL")]
    Url(
        #[source]
        #[from]
        url::ParseError,
    ),

    #[error("An error occurred while sending request to the AI Model")]
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
        AZError,
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

#[test]
fn testing_compilations() {
    let _a = ErrPile::custom("Some message");
    let _b = ErrPile::custom(format!("{} Some other error", "ErrCode:"));
}
