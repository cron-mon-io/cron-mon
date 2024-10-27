use rocket;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::application::services::{get_generate_key_service, get_revoke_key_service};
use crate::errors::Error;
use crate::infrastructure::auth::Jwt;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::paging::Paging;
use crate::infrastructure::repositories::api_key_repo::ApiKeyRepository;
use crate::infrastructure::repositories::Repository;

#[derive(Deserialize)]
pub struct GenerateKeyInfo {
    pub name: String,
}

#[rocket::get("/keys")]
pub async fn list_api_keys(pool: &State<DbPool>, jwt: Jwt) -> Result<Value, Error> {
    let mut repo = ApiKeyRepository::new(pool);
    let keys = repo.all(&jwt.tenant).await?;

    Ok(json!({
        "data": keys,
        "paging": Paging { total: keys.len() }
    }))
}

#[rocket::post("/keys", data = "<info>")]
pub async fn generate_key(
    pool: &State<DbPool>,
    jwt: Jwt,
    info: Json<GenerateKeyInfo>,
) -> Result<Value, Error> {
    let mut service = get_generate_key_service(pool);
    let key = service.generate_key(&info.name, &jwt.tenant).await?;

    Ok(json!({"data": {"key": key}}))
}

#[rocket::delete("/keys/<key_id>")]
pub async fn revoke_key(pool: &State<DbPool>, jwt: Jwt, key_id: Uuid) -> Result<(), Error> {
    let mut service = get_revoke_key_service(pool);
    service.revoke_key(key_id, &jwt.tenant).await?;

    Ok(())
}
