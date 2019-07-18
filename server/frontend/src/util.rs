#[macro_use]

use failure::Fallible;
use yew::{ services::fetch::Response as FetchResponse, services::fetch::Request, services::fetch::FetchService, };
use yew::format::{Nothing, Json};

pub type Response<T> = FetchResponse<Json<Fallible<T>>>;

macro_rules! post_request {
    ($self:ident, $url:expr, $request:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match Request::post($url)
            .header("Content-Type", "text/html")
            .header("Accept", "text/html")
            .body(Json(&$request))
        {
            Ok(req) => {
                $success();
                Some($self.fetch_service.fetch(req, $link.send_back($msg)))
            },
            Err(e) => {
                $error(e);
                None
            },
        };
    };
    ($self:ident, $url:expr, $request:expr, $link:expr, $msg:expr) => {
        match Request::post($url)
            .header("Content-Type", "text/html")
            .header("Accept", "text/html")
            .body(Json(&$request))
        {
            Ok(req) => Some($self.fetch_service.fetch(req, $link.send_back($msg))),
            Err(_) =>  None,
        };
    };
}

macro_rules! get_request {
    ($self:ident, $url:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match Request::get($url)
            .header("Content-Type", "text/html")
            .header("Accept", "text/html")
            .body(Nothing)
        {
            Ok(req) => {
                $success();
                Some($self.fetch_service.fetch(req, $link.send_back($msg)))
            },
            Err(e) => {
                $error(e);
                None
            },
        };
    };
    ($self:ident, $url:expr, $link:expr, $msg:expr) => {
        match Request::get($url)
            .header("Content-Type", "text/html")
            .header("Accept", "text/html")
            .body(Nothing)
        {
            Ok(req) => Some($self.fetch_service.fetch(req, $link.send_back($msg))),
            Err(e) => None,
        };
    };
}
