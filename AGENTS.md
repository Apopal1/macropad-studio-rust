# macropad Studio 3

Native Rust only. No web server, WebView, JavaScript frontend or Python runtime.
Read ROADMAP.md, TASKS.md and docs/WORK_LOG.md before changes.
Work in small sequential tasks; update the task checklist and append decisions/results to the work log.
Preserve the sibling legacy project and firmware. Do not flash devices during this migration.
Dependencies: domain <- storage/platform <- runtime <- desktop; domain must remain independent of platform/UI.
Never run network, disk, HID or macro execution on the UI thread. Use bounded channels and explicit errors.
Never log credentials, user text/macros, raw config contents or HID paths. Profile exports exclude HA credentials.
Never claim hardware connectivity or Windows verification from demo mode or Linux tests.
Before committing: cargo fmt --all --check; cargo test --workspace; cargo clippy --workspace --all-targets -- -D warnings.
Update migration status honestly; no simulated success for unsupported operations.
