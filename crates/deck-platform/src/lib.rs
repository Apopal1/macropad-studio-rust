//! OS and transport adapters. Unsupported platforms fail explicitly.
pub mod ha;
pub mod usb;
#[cfg(windows)]
mod windows;
use anyhow::{Result, bail};
use crossbeam_channel::{Receiver, bounded};
use deck_domain::Action;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
#[derive(Debug, Clone)]
pub struct PadInput {
    pub bank: u8,
    pub key: u8,
}
#[derive(Debug, Clone, Default)]
pub struct InputStatus {
    pub registered: usize,
    pub conflicts: Vec<u8>,
    pub dropped: usize,
    pub available: bool,
}
pub struct InputListener {
    pub inputs: Receiver<PadInput>,
    pub status: Receiver<InputStatus>,
    pub dropped: Arc<AtomicUsize>,
    #[cfg(windows)]
    thread_id: u32,
}
impl InputListener {
    pub fn start() -> Self {
        #[cfg(windows)]
        {
            windows::listen()
        }
        #[cfg(not(windows))]
        {
            let (_, inputs) = bounded(1);
            let (s, status) = bounded(1);
            let _ = s.send(InputStatus::default());
            Self {
                inputs,
                status,
                dropped: Arc::new(AtomicUsize::new(0)),
            }
        }
    }
    pub fn drop_count(&self) -> usize {
        self.dropped.load(Ordering::Relaxed)
    }
}
#[cfg(windows)]
impl Drop for InputListener {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::PostThreadMessageW(
                self.thread_id,
                0x12,
                0,
                0,
            );
        }
    }
}
pub fn execute(action: &Action) -> Result<()> {
    action.validate()?;
    #[cfg(windows)]
    {
        windows::execute(action)
    }
    #[cfg(not(windows))]
    {
        let _ = action;
        bail!("Οι ενέργειες Windows είναι διαθέσιμες μόνο στα Windows. Χρησιμοποίησε προσομοίωση.")
    }
}
