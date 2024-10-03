use async_trait::async_trait;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use moka::sync::Cache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::{Jwt, JwtAuth};
use crate::errors::Error;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Jwk {
    kid: String,
    kty: String,
    alg: String,
    n: String,
    e: String,
}

pub struct JwtAuthService {
    client: Client,
    cache: Cache<String, Jwk>,
    certs_url: String,
}

#[async_trait]
impl JwtAuth for JwtAuthService {
    async fn decode_jwt(&self, token: &str) -> Result<Jwt, Error> {
        let kid = self.get_kid(token)?;

        let decoding_key = self.get_decoding_key(&kid).await?;
        let mut validator = Validation::new(Algorithm::RS256);
        validator.set_audience(&["cron-mon"]);
        match jsonwebtoken::decode::<Jwt>(token, &decoding_key, &validator) {
            Ok(token_data) => Ok(token_data.claims),
            Err(e) => match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    Err(Error::Unauthorized("Token has expired".to_string()))
                }
                _ => Err(Error::AuthenticationError(e.to_string())),
            },
        }
    }
}

impl JwtAuthService {
    pub fn new(certs_url: String, cache: Cache<String, Jwk>) -> Self {
        Self {
            client: Client::new(),
            certs_url,
            cache,
        }
    }

    fn get_kid(&self, token: &str) -> Result<String, Error> {
        match jsonwebtoken::decode_header(token) {
            Ok(header) => match header.kid {
                Some(kid) => Ok(kid),
                None => Err(Error::AuthenticationError(
                    "Token header doesn't include 'kid'".to_string(),
                )),
            },
            Err(e) => Err(Error::AuthenticationError(format!(
                "Failed to decode token header: {}",
                e
            ))),
        }
    }

    async fn get_decoding_key(&self, kid: &str) -> Result<DecodingKey, Error> {
        let jwk =
            match self.cache.get(kid) {
                Some(cached) => cached,
                None => {
                    let jwks = self.fetch_jwks().await?;
                    let jwk = jwks.keys.iter().find(|k| k.kid == kid).ok_or(
                        Error::AuthenticationError(
                            "Failed to find relevant JWK for token".to_string(),
                        ),
                    )?;
                    self.cache.insert(kid.to_string(), jwk.clone());
                    jwk.clone()
                }
            };

        match DecodingKey::from_rsa_components(&jwk.n, &jwk.e) {
            Ok(decoding_key) => Ok(decoding_key),
            Err(_) => Err(Error::AuthenticationError(
                "Failed to create decoding key".to_string(),
            )),
        }
    }

