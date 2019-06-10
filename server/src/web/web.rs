
extern crate actix_web;

use actix_web::{
    get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};

#[get("/index.html")]
fn index(req: HttpRequest, name: web::Path<String>) -> String {
    println!("REQ: {:?}", req);
    format!("Hello: {}!\r\n", name)
}

#[get("/")]
fn no_params() -> &'static str {
    "Hello world!\r\n"
}

pub fn init() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(index)
            .service(no_params)
    })
    .bind("0.0.0.0:8080")?
    .workers(1)
    .run()
}

#[allow(dead_code)]
fn main() {
    init();
}
