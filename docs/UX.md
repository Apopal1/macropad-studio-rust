# UI / UX specification

Visual direction: restrained graphite surfaces, mint accent, generous spacing and readable Greek. No decorative dashboards, fake connected indicators or excessive animation. Key assignment is the primary screen.

- Persistent left navigation: Deck, Display, Home Assistant, Activity, Settings.
- Central 3×3 hardware-shaped key preview, OLED above, three illustrative knobs below. Knobs are clearly unimplemented until firmware/input protocol exists.
- Modes are named, numbered and show K9+K1–K8. Selected editor mode and active runtime mode are distinct.
- Right inspector: label, action type and parameters, move/remove steps, add step, delayed test.
- Explicit Save/Revert and dirty indicator. Import stages a draft; export validates and excludes credentials. Destructive discard/close requires a native confirmation.
- Real errors remain visible until resolved; connection state is never inferred from app state.
- Focusable standard egui widgets, keyboard selection, Ctrl+S save, reduced motion, UI zoom, high contrast text and tooltips. No accessibility compliance claim without assistive-technology testing.
- Display editor supports 128×64 monochrome frames with fixed mode header preview; per-key artwork shares firmware behavior across modes.
- HA form has a password-masked token, safe connection test and entity catalog. Network actions execute only on explicit macro test or physical press.

References: https://www.w3.org/WAI/WCAG22/quickref/ (interaction guidance, not a native-app conformance claim); https://docs.rs/eframe/ ; https://github.com/emilk/egui .
