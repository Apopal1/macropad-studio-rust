//! Typed application boundary. All blocking work is owned by background threads.
mod device;
mod images;
mod io;
use anyhow::{Context, Result, bail};
use crossbeam_channel::{Receiver, Sender, bounded, never, select};
use deck_domain::{
    Action, ActionKind, Config,
    artwork::{Artwork, Clip},
};
use deck_platform::{
    InputListener, InputStatus, PadInput,
    ha::{Catalog, HomeAssistant},
    usb::UsbDeck,
};
use deck_storage::{Store, atomic_write, read_bounded};
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
#[derive(Clone, Default)]
pub struct Status {
    pub usb_connected: bool,
    pub usb_message: String,
    pub firmware: String,
    pub input: InputStatus,
    pub paused: bool,
    pub ha_connected: bool,
    pub ha_url: String,
    pub demo: bool,
    pub data_dir: String,
    pub running_jobs: usize,
    pub io_busy: bool,
}
#[derive(Clone)]
pub struct Activity {
    pub message: String,
    pub error: bool,
    pub time: String,
}
#[allow(clippy::large_enum_variant)]
pub enum Event {
    Ready {
        config: Config,
        artwork: Artwork,
        status: Status,
    },
    Saved(Config),
    Imported(Config),
    ArtworkImported {
        key: u8,
        clip: Clip,
    },
    ArtworkSaved(Artwork),
    Status(Status),
    Catalog(Catalog),
    Activity(Activity),
    ApplicationPicked {
        profile: String,
        key: u8,
        step: usize,
        path: String,
    },
    Notice(String),
    Fatal(String),
}
pub enum Command {
    Save(Config),
    Activate(String),
    Pause(bool),
    Test {
        profile: String,
        key: u8,
        simulate: bool,
    },
    ImportConfig,
    ExportConfig(Config),
    SaveArtwork(Artwork),
    ImportImage(u8),
    ExportArtwork(Artwork),
    ImportArtwork,
    Upload {
        key: u8,
        clip: Clip,
    },
    ConnectHa {
        url: String,
        token: String,
    },
    RefreshHa,
    DisconnectHa,
    RefreshDevice,
    PickApplication {
        profile: String,
        key: u8,
        step: usize,
    },
    Shutdown,
}
// File selection results contain no user data in activity logs.
enum Internal {
    Device(device::Update),
    IoDone(std::result::Result<Vec<Event>, String>),
    Notice { error: bool, message: String },
    Mode(Option<String>),
    Done { error: bool, message: String },
}
// Application picker is surfaced as a typed event, not an untyped path/message channel.
pub struct Runtime {
    pub events: Receiver<Event>,
    tx: Sender<Command>,
    stop: Arc<AtomicBool>,
    epoch: Arc<AtomicU64>,
}
impl Runtime {
    pub fn start(demo: bool, root: Option<PathBuf>, wake: Arc<dyn Fn() + Send + Sync>) -> Self {
        let (tx, rx) = bounded(32);
        let (ev, events) = bounded(128);
        let stop = Arc::new(AtomicBool::new(false));
        let epoch = Arc::new(AtomicU64::new(0));
        let thread_stop = stop.clone();
        let thread_epoch = epoch.clone();
        std::thread::spawn(move || {
            if let Err(e) = controller(
                demo,
                root,
                rx,
                ev.clone(),
                wake.clone(),
                thread_stop,
                thread_epoch,
            ) {
                let _ = ev.try_send(Event::Fatal(format!("{e:#}")));
                wake();
            }
        });
        Self {
            events,
            tx,
            stop,
            epoch,
        }
    }
    pub fn send(&self, c: Command) -> Result<()> {
        self.tx.try_send(c).map_err(|_| {
            anyhow::anyhow!(
                "Η ουρά είναι γεμάτη ή η υπηρεσία σταμάτησε. Περίμενε και δοκίμασε ξανά."
            )
        })
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.epoch.fetch_add(1, Ordering::Relaxed);
        let _ = self.tx.try_send(Command::Shutdown);
    }
}
struct Out {
    tx: Sender<Event>,
    wake: Arc<dyn Fn() + Send + Sync>,
    history: VecDeque<Activity>,
}
impl Out {
    fn send(&self, e: Event) {
        let _ = self.tx.try_send(e);
        (self.wake)();
    }
    fn log(&mut self, message: impl Into<String>, error: bool) {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            % 86400;
        let a = Activity {
            message: message.into(),
            error,
            time: format!(
                "{:02}:{:02}:{:02} UTC",
                secs / 3600,
                secs / 60 % 60,
                secs % 60
            ),
        };
        if self.history.len() >= 200 {
            self.history.pop_front();
        }
        self.history.push_back(a.clone());
        self.send(Event::Activity(a));
    }
}
struct Job {
    actions: Vec<Action>,
    simulate: bool,
    epoch: u64,
    test_delay: bool,
}
fn controller(
    demo: bool,
    root: Option<PathBuf>,
    commands: Receiver<Command>,
    events: Sender<Event>,
    wake: Arc<dyn Fn() + Send + Sync>,
    stop: Arc<AtomicBool>,
    epoch: Arc<AtomicU64>,
) -> Result<()> {
    let store = Store::open(root)?;
    let mut config = store.load_config()?;
    let mut artwork = store.load_artwork()?;
    let ha = HomeAssistant::new()?;
    let listener = if demo {
        None
    } else {
        Some(InputListener::start())
    };
    let mut input_rx = listener
        .as_ref()
        .map(|l| l.inputs.clone())
        .unwrap_or_else(never);
    let (jobs_tx, jobs_rx) = bounded::<Job>(8);
    let (internal_tx, internal) = bounded::<Internal>(32);
    let paused = Arc::new(AtomicBool::new(false));
    {
        let ha = ha.clone();
        let tx = internal_tx.clone();
        let stop = stop.clone();
        let epoch = epoch.clone();
        let pause = paused.clone();
        std::thread::spawn(move || executor(jobs_rx, tx, ha, stop, epoch, pause));
    }
    let mut out = Out {
        tx: events,
        wake,
        history: VecDeque::new(),
    };
    let mut status = Status {
        demo,
        data_dir: store.root.to_string_lossy().into(),
        usb_message: if demo {
            "Προσομοίωση · χωρίς πρόσβαση σε συσκευή"
        } else {
            "USB δεν έχει ελεγχθεί"
        }
        .into(),
        ..Default::default()
    };
    out.send(Event::Ready {
        config: config.clone(),
        artwork: artwork.clone(),
        status: status.clone(),
    });
    out.log(
        if demo {
            "Η προσομοίωση είναι ενεργή. Δεν εκτελούνται εξωτερικές ενέργειες."
        } else {
            "Το native Studio ξεκίνησε."
        },
        false,
    );
    let io_busy = Arc::new(AtomicBool::new(false));
    if !demo && store.secret_path().exists() {
        let h = ha.clone();
        let path = store.secret_path();
        io::launch(&io_busy, internal_tx.clone(), move || {
            Ok(vec![Event::Catalog(h.restore(&std::fs::read(path)?)?)])
        })?;
    }
    let device = if !demo && cfg!(windows) {
        Some(device::DeviceWorker::start(
            config.clone(),
            internal_tx.clone(),
            stop.clone(),
        ))
    } else {
        None
    };
    if !demo && !cfg!(windows) {
        status.usb_message = "Προεπισκόπηση Linux · USB/Windows adapter μη διαθέσιμος".into();
        out.send(Event::Status(status.clone()));
    }
    while !stop.load(Ordering::Relaxed) {
        if let Some(l) = &listener {
            while let Ok(s) = l.status.try_recv() {
                status.input = s;
                out.send(Event::Status(status.clone()));
            }
            status.input.dropped = l.drop_count();
        }
        select! {
         recv(commands)->c=>{let Ok(c)=c else{break};let result:Result<()>= (|| {match c {
          Command::Shutdown=>{stop.store(true,Ordering::Relaxed);},
          Command::Save(c)=>{c.validate()?;store.save_config(&c)?;config=c;if let Some(d)=&device{d.update(&config,true);}out.send(Event::Saved(config.clone()));out.log("Αποθηκεύτηκαν τα modes τοπικά.",false);},
          Command::Activate(id)=>{activate(&mut config,&store,id)?;if let Some(d)=&device{d.update(&config,true);}out.send(Event::Saved(config.clone()));out.log("Άλλαξε το ενεργό mode της εφαρμογής.",false);},
          Command::Pause(value)=>{paused.store(value,Ordering::Relaxed);epoch.fetch_add(1,Ordering::Relaxed);status.paused=value;out.log(if value{"Τα macros τέθηκαν σε παύση· ακυρώθηκαν οι εκκρεμείς ακολουθίες."}else{"Τα macros συνεχίζονται."},false);},
          Command::Test{profile,key,simulate}=>{queue_key(&config,&profile,key,simulate||demo,true,&jobs_tx,&epoch,&paused)?;status.running_jobs+=1;out.log(if simulate||demo{"Προγραμματίστηκε προσομοίωση του αποθηκευμένου πλήκτρου."}else{"Δοκιμή σε 3 δευτερόλεπτα. Επίλεξε το παράθυρο στόχο."},false);},
          Command::SaveArtwork(a)=>{store.save_artwork(&a)?;artwork=a;out.send(Event::ArtworkSaved(artwork.clone()));out.log("Αποθηκεύτηκαν τα γραφικά στον υπολογιστή.",false);},
          c @ (Command::ImportConfig|Command::ExportConfig(_)|Command::ImportImage(_)|Command::ExportArtwork(_)|Command::ImportArtwork|Command::PickApplication{..})=>{io::launch(&io_busy,internal_tx.clone(),move||io::file_command(c))?;},
          Command::Upload{key,clip}=>{if demo{bail!("Η προσομοίωση δεν γράφει στη συσκευή.");}device.as_ref().context("USB adapter μη διαθέσιμος.")?.upload(key,clip)?;out.log("Η μεταφορά OLED μπήκε στην ουρά. Η επιβεβαίωση θα εμφανιστεί μετά την αποθήκευση στο pad.",false);},
          Command::ConnectHa{url,token}=>{if demo{bail!("Η προσομοίωση δεν συνδέεται σε Home Assistant.");}let h=ha.clone();let path=store.secret_path();io::launch(&io_busy,internal_tx.clone(),move||{let cat=h.connect(&url,token)?;if let Some(b)=h.protected()?{atomic_write(&path,&b,false)?;}Ok(vec![Event::Catalog(cat)])})?;},
          Command::RefreshHa=>{let h=ha.clone();io::launch(&io_busy,internal_tx.clone(),move||Ok(vec![Event::Catalog(h.refresh()?)]))?;},
          Command::DisconnectHa=>{if io_busy.load(Ordering::Relaxed){bail!("Περίμενε να ολοκληρωθεί η σύνδεση/επιλογή αρχείου.");}ha.disconnect();if store.secret_path().exists(){std::fs::remove_file(store.secret_path())?;}status.ha_connected=false;status.ha_url.clear();out.send(Event::Catalog(Catalog::default()));out.log("Αφαιρέθηκε η σύνδεση HA.",false);},
          Command::RefreshDevice=>{device.as_ref().context("USB adapter μη διαθέσιμος.")?.refresh()?;},
         }Ok(())})();if let Err(e)=result{out.log(format!("{e:#}"),true);out.send(Event::Notice(format!("{e:#}")));}status.ha_connected=ha.connected();status.io_busy=io_busy.load(Ordering::Relaxed);out.send(Event::Status(status.clone()));},
         recv(input_rx)->v=>{match v{Ok(PadInput{bank,key})=>{if let Some(p)=config.profiles.get(bank as usize){let id=p.id.clone();if config.active_profile!=id {match activate(&mut config,&store,id.clone()){Ok(())=>{if let Some(d)=&device{d.update(&config,false);}out.send(Event::Saved(config.clone()));},Err(e)=>out.log(e.to_string(),true)}}match queue_key(&config,&id,key,false,false,&jobs_tx,&epoch,&paused){Ok(())=>{status.running_jobs+=1;out.log(format!("Φυσικό πλήκτρο K{key} · mode {}.",bank+1),false);},Err(e)=>out.log(e.to_string(),true)}out.send(Event::Status(status.clone()));}},Err(_)=>{input_rx=never();status.input.available=false;out.log("Η υπηρεσία εισόδων σταμάτησε.",true);}}},
         recv(internal)->v=>{match v {
          Ok(Internal::Done{error,message})=>{status.running_jobs=status.running_jobs.saturating_sub(1);out.log(message,error);},
          Ok(Internal::Notice{error,message})=>out.log(message,error),
          Ok(Internal::IoDone(result))=>{match result{Ok(events)=>{for event in events{out.send(event);}out.log("Η εργασία αρχείων/σύνδεσης ολοκληρώθηκε.",false);},Err(e)=>out.log(e,true)}status.ha_connected=ha.connected();status.ha_url=ha.url();},
          Ok(Internal::Device(u))=>{status.usb_connected=u.connected;status.usb_message=u.message;status.firmware=u.firmware;if let Some(d)=&device&&d.revision()==u.revision&&let Some(mode)=u.mode&&let Some(p)=config.profiles.get(mode as usize){let id=p.id.clone();if config.active_profile!=id{match activate(&mut config,&store,id){Ok(())=>{d.update(&config,false);out.send(Event::Saved(config.clone()));},Err(e)=>out.log(e.to_string(),true)}}}},
          Ok(Internal::Mode(id))=>{let id=id.unwrap_or_else(||{let i=config.profiles.iter().position(|p|p.id==config.active_profile).unwrap_or(0);config.profiles[(i+1)%config.profiles.len().min(8)].id.clone()});match activate(&mut config,&store,id){Ok(())=>{if let Some(d)=&device{d.update(&config,true);}out.send(Event::Saved(config.clone()));},Err(e)=>out.log(e.to_string(),true)}},Err(_)=>{}}status.io_busy=io_busy.load(Ordering::Relaxed);out.send(Event::Status(status.clone()));},
         default(Duration::from_millis(100))=>{}
        }
    }
    epoch.fetch_add(1, Ordering::Relaxed);
    drop(jobs_tx);
    drop(listener);
    Ok(())
}
fn activate(config: &mut Config, store: &Store, id: String) -> Result<()> {
    let mut c = config.clone();
    c.active_profile = id;
    c.validate()?;
    store.save_config(&c)?;
    *config = c;
    Ok(())
}
#[allow(clippy::too_many_arguments)] // One bounded dispatch operation; arguments describe an immutable input snapshot.
fn queue_key(
    c: &Config,
    profile: &str,
    key: u8,
    simulate: bool,
    test_delay: bool,
    tx: &Sender<Job>,
    epoch: &AtomicU64,
    paused: &AtomicBool,
) -> Result<()> {
    if paused.load(Ordering::Relaxed) {
        bail!("Τα macros είναι σε παύση.");
    }
    let p = c
        .profiles
        .iter()
        .find(|p| p.id == profile)
        .context("Δεν βρέθηκε mode.")?;
    let k = p
        .keys
        .iter()
        .find(|k| k.id == key)
        .context("Δεν βρέθηκε πλήκτρο.")?;
    tx.try_send(Job {
        actions: k.actions.clone(),
        simulate,
        test_delay,
        epoch: epoch.load(Ordering::Relaxed),
    })
    .map_err(|_| anyhow::anyhow!("Η ουρά macros είναι γεμάτη. Το πάτημα δεν εκτελέστηκε."))
}
fn wait(
    ms: u64,
    job: &Job,
    stop: &AtomicBool,
    epoch: &AtomicU64,
    paused: &AtomicBool,
) -> Result<()> {
    let start = Instant::now();
    loop {
        if stop.load(Ordering::Relaxed)
            || paused.load(Ordering::Relaxed)
            || epoch.load(Ordering::Relaxed) != job.epoch
        {
            bail!("Η ακολουθία ακυρώθηκε.");
        }
        let left = Duration::from_millis(ms).saturating_sub(start.elapsed());
        if left.is_zero() {
            return Ok(());
        }
        std::thread::sleep(left.min(Duration::from_millis(40)));
    }
}
fn executor(
    rx: Receiver<Job>,
    tx: Sender<Internal>,
    ha: HomeAssistant,
    stop: Arc<AtomicBool>,
    epoch: Arc<AtomicU64>,
    paused: Arc<AtomicBool>,
) {
    while let Ok(job) = rx.recv() {
        let r = (|| -> Result<()> {
            if job.test_delay {
                wait(3000, &job, &stop, &epoch, &paused)?;
            }
            for a in &job.actions {
                wait(0, &job, &stop, &epoch, &paused)?;
                if job.simulate {
                    continue;
                }
                match a.kind {
                    ActionKind::Delay => wait(a.value.parse()?, &job, &stop, &epoch, &paused)?,
                    ActionKind::HomeAssistant => ha.call(&a.value, &a.args)?,
                    ActionKind::Mode => tx.send(Internal::Mode(Some(a.value.clone())))?,
                    ActionKind::ModeNext => tx.send(Internal::Mode(None))?,
                    _ => deck_platform::execute(a)?,
                }
            }
            Ok(())
        })();
        let (error, message) = match r {
            Ok(()) => (
                false,
                if job.simulate {
                    format!(
                        "Προσομοίωση ολοκληρώθηκε: {} βήματα, χωρίς εξωτερικές ενέργειες.",
                        job.actions.len()
                    )
                } else {
                    format!("Ολοκληρώθηκε ακολουθία {} βημάτων.", job.actions.len())
                },
            ),
            Err(e) => (true, format!("Η ακολουθία σταμάτησε: {e}")),
        };
        if tx.send(Internal::Done { error, message }).is_err() {
            break;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pause_invalidates_queued_job() {
        let (tx, rx) = bounded(1);
        let e = AtomicU64::new(1);
        let p = AtomicBool::new(false);
        queue_key(&Config::default(), "windows", 1, true, false, &tx, &e, &p).unwrap();
        let j = rx.recv().unwrap();
        e.fetch_add(1, Ordering::Relaxed);
        assert!(wait(0, &j, &AtomicBool::new(false), &e, &p).is_err());
    }
    #[test]
    fn queue_overload_is_explicit() {
        let (tx, _rx) = bounded(1);
        let e = AtomicU64::new(0);
        let p = AtomicBool::new(false);
        queue_key(&Config::default(), "windows", 1, true, false, &tx, &e, &p).unwrap();
        assert!(queue_key(&Config::default(), "windows", 1, true, false, &tx, &e, &p).is_err());
    }
}
