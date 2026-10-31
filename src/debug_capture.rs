//! Opt-in original control/NAV captures. A bounded queue keeps disk I/O off the
//! forwarding task; files are private and the process has a fixed disk budget.
use log::{info, warn};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

pub(crate) const MAX_MESSAGE_BYTES: usize = 256 * 1024;
const MAX_FILES: u64 = 128;
const MAX_DISK_BYTES: u64 = 32 * 1024 * 1024;
const QUEUE_LEN: usize = 4;
static QUEUE: OnceLock<mpsc::Sender<Capture>> = OnceLock::new();
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct Capture {
    directory: PathBuf,
    label: String,
    payload: Vec<u8>,
}

#[derive(Default)]
struct Budget {
    files: u64,
    bytes: u64,
}

impl Budget {
    fn reserve(&mut self, bytes: usize) -> bool {
        if bytes > MAX_MESSAGE_BYTES
            || self.files >= MAX_FILES
            || self.bytes.saturating_add(bytes as u64) > MAX_DISK_BYTES
        {
            return false;
        }
        self.files += 1;
        self.bytes += bytes as u64;
        true
    }
}

pub(crate) fn enqueue(directory: &str, side: &str, channel: u8, id: u16, payload: &[u8]) {
    if directory.is_empty() || payload.len() > MAX_MESSAGE_BYTES {
        return;
    }
    let directory = Path::new(directory);
    if !directory.is_absolute() {
        warn!("packet capture disabled for relative directory");
        return;
    }
    let sender = QUEUE.get_or_init(|| {
        let (tx, mut rx) = mpsc::channel::<Capture>(QUEUE_LEN);
        tokio::spawn(async move {
            let mut budget = Budget::default();
            let mut budget_reported = false;
            while let Some(capture) = rx.recv().await {
                if !budget.reserve(capture.payload.len()) {
                    if !budget_reported {
                        warn!("private packet capture budget exhausted; logging continues without files");
                        budget_reported = true;
                    }
                    continue;
                }
                match write_capture(&capture.directory, &capture.label, &capture.payload).await {
                    Ok(name) => info!("private original packet captured: {} bytes={} sha256={}",
                        name, capture.payload.len(), hex::encode(Sha256::digest(&capture.payload))),
                    Err(_) => warn!("private packet capture failed (path/error details omitted)"),
                }
            }
        });
        tx
    });
    // Reserve a queue slot before allocating/copying a full payload.
    match sender.try_reserve() {
        Ok(slot) => slot.send(Capture {
            directory: directory.to_path_buf(),
            label: format!("{}-ch{:02x}-id{:04x}", side, channel, id),
            payload: payload.to_vec(),
        }),
        Err(_) => warn!("private packet capture queue full; packet file skipped"),
    }
}

async fn write_capture(directory: &Path, label: &str, payload: &[u8]) -> std::io::Result<String> {
    let mut builder = tokio::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    builder.mode(0o700);
    builder.create(directory).await?;
    // Reject symlink/non-private final directories rather than changing permissions
    // on a possibly shared existing location.
    let metadata = tokio::fs::symlink_metadata(directory).await?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
        }
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let seq = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let name = format!(
        "packet-{}-{}-{}-{}.bin",
        std::process::id(),
        stamp,
        seq,
        label
    );
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(directory.join(&name)).await?;
    file.write_all(payload).await?;
    file.flush().await?;
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_bounds_files_message_and_total_bytes() {
        let mut budget = Budget::default();
        assert!(!budget.reserve(MAX_MESSAGE_BYTES + 1));
        for _ in 0..MAX_FILES {
            assert!(budget.reserve(MAX_MESSAGE_BYTES));
        }
        assert_eq!(budget.bytes, MAX_DISK_BYTES);
        assert!(!budget.reserve(1));
    }

    #[tokio::test]
    async fn binary_capture_preserves_original_octets_and_private_permissions() {
        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("private");
        let payload = [0, 6, 0xff, 0, 0xfe];
        let name = write_capture(&directory, "HU-ch00-id0006", &payload)
            .await
            .unwrap();
        let file = directory.join(name);
        assert_eq!(tokio::fs::read(&file).await.unwrap(), payload);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(file).unwrap().permissions().mode() & 0o777,
                0o600
            );
            assert_eq!(
                std::fs::metadata(directory).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn shared_or_symlink_capture_directory_is_rejected() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let temp = tempfile::tempdir().unwrap();
        let shared = temp.path().join("shared");
        std::fs::create_dir(&shared).unwrap();
        std::fs::set_permissions(&shared, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(write_capture(&shared, "test", b"test").await.is_err());
        let link = temp.path().join("link");
        symlink(&shared, &link).unwrap();
        assert!(write_capture(&link, "test", b"test").await.is_err());
    }
}
