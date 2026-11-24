use super::*;
use anyhow::{Context, ensure};
use deck_domain::{ActionKind, bank_modifiers, parse_hotkey};
use std::{
    mem::size_of,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    System::Threading::GetCurrentThreadId,
    UI::{Input::KeyboardAndMouse::*, Shell::ShellExecuteW, WindowsAndMessaging::*},
};
fn send(entries: &[(u16, u16, u32)]) -> Result<()> {
    let values: Vec<INPUT> = entries
        .iter()
        .map(|&(vk, scan, flags)| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: scan,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        })
        .collect();
    // SAFETY: INPUT union is initialized with KEYBDINPUT and exact ABI size; buffers remain live for synchronous call.
    let sent = unsafe {
        SendInput(
            values.len() as u32,
            values.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    };
    if sent != values.len() as u32 {
        let release: Vec<INPUT> = entries
            .iter()
            .filter(|e| e.2 & KEYEVENTF_KEYUP == 0)
            .map(|&(vk, scan, flags)| INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: scan,
                        dwFlags: flags | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            })
            .collect();
        unsafe {
            SendInput(
                release.len() as u32,
                release.as_ptr(),
                size_of::<INPUT>() as i32,
            );
        }
        bail!("Τα Windows μπλόκαραν την αποστολή. Έλεγξε τα δικαιώματα του προγράμματος στόχου.")
    }
    Ok(())
}
fn combo(value: &str) -> Result<()> {
    let keys = parse_hotkey(value)?;
    let flag = |k: &u16| {
        if [33, 34, 35, 36, 37, 38, 39, 40, 44, 45, 46, 91].contains(k) {
            KEYEVENTF_EXTENDEDKEY
        } else {
            0
        }
    };
    let mut e: Vec<_> = keys.iter().map(|k| (*k, 0, flag(k))).collect();
    e.extend(
        keys.iter()
            .rev()
            .map(|k| (*k, 0, flag(k) | KEYEVENTF_KEYUP)),
    );
    send(&e)
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn open(target: &str, args: &str) -> Result<()> {
    let op = wide("open");
    let target = wide(target);
    let args = wide(args);
    let result = unsafe {
        ShellExecuteW(
            null_mut(),
            op.as_ptr(),
            target.as_ptr(),
            args.as_ptr(),
            null(),
            SW_SHOWNORMAL,
        )
    };
    ensure!(
        result as isize > 32,
        "Δεν άνοιξε η εφαρμογή ή διεύθυνση. Έλεγξε τη διαδρομή."
    );
    Ok(())
}
pub fn execute(a: &Action) -> Result<()> {
    use ActionKind::*;
    match a.kind {
        Hotkey => combo(&a.value),
        DiscordMute => combo("Ctrl+Alt+Shift+M"),
        DiscordDeafen => combo("Ctrl+Alt+Shift+D"),
        Text => {
            let u: Vec<_> = a.value.encode_utf16().collect();
            for chunk in u.chunks(64) {
                let e: Vec<_> = chunk
                    .iter()
                    .flat_map(|u| {
                        [
                            (0, *u, KEYEVENTF_UNICODE),
                            (0, *u, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP),
                        ]
                    })
                    .collect();
                send(&e)?;
            }
            Ok(())
        }
        Application => open(&a.value, &a.args),
        Url => open(&a.value, ""),
        VolumeUp | VolumeDown | VolumeMute | PlayPause | NextTrack | PreviousTrack => {
            let key = match a.kind {
                VolumeUp => 0xaf,
                VolumeDown => 0xae,
                VolumeMute => 0xad,
                PlayPause => 0xb3,
                NextTrack => 0xb0,
                _ => 0xb1,
            };
            send(&[
                (key, 0, KEYEVENTF_EXTENDEDKEY),
                (key, 0, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP),
            ])
        }
        _ => bail!("Η ενέργεια εκτελείται από τον runtime controller."),
    }
}
pub fn listen() -> InputListener {
    let (tx, inputs) = bounded(32);
    let (status_tx, status) = bounded(1);
    let (id_tx, id_rx) = bounded(1);
    let dropped = Arc::new(AtomicUsize::new(0));
    let drops = dropped.clone();
    std::thread::spawn(move || {
        let mut msg: MSG = unsafe { std::mem::zeroed() };
        // SAFETY: creates this thread's Windows message queue before publishing its ID.
        unsafe {
            PeekMessageW(&mut msg, null_mut(), 0, 0, PM_NOREMOVE);
        }
        let _ = id_tx.send(unsafe { GetCurrentThreadId() });
        let mut good = Vec::new();
        let mut bad = Vec::new();
        for bank in 0..8 {
            for key in 1..=9 {
                let id = bank * 9 + key;
                let ok = unsafe {
                    RegisterHotKey(
                        null_mut(),
                        id as i32,
                        MOD_NOREPEAT | bank_modifiers(bank),
                        0x7b + key as u32,
                    )
                };
                if ok != 0 { good.push(id) } else { bad.push(id) }
            }
        }
        let _ = status_tx.send(InputStatus {
            registered: good.len(),
            conflicts: bad,
            dropped: 0,
            available: true,
        });
        loop {
            let r = unsafe { GetMessageW(&mut msg, null_mut(), 0, 0) };
            if r <= 0 {
                break;
            }
            if msg.message == WM_HOTKEY {
                let id = msg.wParam as u8;
                if (1..=72).contains(&id)
                    && tx
                        .try_send(PadInput {
                            bank: (id - 1) / 9,
                            key: (id - 1) % 9 + 1,
                        })
                        .is_err()
                {
                    drops.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        for id in good {
            unsafe {
                UnregisterHotKey(null_mut(), id as i32);
            }
        }
    });
    let thread_id = id_rx
        .recv()
        .context("hotkey thread initialization")
        .unwrap_or(0);
    InputListener {
        inputs,
        status,
        dropped,
        thread_id,
    }
}
