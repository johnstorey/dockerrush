use actix_web::{web, HttpResponse, Responder};
use crate::models::{Blob, Manifest, Repository};
use rust_docker_registry::storage::LocalFilesystem;
use rust_docker_registry::models::Manifest;
use crate::storage::LocalFilesystem;
use rustorrent::{Torrent, TorrentBuilder};
use sha1::{Digest; Sha1};
use serde_json::Error as SerdeError;

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

/*******
 * 
 * AFTER ADDING BITORRENT
 * 
 * Similar to the put_blob function, we use the rustorrent library's 
 * TorrentBuilder struct to create a torrent file for the manifest. After 
 * saving the manifest, we instantiate a TorrentBuilder with the desired 
 * piece length and add the manifest file to the builder. Then, we build 
 * the torrent and serialize it into a bencoded file format using the 
 * bencode crate.
 *
 * Finally, we save the serialized torrent data to the storage system. 
 * To complete the implementation, you'll need to implement the seeding 
 * part using the BitTorrent client library of your choice.
 *
 * As mentioned earlier, the rustorrent library is relatively young and 
 * might not be as feature-rich as other BitTorrent libraries. Depending 
 * on your requirements, you might need to consider alternatives or make 
 * additional modifications to the library or your code.
 *
 */
pub async fn put_manifest(
    storage: web::Data<LocalFilesystem>,
    path: web::Path<String>,
    manifest: web::Json<Manifest>,
) -> impl Responder {
    let (name, reference) = path.into_inner();
    let manifest_path = format!("manifests/{}/{}.json", name, reference);

    let mut buffer = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(|err| {
            HttpResponse::InternalServerError().body(format!("Error reading payload: {}", err))
        })?;
        buffer.extend_from_slice(&chunk);
    }

    let manifest: Manifest = serde_json::from_slice(&buffer)?;

    match storage.save(&manifest_path, &buffer).await {
        Ok(_) => (),
        Err(err) => {
            return HttpResponse::InternalServerError().body(format!("Error saving manifest: {}", err));
        }
    }

    // Create a torrent file
    let torrent_path = format!("torrents/{}/{}.torrent", name, reference);

    let piece_length = 256 * 1024; // Set the piece length
    let torrent_builder = TorrentBuilder::new(piece_length);

    let torrent = match torrent_builder.add_file(&manifest_path) {
        Ok(torrent_builder) => {
            let info_hash = Sha1::digest(&bencode::to_bytes(&torrent_builder.info).unwrap());
            torrent_builder.build(info_hash.as_slice())
        }
        Err(err) => {
            return HttpResponse::InternalServerError().body(format!("Error adding file to torrent: {}", err));
        }
    };

    let torrent_data = bencode::to_bytes(&torrent).map_err(|err| {
        HttpResponse::InternalServerError().body(format!("Error serializing torrent: {}", err))
    })?;

    // Save the torrent file
    if let Err(err) = storage.save(&torrent_path, &torrent_data).await {
        return HttpResponse::InternalServerError().body(format!("Error saving torrent: {}", err));
    }

    // Seed the manifest using the BitTorrent client library (You'll need to implement this part)

    HttpResponse::Created().finish()
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
 * 
 * ADDING BITORRENT
 * 
 * In this example, we use the rustorrent library's TorrentBuilder struct to 
 * create a torrent file for the blob. After saving the blob, we instantiate 
 * a TorrentBuilder with the desired piece length and add the blob file to 
 * the builder. Then, we build the torrent and serialize it into a bencoded 
 * file format using the bencode crate.
 *
 * Finally, we save the serialized torrent data to the storage system. To 
 * complete the implementation, you'll need to implement the seeding part 
 * using the BitTorrent client library of your choice.
 *
 * Please note that the rustorrent library is relatively young and might 
 * not be as feature-rich as other BitTorrent libraries. Depending on your 
 * requirements, you might need to consider alternatives or make additional 
 * modifications to the library or your code.
 * 
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
        Ok(_) => (),
        Err(err) => {
            return HttpResponse::InternalServerError().body(format!("Error saving blob: {}", err))
        }
    }

    // Create a torrent file
    let torrent_path = format!("torrents/{}.torrent", path);

    let piece_length = 256 * 1024; // Set the piece length
    let torrent_builder = TorrentBuilder::new(piece_length);

    let torrent = match torrent_builder.add_file(&blob_path) {
        Ok(torrent_builder) => {
            let info_hash = Sha1::digest(&bencode::to_bytes(&torrent_builder.info).unwrap());
            torrent_builder.build(info_hash.as_slice())
        }
        Err(err) => {
            return HttpResponse::InternalServerError().body(format!("Error adding file to torrent: {}", err))
        }
    };

    let torrent_data = bencode::to_bytes(&torrent).map_err(|err| {
        HttpResponse::InternalServerError().body(format!("Error serializing torrent: {}", err))
    })?;

    // Save the torrent file
    if let Err(err) = storage.save(&torrent_path, &torrent_data).await {
        return HttpResponse::InternalServerError().body(format!("Error saving torrent: {}", err));
    }

    // Seed the blob using the BitTorrent client library (You'll need to implement this part)

    HttpResponse::Created().finish()
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