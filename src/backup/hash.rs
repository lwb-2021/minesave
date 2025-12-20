use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use ring::digest::{Digest, SHA256};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};
use walkdir::WalkDir;

use crate::error::Result;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sha256Sum([u64; 4]);
impl From<Digest> for Sha256Sum {
    fn from(value: Digest) -> Self {
        let mut array = [0u8; 32];
        array.copy_from_slice(value.as_ref());
        Self(unsafe { std::mem::transmute::<[u8; 32], [u64; 4]>(array) })
    }
}

pub async fn create_full_copy_with_hash<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q,
) -> Result<HashMap<PathBuf, Sha256Sum>> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    let mut results = HashMap::new();
    let mut handles = Vec::new();
    for entry in WalkDir::new(src) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        let src_path = path.to_path_buf();
        let relative_path = path.strip_prefix(src).unwrap().to_path_buf();
        let target_path = dst.join(&relative_path);
        let parent = target_path.parent().map(|p| p.to_path_buf());

        let handle = tokio::spawn(async move {
            if let Some(parent) = &parent {
                fs::create_dir_all(parent).await?;
            }

            let hash = copy_file_and_hash(&src_path, &target_path).await?;
            Ok((relative_path, hash))
        });
        handles.push(handle);
    }

    for handle in handles {
        match handle.await {
            Ok(Ok((k, v))) => results.insert(k, v),
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(e.into()),
        };
    }

    Ok(results)
}
pub async fn hash<P: AsRef<Path>>(src: P) -> Result<Sha256Sum> {
    let src = src.as_ref();
    let mut src_file = File::open(src).await?;
    let mut context = ring::digest::Context::new(&SHA256);

    let mut buffer = vec![0u8; 8192];
    loop {
        let bytes_read = src_file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        context.update(&buffer[..bytes_read]);
    }
    Ok(context.finish().into())
}

async fn copy_file_and_hash<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<Sha256Sum> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    let mut src_file = File::open(src).await?;
    let mut dst_file = File::create(dst).await?;
    let mut context = ring::digest::Context::new(&SHA256);
    let mut buffer = vec![0u8; 8192];

    loop {
        let bytes_read = src_file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        context.update(&buffer[..bytes_read]);
        dst_file.write_all(&buffer[..bytes_read]).await?;
    }

    Ok(context.finish().into())
}
