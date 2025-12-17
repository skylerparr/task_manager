pub mod access;
pub mod controllers;
pub mod event_loop;
pub mod models;
pub mod schema;

use crate::event_loop::EventLoop;
use actix::prelude::*;
use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    SyncArbiter::start(4, || EventLoop);
    HttpServer::new(|| {
        App::new()
            .service(controllers::create)
            .service(controllers::delete)
            .service(controllers::get)
            .service(controllers::index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
