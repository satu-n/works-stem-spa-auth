#[macro_use]
extern crate diesel;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware, web, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use actix_cors::Cors;

mod schema;
mod models;
mod handlers;
mod errors;
mod utils;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG",
        format!("{}=debug,actix_web=info,actix_server=info",
            utils::env_var("APP_NAME")),
    );
    env_logger::init();
    
    // create db connection pool
    let db_url = utils::env_var("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool: models::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // start http server
    HttpServer::new(move || {
        let cors = Cors::permissive(); // TODO tighten for production
        App::new()
            .wrap(cors)
            .data(pool.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(utils::SECRET_KEY.as_bytes())
                    .name("auth")
                    .path("/")
                    // .domain(utils::env_var("COOKIE_DOMAIN").as_str()) // TODO if cross domain
                    .max_age(86400)
                    .secure(
                        utils::env_var("API_PROTOCOL") == "https"
                        ), // TODO https
            ))
            // limit the maximum amount of data that server will accept
            .data(web::JsonConfig::default().limit(4096))
            // everything under '/api/' route
            .service(
                web::scope("/api")
                    .service(
                        web::resource("/invite")
                            .route(web::post().to(handlers::invite::invite)),
                    )
                    .service(
                        web::resource("/register")
                            .route(web::post().to(handlers::register::register)),
                    )
                    .service(
                        web::resource("/auth")
                            .route(web::get().to(handlers::auth::get_me))
                            .route(web::post().to(handlers::auth::login))
                            .route(web::delete().to(handlers::auth::logout)),
                    ),
            )
    })
    .bind(format!("0.0.0.0:3000"))?
    .run()
    .await
}
