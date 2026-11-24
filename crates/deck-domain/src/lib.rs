//! Platform-independent contracts. No UI, disk, network or OS input side effects.
pub mod artwork;
pub mod protocol;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

pub const SCHEMA: u32 = 3;
pub const MAX_PROFILES: usize = 12;
pub const HARDWARE_MODES: usize = 8;
pub const KEY_ORDER: [u8; 9] = [7, 8, 9, 4, 5, 6, 1, 2, 3];
#[derive(Debug, Error)]
#[error("{0}")]
pub struct ValidationError(pub String);
type Result<T> = std::result::Result<T, ValidationError>;
fn require(ok: bool, text: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(ValidationError(text.into()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Hotkey,
    Text,
    Application,
    Url,
    VolumeUp,
    VolumeDown,
    VolumeMute,
    PlayPause,
    NextTrack,
    PreviousTrack,
    DiscordMute,
    DiscordDeafen,
    Delay,
    HomeAssistant,
    Mode,
    ModeNext,
}
impl ActionKind {
    pub const ALL: [Self; 16] = [
        Self::Hotkey,
        Self::Text,
        Self::Application,
        Self::Url,
        Self::VolumeUp,
        Self::VolumeDown,
        Self::VolumeMute,
        Self::PlayPause,
        Self::NextTrack,
        Self::PreviousTrack,
        Self::DiscordMute,
        Self::DiscordDeafen,
        Self::Delay,
        Self::HomeAssistant,
        Self::Mode,
        Self::ModeNext,
    ];
    pub fn title(self) -> &'static str {
        match self {
            Self::Hotkey => "Συντόμευση",
            Self::Text => "Κείμενο",
            Self::Application => "Άνοιγμα εφαρμογής",
            Self::Url => "Άνοιγμα URL",
            Self::VolumeUp => "Ένταση +",
            Self::VolumeDown => "Ένταση −",
            Self::VolumeMute => "Σίγαση ήχου",
            Self::PlayPause => "Play / Pause",
            Self::NextTrack => "Επόμενο κομμάτι",
            Self::PreviousTrack => "Προηγούμενο κομμάτι",
            Self::DiscordMute => "Discord mute",
            Self::DiscordDeafen => "Discord deafen",
            Self::Delay => "Αναμονή",
            Self::HomeAssistant => "Home Assistant",
            Self::Mode => "Αλλαγή mode",
            Self::ModeNext => "Επόμενο mode",
        }
    }
    pub fn default_action(self) -> Action {
        Action {
            kind: self,
            value: match self {
                Self::Hotkey => "Ctrl+Shift+S",
                Self::Delay => "250",
                Self::Url => "https://example.com",
                Self::Application => "calc.exe",
                Self::HomeAssistant => "light.toggle",
                _ => "",
            }
            .into(),
            args: if self == Self::HomeAssistant {
                "{}".into()
            } else {
                String::new()
            },
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    #[serde(rename = "type")]
    pub kind: ActionKind,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub args: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    pub id: u8,
    pub label: String,
    pub actions: Vec<Action>,
    #[serde(default)]
    pub tint: u8,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub oled_name: String,
    pub keys: Vec<Key>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub active_profile: String,
    pub profiles: Vec<Profile>,
}
impl Config {
    pub fn active(&self) -> &Profile {
        self.profiles
            .iter()
            .find(|p| p.id == self.active_profile)
            .unwrap_or(&self.profiles[0])
    }
    pub fn validate(&self) -> Result<()> {
        require(
            self.version == SCHEMA,
            "Μη υποστηριζόμενη έκδοση ρυθμίσεων.",
        )?;
        require(
            (1..=MAX_PROFILES).contains(&self.profiles.len()),
            "Χρειάζονται 1–12 προφίλ.",
        )?;
        let mut ids = HashSet::new();
        for p in &self.profiles {
            require(
                !p.id.is_empty() && p.id.len() <= 80 && ids.insert(&p.id),
                "Μη έγκυρο ή διπλό ID προφίλ.",
            )?;
            require(
                !p.name.trim().is_empty() && p.name.chars().count() <= 50 && !p.name.contains('\0'),
                "Το όνομα mode χρειάζεται 1–50 χαρακτήρες.",
            )?;
            require(
                p.oled_name.len() <= 16 && p.oled_name.bytes().all(|b| (32..=126).contains(&b)),
                "Το όνομα OLED επιτρέπει έως 16 λατινικούς χαρακτήρες.",
            )?;
            require(p.keys.len() == 9, "Κάθε mode χρειάζεται ακριβώς 9 πλήκτρα.")?;
            let mut keys = HashSet::new();
            for k in &p.keys {
                require(
                    (1..=9).contains(&k.id) && keys.insert(k.id),
                    "Μη έγκυρο ή διπλό ID πλήκτρου.",
                )?;
                require(
                    k.label.chars().count() <= 40 && !k.label.contains('\0') && k.tint < 5,
                    "Μη έγκυρη ετικέτα ή χρώμα πλήκτρου.",
                )?;
                require(
                    k.actions.len() <= 12,
                    "Επιτρέπονται έως 12 ενέργειες ανά πλήκτρο.",
                )?;
                let mut delay = 0;
                for a in &k.actions {
                    a.validate()?;
                    if a.kind == ActionKind::Delay {
                        delay += a.value.parse::<u32>().unwrap_or(0);
                    }
                    if a.kind == ActionKind::Mode {
                        require(
                            self.profiles
                                .iter()
                                .take(HARDWARE_MODES)
                                .any(|p| p.id == a.value),
                            "Η αλλαγή mode χρειάζεται ένα από τα πρώτα 8 προφίλ.",
                        )?;
                    }
                }
                require(
                    delay <= 15000,
                    "Η συνολική αναμονή ανά macro δεν μπορεί να ξεπερνά 15 δευτερόλεπτα.",
                )?;
            }
        }
        require(
            self.profiles
                .iter()
                .take(HARDWARE_MODES)
                .any(|p| p.id == self.active_profile),
            "Το ενεργό mode πρέπει να είναι ένα από τα πρώτα 8 προφίλ.",
        )
    }
    pub fn import(text: &str) -> Result<Self> {
        require(
            text.len() <= 2 * 1024 * 1024,
            "Το αρχείο ρυθμίσεων είναι πολύ μεγάλο.",
        )?;
        let mut value: serde_json::Value = serde_json::from_str(text)
            .map_err(|_| ValidationError("Το JSON δεν είναι έγκυρο.".into()))?;
        let old = value.get("version").and_then(|v| v.as_u64()) == Some(1);
        if old {
            value["version"] = SCHEMA.into();
        }
        let mut config: Self = serde_json::from_value(value)
            .map_err(|e| ValidationError(format!("Μη αναγνωρίσιμη δομή ρυθμίσεων: {e}")))?;
        if old {
            for p in &mut config.profiles {
                p.keys.sort_by_key(|k| k.id);
            }
        }
        config.validate()?;
        Ok(config)
    }
}
impl Default for Config {
    fn default() -> Self {
        let make = |id: &str, name: &str, specs: &[(&str, ActionKind, &str)]| Profile {
            id: id.into(),
            name: name.into(),
            oled_name: name.into(),
            keys: (1..=9)
                .map(|i| {
                    let s = specs.get(i - 1);
                    Key {
                        id: i as u8,
                        label: s.map_or("Χωρίς ενέργεια", |s| s.0).into(),
                        tint: if id == "ha" {
                            3
                        } else if id == "media" {
                            2
                        } else {
                            0
                        },
                        actions: s
                            .map(|s| {
                                vec![Action {
                                    kind: s.1,
                                    value: s.2.into(),
                                    args: String::new(),
                                }]
                            })
                            .unwrap_or_default(),
                    }
                })
                .collect(),
        };
        Self {
            version: SCHEMA,
            active_profile: "windows".into(),
            profiles: vec![
                make(
                    "windows",
                    "Windows",
                    &[
                        ("Discord mute", ActionKind::DiscordMute, ""),
                        ("Ένταση −", ActionKind::VolumeDown, ""),
                        ("Ένταση +", ActionKind::VolumeUp, ""),
                        ("Discord deafen", ActionKind::DiscordDeafen, ""),
                        ("Play / Pause", ActionKind::PlayPause, ""),
                        ("Σίγαση", ActionKind::VolumeMute, ""),
                        ("Αντιγραφή", ActionKind::Hotkey, "Ctrl+C"),
                        ("Επικόλληση", ActionKind::Hotkey, "Ctrl+V"),
                        ("Αριθμομηχανή", ActionKind::Application, "calc.exe"),
                    ],
                ),
                make(
                    "media",
                    "Media",
                    &[
                        ("Ένταση −", ActionKind::VolumeDown, ""),
                        ("Σίγαση", ActionKind::VolumeMute, ""),
                        ("Ένταση +", ActionKind::VolumeUp, ""),
                        ("Προηγούμενο", ActionKind::PreviousTrack, ""),
                        ("Play / Pause", ActionKind::PlayPause, ""),
                        ("Επόμενο", ActionKind::NextTrack, ""),
                    ],
                ),
                make("ha", "Home Assistant", &[]),
            ],
        }
    }
}
impl Action {
    pub fn validate(&self) -> Result<()> {
        require(
            self.value.chars().count() <= 4000
                && self.args.len() <= 8000
                && !self.value.contains('\0')
                && !self.args.contains('\0'),
            "Μη έγκυρη τιμή ενέργειας.",
        )?;
        use ActionKind::*;
        match self.kind {
            Hotkey => {
                parse_hotkey(&self.value)?;
            }
            Delay => require(
                self.value.parse::<u32>().is_ok_and(|v| v <= 5000),
                "Η αναμονή πρέπει να είναι 0–5000 ms.",
            )?,
            Application => require(
                !self.value.trim().is_empty()
                    && (self.value.to_lowercase().ends_with(".exe")
                        || self.value.to_lowercase().ends_with(".lnk")),
                "Διάλεξε εφαρμογή .exe ή συντόμευση .lnk.",
            )?,
            Url => {
                let u = url::Url::parse(&self.value)
                    .map_err(|_| ValidationError("Μη έγκυρο URL.".into()))?;
                require(
                    matches!(u.scheme(), "http" | "https")
                        && u.host_str().is_some()
                        && u.username().is_empty()
                        && u.password().is_none(),
                    "Χρειάζεται HTTP/HTTPS URL χωρίς ενσωματωμένα στοιχεία σύνδεσης.",
                )?;
            }
            HomeAssistant => {
                let parts: Vec<_> = self.value.split('.').collect();
                require(
                    parts.len() == 2
                        && parts.iter().all(|s| {
                            !s.is_empty()
                                && s.bytes().all(|b| {
                                    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'
                                })
                        }),
                    "Χρειάζεται ενέργεια HA όπως light.toggle.",
                )?;
                let v: serde_json::Value = serde_json::from_str(&self.args).map_err(|_| {
                    ValidationError("Οι παράμετροι HA χρειάζονται έγκυρο JSON object.".into())
                })?;
                require(v.is_object(), "Οι παράμετροι HA χρειάζονται JSON object.")?;
            }
            _ => {}
        };
        Ok(())
    }
}
/// Windows virtual keys, with modifiers first. F13–F21 outputs would feed back into pad inputs.
pub fn parse_hotkey(value: &str) -> Result<Vec<u16>> {
    let mut result = Vec::new();
    let mut terminal = false;
    for s in value.split('+').map(|s| s.trim().to_ascii_uppercase()) {
        require(
            !s.is_empty() && !terminal,
            "Χρειάζονται modifiers και ένα τελικό πλήκτρο.",
        )?;
        let modifier = match s.as_str() {
            "CTRL" | "CONTROL" => Some(17),
            "ALT" => Some(18),
            "SHIFT" => Some(16),
            "WIN" | "WINDOWS" => Some(91),
            _ => None,
        };
        if let Some(m) = modifier {
            require(!result.contains(&m), "Διπλό modifier.")?;
            result.push(m);
            continue;
        }
        let k = if s.len() == 1 && s.as_bytes()[0].is_ascii_alphanumeric() {
            s.as_bytes()[0] as u16
        } else if let Some(f) = s.strip_prefix('F').and_then(|s| s.parse::<u16>().ok()) {
            require(
                (1..=24).contains(&f) && !(13..=21).contains(&f),
                "Τα F13–F21 είναι δεσμευμένες είσοδοι του pad.",
            )?;
            111 + f
        } else {
            match s.as_str() {
                "ENTER" => 13,
                "TAB" => 9,
                "ESC" | "ESCAPE" => 27,
                "SPACE" => 32,
                "BACKSPACE" => 8,
                "DELETE" => 46,
                "INSERT" => 45,
                "HOME" => 36,
                "END" => 35,
                "PAGEUP" => 33,
                "PAGEDOWN" => 34,
                "LEFT" => 37,
                "UP" => 38,
                "RIGHT" => 39,
                "DOWN" => 40,
                "PRINTSCREEN" => 44,
                _ => return Err(ValidationError("Μη υποστηριζόμενο πλήκτρο.".into())),
            }
        };
        result.push(k);
        terminal = true;
    }
    require(terminal, "Λείπει το τελικό πλήκτρο.")?;
    Ok(result)
}
pub fn bank_modifiers(bank: u8) -> u32 {
    (if bank & 1 != 0 { 2 } else { 0 })
        | (if bank & 2 != 0 { 4 } else { 0 })
        | (if bank & 4 != 0 { 1 } else { 0 })
}
pub fn oled_label(p: &Profile, index: usize) -> String {
    if !p.oled_name.is_empty() {
        return p.oled_name.clone();
    }
    let mut s = String::new();
    for c in p.name.to_uppercase().chars() {
        let r = match c {
            'Α' | 'Ά' => "A",
            'Β' => "V",
            'Γ' => "G",
            'Δ' => "D",
            'Ε' | 'Έ' => "E",
            'Ζ' => "Z",
            'Η' | 'Ή' => "I",
            'Θ' => "TH",
            'Ι' | 'Ί' | 'Ϊ' => "I",
            'Κ' => "K",
            'Λ' => "L",
            'Μ' => "M",
            'Ν' => "N",
            'Ξ' => "X",
            'Ο' | 'Ό' => "O",
            'Π' => "P",
            'Ρ' => "R",
            'Σ' => "S",
            'Τ' => "T",
            'Υ' | 'Ύ' | 'Ϋ' => "Y",
            'Φ' => "F",
            'Χ' => "CH",
            'Ψ' => "PS",
            'Ω' | 'Ώ' => "O",
            _ => "",
        };
        if c.is_ascii() && !c.is_ascii_control() {
            s.push(c);
        } else {
            s.push_str(r);
        }
    }
    let s = s.trim().chars().take(16).collect::<String>();
    if s.is_empty() {
        format!("MODE {}", index + 1)
    } else {
        s
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_round_trip() {
        let c = Config::default();
        c.validate().unwrap();
        assert_eq!(
            Config::import(&serde_json::to_string(&c).unwrap()).unwrap(),
            c
        );
    }
    #[test]
    fn legacy_import_preserves_actions() {
        let mut v = serde_json::to_value(Config::default()).unwrap();
        v["version"] = 1.into();
        let c = Config::import(&v.to_string()).unwrap();
        assert_eq!(c.version, 3);
        assert_eq!(
            c.profiles[0].keys[0].actions[0].kind,
            ActionKind::DiscordMute
        );
    }
    #[test]
    fn unknown_fields_rejected() {
        let mut v = serde_json::to_value(Config::default()).unwrap();
        v["secret"] = true.into();
        assert!(Config::import(&v.to_string()).is_err());
    }
    #[test]
    fn duplicate_keys_rejected() {
        let mut c = Config::default();
        c.profiles[0].keys[1].id = 1;
        assert!(c.validate().is_err());
    }
    #[test]
    fn delay_total_is_bounded() {
        let mut c = Config::default();
        c.profiles[0].keys[0].actions = vec![
            Action {
                kind: ActionKind::Delay,
                value: "5000".into(),
                args: "".into()
            };
            4
        ];
        assert!(c.validate().is_err());
    }
    #[test]
    fn loops_and_invalid_hotkeys_rejected() {
        for s in [
            "F13",
            "Ctrl+F21",
            "Ctrl+Ctrl+A",
            "Ctrl",
            "Ctrl++A",
            "A+B",
            "F0",
            "F25",
        ] {
            assert!(parse_hotkey(s).is_err(), "{s}");
        }
        assert_eq!(parse_hotkey("Ctrl+Shift+S").unwrap(), [17, 16, 83]);
    }
    #[test]
    fn ha_requires_object_and_safe_service() {
        for (v, a) in [
            ("light/turn_off", "{}"),
            ("light.turn_off", "[]"),
            ("light.turn_off", "bad"),
        ] {
            assert!(
                Action {
                    kind: ActionKind::HomeAssistant,
                    value: v.into(),
                    args: a.into()
                }
                .validate()
                .is_err()
            );
        }
    }
    #[test]
    fn mode_reference_must_exist() {
        let mut c = Config::default();
        c.profiles[0].keys[0].actions = vec![Action {
            kind: ActionKind::Mode,
            value: "missing".into(),
            args: "".into(),
        }];
        assert!(c.validate().is_err());
    }
    #[test]
    fn banks_are_unique() {
        let mods: HashSet<_> = (0..8).map(bank_modifiers).collect();
        assert_eq!(mods.len(), 8);
    }
    #[test]
    fn greek_oled_transliteration() {
        let p = Profile {
            id: "x".into(),
            name: "Φώτα".into(),
            oled_name: "".into(),
            keys: vec![],
        };
        assert_eq!(oled_label(&p, 0), "FOTA");
    }
}
