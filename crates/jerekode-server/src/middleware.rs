//! HTTP middleware: env-gated basic auth and CORS helpers (OpenCode-compatible enough).

use crate::options::BasicAuthMode;
use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, Request, Response, StatusCode, header},
    middleware::Next,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;

/// Resolve server password from OpenCode / jerekode env names.
pub fn server_password_from_env() -> Option<String> {
    env_first(&[
        "OPENCODE_SERVER_PASSWORD",
        "JEREKODE_SERVER_PASSWORD",
        "JEREKO_SERVER_PASSWORD",
    ])
}

/// Resolve server username; defaults to `"opencode"` when password is set.
pub fn server_username_from_env() -> String {
    env_first(&[
        "OPENCODE_SERVER_USERNAME",
        "JEREKODE_SERVER_USERNAME",
        "JEREKO_SERVER_USERNAME",
    ])
    .unwrap_or_else(|| "opencode".into())
}

fn env_first(names: &[&str]) -> Option<String> {
    names
        .iter()
        .find_map(|n| std::env::var(n).ok())
        .filter(|v| !v.is_empty())
}

fn resolve_credentials(mode: &BasicAuthMode) -> Option<(String, String)> {
    match mode {
        BasicAuthMode::FromEnv => {
            let password = server_password_from_env()?;
            Some((server_username_from_env(), password))
        }
        BasicAuthMode::Fixed { username, password } => Some((username.clone(), password.clone())),
        BasicAuthMode::Disabled => None,
    }
}

/// Axum middleware: require HTTP basic auth when configured (env or fixed).
/// OPTIONS (CORS preflight) is always allowed through.
pub async fn basic_auth_middleware(
    axum::extract::State(mode): axum::extract::State<BasicAuthMode>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    if req.method() == axum::http::Method::OPTIONS {
        return next.run(req).await;
    }

    let Some((username, password)) = resolve_credentials(&mode) else {
        return next.run(req).await;
    };

    if credentials_match(req.headers(), &username, &password) {
        return next.run(req).await;
    }

    let mut res = Response::new(Body::from("Unauthorized"));
    *res.status_mut() = StatusCode::UNAUTHORIZED;
    res.headers_mut().insert(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static("Basic realm=\"jerekode\""),
    );
    res
}

fn credentials_match(headers: &HeaderMap, username: &str, password: &str) -> bool {
    let Some(header) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    let Some(encoded) = header
        .strip_prefix("Basic ")
        .or_else(|| header.strip_prefix("basic "))
    else {
        return false;
    };
    let Ok(decoded) = B64.decode(encoded.trim()) else {
        return false;
    };
    let Ok(pair) = String::from_utf8(decoded) else {
        return false;
    };
    let Some((user, pass)) = pair.split_once(':') else {
        return false;
    };
    user == username && pass == password
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_match_accepts_valid_basic() {
        let mut headers = HeaderMap::new();
        let token = B64.encode("opencode:secret");
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {token}")).unwrap(),
        );
        assert!(credentials_match(&headers, "opencode", "secret"));
        assert!(!credentials_match(&headers, "opencode", "wrong"));
    }
}
