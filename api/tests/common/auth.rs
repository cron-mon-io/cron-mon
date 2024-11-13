use std::time::{SystemTime, UNIX_EPOCH};

use rocket::http::Header;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use test_utils::{encode_jwt, RSA_EXPONENT, RSA_MODULUS};

use cron_mon_api::infrastructure::auth::Jwt;

pub async fn setup_mock_jwks_server(kid: &str) -> MockServer {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/certs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "keys": [
                {
                    "kid": kid.to_string(),
                    "kty": "RSA".to_string(),
                    "alg": "RS256".to_string(),
                    "n": RSA_MODULUS.to_string(),
                    "e": RSA_EXPONENT.to_string(),
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    std::env::set_var("KEYCLOAK_CERTS_URL", format!("{}/certs", mock_server.uri()));

    mock_server
}

pub fn create_auth_header<'a>(kid: &str, name: &str, tenant: &str) -> Header<'a> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Header::new(
        "Authorization",
        format!(
            "Bearer {}",
            encode_jwt(
                kid,
                &Jwt {
                    acr: "acr".to_string(),
                    azp: "azp".to_string(),
                    iss: "iss".to_string(),
                    jti: "jti".to_string(),
                    iat: now,
                    auth_time: now,
                    exp: now + 3600,
                    sub: "test-user".to_string(),
                    name: name.to_string(),
                    tenant: tenant.to_string(),
                }
            )
        ),
    )
}
