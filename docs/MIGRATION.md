# Migration matrix

Legacy source reviewed: sibling `control_deck_web`, Studio 1.6 development, firmware 3.0.

| Feature | Legacy contract | Native v3 target | Acceptance |
|---|---|---|---|
| Profiles | schema 1, 1–12 profiles, 9 keys | schema 3, explicit import | round trip and rejection tests |
| Hardware modes | first 8, K9 + K1–K8 | same key IDs and bank encoding | protocol + Windows + actual pad |
| Actions | up to12, delays 0–5000 / total15000ms | typed Rust actions | validation and dispatch tests |
| Input | F13–F21 with Ctrl/Shift/Alt bank bits | RegisterHotKey, no input feedback loops | 72 registrations + conflict reporting |
| Windows | hotkey/text/app/url/media/Discord | native APIs, no shell command eval | real Windows smoke tests |
| HA | arbitrary domain.service + JSON | Rust client, separate protected token | local mock + actual HA |
| OLED | 128×64,24frames,max14px header | per-key frames + HID chunk protocol | firmware reference tests + actual pad |
| Native lifecycle | tray/single instance/startup | native Rust | Windows acceptance |
| Installation | previous Python/WebView installer | standalone executable / future installer | clean Windows install |

The old executable and data remain valid. v3 uses a new data directory. Import never accesses HA token files or copies legacy credentials. Future importer changes must reject unsupported fields rather than silently lose behavior. Potentiometer/app-volume routing did not exist in current firmware and remains a separate future capability.