    async fn fetch_jwks(&self) -> Result<Jwks, Error> {
        info!("Fetching JWKS from: {}", &self.certs_url);
        let response = self
            .client
            .get(&self.certs_url)
            .send()
            .await
            .map_err(|e| Error::AuthenticationError(format!("Failed to fetch JWKS: {}", e)))?;

        let status = response.status();
        if status.is_success() {
            let jwks = response
                .json::<Jwks>()
                .await
                .map_err(|e| Error::AuthenticationError(format!("Failed to parse JWKS: {}", e)))?;
            Ok(jwks)
        } else {
            // Pretty sure it's safe to unwrap here because I wasn't able to create a test case for
            // `text` returning an error.
            let error = response.text().await.unwrap();
            Err(Error::AuthenticationError(format!(
                "JWKS endpoint responded with {}: {}",
                status.as_u16(),
                error
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use moka::sync::Cache;
    use rstest::rstest;
    use serde_json::json;
    use test_utils::{encode_jwt, RSA_EXPONENT, RSA_MODULUS};
    use tracing_test::traced_test;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use test_utils::logging::TracingLog;

    use crate::errors::Error;
    use crate::infrastructure::auth::{Jwt, JwtAuth};

    use super::{Jwk, Jwks, JwtAuthService};

    #[traced_test]
    #[tokio::test]
    async fn test_decode_jwt_without_cached_jwk() {
        // WARNING: This is a valid JWK but it absolutely should not be used in production.
        let jwk = Jwk {
            kid: "DgdoxSuZTY1qxPoCQYjNU9sjzNrQN2-vbMDhWX0ZY9M".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            n: RSA_MODULUS.to_string(),
            e: RSA_EXPONENT.to_string(),
        };

        // We need to create a valid token for this test, because we need a real signature so that
        // jsonwebtoken can decode and verify it, and we need dynamic iat and exp otherwise
        // jsonwebtoken will reject it as expired.
        let original_jwt = setup_jwt();
        let token = encode_jwt(&jwk.kid, &original_jwt);

        // Mock the JWKS endpoint to return our dummy JWK.
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Jwks {
                keys: vec![jwk.clone()],
            }))
            .mount(&mock_server)
            .await;

        let cache = Cache::new(10);
        let auth_service =
            JwtAuthService::new(format!("{}/certs", mock_server.uri()), cache.clone());

        // Decode the token - we should get the original JWT back.
        let jwt = auth_service.decode_jwt(&token).await;
        assert_eq!(jwt, Ok(original_jwt));

        // We should have fetched the JWK.
        let received_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(received_requests.len(), 1);
        assert_eq!(received_requests[0].url.path(), "/certs");
        logs_assert(|logs| {
            let logs = TracingLog::from_logs(logs);
            assert_eq!(logs.len(), 1);
            assert_eq!(
                logs[0].body,
                format!("Fetching JWKS from: {}/certs", mock_server.uri())
            );
            Ok(())
        });

        // We should have cached the JWK.
        let cached_jwk = cache.get(&jwk.kid);
        assert!(cached_jwk.is_some());
    }

    #[traced_test]
    #[tokio::test]
    async fn test_decode_jwt_with_cached_jwk() {
        // WARNING: This is a valid JWK but it absolutely should not be used in production.
        let jwk = Jwk {
            kid: "DgdoxSuZTY1qxPoCQYjNU9sjzNrQN2-vbMDhWX0ZY9M".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            n: RSA_MODULUS.to_string(),
            e: RSA_EXPONENT.to_string(),
        };

        // We need to create a valid token for this test, because we need a real signature so that
        // jsonwebtoken can decode and verify it, and we need dynamic iat and exp otherwise
        // jsonwebtoken will reject it as expired.
        let original_jwt = setup_jwt();
        let token = encode_jwt(&jwk.kid, &original_jwt);

        // Setup a cache with the dummy JWK.
        let cache = Cache::new(10);
        cache.insert(jwk.kid.clone(), jwk.clone());

        let auth_service =
            JwtAuthService::new("http://127.0.0.1:1234/certs".to_string(), cache.clone());

        // We should have not fetched the JWK.
        logs_assert(|logs| {
            assert!(logs.is_empty());
            Ok(())
        });

        // Decode the token - we should get the original JWT back.
        let jwt = auth_service.decode_jwt(&token).await;
        assert_eq!(jwt, Ok(original_jwt));
    }

    #[rstest]
    #[case(
        "not-a-jwt",
        Error::AuthenticationError("Failed to decode token header: InvalidToken".to_string())
    )]
    #[case(
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.\
         dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U",
         Error::AuthenticationError("Token header doesn't include 'kid'".to_string())
    )]
    #[tokio::test]
    async fn test_decode_jwt_with_invalid_token(#[case] token: &str, #[case] expected: Error) {
        let auth_service =
            JwtAuthService::new("http://localhost:1234/certs".to_string(), Cache::new(10));

        // Decode the token - we should get an error because it's not a valid JWT.
        let jwt = auth_service.decode_jwt(token).await;
        assert_eq!(jwt, Err(expected));
    }

    #[tokio::test]
    async fn test_decode_jwt_with_unsupport_kid() {
        // Setup a fake JWK and mock the JWKS endpoint to return it.
        let jwk = Jwk {
            kid: "test".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            n: "dGVzdA".to_string(),
            e: "QVFBQg".to_string(),
        };
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Jwks {
                keys: vec![jwk.clone()],
            }))
            .mount(&mock_server)
            .await;

        let auth_service =
            JwtAuthService::new(format!("{}/certs", mock_server.uri()), Cache::new(10));

        let jwt_result = auth_service.decode_jwt(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IkRnZG94U3VaVFkxcXhQb0NRWWpOVTlzanpOclFOMi\
            12Yk1EaFdYMFpZOU0ifQ.eyJzdWIiOiIxMjM0NTY3ODkwIiwiaWF0IjoxNzI0NTI5OTk3LCJuYW1lIjoiSG93YX\
            JkIFNtaXRoIn0.QglRgN6cb1UvbOGuif7fUbfdM8yurYIYX6adGsRtsaY"
        ).await;
        assert_eq!(
            jwt_result,
            Err(Error::AuthenticationError(
                "Failed to find relevant JWK for token".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn test_decode_jwt_with_no_jwks_endpoint_or_cache() {
        // WARNING: This is a valid JWK but it absolutely should not be used in production.
        let jwk = Jwk {
            kid: "DgdoxSuZTY1qxPoCQYjNU9sjzNrQN2-vbMDhWX0ZY9M".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            n: RSA_MODULUS.to_string(),
            e: RSA_EXPONENT.to_string(),
        };

        // We need to create a valid token for this test, because we need a real signature so that
        // jsonwebtoken can decode and verify it, and we need dynamic iat and exp otherwise
        // jsonwebtoken will reject it as expired.
        let original_jwt = setup_jwt();
        let token = encode_jwt(&jwk.kid, &original_jwt);

        let auth_service =
            JwtAuthService::new("http://localhost:1234/certs".to_string(), Cache::new(10));

        // Decode the token - we should get an error because we can't fetch the JWK.
        let jwt = auth_service.decode_jwt(&token).await;
        assert_eq!(
            jwt,
            Err(Error::AuthenticationError(
                "Failed to fetch JWKS: error sending request for url (http://localhost:1234/certs)"
                    .to_string()
            ))
        );
    }

    #[rstest]
    #[case(
        "not-a-valid-token",
        Err(Error::AuthenticationError("Failed to decode token header: InvalidToken".to_string()))
    )]
    #[case(
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6InRlc3QifQ.\
         eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4iLCJpYXQiOjE1MTYyMzkwMjJ9.\
         ofkIpeXliqh8nI8b5VGYM106-Km3TXJsz3kCoorUd-0",
         Ok("test".to_string())
    )]
    fn test_get_kid(#[case] token: &str, #[case] expected: Result<String, Error>) {
        let auth_service =
            JwtAuthService::new("http://localhost:1234/certs".to_string(), Cache::new(10));

        let kid = auth_service.get_kid(token);
        assert_eq!(kid, expected);
    }

    #[rstest]
    #[case("dGVzdA".to_string(), "QVFBQg".to_string(), true)]
    #[case("123".to_string(), "QVFBQg".to_string(), false)]
    #[case("dGVzdA".to_string(), "123".to_string(), false)]
    #[case("123".to_string(), "123".to_string(), false)]
    #[tokio::test]
    async fn test_get_decoding_key_when_cached(
        #[case] n: String,
        #[case] e: String,
        #[case] is_ok: bool,
    ) {
        // Setup a fake JWK and put it in a cache.
        let jwk = Jwk {
            kid: "test".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            n,
            e,
        };
        let cache = Cache::new(10);
        cache.insert("test".to_string(), jwk.clone());

        let auth_service = JwtAuthService::new("http://localhost:1234/certs".to_string(), cache);

        // Attempt to get the decoding key. We shouldn't need a mock server because this should be
        // using the cache.
        let decoding_key = auth_service.get_decoding_key(&jwk.kid).await;
        assert_eq!(decoding_key.is_ok(), is_ok);
    }

    #[tokio::test]
    async fn test_get_decoding_key_when_not_cached() {
        // Setup a fake JWK and mock the JWKS endpoint to return it.
        let jwk = Jwk {
            kid: "test".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            n: "dGVzdA".to_string(),
            e: "QVFBQg".to_string(),
        };
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/certs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Jwks {
                keys: vec![jwk.clone()],
            }))
            .mount(&mock_server)
            .await;

        let cache = Cache::new(10);
        let auth_service =
            JwtAuthService::new(format!("{}/certs", mock_server.uri()), cache.clone());

        // Attempt to get the decoding key. This should fetch the JWK from the mock server.
        let decoding_key = auth_service.get_decoding_key(&jwk.kid).await;
        assert!(decoding_key.is_ok());

        // We should have cached the JWK.
        let cached_jwk = cache.get(&jwk.kid).unwrap();
        assert_eq!(cached_jwk, jwk);
    }

    #[rstest]
    #[case(
        ResponseTemplate::new(200).set_body_json(Jwks {
            keys: vec![Jwk {
                kid: "test".to_string(),
                kty: "RSA".to_string(),
                alg: "RS256".to_string(),
                n: "test".to_string(),
                e: "AQAB".to_string(),
            }],
        }),
        Ok(Jwks {
            keys: vec![Jwk {
                kid: "test".to_string(),
                kty: "RSA".to_string(),
                alg: "RS256".to_string(),
                n: "test".to_string(),
                e: "AQAB".to_string(),
            }]
        })
    )]
    #[case(
        ResponseTemplate::new(200).set_body_json(
            json!({"not_keys": [{"foo": 42}]})
        ),
        Err(Error::AuthenticationError(
            "Failed to parse JWKS: error decoding response body".to_string()
        ))
    )]
    #[case(
        ResponseTemplate::new(404).set_body_json(
            json!({"error": "Not Found"})
        ),
        Err(Error::AuthenticationError(
            "JWKS endpoint responded with 404: {\"error\":\"Not Found\"}".to_string()
        ))
    )]
    #[tokio::test]
    async fn test_fetch_jwks(
        #[case] response_template: ResponseTemplate,
        #[case] expected: Result<Jwks, Error>,
    ) {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/certs"))
            .respond_with(response_template)
            .mount(&mock_server)
            .await;

        let auth_service =
            JwtAuthService::new(format!("{}/certs", mock_server.uri()), Cache::new(10));

        let jwks = auth_service.fetch_jwks().await;
        assert_eq!(jwks, expected)
    }

    #[tokio::test]
    async fn test_fetch_jwks_with_no_server() {
        let auth_service =
            JwtAuthService::new("http://localhost:1234/certs".to_string(), Cache::new(10));

        let jwks = auth_service.fetch_jwks().await;
        assert_eq!(
            jwks,
            Err(Error::AuthenticationError(
                "Failed to fetch JWKS: error sending request for url (http://localhost:1234/certs)"
                    .to_string()
            ))
        )
    }

    fn setup_jwt() -> Jwt {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Jwt {
            jti: "jti".to_string(),
            sub: "1234567890".to_string(),
            name: "John Doe".to_string(),
            acr: "acr".to_string(),
            azp: "azp".to_string(),
            tenant: "tenant".to_string(),
            iss: "iss".to_string(),
            iat: now,
            auth_time: now,
            exp: now + 3600,
        }
    }
}
