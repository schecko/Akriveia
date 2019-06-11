extern crate actix_web;
extern crate actix_files;

use actix_files as fs;
use actix_web::{
    get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(fs::Files::new("/", "static/").index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?
    .run()
}

