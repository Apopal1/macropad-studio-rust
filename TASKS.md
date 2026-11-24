# Task checklist

Status: `[ ]` pending, `[x]` completed. Each task must have evidence in the work log.

## Foundation
- [x] F01 Create isolated v3 folder and roadmap before implementation.
- [x] F02 Inventory legacy features and schemas.
- [x] F03 Record architecture, UX and security decisions.
- [x] F04 Create private GitHub repository and initial roadmap commit.
- [x] F05 Install/verify Rust toolchain and dependencies.

## Rust vertical slice
- [x] R01 Define versioned config, profiles, actions and validation.
- [x] R02 Implement recoverable atomic config persistence.
- [x] R03 Implement typed in-process commands and bounded worker queue.
- [x] R04 Implement profile/runtime status and bounded action execution.
- [x] R05 Implement legacy profile import; legacy OLED bank import remains D04.

## Editor
- [x] U01 Build responsive navigation, mode list and 3×3 deck preview.
- [x] U02 Build key inspector and action sequence editing.
- [x] U03 Save/revert, dirty state and import/export.
- [x] U04 Diagnostics and explicit demo/unsupported states.
- [x] U05 Headless native visual QA and real widget pointer interaction.
- [ ] U06 Actual Windows keyboard, focus and assistive-technology acceptance.

## Platform/integrations
- [x] W01 Implement Windows input and multimedia adapter (runtime acceptance W03).
- [x] W02 Implement global mode/key listener (runtime acceptance W03).
- [ ] W03 Windows smoke tests on real computer.
- [x] H01 Rust HA connection/catalog/service client.
- [x] H02 Implement DPAPI credential storage and pass local HA mock tests; DPAPI runtime acceptance remains W03.
- [ ] H03 Real Home Assistant verification.
- [x] D01 Port device protocol and mode/OLED sync (actual device acceptance D03).
- [x] D02 Native per-key bitmap/GIF editor, bounded image import and CDAN transfer.
- [ ] D04 Import legacy OLED-bank v2 JSON.
- [ ] D05 Native text/shape OLED tools and idle artwork slot.
- [ ] D03 Actual ESP32-S3 verification.
- [ ] N01 Native tray, single instance and startup.
- [ ] N02 Windows installer and upgrade tests.

## Delivery
- [x] Q01 Rust and UI automated checks.
- [x] Q02 Author CI and development/Windows instructions (remote CI run awaits Q03).
- [x] Q03 Push verified source and progress to new GitHub repository.

- [ ] Q04 Verify the first GitHub Windows/Linux CI run and release artifacts.

## Next small tasks, in order
- [ ] N03 Tray icon/menu; preserve macros when editor hides.
- [ ] N04 Focus existing native instance instead of showing a lock error.
- [ ] N05 Optional Windows startup, explicitly controlled in Settings.
- [ ] D06 Mock HID transport tests for ACK timeout, disconnect and upload recovery.
- [ ] P01 Measure Windows idle CPU, memory, startup and dispatch latency.
- [ ] P02 Compare locked egui baseline with current supported release before feature expansion.
- [ ] A01 Define three-potentiometer input protocol and Windows per-app audio routing.

Private repository and source publication are verified. Initial GitHub CI is running; local implementation is not the same as physical acceptance.
