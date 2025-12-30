use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use crate::config::{Config, OAuth2Config};

const REDIRECT_URI: &str = "http://localhost:7171/auth/redirect";
const LISTEN_ADDR: &str = "127.0.0.1:7171";
const SCOPES: &str = "openid profile read_user write_repository api";
// Same client ID as glab for gitlab.com
const DEFAULT_CLIENT_ID: &str = "41d48f9422ebd655dd9cf2947d6979681dfaddc6d0c56f7628f6ada59559af1e";

pub fn default_client_id() -> &'static str {
    DEFAULT_CLIENT_ID
}

pub struct AuthFlow {
    host: String,
    client_id: String,
    code_verifier: String,
}

impl AuthFlow {
    pub fn new(host: &str, client_id: &str) -> Self {
        Self {
            host: host.trim_end_matches('/').to_string(),
            client_id: client_id.to_string(),
            code_verifier: generate_code_verifier(),
        }
    }

    fn code_challenge(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.code_verifier.as_bytes());
        URL_SAFE_NO_PAD.encode(hasher.finalize())
    }

    pub fn authorization_url(&self) -> String {
        let challenge = self.code_challenge();
        format!(
            "{}/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256",
            self.host,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(REDIRECT_URI),
            urlencoding::encode(SCOPES),
            urlencoding::encode(&challenge),
        )
    }

    pub fn wait_for_callback(&self) -> Result<String> {
        let listener = TcpListener::bind(LISTEN_ADDR)
            .context("Failed to bind to port 7171. Is another instance running?")?;

        println!("Waiting for authorization callback...");

        let (mut stream, _) = listener.accept().context("Failed to accept connection")?;
        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        let code = extract_code_from_request(&request_line)?;

        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
            <html><body><h1>Authorization successful!</h1>\
            <p>You can close this window and return to the terminal.</p></body></html>";
        stream.write_all(response.as_bytes())?;

        Ok(code)
    }

    pub async fn exchange_code(&self, code: &str) -> Result<OAuth2Config> {
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/oauth/token", self.host))
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("code", code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", REDIRECT_URI),
                ("code_verifier", &self.code_verifier),
            ])
            .send()
            .await
            .context("Failed to exchange authorization code")?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("Token exchange failed: {}", body));
        }

        parse_token_response(&self.client_id, &body)
    }
}

pub async fn refresh_token(config: &mut Config) -> Result<()> {
    let oauth2 = config
        .oauth2
        .as_ref()
        .ok_or_else(|| anyhow!("No OAuth2 configuration found"))?;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/oauth/token", config.host()))
        .form(&[
            ("client_id", oauth2.client_id.as_str()),
            ("refresh_token", oauth2.refresh_token.as_str()),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .context("Failed to refresh token")?;

    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(anyhow!("Token refresh failed: {}", body));
    }

    let new_oauth2 = parse_token_response(&oauth2.client_id, &body)?;
    config.oauth2 = Some(new_oauth2);
    config.save()?;

    Ok(())
}

fn generate_code_verifier() -> String {
    let bytes: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen()).collect();
    URL_SAFE_NO_PAD.encode(&bytes)
}

fn extract_code_from_request(request_line: &str) -> Result<String> {
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(anyhow!("Invalid HTTP request"));
    }

    let path = parts[1];
    let query_start = path
        .find('?')
        .ok_or_else(|| anyhow!("No query string in callback"))?;
    let query = &path[query_start + 1..];

    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if let (Some(key), Some(value)) = (kv.next(), kv.next()) {
            if key == "code" {
                return Ok(urlencoding::decode(value)?.into_owned());
            }
            if key == "error" {
                let desc = query
                    .split('&')
                    .find_map(|p| {
                        let mut kv = p.splitn(2, '=');
                        if kv.next() == Some("error_description") {
                            kv.next()
                                .map(|v| urlencoding::decode(v).unwrap_or_default().into_owned())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                return Err(anyhow!("Authorization failed: {} - {}", value, desc));
            }
        }
    }

    Err(anyhow!("No authorization code in callback"))
}

fn parse_token_response(client_id: &str, body: &str) -> Result<OAuth2Config> {
    let json: serde_json::Value =
        serde_json::from_str(body).context("Failed to parse token response")?;

    let access_token = json["access_token"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing access_token"))?
        .to_string();

    let refresh_token = json["refresh_token"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing refresh_token"))?
        .to_string();

    let expires_in = json["expires_in"].as_i64().unwrap_or(7200);
    let expires_at = Utc::now() + Duration::seconds(expires_in);

    Ok(OAuth2Config {
        client_id: client_id.to_string(),
        access_token,
        refresh_token,
        expires_at,
    })
}
