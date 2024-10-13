use async_trait::async_trait;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use crate::errors::Error;

pub struct ApiKey(pub String);

#[async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("X-API-Key") {
            Some(key) => Outcome::Success(ApiKey(key.to_owned())),
            None => Outcome::Error((
                Status::Unauthorized,
                Error::Unauthorized("X-API-Key is required".to_string()),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use rocket::http::Accept;
    use rocket::http::Header;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use rstest::{fixture, rstest};

    use super::ApiKey;

    #[fixture]
    fn client() -> Client {
        #[rocket::get("/")]
        async fn protected_index(api_key: ApiKey) -> String {
            format!("API key: {}", api_key.0)
        }

        let test_rocket = rocket::build().mount("/", rocket::routes![protected_index]);
        Client::tracked(test_rocket)
            .expect("Couldn't create test Rocket app for ApiKey request guard test")
    }

    #[rstest]
    fn test_api_key_provided(client: Client) {
        let response = client
            .get("/")
            .header(Accept::JSON)
            .header(Header::new("X-Api-Key", "foo-key"))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string().unwrap(), "API key: foo-key");
    }

    #[rstest]
    fn test_api_key_not_provided(client: Client) {
        let response = client.get("/").header(Accept::JSON).dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
    }
}
