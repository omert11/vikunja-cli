use anyhow::{bail, Result};

pub struct Config {
    pub api_url: String,
    pub api_token: String,
}

pub fn load() -> Result<Config> {
    let api_url = std::env::var("VIKUNJA_API_URL")
        .ok()
        .filter(|s| !s.is_empty());
    let api_token = std::env::var("VIKUNJA_API_TOKEN")
        .ok()
        .filter(|s| !s.is_empty());
    match (api_url, api_token) {
        (Some(api_url), Some(api_token)) => Ok(Config { api_url, api_token }),
        _ => bail!(
            "VIKUNJA_API_URL and VIKUNJA_API_TOKEN must be set.\n\
             Example: export VIKUNJA_API_URL=http://localhost:3456/api/v1\n\
                      export VIKUNJA_API_TOKEN=tk_xxxxxxxx"
        ),
    }
}
