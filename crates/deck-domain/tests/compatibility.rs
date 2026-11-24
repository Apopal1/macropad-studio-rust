//! Golden bytes generated using original Studio 1.6 OLEDAnimation.pack/OLEDCore.packet.
use deck_domain::artwork::{Clip, Frame, crc32};
#[test]
fn cdan_matches_legacy_javascript_encoder() {
    let bitmap: Vec<u8> = (0..1024).map(|i| ((i * 17 + 3) & 255) as u8).collect();
    let inverse: Vec<_> = bitmap.iter().map(|b| b ^ 255).collect();
    let c = Clip {
        frames: vec![
            Frame {
                bitmap,
                duration_ms: 125,
            },
            Frame {
                bitmap: inverse,
                duration_ms: 250,
            },
        ],
    };
    let blob = c.pack().unwrap();
    assert_eq!(blob, include_bytes!("fixtures/legacy-cdan.bin"));
    let r = include_bytes!("fixtures/legacy-upload-begin.bin");
    assert_eq!(&r[..5], &[63, 0xcd, 1, 10, 17]);
    assert_eq!(
        u32::from_le_bytes(r[9..13].try_into().unwrap()),
        crc32(&blob)
    );
    assert_eq!(
        u16::from_le_bytes(r[13..15].try_into().unwrap()) as usize,
        blob.len()
    );
    assert_eq!(r[15], 7);
}
