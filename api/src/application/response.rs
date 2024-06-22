use std::io::Cursor;

use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use serde_json::json;

use crate::errors::AppError;

impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let status = match self {
            AppError::RepositoryError(_) => Status::InternalServerError,
            AppError::MonitorNotFound(_) => Status::NotFound,
            AppError::JobNotFound(_, _) => Status::NotFound,
            AppError::JobAlreadyFinished(_) => Status::BadRequest,
        };
        let body = json!({ "error": self.to_string() }).to_string();
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

    use super::AppError;

    #[rocket::get("/repo_error")]
    fn repo_error() -> Result<(), AppError> {
        Err(AppError::RepositoryError(
            "something went wrong".to_string(),
        ))
    }

    #[rocket::get("/monitor_not_found")]
    fn monitor_not_found() -> Result<(), AppError> {
        Err(AppError::MonitorNotFound(gen_uuid(
            "41ebffb4-a188-48e9-8ec1-61380085cde3",
        )))
    }

    #[rocket::get("/job_not_found")]
    fn job_not_found() -> Result<(), AppError> {
        Err(AppError::JobNotFound(
            gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
        ))
    }

    #[rocket::get("/job_already_finished")]
    fn job_already_finished() -> Result<(), AppError> {
        Err(AppError::JobAlreadyFinished(gen_uuid(
            "01a92c6c-6803-409d-b675-022fff62575a",
        )))
    }

    #[fixture]
    fn test_client() -> Client {
        let test_rocket = rocket::build().mount(
            "/",
            rocket::routes![
                repo_error,
                monitor_not_found,
                job_not_found,
                job_already_finished
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
            json!({"error": "Failed to read or write data: something went wrong"})
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
                "error": "Failed to find monitor with id '41ebffb4-a188-48e9-8ec1-61380085cde3'"
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
                "error": "Failed to find job with id '01a92c6c-6803-409d-b675-022fff62575a' \
                          in Monitor('41ebffb4-a188-48e9-8ec1-61380085cde3')"
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
                "error": "Job('01a92c6c-6803-409d-b675-022fff62575a') is already finished"
            })
        );
    }
}
