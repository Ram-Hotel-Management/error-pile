use core::fmt;
use std::error::Error;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ErrPile, PileResult};

/// Accomdate the use for mapping to correct response
/// from Microsoft Graph response
/// TODO implement ERROR trait this struct
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MSResponseErrorInner {
    pub code: String,
    pub inner_error: Value,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MSResponseError {
    pub error: MSResponseErrorInner,
}

#[derive(Debug, Serialize, Deserialize)]
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
            "Could not parse Ok variant or the Err variant | Response: {value:?}"
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
pub struct AZWarning {
    /** One of a server-defined set of warning codes. */
    code: String,
    /** A human-readable representation of the warning. */
    message: String,
    /** The target of the error. */
    target: Option<String>,
}

/////////////////////////////// END OF AZURE Document Intelligence Errors ////////////////////////
