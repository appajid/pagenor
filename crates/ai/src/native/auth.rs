use super::*;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    time::Instant,
};
fn random() -> String {
    URL_SAFE_NO_PAD.encode(rand::random::<[u8; 32]>())
}
fn metadata() -> AiResult<Value> {
    json_response(
        client()?.get("https://auth.openai.com/.well-known/openid-configuration").send().map_err(|_| "Could not read authorization metadata")?,
    )
}
fn official(url: &str) -> AiResult<&str> {
    if url::Url::parse(url).is_ok_and(|u| {
        u.scheme() == "https" && u.host_str() == Some("auth.openai.com") && u.port().is_none() && u.username().is_empty() && u.password().is_none()
    }) {
        Ok(url)
    } else {
        Err("Authorization endpoint does not match the expected service".into())
    }
}
/// Loopback callback is local to this attempt, with fresh state, nonce and PKCE.
pub fn login(mut settings: Settings, cancel: &Cancellation) -> AiResult<Settings> {
    settings.validate()?;
    if settings.account.is_empty() && settings.accounts.len() >= 32 {
        return Err("Account limit reached. Sign out an unused account first.".into());
    }
    let old = if settings.account.is_empty() {
        None
    } else {
        Some(
            serde_json::from_str::<Credential>(&secret(&format!("account:{}", settings.account))?)
                .map_err(|_| "Selected account credentials are unavailable. Add the account and sign in again.")?,
        )
    };
    if let Some(c) = &old {
        c.validate()?;
    }
    if settings.host_id.is_empty() {
        let r = rand::random::<[u8; 16]>();
        settings.host_id = format!(
            "urn:uuid:{:08x}-{:04x}-4{:03x}-a{:03x}-{:012x}",
            u32::from_be_bytes([r[0], r[1], r[2], r[3]]),
            u16::from_be_bytes([r[4], r[5]]),
            u16::from_be_bytes([r[6], r[7]]) & 0xfff,
            u16::from_be_bytes([r[8], r[9]]) & 0xfff,
            u64::from_be_bytes([0, 0, r[10], r[11], r[12], r[13], r[14], r[15]])
        );
        settings.save(None)?;
    }
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|_| "Could not start the local authorization callback")?;
    listener.set_nonblocking(true).map_err(|_| "Could not configure the authorization callback")?;
    let redirect = format!("http://127.0.0.1:{}/auth/callback", listener.local_addr().map_err(|_| "Callback address is unavailable")?.port());
    let (state, nonce, verifier) = (random(), random(), random());
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let mut url = url::Url::parse("https://auth.openai.com/api/accounts/authorize").map_err(|_| "Invalid authorization URL")?;
    url.query_pairs_mut().extend_pairs([
        ("client_id", old.as_ref().map(|c| c.client_id.as_str()).unwrap_or("dynamic_agent_client")),
        ("ext_agent_host_id", &settings.host_id),
        ("response_type", "code"),
        ("redirect_uri", &redirect),
        ("scope", "openid profile email offline_access resource.invoke chatgpt.tokens.use.direct"),
        ("resource", "https://api.openai.com/v1"),
        ("state", &state),
        ("nonce", &nonce),
        ("code_challenge_method", "S256"),
        ("code_challenge", &challenge),
    ]);
    if let Some(c) = &old {
        url.query_pairs_mut().append_pair("id_token_hint", &c.id_token);
    } else {
        url.query_pairs_mut().append_pair("agent_name_hint", "Pagenor");
    }
    webbrowser::open(url.as_str()).map_err(|_| "Could not open the system browser")?;
    let deadline = Instant::now() + Duration::from_secs(300);
    let params = loop {
        cancel.check()?;
        if Instant::now() > deadline {
            return Err("Sign-in timed out. Retry.".into());
        }
        match listener.accept() {
            Ok((mut socket, _)) => {
                socket.set_read_timeout(Some(Duration::from_secs(2))).map_err(|_| "Could not read the callback")?;
                let mut buf = [0; 16384];
                let n = socket.read(&mut buf).unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);
                let first = request.lines().next().unwrap_or("");
                let path = first.strip_prefix("GET ").and_then(|s| s.split_whitespace().next()).unwrap_or("");
                let callback = url::Url::parse(&format!("http://127.0.0.1{path}"));
                let p = callback.as_ref().ok().map(|u| u.query_pairs().into_owned().collect::<HashMap<_, _>>()).unwrap_or_default();
                let valid = callback.as_ref().is_ok_and(|u| u.path() == "/auth/callback") && p.get("state") == Some(&state);
                let body = if valid { "Authorization received. Return to Pagenor." } else { "Invalid callback." };
                let _ = write!(
                    socket,
                    "HTTP/1.1 {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    if valid { "200 OK" } else { "400 Bad Request" },
                    body.len(),
                    body
                );
                if valid {
                    break p;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(100)),
            Err(_) => return Err("Authorization callback failed".into()),
        }
    };
    if params.contains_key("error") {
        return Err("Authorization was not completed. Sign in again.".into());
    }
    let cid = params
        .get("client_id")
        .cloned()
        .or_else(|| old.as_ref().map(|c| c.client_id.clone()))
        .filter(|c| !c.is_empty() && c.len() <= 512 && c != "dynamic_agent_client")
        .ok_or("Missing registered client identity")?;
    if old.as_ref().is_some_and(|o| o.client_id != cid) {
        return Err("Registered account identity does not match".into());
    }
    let code = params.get("code").ok_or("Authorization callback has no code")?;
    cancel.check()?;
    let v = json_response(
        client()?
            .post("https://auth.openai.com/api/accounts/oauth/token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", &cid),
                ("code", code),
                ("code_verifier", &verifier),
                ("redirect_uri", &redirect),
                ("resource", "https://api.openai.com/v1"),
            ])
            .send()
            .map_err(|_| "Authorization exchange failed; sign in again")?,
    )?;
    let token = v["id_token"].as_str().ok_or("Missing identity token")?;
    let meta = metadata()?;
    let jwks = json_response(
        client()?
            .get(official(meta["jwks_uri"].as_str().ok_or("Missing signing-key endpoint")?)?)
            .send()
            .map_err(|_| "Could not read signing keys")?,
    )?;
    let set: jsonwebtoken::jwk::JwkSet = serde_json::from_value(jwks).map_err(|_| "Invalid signing key")?;
    let header = jsonwebtoken::decode_header(token).map_err(|_| "Invalid identity token format")?;
    if header.alg != jsonwebtoken::Algorithm::RS256 {
        return Err("Unsupported identity signature algorithm".into());
    }
    let jwk = set.find(header.kid.as_deref().ok_or("Missing signature identifier")?).ok_or("Signing key was not found")?;
    let key = jsonwebtoken::DecodingKey::from_jwk(jwk).map_err(|_| "Invalid signing key")?;
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&[&cid]);
    validation.set_issuer(&["https://auth.openai.com"]);
    let claims = jsonwebtoken::decode::<Value>(token, &key, &validation).map_err(|_| "Identity token validation failed")?.claims;
    if claims["nonce"].as_str() != Some(&nonce) {
        return Err("Identity nonce does not match".into());
    }
    let subject = claims["sub"].as_str().ok_or("Missing account identity")?.to_string();
    if old.as_ref().is_some_and(|o| o.subject != subject) {
        return Err("Returned account does not match the selected account".into());
    }
    let mut c = Credential {
        client_id: cid.clone(),
        subject,
        email: claims["email"].as_str().unwrap_or("ChatGPT").into(),
        access_token: String::new(),
        refresh_token: String::new(),
        id_token: String::new(),
        expires_at: 0,
        scopes: vec![],
    };
    update_token(&mut c, &v)?;
    cancel.check()?;
    put_secret(&format!("account:{cid}"), &serde_json::to_string(&c).map_err(|_| "Could not encode credentials")?)?;
    let account = Account {
        id: cid.clone(),
        label: format!("{} · {}", c.email, cid.chars().rev().take(6).collect::<Vec<_>>().into_iter().rev().collect::<String>()),
        sharing: c.scopes.iter().any(|s| s == "chatgpt.tokens.use.direct"),
    };
    settings.accounts.retain(|a| a.id != cid);
    settings.accounts.push(account);
    settings.account = cid;
    settings.provider = "chatgpt".into();
    settings.model.clear();
    settings.save(None)?;
    Ok(settings)
}
pub fn logout(mut s: Settings) -> AiResult<Settings> {
    s.validate()?;
    let name = format!("account:{}", s.account);
    if let Ok(c) = serde_json::from_str::<Credential>(&secret(&name)?)
        && let Ok(meta) = metadata()
        && let Some(url) = meta["revocation_endpoint"].as_str()
        && let Ok(url) = official(url)
    {
        let _ = client()?
            .post(url)
            .form(&[("token", c.refresh_token.as_str()), ("token_type_hint", "refresh_token"), ("client_id", c.client_id.as_str())])
            .send();
    }
    match entry(&name)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(_) => return Err("Could not delete local credentials".into()),
    }
    s.accounts.retain(|a| a.id != s.account);
    s.account.clear();
    s.model.clear();
    s.save(None)?;
    Ok(s)
}
