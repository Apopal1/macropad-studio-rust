use anyhow::{Result, bail};
use deck_domain::{Config, artwork::Clip, protocol::DeviceStatus};
#[cfg(windows)]
use {
    anyhow::ensure,
    deck_domain::{oled_label, protocol::*},
    std::{thread, time::Duration},
};
pub struct UsbDeck {
    #[cfg(windows)]
    device: hidapi::HidDevice,
    #[cfg(windows)]
    seq: u8,
}
impl UsbDeck {
    pub fn open() -> Result<Self> {
        #[cfg(windows)]
        {
            let api = hidapi::HidApi::new()?;
            let mut found = Vec::new();
            for info in api
                .device_list()
                .filter(|d| d.usage_page() == USAGE_PAGE && d.usage() == USAGE)
            {
                if let Ok(d) = info.open_device(&api) {
                    let mut deck = Self { device: d, seq: 0 };
                    if deck.status().is_ok() {
                        found.push(deck);
                    }
                }
            }
            ensure!(
                found.len() == 1,
                if found.is_empty() {
                    "Δεν βρέθηκε Control Deck στο USB-NATIVE. Έλεγξε καλώδιο και firmware."
                } else {
                    "Βρέθηκαν πολλά Control Deck. Κράτησε συνδεδεμένο μόνο ένα."
                }
            );
            Ok(found.remove(0))
        }
        #[cfg(not(windows))]
        {
            bail!("Η native σύνδεση USB του pad απαιτεί Windows σε αυτή την έκδοση.")
        }
    }
    pub fn status(&mut self) -> Result<DeviceStatus> {
        #[cfg(windows)]
        {
            let mut r = [0; 64];
            r[0] = REPORT_ID;
            let n = self.device.get_feature_report(&mut r)?;
            Ok(parse_status(&r[..n])?)
        }
        #[cfg(not(windows))]
        {
            bail!("USB adapter μη διαθέσιμος.")
        }
    }
    pub fn configure(&mut self, c: &Config, select: bool) -> Result<()> {
        #[cfg(windows)]
        {
            ensure!(
                self.status()?.supports_modes,
                "Χρειάζεται firmware 3.0 για modes."
            );
            let profiles = &c.profiles[..c.profiles.len().min(8)];
            self.command(22, ((1u16 << profiles.len()) - 1) as u8, "")?;
            for (i, p) in profiles.iter().enumerate() {
                self.command(20, i as u8, &oled_label(p, i))?;
            }
            if select {
                let i = profiles
                    .iter()
                    .position(|p| p.id == c.active_profile)
                    .unwrap();
                self.command(21, i as u8, "")?;
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            let _ = (c, select);
            bail!("USB adapter μη διαθέσιμος.")
        }
    }
    #[cfg(windows)]
    fn command(&mut self, cmd: u8, value: u8, label: &str) -> Result<DeviceStatus> {
        self.seq = self.seq.wrapping_add(1);
        self.packet(mode_command(cmd, self.seq, value, label)?)
    }
    #[cfg(windows)]
    fn packet(&mut self, r: [u8; 64]) -> Result<DeviceStatus> {
        self.device.send_feature_report(&r)?;
        for _ in 0..8 {
            let s = self.status()?;
            if s.command == r[3] && s.seq == r[4] {
                ensure!(
                    s.error == 0,
                    "Το pad απέρριψε την ενέργεια (κωδικός {}).",
                    s.error
                );
                return Ok(s);
            }
            thread::sleep(Duration::from_millis(25));
        }
        bail!("Δεν επιβεβαιώθηκε το πακέτο USB.")
    }
    pub fn upload(&mut self, slot: u8, clip: &Clip) -> Result<()> {
        #[cfg(windows)]
        {
            ensure!((1..=9).contains(&slot), "Μη έγκυρη θέση artwork.");
            let blob = clip.pack()?;
            let crc = deck_domain::artwork::crc32(&blob);
            self.ready(false)?;
            let mut begin = self.base_packet(10);
            begin[9..13].copy_from_slice(&crc.to_le_bytes());
            begin[13..15].copy_from_slice(&(blob.len() as u16).to_le_bytes());
            begin[15] = slot;
            self.packet(begin)?;
            for (offset, chunk) in blob.chunks(55).enumerate() {
                let mut r = self.base_packet(11);
                let offset = (offset * 55) as u16;
                r[5..7].copy_from_slice(&offset.to_le_bytes());
                r[7] = chunk.len() as u8;
                r[9..9 + chunk.len()].copy_from_slice(chunk);
                self.packet(r)?;
            }
            let mut r = self.base_packet(12);
            r[8] = 1;
            self.packet(r)?;
            let s = self.ready(true)?;
            ensure!(
                s.saved_crc == crc && s.saved_slot == slot && s.slot_mask & (1 << slot) != 0,
                "Το pad δεν επιβεβαίωσε το αποθηκευμένο artwork."
            );
            Ok(())
        }
        #[cfg(not(windows))]
        {
            let _ = (slot, clip);
            bail!("USB adapter μη διαθέσιμος.")
        }
    }
    #[cfg(windows)]
    fn base_packet(&mut self, cmd: u8) -> [u8; 64] {
        self.seq = self.seq.wrapping_add(1);
        let mut r = [0; 64];
        r[..5].copy_from_slice(&[63, 0xcd, 1, cmd, self.seq]);
        r
    }
    #[cfg(windows)]
    fn ready(&mut self, check_error: bool) -> Result<DeviceStatus> {
        for _ in 0..100 {
            let s = self.status()?;
            if !s.busy {
                ensure!(
                    !check_error || s.error == 0,
                    "Αποτυχία αποθήκευσης στο pad ({}).",
                    s.error
                );
                return Ok(s);
            }
            thread::sleep(Duration::from_millis(50));
        }
        bail!("Η αποθήκευση OLED δεν ολοκληρώθηκε εγκαίρως.")
    }
}
