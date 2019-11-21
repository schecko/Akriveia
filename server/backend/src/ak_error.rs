use actix_web::{ error, HttpResponse, http::StatusCode, };
use serde_derive::{ Deserialize, Serialize, };
use std::fmt;
use common::*;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum AkErrorType {
    InternalFailure,
    NotFound,
    Unauthorized,
    Validation,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AkError {
    pub reason: String,
    pub t: AkErrorType,
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
            AkErrorType::InternalFailure => HttpResponse::InternalServerError().finish(),
            AkErrorType::NotFound => HttpResponse::NotFound().finish(),
            AkErrorType::Unauthorized => HttpResponse::Unauthorized().finish(),
            AkErrorType::Validation => HttpResponse::BadRequest().finish(),
        }
    }
}
