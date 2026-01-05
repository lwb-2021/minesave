use std::{
    collections::HashMap,
    fmt::{Display, Write},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use async_compression::tokio::write::ZstdEncoder;
use log::{debug, error, warn};
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
impl Display for Sha256Sum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{:016x}{:016x}{:016x}{:016x}",
            self.0[0], self.0[1], self.0[2], self.0[3],
        ))
    }
}
impl From<Digest> for Sha256Sum {
    fn from(value: Digest) -> Self {
        let mut array = [0u8; 32];
        array.copy_from_slice(value.as_ref());
        Self(unsafe { std::mem::transmute::<[u8; 32], [u64; 4]>(array) })
    }
}

pub async fn copy_to_storage<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q,
) -> Result<HashMap<PathBuf, Sha256Sum>> {
    fs::create_dir_all(&dst).await?;

    let mut results = HashMap::new();
    let mut handles = Vec::new();

    for entry in WalkDir::new(&src) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        let src_path = path.to_path_buf();
        let relative_path = path.strip_prefix(&src).unwrap().to_path_buf();
        let dst_cloned = dst.as_ref().to_path_buf();

        let handle = tokio::spawn(async move {
            let hash = copy_file_to_storage(
                src_path,
                dst_cloned,
                relative_path.to_string_lossy().to_string(),
            )
            .await?;
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
pub async fn hash_diff<P: AsRef<Path>>(
    src: P,
    old_hash: &HashMap<PathBuf, Sha256Sum>,
) -> Result<HashMap<PathBuf, Sha256Sum>> {
    let mut result = HashMap::new();
    for entry in WalkDir::new(&src) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        let src_path = path.to_path_buf();
        let relative_path = path.strip_prefix(&src).unwrap().to_path_buf();
        let new_hash = hash(&src_path).await?;

        if let Some(old) = old_hash.get(&relative_path)
            && old == &new_hash
        {
            continue;
        }

        result.insert(relative_path, new_hash);
    }

    Ok(result)
}
pub async fn hash<P: AsRef<Path>>(src: P) -> Result<Sha256Sum> {
    let mut src_file = File::open(&src).await?;
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

pub async fn copy_file_to_storage<P: AsRef<Path>, Q: AsRef<Path>>(
    src: P,
    dst: Q,
    name: String,
) -> Result<Sha256Sum> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    let tmp_name = name.replace("/", ".");

    let mut src_file = File::open(src).await?;
    let dst_file = File::create(dst.join(&tmp_name)).await?;
    let mut encoder = ZstdEncoder::with_quality(dst_file, async_compression::Level::Precise(15));
    let mut context = ring::digest::Context::new(&SHA256);
    let mut buffer = vec![0u8; 8192];

    loop {
        let bytes_read = src_file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        context.update(&buffer[..bytes_read]);
        encoder.write_all(&buffer[..bytes_read]).await?;
    }
    let hash: Sha256Sum = context.finish().into();

    encoder.flush().await?;
    encoder.into_inner().shutdown().await?;

    let dst_file = dst.join(hash.to_string()).with_extension("zst");
    if std::fs::exists(&dst_file)? {
        fs::remove_file(dst.join(&tmp_name)).await?;
        debug!("exists: {:?}, hash={}", name, hash);
        return Ok(hash);
    }
    debug!("copied: {:?}, hash={}", name, hash);
    fs::rename(dst.join(&tmp_name), &dst_file).await?;
    Ok(hash)
}
