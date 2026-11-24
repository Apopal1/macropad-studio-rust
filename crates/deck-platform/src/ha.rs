#[cfg(not(windows))]
use anyhow::bail;
use anyhow::{Context, Result, ensure};
use reqwest::{blocking::Client, redirect::Policy};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    io::Read,
    sync::{Arc, Mutex},
    time::Duration,
};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub state: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub response_required: bool,
}
#[derive(Clone, Default, Debug)]
pub struct Catalog {
    pub entities: Vec<Entity>,
    pub services: Vec<Service>,
}
#[derive(Serialize, Deserialize)]
struct Credentials {
    url: String,
    token: String,
}
struct Session {
    credentials: Credentials,
    catalog: Catalog,
}
#[derive(Clone)]
pub struct HomeAssistant {
    client: Client,
    session: Arc<Mutex<Option<Session>>>,
}
impl HomeAssistant {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .redirect(Policy::none())
                .connect_timeout(Duration::from_secs(3))
                .timeout(Duration::from_secs(8))
                .build()?,
            session: Arc::new(Mutex::new(None)),
        })
    }
    pub fn connected(&self) -> bool {
        self.session.lock().is_ok_and(|s| s.is_some())
    }
    pub fn url(&self) -> String {
        self.session
            .lock()
            .ok()
            .and_then(|s| s.as_ref().map(|s| s.credentials.url.clone()))
            .unwrap_or_default()
    }
    pub fn disconnect(&self) {
        if let Ok(mut s) = self.session.lock() {
            *s = None;
        }
    }
    fn request(&self, c: &Credentials, path: &str, body: Option<&Value>) -> Result<Value> {
        let url = format!("{}{path}", c.url);
        let req = if let Some(b) = body {
            self.client.post(&url).json(b)
        } else {
            self.client.get(&url)
        };
        let response = req.bearer_auth(&c.token).send().map_err(|_| {
            anyhow::anyhow!("Δεν απάντησε το HA. Έλεγξε διεύθυνση, δίκτυο και πιστοποιητικό.")
        })?;
        let status = response.status();
        ensure!(
            status.is_success(),
            "Το HA επέστρεψε HTTP {}. Έλεγξε token και δικαιώματα.",
            status.as_u16()
        );
        let mut bytes = Vec::new();
        response
            .take(8 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .context("Δεν διαβάστηκε η απάντηση HA.")?;
        ensure!(
            bytes.len() <= 8 * 1024 * 1024,
            "Η απάντηση HA είναι πολύ μεγάλη."
        );
        serde_json::from_slice(&bytes).context("Το HA δεν επέστρεψε έγκυρο JSON.")
    }
    pub fn connect(&self, url: &str, token: String) -> Result<Catalog> {
        let url = validate_url(url)?;
        ensure!(
            !token.is_empty() && token.len() <= 8192 && !token.chars().any(char::is_whitespace),
            "Μη έγκυρο token HA."
        );
        let credentials = Credentials { url, token };
        self.request(&credentials, "/api/", None)?;
        let catalog = self.fetch_catalog(&credentials)?;
        *self
            .session
            .lock()
            .map_err(|_| anyhow::anyhow!("HA lock error"))? = Some(Session {
            credentials,
            catalog: catalog.clone(),
        });
        Ok(catalog)
    }
    fn fetch_catalog(&self, c: &Credentials) -> Result<Catalog> {
        let states = self.request(c, "/api/states", None)?;
        let services = self.request(c, "/api/services", None)?;
        let entities = states
            .as_array()
            .context("Μη έγκυρος κατάλογος HA.")?
            .iter()
            .filter_map(|v| {
                Some(Entity {
                    id: v["entity_id"].as_str()?.into(),
                    name: v["attributes"]["friendly_name"]
                        .as_str()
                        .or(v["entity_id"].as_str())?
                        .into(),
                    state: v["state"].as_str().unwrap_or("unknown").into(),
                })
            })
            .collect();
        let mut actions = Vec::new();
        for domain in services.as_array().context("Μη έγκυρες ενέργειες HA.")? {
            if let (Some(d), Some(s)) = (domain["domain"].as_str(), domain["services"].as_object())
            {
                for (id, m) in s {
                    actions.push(Service {
                        id: format!("{d}.{id}"),
                        name: m["name"].as_str().unwrap_or(id).into(),
                        response_required: m["response"]["optional"].as_bool() == Some(false),
                    });
                }
            }
        }
        Ok(Catalog {
            entities,
            services: actions,
        })
    }
    pub fn refresh(&self) -> Result<Catalog> {
        let c = {
            let s = self.session.lock().unwrap();
            let s = s.as_ref().context("Σύνδεσε πρώτα το HA.")?;
            Credentials {
                url: s.credentials.url.clone(),
                token: s.credentials.token.clone(),
            }
        };
        let catalog = self.fetch_catalog(&c)?;
        if let Some(s) = self.session.lock().unwrap().as_mut() {
            s.catalog = catalog.clone();
        }
        Ok(catalog)
    }
    pub fn call(&self, service: &str, data: &str) -> Result<()> {
        let a = deck_domain::Action {
            kind: deck_domain::ActionKind::HomeAssistant,
            value: service.into(),
            args: data.into(),
        };
        a.validate()?;
        let (c, response) = {
            let s = self.session.lock().unwrap();
            let s = s
                .as_ref()
                .context("Το Home Assistant δεν είναι συνδεδεμένο.")?;
            let response = s
                .catalog
                .services
                .iter()
                .find(|s| s.id == service)
                .is_some_and(|s| s.response_required);
            (
                Credentials {
                    url: s.credentials.url.clone(),
                    token: s.credentials.token.clone(),
                },
                response,
            )
        };
        let path = format!(
            "/api/services/{}{}",
            service.replace('.', "/"),
            if response { "?return_response" } else { "" }
        );
        self.request(&c, &path, Some(&serde_json::from_str(data)?))?;
        Ok(())
    }
    pub fn protected(&self) -> Result<Option<Vec<u8>>> {
        #[cfg(windows)]
        {
            let s = self.session.lock().unwrap();
            let s = s.as_ref().context("HA disconnected")?;
            Ok(Some(protect(&serde_json::to_vec(&s.credentials)?, false)?))
        }
        #[cfg(not(windows))]
        {
            Ok(None)
        }
    }
    pub fn restore(&self, bytes: &[u8]) -> Result<Catalog> {
        #[cfg(windows)]
        {
            let c: Credentials = serde_json::from_slice(&protect(bytes, true)?)?;
            self.connect(&c.url, c.token)
        }
        #[cfg(not(windows))]
        {
            let _ = bytes;
            bail!("Ανάκτηση token μόνο με Windows DPAPI.")
        }
    }
}
pub fn validate_url(value: &str) -> Result<String> {
    ensure!(
        value.len() <= 1000
            && !value
                .trim()
                .chars()
                .any(|c| c.is_whitespace() || c.is_control()),
        "Μη έγκυρη διεύθυνση HA."
    );
    let u = url::Url::parse(value.trim()).context("Χρειάζεται η βασική διεύθυνση HA.")?;
    ensure!(
        matches!(u.scheme(), "http" | "https")
            && u.host_str().is_some()
            && u.username().is_empty()
            && u.password().is_none()
            && u.query().is_none()
            && u.fragment().is_none(),
        "Χρειάζεται HTTP/HTTPS διεύθυνση HA χωρίς κωδικούς, query ή fragment."
    );
    ensure!(
        !u.path()
            .split('/')
            .any(|p| matches!(p, "api" | "lovelace" | "dashboard" | "profile" | "config")),
        "Βάλε τη βασική διεύθυνση HA, χωρίς /api ή /lovelace."
    );
    Ok(u.as_str().trim_end_matches('/').into())
}
#[cfg(windows)]
fn protect(bytes: &[u8], decrypt: bool) -> Result<Vec<u8>> {
    use windows_sys::Win32::{Foundation::LocalFree, Security::Cryptography::*};
    let source = CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr().cast_mut(),
    };
    let mut out = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    // SAFETY: DPAPI borrows input for synchronous call, allocates output with LocalAlloc. LocalFree is always called after copying.
    let ok = unsafe {
        if decrypt {
            CryptUnprotectData(
                &source,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        } else {
            CryptProtectData(
                &source,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        }
    };
    ensure!(
        ok != 0,
        "Δεν ήταν δυνατή η προστασία/ανάκτηση token μέσω DPAPI."
    );
    let result = unsafe { std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec() };
    unsafe {
        LocalFree(out.pbData.cast());
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_urls_rejected() {
        for u in [
            "file:///etc/passwd",
            "http://a:b@localhost",
            "http://localhost/api",
            "https://host/#x",
            "https://host/?x=1",
            "https://host/lovelace",
        ] {
            assert!(validate_url(u).is_err(), "{u}");
        }
        assert_eq!(
            validate_url("http://127.0.0.1:8123/").unwrap(),
            "http://127.0.0.1:8123"
        );
    }
    #[test]
    fn ha_mock_connection_catalog_and_dispatch() {
        use std::{io::Write, net::TcpListener};
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        let worker = std::thread::spawn(move || {
            let mut paths = Vec::new();
            for body in [
                r#"{"message":"API running."}"#,
                r#"[{"entity_id":"light.office","state":"on","attributes":{"friendly_name":"Office"}}]"#,
                r#"[{"domain":"light","services":{"turn_off":{"name":"Turn off"}}}]"#,
                "[]",
            ] {
                let (mut s, _) = l.accept().unwrap();
                s.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                let mut b = [0; 16384];
                let n = s.read(&mut b).unwrap();
                let request = String::from_utf8_lossy(&b[..n]);
                paths.push(request.lines().next().unwrap().to_string());
                assert!(
                    request
                        .to_ascii_lowercase()
                        .contains("authorization: bearer test-token")
                );
                write!(s,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
            }
            paths
        });
        let h = HomeAssistant::new().unwrap();
        let c = h
            .connect(&format!("http://{addr}"), "test-token".into())
            .unwrap();
        assert_eq!(c.entities[0].id, "light.office");
        h.call("light.turn_off", r#"{"entity_id":"light.office"}"#)
            .unwrap();
        let paths = worker.join().unwrap();
        assert!(paths[3].starts_with("POST /api/services/light/turn_off "));
        h.disconnect();
        assert!(!h.connected());
    }
}
