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
        match jsonwebtoken::decode::<Jwt>(token, &decoding_key, &Validation::new(Algorithm::RS256))
        {
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
        // TODO: Pass logger through.
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
            let response_text = response.text().await;
            let error = match &response_text {
                Ok(text) => text,
                Err(_) => "[Could not read body]",
            };
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

    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use moka::sync::Cache;
    use rstest::rstest;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::errors::Error;
    use crate::infrastructure::auth::{Jwt, JwtAuth};

    use super::{Jwk, Jwks, JwtAuthService};

    // WARNING: This is a valid private key but it absolutely should not be used in production.
    const DUMMY_PRIVATE_KEY: &str = "-----BEGIN RSA PRIVATE KEY-----\n\
         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\n\
         -----END RSA PRIVATE KEY-----";

    #[tokio::test]
    async fn test_decode_jwt_without_cached_jwk() {
        // WARNING: This is a valid JWK but it absolutely should not be used in production.
        let jwk = Jwk {
            kid: "DgdoxSuZTY1qxPoCQYjNU9sjzNrQN2-vbMDhWX0ZY9M".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            n: "r31yIMCMynfhVFbAu_oZvolDpPbUctgUtbq9pRhybGLHxSfGFYQBVJnJMRRclit_fninHBUeHpyqi_A0TUlJ8gg\
                n73AaRYUFoMdczJQvCXZX0YS4vYQm0tSjnApg5bRHIpWCElu6oOKxWplYFQdLcv6ntGK9HfJG2eDU4wDAfyqBor\
                is_0vkKvAtfMVuSwALFAnJhXLDugpmEE8x2SNMLzOkAPtVFEvK3bd8eC-0zEWHYMnd7uhyIshskUglqUDbY8V8h\
                GLLvgvDlpMkBfUO_xyptabKRnftWNM40ZIVJp3GSkho3RsZILXfB8DS_2wzXR-vyTb4rugd2xDXGwlUb0pcv0_C\
                TkTbnxT4lfLATzr_ddespiqQ6-BgGBuMPctlkeWSsvLNbB8BbWMkgkYfRnc69vDnaHvtd0eUU8fW8ct-JK2VL6C\
                -ULh-qg3AWrznKLDmlx9XaFtfUWt-3AgEqPz8u3BnNtGLvDnVSLLqOM0dqmbji-x7ZUjf2pewO67SDkUWLosR6S\
                WhLVjudA8xhmemFZT2PP97Rn6yo48WGtHDrYjL3uusqGA4CBiQOeJsMRvUrGAWVSWYSfjGHBc1jIARVC335i1jv\
                BG9BpIryPY19_awFgIEnufAF_U2G1Ad_631nIckg6TUOW8cbQQcYh4uMGJalt6dQu0YOhUoXNE".to_string(),
            e: "AQAB".to_string(),
        };

        // We need to create a valid token for this test, because we need a real signature so that
        // jsonwebtoken can decode and verify it, and we need dynamic iat and exp otherwise
        // jsonwebtoken will reject it as expired.
        let original_jwt = setup_jwt();
        let token = encode(
            &Header {
                alg: Algorithm::RS256,
                kid: Some(jwk.kid.clone()),
                ..Default::default()
            },
            &original_jwt,
            &EncodingKey::from_rsa_pem(DUMMY_PRIVATE_KEY.as_bytes()).unwrap(),
        )
        .unwrap();

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

        // We should have cached the JWK.
        let cached_jwk = cache.get(&jwk.kid);
        assert!(cached_jwk.is_some());
    }

    #[tokio::test]
    async fn test_decode_jwt_with_cached_jwk() {
        // WARNING: This is a valid JWK but it absolutely should not be used in production.
        let jwk = Jwk {
            kid: "DgdoxSuZTY1qxPoCQYjNU9sjzNrQN2-vbMDhWX0ZY9M".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            n: "r31yIMCMynfhVFbAu_oZvolDpPbUctgUtbq9pRhybGLHxSfGFYQBVJnJMRRclit_fninHBUeHpyqi_A0TUlJ8gg\
                n73AaRYUFoMdczJQvCXZX0YS4vYQm0tSjnApg5bRHIpWCElu6oOKxWplYFQdLcv6ntGK9HfJG2eDU4wDAfyqBor\
                is_0vkKvAtfMVuSwALFAnJhXLDugpmEE8x2SNMLzOkAPtVFEvK3bd8eC-0zEWHYMnd7uhyIshskUglqUDbY8V8h\
                GLLvgvDlpMkBfUO_xyptabKRnftWNM40ZIVJp3GSkho3RsZILXfB8DS_2wzXR-vyTb4rugd2xDXGwlUb0pcv0_C\
                TkTbnxT4lfLATzr_ddespiqQ6-BgGBuMPctlkeWSsvLNbB8BbWMkgkYfRnc69vDnaHvtd0eUU8fW8ct-JK2VL6C\
                -ULh-qg3AWrznKLDmlx9XaFtfUWt-3AgEqPz8u3BnNtGLvDnVSLLqOM0dqmbji-x7ZUjf2pewO67SDkUWLosR6S\
                WhLVjudA8xhmemFZT2PP97Rn6yo48WGtHDrYjL3uusqGA4CBiQOeJsMRvUrGAWVSWYSfjGHBc1jIARVC335i1jv\
                BG9BpIryPY19_awFgIEnufAF_U2G1Ad_631nIckg6TUOW8cbQQcYh4uMGJalt6dQu0YOhUoXNE".to_string(),
            e: "AQAB".to_string(),
        };

        // We need to create a valid token for this test, because we need a real signature so that
        // jsonwebtoken can decode and verify it, and we need dynamic iat and exp otherwise
        // jsonwebtoken will reject it as expired.
        let original_jwt = setup_jwt();
        let token = encode(
            &Header {
                alg: Algorithm::RS256,
                kid: Some(jwk.kid.clone()),
                ..Default::default()
            },
            &original_jwt,
            &EncodingKey::from_rsa_pem(DUMMY_PRIVATE_KEY.as_bytes()).unwrap(),
        )
        .unwrap();

        // Setup a cache with the dummy JWK.
        let cache = Cache::new(10);
        cache.insert(jwk.kid.clone(), jwk.clone());

        let auth_service =
            JwtAuthService::new("http://127.0.0.1:1234/certs".to_string(), cache.clone());

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
            n: "r31yIMCMynfhVFbAu_oZvolDpPbUctgUtbq9pRhybGLHxSfGFYQBVJnJMRRclit_fninHBUeHpyqi_A0TUlJ8gg\
                n73AaRYUFoMdczJQvCXZX0YS4vYQm0tSjnApg5bRHIpWCElu6oOKxWplYFQdLcv6ntGK9HfJG2eDU4wDAfyqBor\
                is_0vkKvAtfMVuSwALFAnJhXLDugpmEE8x2SNMLzOkAPtVFEvK3bd8eC-0zEWHYMnd7uhyIshskUglqUDbY8V8h\
                GLLvgvDlpMkBfUO_xyptabKRnftWNM40ZIVJp3GSkho3RsZILXfB8DS_2wzXR-vyTb4rugd2xDXGwlUb0pcv0_C\
                TkTbnxT4lfLATzr_ddespiqQ6-BgGBuMPctlkeWSsvLNbB8BbWMkgkYfRnc69vDnaHvtd0eUU8fW8ct-JK2VL6C\
                -ULh-qg3AWrznKLDmlx9XaFtfUWt-3AgEqPz8u3BnNtGLvDnVSLLqOM0dqmbji-x7ZUjf2pewO67SDkUWLosR6S\
                WhLVjudA8xhmemFZT2PP97Rn6yo48WGtHDrYjL3uusqGA4CBiQOeJsMRvUrGAWVSWYSfjGHBc1jIARVC335i1jv\
                BG9BpIryPY19_awFgIEnufAF_U2G1Ad_631nIckg6TUOW8cbQQcYh4uMGJalt6dQu0YOhUoXNE".to_string(),
            e: "AQAB".to_string(),
        };

        // We need to create a valid token for this test, because we need a real signature so that
        // jsonwebtoken can decode and verify it, and we need dynamic iat and exp otherwise
        // jsonwebtoken will reject it as expired.
        let original_jwt = setup_jwt();
        let token = encode(
            &Header {
                alg: Algorithm::RS256,
                kid: Some(jwk.kid.clone()),
                ..Default::default()
            },
            &original_jwt,
            &EncodingKey::from_rsa_pem(DUMMY_PRIVATE_KEY.as_bytes()).unwrap(),
        )
        .unwrap();

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
