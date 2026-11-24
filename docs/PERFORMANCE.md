# Performance evidence and budget

Native Rust and glow reduce runtime layers; they do not prove a specific CPU/RAM improvement. No comparison with the old Windows app has been measured yet.

Implementation constraints:
- No continuous UI repaint loop. Input/events drive rendering; OLED playback schedules only its own frame timer.
- Native window, no browser engine or localhost service.
- Bounded32 controller inputs,32 physical inputs,8 macro jobs,4 USB work items,32 worker results and128 UI events.
- USB operations have one handle owner. Native dialogs/image decoding/HA connection work are isolated from input dispatch.
- Only one file/HA setup operation at a time; HA requests use connect/request timeouts and bounded bodies.
- Artwork max9×24×1024 bitmap bytes plus metadata. GIF input512×512,24frames and8MiB compressed; still images2048×2048 and8MiB.
- 200 activity rows, no credentials or macro content in logs.
- Release profile optimizes size, strips symbols and enables thin LTO. Framework versions are locked; eframe0.33.3 is the tested baseline, not a claim to use the newest available crate.

Target measurements pending Windows acceptance:

| Metric | Baseline | v3 | Method |
|---|---|---|---|
| Idle CPU after60s | pending | pending | Task Manager / Performance Monitor |
| Private working set | pending | pending | same scene and driver |
| Cold/warm startup | pending | pending |10runs, median / p95 |
| Input-to-dispatch latency | pending | pending | timestamped test adapter |
| Release executable bytes | pending | build report | exact filesystem size |

Do not report Linux headless test memory as Windows native application memory. Headless UI images are actual egui geometry rasterized by a test-only Rust software renderer; they are not a measurement of the glow backend or OS compositor.
