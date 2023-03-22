pub mod local_filesystem;

pub use local_filesystem::LocalFilesystem;

use async_trait::async_trait;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn save(&self, path: &str, data: &[u8]) -> std::io::Result<()>;
    async fn load(&self, path: &str) -> std::io::Result<Vec<u8>>;
}
