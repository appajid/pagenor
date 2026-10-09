//! Native services. No credentials cross the UI/control response or document boundary.
mod auth;
mod transport;
pub use auth::{login, logout};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Read;
const MAX_SETTINGS: usize = 1 << 20;
const MAX_SECRET: usize = 1 << 20;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
pub use transport::{Model, conversation, models, test_connection};

pub type AiResult<T> = std::result::Result<T, String>;
#[derive(Clone, Default)]
pub struct Cancellation(pub Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn check(&self) -> AiResult<()> {
        if self.0.load(Ordering::SeqCst) { Err("Stopped".into()) } else { Ok(()) }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub provider: String,
    pub endpoint: String,
    pub model: String,
    pub account: String,
    pub accounts: Vec<Account>,
    pub host_id: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            provider: "compatible".into(),
            endpoint: "https://api.openai.com/v1".into(),
            model: String::new(),
            account: String::new(),
            accounts: vec![],
            host_id: String::new(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub label: String,
    pub sharing: bool,
}
#[derive(Clone, Serialize, Deserialize)]
struct Credential {
    client_id: String,
    subject: String,
    email: String,
    access_token: String,
    refresh_token: String,
    id_token: String,
    expires_at: u64,
    scopes: Vec<String>,
}
fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}
fn client() -> AiResult<Client> {
    Client::builder()
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Could not create the network client".into())
}
fn entry(name: &str) -> AiResult<keyring::Entry> {
    keyring::Entry::new("Pagenor AI", name).map_err(|_| "System credential storage is unavailable".into())
}
fn secret(name: &str) -> AiResult<String> {
    match entry(name)?.get_password() {
        Ok(v) if v.len() <= MAX_SECRET => Ok(v),
        Ok(_) => Err("Credential exceeds the size limit".into()),
        Err(keyring::Error::NoEntry) => Ok(String::new()),
        Err(_) => Err("Could not read system credentials. Unlock the credential store and retry.".into()),
    }
}
fn put_secret(name: &str, value: &str) -> AiResult<()> {
    if value.len() > MAX_SECRET || name.len() > 8192 {
        return Err("Credential exceeds the size limit".into());
    }
    entry(name)?.set_password(value).map_err(|_| "Could not store credentials; settings were not saved.".into())
}
fn config_path() -> AiResult<PathBuf> {
    let base = if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support/Pagenor"))
    } else if cfg!(windows) {
        std::env::var_os("APPDATA").map(|h| PathBuf::from(h).join("Pagenor"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .map(|p| p.join("pagenor"))
    };
    Ok(base.ok_or("Could not locate the user settings directory")?.join("ai.json"))
}
impl Settings {
    pub fn load() -> AiResult<Self> {
        let path = config_path()?;
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(_) => return Err("Could not read AI settings".into()),
        };
        let mut bytes = Vec::new();
        file.take(MAX_SETTINGS as u64 + 1).read_to_end(&mut bytes).map_err(|_| "Could not read AI settings")?;
        if bytes.len() > MAX_SETTINGS {
            return Err("AI settings exceed the size limit".into());
        }
        let settings: Self = serde_json::from_slice(&bytes).map_err(|_| "AI settings are damaged. Review and save the settings again.")?;
        settings.validate()?;
        Ok(settings)
    }
    pub fn validate(&self) -> AiResult<()> {
        if self.endpoint.len() > 4096
            || self.model.len() > 512
            || self.account.len() > 512
            || self.host_id.len() > 128
            || self.accounts.len() > 32
            || self.accounts.iter().any(|a| a.id.is_empty() || a.id.len() > 512 || a.label.len() > 2048)
        {
            return Err("AI settings exceed field/count limits".into());
        }
        endpoint(&self.endpoint)?;
        if !matches!(self.provider.as_str(), "compatible" | "chatgpt") {
            return Err("Unknown model provider".into());
        }
        Ok(())
    }
    pub fn save(&self, key: Option<&str>) -> AiResult<()> {
        self.validate()?;
        if key.is_some_and(|k| k.len() > 65536) {
            return Err("API key exceeds the size limit".into());
        }
        if !matches!(self.provider.as_str(), "compatible" | "chatgpt") {
            return Err("Unknown model provider".into());
        }
        if let Some(key) = key {
            put_secret(&format!("api:{}", self.endpoint.trim_end_matches('/')), key)?;
        }
        let path = config_path()?;
        std::fs::create_dir_all(path.parent().ok_or("Invalid settings directory")?).map_err(|_| "Could not create the settings directory")?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self).map_err(|_| "Could not encode settings")?).map_err(|_| "Could not write AI settings")?;
        std::fs::rename(tmp, path).map_err(|_| "Could not save AI settings".into())
    }
}
fn endpoint(value: &str) -> AiResult<url::Url> {
    let u = url::Url::parse(value).map_err(|_| "Invalid service endpoint")?;
    if !matches!(u.scheme(), "http" | "https")
        || u.host_str().is_none()
        || !u.username().is_empty()
        || u.password().is_some()
        || u.query().is_some()
        || u.fragment().is_some()
    {
        return Err("The endpoint must be an HTTP(S) URL without credentials, query parameters or fragments".into());
    }
    Ok(u)
}
fn json_response(response: reqwest::blocking::Response) -> AiResult<Value> {
    let status = response.status();
    if !status.is_success() {
        return Err(format!("Service returned HTTP {}. Check account permissions, usage limits, model and endpoint.", status.as_u16()));
    }
    let mut bytes = Vec::new();
    response.take(16 * 1024 * 1024 + 1).read_to_end(&mut bytes).map_err(|_| "Could not read the response")?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("Response exceeds the size limit".into());
    }
    serde_json::from_slice(&bytes).map_err(|_| "Service returned invalid JSON".into())
}
fn credentials(settings: &Settings) -> AiResult<(String, String)> {
    settings.validate()?;
    if settings.provider == "chatgpt" {
        let mut c: Credential = serde_json::from_str(&secret(&format!("account:{}", settings.account))?).map_err(|_| "Sign in with ChatGPT first")?;
        c.validate()?;
        if c.expires_at <= now().saturating_add(120) {
            let v = json_response(
                client()?
                    .post("https://auth.openai.com/api/accounts/oauth/token")
                    .form(&[
                        ("grant_type", "refresh_token"),
                        ("client_id", c.client_id.as_str()),
                        ("refresh_token", c.refresh_token.as_str()),
                        ("resource", "https://api.openai.com/v1"),
                    ])
                    .send()
                    .map_err(|_| "Token refresh failed; sign in again")?,
            )?;
            update_token(&mut c, &v)?;
            put_secret(&format!("account:{}", settings.account), &serde_json::to_string(&c).map_err(|_| "Could not encode credentials")?)?;
        }
        if !c.scopes.iter().any(|s| s == "chatgpt.tokens.use.direct") {
            return Err("This account has not granted ChatGPT plan inference permission.".into());
        }
        Ok(("https://api.openai.com/v1".into(), c.access_token))
    } else {
        endpoint(&settings.endpoint)?;
        Ok((settings.endpoint.trim_end_matches('/').into(), secret(&format!("api:{}", settings.endpoint.trim_end_matches('/')))?))
    }
}
fn update_token(c: &mut Credential, v: &Value) -> AiResult<()> {
    c.access_token = v["access_token"].as_str().filter(|s| !s.is_empty()).ok_or("Authorization response has no access token")?.into();
    if let Some(t) = v["refresh_token"].as_str() {
        c.refresh_token = t.into();
    }
    if let Some(t) = v["id_token"].as_str() {
        c.id_token = t.into();
    }
    c.expires_at = now().saturating_add(v["expires_in"].as_u64().unwrap_or(3600).min(365 * 24 * 3600));
    if let Some(s) = v["scope"].as_str() {
        c.scopes = s.split_whitespace().map(str::to_owned).collect();
    }
    c.validate()?;
    Ok(())
}
impl Credential {
    fn validate(&self) -> AiResult<()> {
        if self.client_id.is_empty()
            || self.client_id.len() > 512
            || self.subject.len() > 4096
            || self.email.len() > 1024
            || self.access_token.len() > 65536
            || self.refresh_token.len() > 65536
            || self.id_token.len() > 65536
            || self.scopes.len() > 128
            || self.scopes.iter().any(|s| s.len() > 256)
        {
            return Err("Credential exceeds field/count limits".into());
        }
        Ok(())
    }
}

pub fn open_usage() -> AiResult<()> {
    webbrowser::open("https://chatgpt.com/settings/usage").map_err(|_| "Could not open the browser".into())
}
