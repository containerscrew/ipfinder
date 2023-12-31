mod infrastructure;
mod routes;
mod models;
mod app;

#[cfg(test)]
mod test;

use std::env;
use std::sync::Arc;
use actix_web::{App, HttpServer, web};
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use dotenv::dotenv;
use env_logger::Env;
use crate::app::db_ops::DbOps;
use crate::infrastructure::{Db};
use crate::routes::{delete_ip, get_ip, health, insert_ip, update_ip};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    get_env();

    println!("Ipfinder API started!");

    // Load env variables
    let db_endpoint : String= env::var("DB_ENDPOINT").expect("You must set the DB_ENDPOINT environment var!");
    let db_name: String = env::var("DB_NAME").expect("You must set the DB_NAME environment var!");
    let collection_name: String = env::var("COLLECTION_NAME").expect("You must set the COLLECTION_NAME environment var!");

    // Register client
    let db = Arc::new(Db::new(
       db_endpoint,
       db_name,
       collection_name
    ).await.expect("Can't connect to database"));

    // Create index
    db.create_ips_index().await;

    start_server(db).await
}

pub fn get_env() {
    env::set_var("RUST_LOG", "actix_web=debug");
    dotenv().ok();

    // Init default logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));
}

fn start_server(db: Arc<dyn DbOps+Send+Sync>) -> Server {
    HttpServer::new(move || {
        App::new()
                .service(web::scope("/api/v1/ipfinder")
                .app_data(web::Data::new(db.clone()))
                .wrap(Logger::default())
                .wrap(Logger::new("%a %{User-Agent}i"))
                .service(health)
                .service(insert_ip)
                .service(get_ip)
                .service(update_ip)
                .service(delete_ip)
        )
    }).workers(5)
        .bind(("127.0.0.1", 8080)).expect("Unable to bind address!")
        .shutdown_timeout(30)
        .run()
}
