use std::io::Cursor;

use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use serde_json::json;

use crate::errors::Error;

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let (status, reason) = match self {
            Error::RepositoryError(_) => (Status::InternalServerError, "Repository Error"),
            Error::MonitorNotFound(_) => (Status::NotFound, "Monitor Not Found"),
            Error::JobNotFound(_, _) => (Status::NotFound, "Job Not Found"),
            Error::JobAlreadyFinished(_) => (Status::BadRequest, "Job Already Finished"),
            // Both of these could either be server-side or client-side. For now we'll handle the
            // client providing invalid data outside of where we return these, allowing us to
            // default to server-side errors.
            Error::InvalidMonitor(_) => (Status::InternalServerError, "Invalid Monitor"),
            Error::InvalidJob(_) => (Status::InternalServerError, "Invalid Job"),
        };
        let body =
            json!({ "error": {"code": status.code, "reason": reason, "description": self.to_string()} })
                .to_string();
        Response::build()
            .status(status)
            .header(ContentType::JSON)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
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

    use test_utils::gen_uuid;

    use super::Error;

    #[rocket::get("/repo_error")]
    fn repo_error() -> Result<(), Error> {
        Err(Error::RepositoryError("something went wrong".to_string()))
    }

    #[rocket::get("/monitor_not_found")]
    fn monitor_not_found() -> Result<(), Error> {
        Err(Error::MonitorNotFound(gen_uuid(
            "41ebffb4-a188-48e9-8ec1-61380085cde3",
        )))
    }

    #[rocket::get("/job_not_found")]
    fn job_not_found() -> Result<(), Error> {
        Err(Error::JobNotFound(
            gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
        ))
    }

    #[rocket::get("/job_already_finished")]
    fn job_already_finished() -> Result<(), Error> {
        Err(Error::JobAlreadyFinished(gen_uuid(
            "01a92c6c-6803-409d-b675-022fff62575a",
        )))
    }

    #[rocket::get("/invalid_monitor")]
    fn invalid_monitor() -> Result<(), Error> {
        Err(Error::InvalidMonitor("invalid monitor".to_string()))
    }

    #[rocket::get("/invalid_job")]
    fn invalid_job() -> Result<(), Error> {
        Err(Error::InvalidJob("invalid job".to_string()))
    }

    #[fixture]
    fn test_client() -> Client {
        let test_rocket = rocket::build().mount(
            "/",
            rocket::routes![
                repo_error,
                monitor_not_found,
                job_not_found,
                job_already_finished,
                invalid_monitor,
                invalid_job
            ],
        );
        Client::tracked(test_rocket)
            .expect("Couldn't create test Rocket app for DefaultJSON fairing test")
    }

    #[rstest]
    fn test_repository_error(test_client: Client) {
        let response = test_client.get("/repo_error").dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 500,
                    "reason": "Repository Error",
                    "description": "Failed to read or write data: something went wrong"
                }
            })
        );
    }

    #[rstest]
    fn test_monitor_not_found(test_client: Client) {
        let response = test_client.get("/monitor_not_found").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 404,
                    "reason": "Monitor Not Found",
                    "description": "Failed to find monitor with id \
                                    '41ebffb4-a188-48e9-8ec1-61380085cde3'"
                }
            })
        );
    }

    #[rstest]
    fn test_job_not_found(test_client: Client) {
        let response = test_client.get("/job_not_found").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 404,
                    "reason": "Job Not Found",
                    "description": "Failed to find job with id \
                                    '01a92c6c-6803-409d-b675-022fff62575a' in \
                                    Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3')"
                }
            })
        );
    }

    #[rstest]
    fn test_job_already_finished(test_client: Client) {
        let response = test_client.get("/job_already_finished").dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 400,
                    "reason": "Job Already Finished",
                    "description": "Job('01a92c6c-6803-409d-b675-022fff62575a') is already finished"
                }
            })
        );
    }

    #[rstest]
    fn test_invalid_monitor(test_client: Client) {
        let response = test_client.get("/invalid_monitor").dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 500,
                    "reason": "Invalid Monitor",
                    "description": "Invalid Monitor: invalid monitor"
                }
            })
        );
    }

    #[rstest]
    fn test_invalid_job(test_client: Client) {
        let response = test_client.get("/invalid_job").dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 500,
                    "reason": "Invalid Job",
                    "description": "Invalid Job: invalid job"
                }
            })
        );
    }
}
