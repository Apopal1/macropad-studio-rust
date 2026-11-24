# Windows / hardware acceptance checklist

Not run in this Linux workspace. Check on Windows 10/11 x64 with a data cable and current firmware. Start with a copied export and retain the old Studio for rollback. Close its bridge before running native input tests.

- [ ] Launch native .exe without Python, browser or WebView. Correct Greek font and UI scaling on both monitors.
- [ ] Second launch reports existing instance and does not register duplicate hotkeys.
- [ ] Save/restart restores modes; corrupt-file recovery preserves original data.
- [ ] USB vendor collection recognized with firmware identity; no COM-port assumption.
- [ ] 72 hotkeys register. A deliberate F13 conflict appears in diagnostics.
- [ ] USB and BLE: each K1–K9 executes exactly once; K9 modifier combos do not fire ordinary macros.
- [ ] Rapid K9+mode changes route to the bank encoded in each key event.
- [ ] Mode changes from device and UI agree; label stays on OLED above artwork.
- [ ] Windows volume, media, hotkeys, .exe/.lnk, URL and Greek/emoji text work.
- [ ] Partial SendInput failure releases modifiers; elevated-window failures surface clearly.
- [ ] Pause invalidates pending macros; queue overload is visible; closing stops input registration.
- [ ] Discord custom keybinds work outside its keybind editor. No fake mute-state display.
- [ ] HA connection test changes no entities; catalog loads actual entities/services.
- [ ] HA service with response required requests return_response; no retries on failed service call.
- [ ] DPAPI token survives restart for the same user; profile exports contain no token.
- [ ] Each K1–K9 receives its own image/GIF, CRC and stored slot are confirmed.
- [ ] OLED transfer or a native file dialog does not block macro input handling.
- [ ] Unplug mid-transfer produces failure; reconnect reapplies current saved modes.
- [ ] Stable UI focus and keyboard operation; screen-reader and high-DPI checks.
- [ ] Measure idle CPU/RSS, startup, binary size and action latency on the target PC.

Pending release features: tray and background lifecycle, focus existing instance, optional startup, installer, legacy OLED-bank migration and text/shape editor parity. Potentiometer firmware and per-application audio routing are a separate feature.
