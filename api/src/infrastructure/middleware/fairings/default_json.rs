use async_trait::async_trait;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Accept;
use rocket::{Data, Request};

pub struct DefaultJSON;

#[async_trait]
impl Fairing for DefaultJSON {
    fn info(&self) -> Info {
        Info {
            name: "Prefer JSON responses",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        // We could check the Accept header here and only set it to JSON if it's not already set.
        // The reason we don't do that is because we want to make sure that the API always returns
        // JSON, even if the client requests a different format. Note that whilst it feels a bit
        // rude to ignore the client's request, there's nothing technically wrong with us returning
        // JSON when the client asks for a different format. We could respond with a 406 in such
        // cases but Rocket currently doesn't support terminating or responding to requests in
        // Fairings. See https://github.com/rwf2/Rocket/issues/749
        request.replace_header(Accept::JSON);
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use rocket::http::ContentType;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use rstest::*;
    use serde_json::{json, Value};

    use super::*;

    #[rocket::get("/foo")]
    fn foo() -> &'static str {
        "foo"
    }

    #[rocket::get("/bar")]
    fn bar() {
        panic!("This route should not be called");
    }

    #[fixture]
    fn test_client() -> Client {
        let test_rocket = rocket::build()
            .attach(DefaultJSON)
            .mount("/", rocket::routes![foo, bar]);
        Client::tracked(test_rocket)
            .expect("Couldn't create test Rocket app for DefaultJSON fairing test")
    }

    #[rstest]
    fn test_explicit_type_is_returned(test_client: Client) {
        let response = test_client.get("/foo").dispatch();

        assert_eq!(response.status(), Status::Ok);

        assert_eq!(response.content_type(), Some(ContentType::Plain));
        assert_eq!(response.into_string().unwrap(), "foo");
    }

    #[rstest]
    fn test_errors_are_json(test_client: Client) {
        let response = test_client.get("/bar").dispatch();

        assert_eq!(response.status(), Status::InternalServerError);

        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 500,
                    "reason": "Internal Server Error",
                    "description": (
                        "The server encountered an internal error while processing this request."
                    )
                }
            })
        );
    }
}
