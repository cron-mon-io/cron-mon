use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::infrastructure::db_schema::api_key;

// Note that we do not have a corresponding domain model for ApiKeyData, like we do for monitors and
// jobs. This is because the ApiKeyData struct is a direct representation of the database table, and
// we do not need to transform it into a domain model.
#[derive(Clone, Queryable, Identifiable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = api_key)]
#[diesel(primary_key(api_key_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ApiKeyData {
    pub api_key_id: Uuid,
    pub tenant: String,
    pub key: String,
    pub last_used: Option<NaiveDateTime>,
    pub last_used_monitor_id: Option<Uuid>,
    pub last_used_monitor_name: Option<String>,
}
