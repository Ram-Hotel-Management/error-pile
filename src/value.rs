use core::fmt;

use serde_json::Value;

#[derive(Debug)]
pub struct SerdeValue(pub serde_json::Value);

impl SerdeValue {
    /// Extract error information from any JSON response format
    pub fn extract_error_from_json(&self) -> String {
        let json_value = &self.0;

        if let Value::Object(obj) = json_value {
            // Common error field names to check
            let error_fields = [
                "error",
                "err",
                "message",
                "detail",
                "details",
                "description",
                "errorMessage",
                "error_message",
                "reason",
                "title",
            ];

            for field in &error_fields {
                if let Some(error_value) = obj.get(*field) {
                    match error_value {
                        Value::String(s) => return s.clone(),
                        Value::Object(_) => {
                            let nested = Self::extract_error_from_json(&Self(error_value.clone()));
                            if !nested.is_empty() {
                                return nested;
                            }
                        }
                        _ => continue,
                    }
                }
            }

            // Return formatted JSON if no specific error field found
            serde_json::to_string_pretty(&json_value)
                .unwrap_or_else(|_| "Unknown error format".to_string())
        } else if let Value::String(s) = json_value {
            s.clone()
        } else {
            json_value.to_string()
        }
    }
}

impl From<serde_json::Value> for SerdeValue {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl fmt::Display for SerdeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SerdeValue {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
