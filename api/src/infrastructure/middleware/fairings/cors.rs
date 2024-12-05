use async_trait::async_trait;
use rocket::fairing::{Fairing, Info, Kind, Result};
use rocket::http::Header;
use rocket::{options, routes, Build, Request, Response, Rocket};

pub struct CORS;

#[async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "CORS headers in responses",
            kind: Kind::Response | Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result {
        Ok(rocket.mount("/", routes![options_all]))
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        // TODO: Can we make this more configurable so we only return the methods allowed for
        // the route that is being accessed?
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "DELETE, GET, OPTIONS, PATCH, POST",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[options("/<_..>")]
fn options_all() {
    // Nothing to do here since this Fairing already adds the CORS headers below.
}

#[cfg(test)]
mod tests {

    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use rstest::*;

    use super::CORS;

    #[fixture]
    fn test_client() -> Client {
        Client::tracked(rocket::build().attach(CORS))
            .expect("Couldn't create test Rocket app for CORS fairing test")
    }

    #[rstest]
    #[case("/")]
    #[case("/api/v1/health")]
    #[case("/foo/bar")]
    fn test_cors(test_client: Client, #[case] path: &str) {
        let response = test_client.options(path).dispatch();

        assert_eq!(response.status(), Status::Ok);

        let headers = response.headers();
        assert_eq!(headers.get_one("Access-Control-Allow-Origin"), Some("*"));
        assert_eq!(
            headers.get_one("Access-Control-Allow-Methods"),
            Some("DELETE, GET, OPTIONS, PATCH, POST")
        );
        assert_eq!(headers.get_one("Access-Control-Allow-Headers"), Some("*"));
        assert_eq!(
            headers.get_one("Access-Control-Allow-Credentials"),
            Some("true")
        );
    }
}
