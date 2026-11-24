# ADR 0001 — Native Rust UI and in-process services

Status: accepted, 2026-11-24.

User requires both frontend and backend in Rust, and a native application. Use egui/eframe, native glow rendering and an in-process service controller. Do not use Leptos/WASM, Axum, Tauri or a web frontend. Those earlier proposals were planning only, never implemented.

A modular monolith fits a single-user local device controller. Shared domain types eliminate duplicated validation. Actor-style commands isolate mutable application state. A bounded macro executor isolates slow actions from UI and config work. Avoid microservices, arbitrary plugin execution and dynamic scripting.

Glow is selected to avoid the larger default wgpu dependency tree. Native does not mean all controls are OS widgets: egui draws the UI and uses AccessKit for accessibility. Keyboard navigation and actual screen-reader use require verification. Performance claims require measurements on the target PC.

No automatic claim that a rewrite is faster. Track binary size, resident memory, idle CPU, startup and action latency in docs/PERFORMANCE.md. Keep optimizations driven by those measurements.
