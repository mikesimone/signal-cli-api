use base64::Engine;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// signal-cli's `avatar` param (both CLI and JSON-RPC, for profiles and
/// groups alike) is a local file path, not inline image data - passing
/// base64 straight through fails with "File name too long". Spill it to disk
/// first, same as `messages::spill_attachments_to_disk` does for message
/// attachments, and clean up once the RPC call returns.
const AVATAR_SPILL_DIR: &str = "outgoing-attachments";
static AVATAR_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct SpilledAvatar(Option<PathBuf>);

impl Drop for SpilledAvatar {
    fn drop(&mut self) {
        if let Some(path) = &self.0 {
            let _ = std::fs::remove_file(path);
        }
    }
}

pub fn spill_avatar_to_disk(base64_avatar: &str) -> std::io::Result<(PathBuf, SpilledAvatar)> {
    let dir = std::path::Path::new(AVATAR_SPILL_DIR);
    std::fs::create_dir_all(dir)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_avatar)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let unique = AVATAR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = dir.join(format!("avatar-{}-{unique}.jpg", std::process::id()));
    std::fs::write(&path, &bytes)?;
    Ok((path.clone(), SpilledAvatar(Some(path))))
}
