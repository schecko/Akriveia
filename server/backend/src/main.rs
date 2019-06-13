extern crate actix_web;
extern crate actix_files;
extern crate actix_session;
extern crate env_logger;

use std::env;
use actix_session::{CookieSession, Session};
use actix_web::http::{header, Method, StatusCode};
use actix_files as fs;
use actix_web::{
    get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct HelloFrontEnd {
    data: u32,
}

#[get("/hello")]
fn hello(req: HttpRequest) -> HttpResponse {
    println!("hello called");
    let hello_data = HelloFrontEnd {
        data: 0xDEADBEEF,
    };
    HttpResponse::Ok().json(hello_data)
}

fn default_route(req: HttpRequest) -> HttpResponse {
    println!("default route called");
    println!("request was: {:?}", req);
    HttpResponse::NotFound().finish()
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=debug,actix_web=debug");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(hello)
            // these two last !!
            .service(fs::Files::new("/", "static/").index_file("index.html"))
            .default_service(web::resource("").to(default_route))
    })
    .bind("0.0.0.0:8080")?
    .run()
}

