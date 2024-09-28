use async_trait::async_trait;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;

use crate::errors::Error;
use crate::infrastructure::auth::{Jwt, JwtAuth};

#[async_trait]
impl<'r> FromRequest<'r> for Jwt {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("Authorization") {
            Some(bearer_token) => {
                if !bearer_token.to_lowercase().starts_with("bearer ") {
                    return Outcome::Error((
                        Status::Unauthorized,
                        Error::Unauthorized("Invalid bearer token".to_string()),
                    ));
                }

                let token = bearer_token[7..].to_string();

                let auth_service = match get_auth_service(request).await {
                    Ok(auth_service) => auth_service,
                    Err(e) => {
                        return Outcome::Error((Status::InternalServerError, e));
                    }
                };

                match auth_service.decode_jwt(&token).await {
                    Err(e) => Outcome::Error((Status::Unauthorized, e)),
                    Ok(jwt) => Outcome::Success(jwt),
                }
            }
            None => Outcome::Error((
                Status::Unauthorized,
                Error::Unauthorized("No token provided".to_string()),
            )),
        }
    }
}

async fn get_auth_service<'r>(
    request: &'r Request<'_>,
) -> Result<&'r Box<dyn JwtAuth + Send + Sync>, Error> {
    match request
        .guard::<&State<Box<dyn JwtAuth + Send + Sync>>>()
        // #[coverage(off)] code coverage misses awaits in certain scenarios
        // see https://github.com/rust-lang/rust/issues/98712
        .await
        // #[coverage(on)]
    {
        Outcome::Success(auth_service) => {
            Ok(auth_service.inner() as &Box<dyn JwtAuth + Send + Sync>)
        }
        _ => Err(Error::AuthenticationError("JwtAuth not found".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate;
    use rocket::http::Accept;
    use rocket::http::Header;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use serde_json::json;

    use crate::errors::Error;
    use crate::infrastructure::auth::{JwtAuth, MockJwtAuth};

    use super::Jwt;

    #[test]
    fn test_from_request_no_jwt_auth() {
        #[rocket::get("/")]
        async fn protected_index(jwt: Jwt) -> String {
            // We should never get here.
            panic!("Hello, {}!", jwt.name)
        }

        // Setup a Rocket client that doesn't have a JwtAuth service at all.
        let test_rocket = rocket::build().mount("/", rocket::routes![protected_index]);
        let client = Client::tracked(test_rocket)
            .expect("Couldn't create test Rocket app for DefaultJSON fairing test");

        let response = client
            .get("/")
            .header(Accept::JSON)
            .header(Header::new("Authorization", "Bearer token"))
            .dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(
            response.into_json::<serde_json::Value>().unwrap(),
            json!({
                "error": {
                    "code": 500,
                    "reason": "Internal Server Error",
                    "description": "The server encountered an internal error while processing this request."
                }
            })
        );
    }

    #[test]
    fn test_from_request_missing_authorization_header() {
        // Setup a rocket client that uses a mock that should never be called.
        let mut mock = MockJwtAuth::new();
        mock.expect_decode_jwt().never();
        let client = setup_rocket_client(mock);

        let response = client.get("/").header(Accept::JSON).dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
        assert_eq!(
            response.into_json::<serde_json::Value>().unwrap(),
            json!({
                "error": {
                    "code": 401,
                    "reason": "Unauthorized",
                    "description": "The request requires user authentication."
                }
            })
        );
    }

    #[test]
    fn test_from_request_malformed_authorization_header() {
        // Setup a rocket client that uses a mock that should never be called.
        let mut mock = MockJwtAuth::new();
        mock.expect_decode_jwt().never();
        let client = setup_rocket_client(mock);

        let response = client
            .get("/")
            .header(Accept::JSON)
            .header(Header::new("Authorization", "token"))
            .dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
        assert_eq!(
            response.into_json::<serde_json::Value>().unwrap(),
            json!({
                "error": {
                    "code": 401,
                    "reason": "Unauthorized",
                    "description": "The request requires user authentication."
                }
            })
        );
    }

    #[test]
    fn test_from_request_invalid_token() {
        // Setup Rocket client that uses a mock, that should be called once with "foo-token", and
        // return an error.
        let mut mock = MockJwtAuth::new();
        mock.expect_decode_jwt()
            .times(1)
            .with(predicate::eq("foo-token"))
            .returning(|_| Err(Error::Unauthorized("Invalid token".to_string())));

        let client = setup_rocket_client(mock);

        let response = client
            .get("/")
            .header(Accept::JSON)
            .header(Header::new("Authorization", "Bearer foo-token"))
            .dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
        assert_eq!(
            response.into_json::<serde_json::Value>().unwrap(),
            json!({
                "error": {
                    "code": 401,
                    "reason": "Unauthorized",
                    "description": "The request requires user authentication."
                }
            })
        );
    }

    #[test]
    fn test_from_request_valid_token() {
        // Setup a Rocket client that uses a mock, that should be called once with "foo-token", and
        // return a valid JWT.
        let mut mock = MockJwtAuth::new();
        mock.expect_decode_jwt()
            .times(1)
            .with(predicate::eq("foo-token"))
            .returning(|_| {
                Ok(Jwt {
                    jti: "jti".to_string(),
                    sub: "1234567890".to_string(),
                    name: "John Doe".to_string(),
                    acr: "acr".to_string(),
                    azp: "azp".to_string(),
                    tenant: "tenant".to_string(),
                    iss: "iss".to_string(),
                    iat: 1234567890,
                    auth_time: 1234567890,
                    exp: 1234567899,
                })
            });

        let client = setup_rocket_client(mock);

        let response = client
            .get("/")
            .header(Header::new("Authorization", "Bearer foo-token"))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "Hello, John Doe!");
    }

    fn setup_rocket_client(mock: MockJwtAuth) -> Client {
        #[rocket::get("/")]
        async fn protected_index(jwt: Jwt) -> String {
            format!("Hello, {}!", jwt.name)
        }

        let test_rocket = rocket::build()
            .manage(Box::new(mock) as Box<dyn JwtAuth + Send + Sync>)
            .mount("/", rocket::routes![protected_index]);
        Client::tracked(test_rocket)
            .expect("Couldn't create test Rocket app for DefaultJSON fairing test")
    }
}
