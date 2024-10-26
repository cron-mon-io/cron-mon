use rocket;
use rocket::State;
use serde_json::{json, Value};

use crate::errors::Error;
use crate::infrastructure::auth::Jwt;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::paging::Paging;
use crate::infrastructure::repositories::api_key_repo::ApiKeyRepository;
use crate::infrastructure::repositories::Repository;

#[rocket::get("/keys")]
pub async fn list_api_keys(pool: &State<DbPool>, jwt: Jwt) -> Result<Value, Error> {
    let mut repo = ApiKeyRepository::new(pool);
    let keys = repo.all(&jwt.tenant).await?;

    Ok(json!({
        "data": keys,
        "paging": Paging { total: keys.len() }
    }))
}
