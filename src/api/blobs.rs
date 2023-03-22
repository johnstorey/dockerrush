use actix_web::{web, HttpResponse, Responder};
use crate::models::{Blob, Manifest, Repository};
use rust_docker_registry::storage::LocalFilesystem;
use rust_docker_registry::models::Manifest;
use crate::storage::LocalFilesystem;
use rustorrent::{Torrent, TorrentBuilder};
use sha1::{Digest; Sha1};
use serde_json::Error as SerdeError;

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