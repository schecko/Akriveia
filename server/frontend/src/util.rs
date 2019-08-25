use failure::Fallible;
use yew::{ services::fetch::Response as FetchResponse, };
use yew::format::Json;

pub type Response<T> = FetchResponse<Json<Fallible<T>>>;

macro_rules! Log {
    ($($arg:tt)*) => (
        {
            use yew::services::ConsoleService;
            let mut console = ConsoleService::new();
            console.log(format!($($arg)*).as_str());
        }
    )
}

macro_rules! request {
    ($method:ident, $fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        match Request::$method($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
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
    ($method:ident, $fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr) => {
        match Request::$method($url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(Json(&$request))
        {
            Ok(req) => Some($fetch_service.fetch(req, $link.send_back($msg))),
            Err(_) =>  None,
        };
    };
}

macro_rules! post_request {
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        request!(post, $fetch_service, $url, $request, $link, $msg, $success, $error);
    };
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr) => {
        request!(post, $fetch_service, $url, $request, $link, $msg);
    };
}

macro_rules! get_request {
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        request!(get, $fetch_service, $url, $request, $link, $msg, $success, $error);
    };
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr) => {
        request!(get, $fetch_service, $url, $request, $link, $msg);
    };
}

macro_rules! put_request {
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr, $success:expr, $error:expr) => {
        request!(put, $fetch_service, $url, $request, $link, $msg, $success, $error);
    };
    ($fetch_service:expr, $url:expr, $request:expr, $link:expr, $msg:expr) => {
        request!(put, $fetch_service, $url, $request, $link, $msg);
    };
}
