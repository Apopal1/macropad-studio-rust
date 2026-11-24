# Verification

2026-11-24, Linux development workspace, rustc1.100.0 stable.

- Rust workspace native check: passed.
- Rust workspace tests:26 passed (16domain,1legacy protocol golden,3storage,2HA,3runtime/image,1native UI).
- Clippy with warnings denied and formatting check: passed.
- Windows x86_64 GNU cross-check: passed, including Windows input, DPAPI and HID code. This is compiler verification, not Windows execution.
- UI: actual egui widget tree rendered for Deck, OLED, HA, Activity, Settings and compact window. A synthetic pointer event selects the real K7 button. Headless renderer is test-only Rust code.
- Legacy compatibility: CDAN blob matches golden bytes produced by Studio1.6's actual JavaScript encoder; upload-begin field offsets verified against its packet encoder.
- HA: local HTTP mock checks auth header, API handshake, catalog and service POST. No real HA account or token used.
- Persistence: backup/roundtrip, invalid-save protection, corrupt-file preservation and exclusive instance lock.
- Runtime: queue-overload errors, pause invalidation and bounded image placement below mode header.

Optional release cross-link from Linux was attempted but failed due to missing/unsupported Windows import libraries. No Windows executable was produced. The documented native MSVC build and authored Windows CI still require execution.

Not verified: actual Windows window/driver behavior, Windows hotkey/input execution, DPAPI at runtime, HID transfer to the physical pad, actual Home Assistant, accessibility with assistive technology, installer/tray/startup and target-machine performance.

UI review found and fixed a negative layout size, crowded step controls, missing full-width-plus glyph and unnecessary mode-toolbar height. These were real rendering failures found before delivery, not assertions of physical device testing.
