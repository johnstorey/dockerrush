use actix_web::{web, App, HttpServer};
use rust_docker_registry::api::*;
use rust_docker_registry::storage::LocalFilesystem;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let storage = LocalFilesystem::new("./data");

    HttpServer::new(move || {
        App::new()
        .data(storage.clone())
        // Configure your routes here.
        .route("/v2/{name}/manifests/{reference}", web::get().to(get_manifest))
        .route("/v2/{name}/manifests/{reference}", web::put().to(put_manifest))
        .route("/v2/{name}/blobs/{digest}", web::get().to(get_blob))
        .route("/v2/{name}/blobs/{digest}", web::put().to(put_blob))
        .route("/v2/{name}", web::get().to(get_repository))
        .route("/v2/{name}", web::put().to(create_repository))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}