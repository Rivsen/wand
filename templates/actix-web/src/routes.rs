use actix_web::{get, web::ServiceConfig, HttpResponse, Responder};
use serde::Serialize;

pub fn init(cfg: &mut ServiceConfig) {
    cfg
        .service(index)
        .service(ping)
    ;
}

#[derive(Serialize, Debug)]
struct Pong {
    status: String,
    code: i16,
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().json(Pong {
        status: "ok".to_string(),
        code: 200,
    })
}