#[macro_use]

use failure::Fallible;
use yew::{ services::fetch::Response as FetchResponse, services::fetch::Request, services::fetch::FetchService, };
use yew::format::{Nothing, Json};

pub type Response<T> = FetchResponse<Json<Fallible<T>>>;

macro_rules! Log {
    ($($arg:tt)*) => (
        let mut console = ConsoleService::new();
        console.log(format!($($arg)*).as_str());
    )
}


macro_rules! post_request {
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match Request::post($url)
            .header("Content-Type", "text/html")
            .header("Accept", "text/html")
            .body(Json(&$request))
        {
            Ok(req) => {
                $success();
                Some($fetch_service.fetch(req, $link.send_back($msg)))
            },
            Err(e) => {
                $error(e);
                None
            },
        };
    };
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr) => {
        match Request::post($url)
            .header("Content-Type", "text/html")
            .header("Accept", "text/html")
            .body(Json(&$request))
        {
            Ok(req) => Some($fetch_service.fetch(req, $link.send_back($msg))),
            Err(_) =>  None,
        };
    };
}

macro_rules! get_request {
    ($fetch_service:expr, $url:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match Request::get($url)
            .header("Content-Type", "text/html")
            .header("Accept", "text/html")
            .body(Nothing)
        {
            Ok(req) => {
                $success();
                Some($fetch_service.fetch(req, $link.send_back($msg)))
            },
            Err(e) => {
                $error(e);
                None
            },
        };
    };
    ($fetch_service:expr, $url:expr, $link:expr, $msg:expr) => {
        match Request::get($url)
            .header("Content-Type", "text/html")
            .header("Accept", "text/html")
            .body(Nothing)
        {
            Ok(req) => Some($fetch_service.fetch(req, $link.send_back($msg))),
            Err(e) => None,
        };
    };
}
