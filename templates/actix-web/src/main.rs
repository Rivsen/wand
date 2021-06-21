extern crate actix_web_skeleton;

use actix_web::{middleware, App, HttpServer};
use actix_web_skeleton::{routes};
use dotenv::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    let server_url = env::var("SERVER_URL").expect("SERVER_URL is not set in .env");
    let workers_conf = env::var("SERVER_WORKERS");
    let mut workers = num_cpus::get();

    if let Ok(workers_conf) = workers_conf {
        workers = workers_conf.parse::<usize>().expect("Invalid server workers config in .env");
    }

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(routes::init)
    })
        .bind(server_url.clone())?
        .workers(workers)
        .run()
        .await
}
