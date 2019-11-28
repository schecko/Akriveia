use failure::Fallible;
use yew::services::fetch::{ Response as FetchResponse, };
use yew::format::Json;
use chrono::offset::FixedOffset;
use stdweb::web::Date;
pub use chrono::{ DateTime, Utc, format::DelayedFormat, format::StrftimeItems, };

pub type Response<T> = FetchResponse<Json<Fallible<T>>>;

#[derive(Clone, Copy, PartialEq)]
pub enum WebUserType {
    Admin,
    Responder,
}

pub fn format_timestamp<'a>(stamp: &DateTime<Utc>) -> DelayedFormat<StrftimeItems<'a>> {
    let offset_minutes = Date::new().get_timezone_offset();
    let off = FixedOffset::east(offset_minutes * 60);
    let zoned_stamp = stamp.with_timezone(&off);
    zoned_stamp.format("%c")
}


macro_rules! Log {
    ($($arg:tt)*) => (
        {
            use yew::services::ConsoleService;
            let mut console = ConsoleService::new();
            console.log(format!($($arg)*).as_str());
        }
    )
}

macro_rules! put_image {
    ($fetch_service:expr, $url:expr, $body:expr, $link:expr, $msg:expr) => {
        match yew::services::fetch::Request::put($url)
            .header("Content-Type", "image/png,image/jpg")
            .header("Accept", "application/json")
            .body(Ok($body))
        {
            Ok(req) => Some($fetch_service.fetch_binary(req, $link.send_back($msg))),
            Err(_) =>  None,
        };
    };
}

macro_rules! post_request {
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match yew::services::fetch::Request::post($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Json(&$request))
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
        match yew::services::fetch::Request::post($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Json(&$request))
        {
            Ok(req) => Some($fetch_service.fetch(req, $link.send_back($msg))),
            Err(_) =>  None,
        };
    };
}

macro_rules! put_request {
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match yew::services::fetch::Request::put($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Json(&$request))
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
        match yew::services::fetch::Request::put($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Json(&$request))
        {
            Ok(req) => Some($fetch_service.fetch(req, $link.send_back($msg))),
            Err(_) =>  None,
        };
    };
}

macro_rules! get_request {
    ($fetch_service:expr, $url:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match yew::services::fetch::Request::get($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Nothing)
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
        match yew::services::fetch::Request::get($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Nothing)
        {
            Ok(req) => Some($fetch_service.fetch(req, $link.send_back($msg))),
            Err(_) => None,
        };
    };
}

macro_rules! delete_request {
    ($fetch_service:expr, $url:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match yew::services::fetch::Request::delete($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Nothing)
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
        match yew::services::fetch::Request::delete($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(yew::format::Nothing)
        {
            Ok(req) => Some($fetch_service.fetch(req, $link.send_back($msg))),
            Err(_) => None,
        };
    };
}
