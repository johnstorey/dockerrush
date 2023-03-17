use actix_web::{web, HttpResponse, Responder};
use crate::models::{Blob, Manifest, Repository};
use crate::storage::LocalFilesystem;

pub async fn get_manifest(
    storage: web::Data<LocalFilesystem>,
    path: web::Path<String>,
) -> impl Responder {
    let manifest_path = format!("manifests/{}.json", path);

    match storage.load(manifest_path).await {
        Ok(manifest_data) => {
            let manifest: Manifest = serde_json::from_slice(&manifest_data)
                .map_err(|err| {
                    HttpResponse::InternalServerError().body(format!("Error parsing manifest: {}", err))
                })?;
            HttpResponse::Ok().json(manifest)
        }
        Err(err) => HttpResponse::NotFound().body(format!("Manifest not found: {}", err)),
    }
}

pub async fn put_manifest(
    storage: web::Data<LocalFilesystem>,
    path: web::Path<String>,
    manifest: web::Json<Manifest>,
) -> impl Responder {
    let manifest_path = format!("manifests/{}.json", path);

    let manifest_data = serde_json::to_vec(&manifest.0)
        .map_err(|err| {
            HttpResponse::InternalServerError().body(format!("Error serializing manifest: {}", err))
        })?;

    match storage.save(manifest_path, &manifest_data).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error saving manifest: {}", err)),
    }
}

/*****
 *
 * In the put_blob function, we now receive the web::HttpRequest and 
 * web::Payload instead of the web::Json<Blob> parameter to support 
 * binary data uploads. We create the blob_path similar to the get_blob 
 * function. We then read the payload and store it in a web::BytesMut buffer. 
 * After reading the payload, we use the storage.save() method to save the 
 * blob data. If the blob is saved successfully, we return a 201 Created 
 * status. If not, we return a 500 Internal Server Error status with an 
 * error message. 
 */
pub async fn get_blob(
    storage: web::Data<LocalFilesystem>,
    path: web::Path<String>,
) -> impl Responder {
    let blob_path = format!("blobs/{}", path);

    match storage.load(&blob_path).await {
        Ok(blob_data) => HttpResponse::Ok().body(blob_data),
        Err(err) => HttpResponse::NotFound().body(format!("Blob not found: {}", err)),
    }
}

pub async fn put_blob(
    storage: web::Data<LocalFilesystem>,
    path: web::Path<String>,
    req: web::HttpRequest,
    mut payload: web::Payload,
) -> impl Responder {
    let blob_path = format!("blobs/{}", path);

    let mut buffer = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(|err| {
            HttpResponse::InternalServerError().body(format!("Error reading payload: {}", err))
        })?;
        buffer.extend_from_slice(&chunk);
    }

    match storage.save(&blob_path, &buffer).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(err) => HttpResponse::InternalServerError().body(format!("Error saving blob: {}", err)),
    }
}

/******
 * 
 * In the get_repository function, we first create the repo_path by using the given repository name in the path parameter. We then use the storage.load() method to fetch the repository data. If the repository is found, we deserialize it into a Repository object and return it with a 200 OK status. If not, we return a 404 Not Found status with an error message.

 * In the create_repository function, we first create the repo_path in 
 * the same way as in the get_repository function. We then check if 
 * the repository already exists using the storage.exists() method. 
 * If the repository exists, we return a 409 Conflict status with an 
 * error message. If not, we serialize the provided Repository object, 
 * use the storage.save() method to save the repository data, 
 * and return a 201 Created status. 
 * If there's an error during the creation, 
 * we return a 500 Internal Server Error status with an error message.
 * 
 */


pub async fn get_repository(
    storage: web::Data<LocalFilesystem>,
    path: web::Path<String>,
) -> impl Responder {
    // Implement the logic to fetch a repository by its name
}

pub async fn create_repository(
    storage: web::Data<LocalFilesystem>,
    path: web::Path<String>,
    repository: web::Json<Repository>,
) -> impl Responder {
    // Implement the logic to create a new repository
}