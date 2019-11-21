use actix_web::{ error, HttpResponse, http::StatusCode, };
use serde_derive::{ Deserialize, Serialize, };
use std::fmt;
use common::*;
use tokio_postgres::error::Error as TError;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum AkErrorType {
    Internal,
    NotFound,
    Unauthorized,
    Validation,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AkError {
    pub reason: String,
    pub t: AkErrorType,
}

impl AkError {
    fn internal(reason: &str) -> AkError {
        AkError {
            reason: reason.to_string(),
            t: AkErrorType::Internal,
        }
    }

    fn not_found(reason: &str) -> AkError {
        AkError {
            reason: reason.to_string(),
            t: AkErrorType::NotFound,
        }
    }

    fn unauthorized(reason: &str) -> AkError {
        AkError {
            reason: reason.to_string(),
            t: AkErrorType::Unauthorized,
        }
    }

    fn validation(reason: &str) -> AkError {
        AkError {
            reason: reason.to_string(),
            t: AkErrorType::Validation,
        }
    }
}

impl fmt::Display for AkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO which one is better?
        //write!(f, "{}", serde_json::to_string(&Err::<(), _>(self)).unwrap())
        write!(f, "{}", self.reason)
    }
}

impl fmt::Debug for AkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl error::ResponseError for AkError {
    fn error_response(&self) -> HttpResponse {
        match self.t {
            AkErrorType::Internal => HttpResponse::InternalServerError().finish(),
            AkErrorType::NotFound => HttpResponse::NotFound().finish(),
            AkErrorType::Unauthorized => HttpResponse::Unauthorized().finish(),
            AkErrorType::Validation => HttpResponse::BadRequest().finish(),
        }
    }
}

impl From<TError> for AkError {
    fn from(other: TError) -> Self {
        match other.into_source() {
            Some(err) => {
                dbg!(err);
                AkError {
                    t: AkErrorType::Validation,
                    reason: err.to_string(),
                }
            },
            None => {
                AkError {
                    t: AkErrorType::Internal,
                    reason: "Unknown database error.".to_owned(),
                }
            }
        }
    }
}
