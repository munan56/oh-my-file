use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, Serialize)]
pub struct TransferProgress {
    pub id: String,
    pub filename: String,
    pub device_name: String,
    pub total_bytes: u64,
    pub received_bytes: u64,
    pub status: String,
    pub save_path: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Clone)]
pub struct TransferEntry {
    pub progress: TransferProgress,
    pub cancel_token: CancellationToken,
}

#[derive(Clone, Default)]
pub struct TransferStore {
    inner: Arc<RwLock<HashMap<String, TransferEntry>>>,
}

impl TransferStore {
    pub async fn insert(&self, progress: TransferProgress) -> CancellationToken {
        let cancel_token = CancellationToken::new();
        let entry = TransferEntry {
            progress: progress.clone(),
            cancel_token: cancel_token.clone(),
        };

        self.inner.write().await.insert(progress.id.clone(), entry);
        cancel_token
    }

    pub async fn update_progress(&self, id: &str, received_bytes: u64) {
        if let Some(entry) = self.inner.write().await.get_mut(id) {
            entry.progress.received_bytes = received_bytes;
        }
    }

    pub async fn complete(&self, id: &str, save_path: String) {
        if let Some(entry) = self.inner.write().await.get_mut(id) {
            entry.progress.status = "completed".into();
            entry.progress.save_path = Some(save_path);
            entry.progress.error_message = None;
        }
    }

    pub async fn fail(&self, id: &str, message: String) {
        if let Some(entry) = self.inner.write().await.get_mut(id) {
            entry.progress.status = "error".into();
            entry.progress.error_message = Some(message);
        }
    }

    pub async fn cancel(&self, id: &str) -> bool {
        if let Some(entry) = self.inner.write().await.get_mut(id) {
            entry.progress.status = "cancelled".into();
            entry.progress.error_message = Some("用户取消传输".into());
            entry.cancel_token.cancel();
            return true;
        }

        false
    }

    pub async fn snapshot(&self, id: &str) -> Option<TransferProgress> {
        self.inner
            .read()
            .await
            .get(id)
            .map(|entry| entry.progress.clone())
    }
}

pub fn resolve_target_path(base_dir: &Path, filename: &str) -> PathBuf {
    let candidate = base_dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }

    let path = Path::new(filename);
    let stem = path.file_stem().and_then(|value| value.to_str()).unwrap_or("file");
    let ext = path.extension().and_then(|value| value.to_str()).unwrap_or("");

    for index in 1..1000 {
        let next = if ext.is_empty() {
            base_dir.join(format!("{stem}({index})"))
        } else {
            base_dir.join(format!("{stem}({index}).{ext}"))
        };

        if !next.exists() {
            return next;
        }
    }

    candidate
}

pub fn directory_path(path: &Path) -> Option<String> {
    path.parent().map(|parent| parent.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::{directory_path, resolve_target_path};
    use std::fs;
    use std::path::Path;

    #[test]
    fn appends_incrementing_suffix_for_conflicts() {
        let root = std::env::temp_dir().join(format!("oh-my-file-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create temp dir");
        fs::write(root.join("photo.jpg"), b"first").expect("write file");

        let resolved = resolve_target_path(&root, "photo.jpg");
        assert!(resolved.ends_with("photo(1).jpg"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn extracts_parent_directory() {
        let path = Path::new(r"D:\temp\photo.jpg");
        assert_eq!(directory_path(path), Some(r"D:\temp".to_string()));
    }
}
