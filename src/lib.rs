use std::borrow::Cow;

use sea_orm::DbErr;
use serde::Deserialize;
use serde_json::Value;

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
pub enum MSResponse<T> {
    Ok(T),
    Err(MSResponseError),
}

impl<T> From<MSResponse<T>> for PileResult<T> {
    fn from(value: MSResponse<T>) -> Self {
        match value {
            MSResponse::Ok(apivalue) => Ok(apivalue),
            MSResponse::Err(msresponse_error) => Err(ErrPile::Response(msresponse_error)),
        }
    }
}

/// Short hand Result
pub type PileResult<T> = Result<T, ErrPile>;

/// Encapsulates all the possible Error that might be encountered
#[derive(Debug, thiserror::Error)]
pub enum ErrPile {
    #[error("Error connecting/ storing to DB")]
    DB(#[from] DbErr),

    #[error("An error occurred with SSH")]
    Ssh(#[from] russh::Error),

    #[error("An error occurred with sftp connection")]
    Sftp(#[from] russh_sftp::client::error::Error),

    #[error("An invalid username or password was provided. Please try again")]
    Auth,

    #[error("An error occurred while getting data using Microsoft Graph")]
    Graph(#[from] graph_rs_sdk::GraphFailure),

    #[error("Graph Error Message")]
    GraphErrMSg(#[from] graph_rs_sdk::error::ErrorMessage),

    #[error("Error parsing Json Data (Serde)")]
    Json(#[from] serde_json::Error),

    #[error("Request responded with an error")]
    Response(MSResponseError),

    #[error("An error occurred with excel file")]
    Xlsx(#[from] calamine::XlsxError),

    #[error("Error opening zip archive")]
    Zip(#[from] zip::result::ZipError),

    #[error("Error decoding from base64 content bytes")]
    Decode(#[from] base64::DecodeError),

    #[error("A thread panicked while executing a task")]
    Thread(#[from] tokio::task::JoinError),

    #[error("An IO Error Occurred")]
    IO(#[from] std::io::Error),

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
