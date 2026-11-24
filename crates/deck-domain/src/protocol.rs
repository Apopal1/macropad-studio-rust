//! Current firmware v3 vendor feature protocol. Transport and retry policy live in platform.
use crate::{ValidationError, require};
pub const REPORT_ID: u8 = 63;
pub const REPORT_SIZE: usize = 64;
pub const USAGE_PAGE: u16 = 0xff60;
pub const USAGE: u16 = 0x61;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceStatus {
    pub firmware: String,
    pub supports_modes: bool,
    pub mode: u8,
    pub mode_mask: u8,
    pub mode_name: String,
    pub busy: bool,
    pub command: u8,
    pub seq: u8,
    pub error: u8,
    pub saved_crc: u32,
    pub saved_slot: u8,
    pub slot_mask: u16,
}
pub fn parse_status(data: &[u8]) -> Result<DeviceStatus, ValidationError> {
    let r = if data.len() == 64 && data[0] == REPORT_ID {
        &data[1..]
    } else {
        data
    };
    require(
        r.len() == 63 && r[..2] == [0xcd, 1],
        "Η συσκευή δεν απάντησε ως Control Deck.",
    )?;
    Ok(DeviceStatus {
        firmware: format!("{}.{}", r[19], r[20]),
        supports_modes: r[21] & 4 != 0,
        mode: r[34],
        mode_mask: r[35],
        mode_name: r[37..54]
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| char::from(b))
            .collect(),
        busy: r[7] != 0,
        command: r[2],
        seq: r[3],
        error: r[4],
        saved_crc: u32::from_le_bytes(r[14..18].try_into().unwrap()),
        saved_slot: r[31],
        slot_mask: u16::from_le_bytes(r[28..30].try_into().unwrap()),
    })
}
pub fn mode_command(
    command: u8,
    seq: u8,
    value: u8,
    label: &str,
) -> Result<[u8; 64], ValidationError> {
    require(
        label.len() <= 16 && label.bytes().all(|b| (32..=126).contains(&b)),
        "Μη έγκυρη ετικέτα HID.",
    )?;
    let mut r = [0; 64];
    r[..5].copy_from_slice(&[REPORT_ID, 0xcd, 1, command, seq]);
    r[9] = value;
    r[10..10 + label.len()].copy_from_slice(label.as_bytes());
    Ok(r)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_status_rejected() {
        for f in [vec![], vec![0; 64], vec![63, 0xcd, 1]] {
            assert!(parse_status(&f).is_err());
        }
    }
    #[test]
    fn firmware_offsets() {
        let mut r = [0; 64];
        r[..3].copy_from_slice(&[63, 0xcd, 1]);
        r[20] = 3;
        r[22] = 4;
        r[35] = 2;
        r[38..43].copy_from_slice(b"MEDIA");
        let s = parse_status(&r).unwrap();
        assert_eq!(s.firmware, "3.0");
        assert_eq!(s.mode, 2);
        assert_eq!(s.mode_name, "MEDIA");
        assert!(s.supports_modes);
    }
    #[test]
    fn command_offsets() {
        let r = mode_command(12, 7, 2, "MEDIA").unwrap();
        assert_eq!(&r[..5], &[63, 0xcd, 1, 12, 7]);
        assert_eq!(r[9], 2);
        assert_eq!(&r[10..15], b"MEDIA");
    }
}
