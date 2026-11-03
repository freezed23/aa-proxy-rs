//! Shared configuration update policy and durable same-directory writes.
use std::{fs::{self, OpenOptions}, io::{self, Write}, path::Path, sync::atomic::{AtomicU64, Ordering}};
#[cfg(unix)] use std::os::unix::fs::OpenOptionsExt;
static SEQ: AtomicU64 = AtomicU64::new(0);
/// Root keys absent from older clients retain their current values. Explicit false is honored.
pub(crate) fn merge_update(mut current: serde_json::Value, patch: serde_json::Value) -> Result<serde_json::Value, &'static str> {
    let target = current.as_object_mut().ok_or("current config must be an object")?;
    let updates = patch.as_object().ok_or("config update must be an object")?;
    for (key, value) in updates { target.insert(key.clone(), value.clone()); }
    Ok(current)
}
pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let name = path.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing config filename"))?.to_string_lossy();
    let temp = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), SEQ.fetch_add(1, Ordering::Relaxed)));
    let result = (|| {
        let mut options = OpenOptions::new(); options.write(true).create_new(true);
        #[cfg(unix)] options.mode(0o600);
        let mut file = options.open(&temp)?;
        file.write_all(bytes)?; file.sync_all()?; drop(file);
        fs::rename(&temp, path)?;
        fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() { let _ = fs::remove_file(&temp); }
    result
}
#[cfg(test)] mod tests {
 use super::*; use serde_json::json;
 #[test] fn omitted_enum_is_preserved_and_explicit_false_works() {
   let c=json!({"nav_sdr_enum_to_phone_enabled":true,"mitm":true});
   let x=merge_update(c,json!({"mitm":false})).unwrap(); assert_eq!(x["nav_sdr_enum_to_phone_enabled"],true);
   let x=merge_update(x,json!({"nav_sdr_enum_to_phone_enabled":false})).unwrap(); assert_eq!(x["nav_sdr_enum_to_phone_enabled"],false); assert_eq!(x["mitm"],false);
 }
 #[test] fn invalid_root_rejected() { assert!(merge_update(json!({}),json!([])).is_err()); }
 #[test] fn durable_write_reload_and_replacement() {
   let d=tempfile::tempdir().unwrap(); let p=d.path().join("config.toml");
   atomic_write(&p,b"nav_sdr_enum_to_phone_enabled = true\n").unwrap();
   assert!(fs::read_to_string(&p).unwrap().contains("true"));
   atomic_write(&p,b"nav_sdr_enum_to_phone_enabled = false\n").unwrap();
   assert!(fs::read_to_string(&p).unwrap().contains("false")); assert_eq!(fs::read_dir(d.path()).unwrap().count(),1);
   #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; assert_eq!(fs::metadata(&p).unwrap().permissions().mode() & 0o777,0o600); }
 }
 #[test] fn failed_replace_preserves_existing_target_and_cleans_temp() {
   let d=tempfile::tempdir().unwrap(); let p=d.path().join("config.toml"); fs::create_dir(&p).unwrap();
   assert!(atomic_write(&p,b"bad").is_err()); assert!(p.is_dir()); assert_eq!(fs::read_dir(d.path()).unwrap().count(),1);
 }
}
