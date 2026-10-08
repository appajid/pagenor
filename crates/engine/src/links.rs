//! Pagenor project links (Help menu, CLI and MCP).

use serde_json::{Value, json};

/// The app's repository name.
pub const APP: &str = "pagenor";
/// No Pagenor Discord destination has been supplied. Kept for API compatibility.
pub const DISCORD: &str = "";
/// The repository is Pagenor's current project page.
pub const WEBSITE: &str = GITHUB;
pub const APP_PAGE: &str = GITHUB;
/// Pagenor's source repository.
pub const GITHUB: &str = "https://github.com/appajid/pagenor";
/// Where to report bugs and request features.
pub const ISSUES: &str = "https://github.com/appajid/pagenor/issues";

/// All links as JSON (the `app.links` command).
pub fn all() -> Value {
    json!({
        "discord": get("discord"),
        "website": WEBSITE,
        "appPage": APP_PAGE,
        "github": GITHUB,
        "issues": ISSUES,
    })
}

/// A link by key (`discord`, `website`, `appPage`, `github`, `issues`).
pub fn get(key: &str) -> Option<&'static str> {
    let url = match key {
        "discord" => DISCORD,
        "website" => WEBSITE,
        "appPage" => APP_PAGE,
        "github" => GITHUB,
        "issues" => ISSUES,
        _ => return None,
    };
    (!url.is_empty()).then_some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_follow_the_app_slug() {
        assert_eq!(APP_PAGE, GITHUB);
        assert!(GITHUB.ends_with(&format!("/appajid/{APP}")));
        assert!(ISSUES.starts_with(GITHUB));
        assert!(get("discord").is_none());
        assert!(all()["discord"].is_null());
        for k in ["website", "appPage", "github", "issues"] {
            assert!(get(k).is_some_and(|u| u.starts_with("https://")), "{k}");
            assert_eq!(all()[k], get(k).unwrap());
        }
    }
}
