# Control Deck Studio 3 — Roadmap

Created: 2026-11-24. Work starts with this roadmap before implementation.

## Goal
A local-first, native Rust Control Deck application (egui/eframe desktop + Rust services) with a new accessible UI, explicit hardware status, reliable profile editing and a native Windows application architecture. Develop in this new directory; preserve the existing Python Studio and firmware.

## Working agreement
- Work sequentially in small independently reviewable tasks.
- Record each meaningful action, result, decision and blocker in `docs/WORK_LOG.md`.
- Track implemented and pending tasks in `TASKS.md`; never mark untested hardware features complete.
- Keep core behavior independent of HTTP, UI and platform APIs.
- Run focused tests before committing a task. Keep real device and Windows verification separate from Linux tests.
- No secrets, user profiles, tokens, vendor binaries, build outputs or personal data in Git.

## Stack constraint
All application logic and the native UI are Rust. Use egui/eframe with the native glow renderer: no browser, WebView, JavaScript, TypeScript, WASM, Leptos, Tauri or HTTP server. Platform work runs away from the UI thread. Resource use is measured, not assumed.

## Milestones
1. **Discovery and decisions**: inventory legacy behavior; architecture decision records; migration matrix; UX design brief.
2. **Rust vertical slice**: typed domain, validation, profile persistence, typed in-process commands, demo runtime, tests.
3. **New editor**: hardware-shaped 3×3 layout, mode switching, action inspector, save/revert, import/export, diagnostics, accessible responsive design.
4. **Windows actions**: native hotkeys, input dispatch, multimedia, application launch and Unicode text; preserve input-loop protection and bounded macros.
5. **Home Assistant**: Rust HTTP client, connection test/catalog/service dispatch, protected local credentials and explicit execution feedback.
6. **OLED/device**: protocol parity with current firmware, mode synchronization, per-key artwork/GIF migration and physical-device tests.
7. **Native lifecycle**: tray, single instance, startup, installer and Windows smoke tests.
8. **Release parity**: legacy import, backup/rollback, complete regression checks and upgrade instructions.

## Delivery gate
A source baseline can be delivered before full hardware parity only with every unsupported feature visibly identified. Production/release readiness requires Windows, ESP32 and HA verification. Do not flash firmware as part of this software redesign.

## Repository
Private repository created: https://github.com/Apopal1/macropad-studio-rust
Source publication and remote verification are in progress.
