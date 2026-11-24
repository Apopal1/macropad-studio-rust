//! Single owner of the HID handle. OLED transfers cannot stall physical key dispatch.
use super::*;
use std::sync::Mutex;
#[derive(Clone)]
pub struct Update {
    pub connected: bool,
    pub message: String,
    pub firmware: String,
    pub mode: Option<u8>,
    pub revision: u64,
}
enum Work {
    Refresh,
    Upload(u8, Clip),
}
pub struct DeviceWorker {
    desired: Arc<Mutex<(Config, u64, bool)>>,
    tx: Sender<Work>,
}
impl DeviceWorker {
    pub fn start(config: Config, events: Sender<Internal>, stop: Arc<AtomicBool>) -> Self {
        let desired = Arc::new(Mutex::new((config, 1, true)));
        let (tx, rx) = bounded(4);
        let state = desired.clone();
        std::thread::spawn(move || {
            let mut device: Option<UsbDeck> = None;
            let mut applied = 0;
            let mut last = Instant::now() - Duration::from_secs(10);
            while !stop.load(Ordering::Relaxed) {
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(Work::Refresh) => {
                        device = None;
                        applied = 0;
                        last = Instant::now() - Duration::from_secs(10);
                    }
                    Ok(Work::Upload(key, clip)) => {
                        let r = device
                            .as_mut()
                            .context("Σύνδεσε πρώτα το USB-NATIVE.")
                            .and_then(|d| d.upload(key, &clip));
                        let error = r.is_err();
                        let message = r
                            .map(|_| format!("Το pad επιβεβαίωσε την αποθήκευση γραφικού K{key}."))
                            .unwrap_or_else(|e| format!("Αποτυχία μεταφοράς OLED: {e}"));
                        let _ = events.send(Internal::Notice { error, message });
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                    _ => {}
                }
                if last.elapsed() < Duration::from_secs(2) {
                    continue;
                }
                last = Instant::now();
                let (c, revision, select) = state.lock().unwrap().clone();
                let r = (|| -> Result<Update> {
                    if device.is_none() {
                        device = Some(UsbDeck::open()?);
                        applied = 0;
                    }
                    let d = device.as_mut().unwrap();
                    let mut s = d.status()?;
                    if applied != revision && !s.busy {
                        d.configure(&c, select)?;
                        applied = revision;
                        s = d.status()?;
                    }
                    Ok(Update {
                        connected: true,
                        message: if applied == revision {
                            "USB-NATIVE · modes συγχρονισμένα"
                        } else {
                            "USB-NATIVE · αναμονή συγχρονισμού"
                        }
                        .into(),
                        firmware: s.firmware,
                        mode: if applied == revision {
                            Some(s.mode)
                        } else {
                            None
                        },
                        revision,
                    })
                })();
                let u = match r {
                    Ok(u) => u,
                    Err(e) => {
                        device = None;
                        applied = 0;
                        Update {
                            connected: false,
                            message: e.to_string(),
                            firmware: String::new(),
                            mode: None,
                            revision,
                        }
                    }
                };
                let _ = events.try_send(Internal::Device(u));
            }
        });
        Self { desired, tx }
    }
    pub fn update(&self, config: &Config, select: bool) {
        let mut s = self.desired.lock().unwrap();
        let revision = s.1 + 1;
        *s = (config.clone(), revision, select);
    }
    pub fn revision(&self) -> u64 {
        self.desired.lock().unwrap().1
    }
    pub fn refresh(&self) -> Result<()> {
        self.tx
            .try_send(Work::Refresh)
            .map_err(|_| anyhow::anyhow!("Η ουρά USB είναι γεμάτη."))
    }
    pub fn upload(&self, key: u8, clip: Clip) -> Result<()> {
        self.tx
            .try_send(Work::Upload(key, clip))
            .map_err(|_| anyhow::anyhow!("Η ουρά USB είναι γεμάτη."))
    }
}
