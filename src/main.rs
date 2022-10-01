use actix_web::{middleware, web, App, HttpServer};
use crate::config::CONFIG;
use tera::Tera;
use tokio_postgres::NoTls;
use log::error;

mod config;
mod models;
mod routes;

const LETTER_COUNT: i32 = 3;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    if let Err(err) = config::load().await {
        error!("ConfigError: couldn't load or parse config, err: {err}");
        return Ok(());
    }

    let config = CONFIG.read().await;

    let pg = deadpool_postgres::Config {
        host: Some(config.db.host.clone()),
        dbname: Some(config.db.database.clone()),
        user: Some(config.db.user.clone()),
        password: Some(config.db.password.clone()),
        port: config.db.port,
        ..Default::default()
    };
    let address = config.http.address.clone();
    let port = config.http.port;
    drop(config);

    let pool = pg
        .create_pool(None, NoTls)
        .expect("couldn't create db pool");

    HttpServer::new(move || {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        App::new()
            .app_data(web::Data::new(tera))
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(routes::index::index)
            .service(routes::router::router)
            .service(routes::shorten::shorten)
            .default_service(web::to(routes::default::default_handler))
    })
    .bind((address, port))?
    .run()
    .await
}
