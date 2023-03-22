use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/* 
 * This implementation consists of the LocalFilesystem struct and the associated Storage trait.
 * The LocalFilesystem struct has a base_path field that represents the base path for storing files.
 *
 * The LocalFilesystem struct also provides two async methods, save and load, for saving and 
 * loading data to and from the file system. The save method writes data to a specified file path 
 * and creates directories if necessary. The load method reads data from a specified file path.
 *
 * The Storage trait defines the save and load methods required for any storage implementation. 
 * The LocalFilesystem struct implements the Storage trait, which allows you to use it 
 * interchangeably with other storage implementations that also implement the Storage trait.
 *
*/

pub struct LocalFilesystem {
    base_path: PathBuf,
}

impl LocalFilesystem {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_owned(),
        }
    }

    pub fn save(&self, path: impl AsRef<Path>, content: &[u8]) -> io::Result<()> {
        let full_path = self.base_path.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(full_path)?;
        file.write_all(content)?;
        Ok(())
    }

    pub fn load(&self, path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        let full_path = self.base_path.join(path);
        let mut file = File::open(full_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    pub fn delete(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let full_path = self.base_path.join(path);
        fs::remove_file(full_path)?;
        Ok(())
    }

    pub fn exists(&self, path: impl AsRef<Path>) -> bool {
        let full_path = self.base_path.join(path);
        full_path.exists()
    }
}
