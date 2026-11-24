//! One bounded background I/O operation at a time; native dialogs never block the input controller.
use super::*;
pub fn launch(
    busy: &Arc<AtomicBool>,
    tx: Sender<Internal>,
    task: impl FnOnce() -> Result<Vec<Event>> + Send + 'static,
) -> Result<()> {
    if busy.swap(true, Ordering::Relaxed) {
        bail!("Υπάρχει ήδη επιλογή αρχείου ή σύνδεση σε εξέλιξη.");
    }
    let busy = busy.clone();
    std::thread::spawn(move || {
        let result = task().map_err(|e| format!("{e:#}"));
        busy.store(false, Ordering::Relaxed);
        let _ = tx.send(Internal::IoDone(result));
    });
    Ok(())
}
pub fn file_command(c: Command) -> Result<Vec<Event>> {
    let mut events = Vec::new();
    match c {
        Command::ImportConfig => {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("Ρυθμίσεις JSON", &["json"])
                .pick_file()
            {
                events.push(Event::Imported(Config::import(&read_bounded(
                    &p,
                    2 * 1024 * 1024,
                )?)?));
            }
        }
        Command::ExportConfig(c) => {
            c.validate()?;
            if let Some(p) = rfd::FileDialog::new()
                .set_file_name("macropad-profiles.json")
                .add_filter("JSON", &["json"])
                .save_file()
            {
                atomic_write(&p, &serde_json::to_vec_pretty(&c)?, false)?;
            }
        }
        Command::ImportImage(key) => {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("Εικόνα / μικρό GIF", &["png", "jpg", "jpeg", "gif"])
                .pick_file()
            {
                events.push(Event::ArtworkImported {
                    key,
                    clip: images::load_clip(&p)?,
                });
            }
        }
        Command::ExportArtwork(a) => {
            a.validate()?;
            if let Some(p) = rfd::FileDialog::new()
                .set_file_name("macropad-artwork.json")
                .add_filter("JSON", &["json"])
                .save_file()
            {
                atomic_write(&p, &serde_json::to_vec(&a)?, false)?;
            }
        }
        Command::ImportArtwork => {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("Artwork v3", &["json"])
                .pick_file()
            {
                let a: Artwork = serde_json::from_str(&read_bounded(&p, 2 * 1024 * 1024)?)?;
                a.validate()?;
                for (i, c) in a.keys.into_iter().enumerate() {
                    events.push(Event::ArtworkImported {
                        key: i as u8 + 1,
                        clip: c,
                    });
                }
            }
        }
        Command::PickApplication { profile, key, step } => {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("Windows application", &["exe", "lnk"])
                .pick_file()
            {
                events.push(Event::ApplicationPicked {
                    profile,
                    key,
                    step,
                    path: p.to_string_lossy().into(),
                });
            }
        }
        _ => bail!("Μη έγκυρη εντολή αρχείων."),
    };
    Ok(events)
}
