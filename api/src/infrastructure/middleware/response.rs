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
            Error::ApiKeyNotFound(_) => (Status::NotFound, "API Key Not Found"),
            Error::JobNotFound(_, _) => (Status::NotFound, "Job Not Found"),
            Error::AlertConfigNotFound(_) => (Status::NotFound, "Alert Configuration Not Found"),
            Error::JobAlreadyFinished(_) => (Status::BadRequest, "Job Already Finished"),
            Error::ErroneousJobAlertFailure(_) => {
                (Status::InternalServerError, "Late Job Process Failure")
            }
            Error::AlertConfigurationError(_) => {
                (Status::InternalServerError, "Alert Configuration Error")
            }
            // Both of these could either be server-side or client-side. For now we'll handle the
            // client providing invalid data outside of where we return these, allowing us to
            // default to server-side errors.
            Error::InvalidMonitor(_) => (Status::InternalServerError, "Invalid Monitor"),
            Error::InvalidJob(_) => (Status::InternalServerError, "Invalid Job"),
            Error::InvalidAlertConfig(_) => {
                (Status::InternalServerError, "Invalid Alert Configuration")
            }
            Error::NotifyError(_) => (Status::InternalServerError, "Notify Error"),
            Error::Unauthorized(_) => (Status::Unauthorized, "Unauthorized"),
            Error::AuthenticationError(_) => (Status::InternalServerError, "Authentication Error"),
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

    use super::*;

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

    #[rocket::get("/key_not_found")]
    fn key_not_found() -> Result<(), Error> {
        Err(Error::ApiKeyNotFound(gen_uuid(
            "01a92c6c-6803-409d-b675-022fff62575a",
        )))
    }

    #[rocket::get("/job_not_found")]
    fn job_not_found() -> Result<(), Error> {
        Err(Error::JobNotFound(
            gen_uuid("41ebffb4-a188-48e9-8ec1-61380085cde3"),
            gen_uuid("01a92c6c-6803-409d-b675-022fff62575a"),
        ))
    }

    #[rocket::get("/single_alert_config_not_found")]
    fn single_alert_config_not_found() -> Result<(), Error> {
        Err(Error::AlertConfigNotFound(vec![gen_uuid(
            "b6a32bd4-1d3e-4943-9150-a62aa84bc10a",
        )]))
    }

    #[rocket::get("/multiple_alert_config_not_found")]
    fn multiple_alert_config_not_found() -> Result<(), Error> {
        Err(Error::AlertConfigNotFound(vec![
            gen_uuid("b6a32bd4-1d3e-4943-9150-a62aa84bc10a"),
            gen_uuid("b6a32bd4-1d3e-4943-9150-a62aa84bc10b"),
        ]))
    }

    #[rocket::get("/job_already_finished")]
    fn job_already_finished() -> Result<(), Error> {
        Err(Error::JobAlreadyFinished(gen_uuid(
            "01a92c6c-6803-409d-b675-022fff62575a",
        )))
    }

    #[rocket::get("/late_job_process_failure")]
    fn late_job_process_failure() -> Result<(), Error> {
        Err(Error::ErroneousJobAlertFailure(
            "something went wrong".to_string(),
        ))
    }

    #[rocket::get("/alert_config_error")]
    fn alert_config_error() -> Result<(), Error> {
        Err(Error::AlertConfigurationError(
            "something went wrong".to_string(),
        ))
    }

    #[rocket::get("/invalid_monitor")]
    fn invalid_monitor() -> Result<(), Error> {
        Err(Error::InvalidMonitor("invalid monitor".to_string()))
    }

    #[rocket::get("/invalid_job")]
    fn invalid_job() -> Result<(), Error> {
        Err(Error::InvalidJob("invalid job".to_string()))
    }

    #[rocket::get("/invalid_alert_config")]
    fn invalid_alert_config() -> Result<(), Error> {
        Err(Error::InvalidAlertConfig(
            "invalid alert config".to_string(),
        ))
    }

    #[rocket::get("/notify_error")]
    fn notify_error() -> Result<(), Error> {
        Err(Error::NotifyError("something went wrong".to_string()))
    }

    #[rocket::get("/unauthorized")]
    fn unauthorized() -> Result<(), Error> {
        Err(Error::Unauthorized("insufficient permissions".to_string()))
    }

    #[rocket::get("/auth_error")]
    fn auth_error() -> Result<(), Error> {
        Err(Error::AuthenticationError(
            "something went wrong".to_string(),
        ))
    }

    #[fixture]
    fn test_client() -> Client {
        let test_rocket = rocket::build().mount(
            "/",
            rocket::routes![
                repo_error,
                monitor_not_found,
                key_not_found,
                job_not_found,
                single_alert_config_not_found,
                multiple_alert_config_not_found,
                job_already_finished,
                late_job_process_failure,
                alert_config_error,
                invalid_monitor,
                invalid_job,
                invalid_alert_config,
                notify_error,
                unauthorized,
                auth_error
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
    fn test_api_key_not_found(test_client: Client) {
        let response = test_client.get("/key_not_found").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 404,
                    "reason": "API Key Not Found",
                    "description": "Failed to find API key with id \
                                    '01a92c6c-6803-409d-b675-022fff62575a'"
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
    fn test_single_alert_config_not_found(test_client: Client) {
        let response = test_client.get("/single_alert_config_not_found").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 404,
                    "reason": "Alert Configuration Not Found",
                    "description": "Failed to find alert configuration with id \
                                    'b6a32bd4-1d3e-4943-9150-a62aa84bc10a'"
                }
            })
        );
    }

    #[rstest]
    fn test_multiple_alert_config_not_found(test_client: Client) {
        let response = test_client
            .get("/multiple_alert_config_not_found")
            .dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 404,
                    "reason": "Alert Configuration Not Found",
                    "description": "Failed to find alert configurations with ids \
                                    '[b6a32bd4-1d3e-4943-9150-a62aa84bc10a, \
                                      b6a32bd4-1d3e-4943-9150-a62aa84bc10b]'"
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
    fn test_late_job_process_failure(test_client: Client) {
        let response = test_client.get("/late_job_process_failure").dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 500,
                    "reason": "Late Job Process Failure",
                    "description": "Failed to process late job(s): something went wrong"
                }
            })
        );
    }

    #[rstest]
    fn test_alert_config_error(test_client: Client) {
        let response = test_client.get("/alert_config_error").dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 500,
                    "reason": "Alert Configuration Error",
                    "description": "Failed to configure alert: something went wrong"
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

    #[rstest]
    fn test_invalid_alert_config(test_client: Client) {
        let response = test_client.get("/invalid_alert_config").dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 500,
                    "reason": "Invalid Alert Configuration",
                    "description": "Invalid Alert Configuration: invalid alert config"
                }
            })
        );
    }

    #[rstest]
    fn test_notify_error(test_client: Client) {
        let response = test_client.get("/notify_error").dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 500,
                    "reason": "Notify Error",
                    "description": "Failed to notify: something went wrong"
                }
            })
        );
    }

    #[rstest]
    fn test_unauthorized(test_client: Client) {
        let response = test_client.get("/unauthorized").dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 401,
                    "reason": "Unauthorized",
                    "description": "Unauthorized: insufficient permissions"
                }
            })
        );
    }

    #[rstest]
    fn test_authentication_error(test_client: Client) {
        let response = test_client.get("/auth_error").dispatch();

        assert_eq!(response.status(), Status::InternalServerError);
        assert_eq!(response.content_type(), Some(ContentType::JSON));
        assert_eq!(
            response.into_json::<Value>().unwrap(),
            json!({
                "error": {
                    "code": 500,
                    "reason": "Authentication Error",
                    "description": "Authentication error: something went wrong"
                }
            })
        );
    }
}
