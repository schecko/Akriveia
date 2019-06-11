extern crate actix_web;
extern crate actix_files;
extern crate askama;

use askama::Template;
use actix_files as fs;
use actix_web::{
    get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer
};

#[derive(Template)]
#[template(path = "index.html")]
struct Index;

#[get("/")]
fn index() -> Result<HttpResponse, Error> {
    let body = Index.render().unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub fn init() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(index)
    })
    .bind("0.0.0.0:8080")?
    .workers(1)
    .run()
}

#[allow(dead_code)]
fn main() {
    init();
}
