//! Recoverable, atomically replaced documents, protected by one process-wide file lock.
use anyhow::{Context, Result, bail};
use deck_domain::{Config, artwork::Artwork};
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub struct Store {
    pub root: PathBuf,
    _lock: File,
}
impl Store {
    pub fn open(root: Option<PathBuf>) -> Result<Self> {
        let root = match root {
            Some(p) => p,
            None => directories::ProjectDirs::from("gr", "Apostolos", "MacropadStudio3")
                .context("Δεν βρέθηκε φάκελος δεδομένων.")?
                .data_local_dir()
                .to_owned(),
        };
        fs::create_dir_all(&root)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join("instance.lock"))?;
        lock.try_lock_exclusive()
            .context("Το Studio 3 είναι ήδη ανοικτό για αυτόν τον φάκελο δεδομένων.")?;
        Ok(Self { root, _lock: lock })
    }
    pub fn load_config(&self) -> Result<Config> {
        let p = self.root.join("config.json");
        if !p.exists() {
            return Ok(Config::default());
        }
        Config::import(&read_bounded(&p,2*1024*1024)?).context("Δεν φορτώθηκε το config.json. Το αρχικό αρχείο διατηρήθηκε· ανάκτησε το config.previous.json.")
    }
    pub fn save_config(&self, c: &Config) -> Result<()> {
        c.validate()?;
        self.write("config.json", &serde_json::to_vec_pretty(c)?)
    }
    pub fn load_artwork(&self) -> Result<Artwork> {
        let p = self.root.join("artwork.json");
        if !p.exists() {
            return Ok(Artwork::default());
        }
        let a: Artwork = serde_json::from_str(&read_bounded(&p, 2 * 1024 * 1024)?)?;
        a.validate()?;
        Ok(a)
    }
    pub fn save_artwork(&self, a: &Artwork) -> Result<()> {
        a.validate()?;
        self.write("artwork.json", &serde_json::to_vec(a)?)
    }
    pub fn write(&self, name: &str, bytes: &[u8]) -> Result<()> {
        atomic_write(&self.root.join(name), bytes, true)
    }
    pub fn secret_path(&self) -> PathBuf {
        self.root.join("ha-credentials.dpapi")
    }
}
pub fn read_bounded(path: &Path, max: usize) -> Result<String> {
    let f = File::open(path)?;
    let mut b = Vec::new();
    f.take(max as u64 + 1).read_to_end(&mut b)?;
    if b.len() > max {
        bail!("Το αρχείο ξεπερνά το επιτρεπόμενο μέγεθος.")
    }
    String::from_utf8(b).context("Το αρχείο δεν είναι UTF-8.")
}
pub fn atomic_write(path: &Path, bytes: &[u8], backup: bool) -> Result<()> {
    let dir = path.parent().context("Λείπει ο φάκελος αρχείου.")?;
    fs::create_dir_all(dir)?;
    if backup && path.exists() {
        let previous = path.with_file_name(format!(
            "{}.previous.{}",
            path.file_stem().unwrap().to_string_lossy(),
            path.extension().unwrap_or_default().to_string_lossy()
        ));
        let old = fs::read(path)?;
        atomic_write(&previous, &old, false)?;
    }
    let mut temp = tempfile::NamedTempFile::new_in(dir)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| e.error)?;
    #[cfg(unix)]
    File::open(dir)?.sync_all()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_backup_and_lock() {
        let d = tempfile::tempdir().unwrap();
        let s = Store::open(Some(d.path().into())).unwrap();
        let mut c = s.load_config().unwrap();
        s.save_config(&c).unwrap();
        c.profiles[0].name = "Changed".into();
        s.save_config(&c).unwrap();
        assert_eq!(s.load_config().unwrap(), c);
        assert!(d.path().join("config.previous.json").exists());
        assert!(Store::open(Some(d.path().into())).is_err());
    }
    #[test]
    fn invalid_config_cannot_replace_saved() {
        let d = tempfile::tempdir().unwrap();
        let s = Store::open(Some(d.path().into())).unwrap();
        s.save_config(&Config::default()).unwrap();
        let mut bad = Config::default();
        bad.profiles.clear();
        assert!(s.save_config(&bad).is_err());
        assert_eq!(s.load_config().unwrap(), Config::default());
    }
    #[test]
    fn corrupt_file_preserved() {
        let d = tempfile::tempdir().unwrap();
        let s = Store::open(Some(d.path().into())).unwrap();
        fs::write(d.path().join("config.json"), "corrupt").unwrap();
        assert!(s.load_config().is_err());
        assert_eq!(
            fs::read_to_string(d.path().join("config.json")).unwrap(),
            "corrupt"
        );
    }
}
