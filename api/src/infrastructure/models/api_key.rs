use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::models::ApiKey;
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
    pub created_at: NaiveDateTime,
    pub tenant: String,
    pub name: String,
    pub key: String,
    pub masked: String,
    pub last_used: Option<NaiveDateTime>,
    pub last_used_monitor_id: Option<Uuid>,
    pub last_used_monitor_name: Option<String>,
}

impl From<&ApiKeyData> for ApiKey {
    fn from(value: &ApiKeyData) -> Self {
        ApiKey {
            api_key_id: value.api_key_id,
            tenant: value.tenant.clone(),
            name: value.name.clone(),
            key: value.key.clone(),
            masked: value.masked.clone(),
            created: value.created_at,
            last_used: value.last_used,
            last_used_monitor_id: value.last_used_monitor_id,
            last_used_monitor_name: value.last_used_monitor_name.clone(),
        }
    }
}

impl From<&ApiKey> for ApiKeyData {
    fn from(value: &ApiKey) -> Self {
        ApiKeyData {
            api_key_id: value.api_key_id,
            created_at: value.created,
            tenant: value.tenant.clone(),
            name: value.name.clone(),
            key: value.key.clone(),
            masked: value.masked.clone(),
            last_used: value.last_used,
            last_used_monitor_id: value.last_used_monitor_id,
            last_used_monitor_name: value.last_used_monitor_name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use test_utils::{gen_relative_datetime, gen_uuid};

    use super::{ApiKey, ApiKeyData};

    #[test]
    fn test_from_api_key_data() {
        let api_key_id = gen_uuid("fadd7266-648b-4102-8f85-c768655f4297");
        let tenant = "test-tenant".to_owned();
        let name = "test-name".to_owned();
        let key = "test-key".to_owned();
        let masked = "test-************t-key".to_owned();
        let last_used = Some(gen_relative_datetime(0));
        let last_used_monitor_id = Some(gen_uuid("8c8e7032-5de9-4dcf-b267-4e66e27b8a78"));
        let last_used_monitor_name = Some("test-monitor".to_owned());

        let data = ApiKeyData {
            api_key_id,
            created_at: gen_relative_datetime(0),
            tenant: tenant.clone(),
            name: name.clone(),
            key: key.clone(),
            masked: masked.clone(),
            last_used,
            last_used_monitor_id,
            last_used_monitor_name: last_used_monitor_name.clone(),
        };

        let model = ApiKey {
            api_key_id,
            tenant,
            name,
            key,
            masked,
            created: data.created_at,
            last_used,
            last_used_monitor_id,
            last_used_monitor_name,
        };

        assert_eq!(ApiKey::from(&data), model);
    }

    #[test]
    fn test_to_api_key_data() {
        let api_key_id = gen_uuid("fadd7266-648b-4102-8f85-c768655f4297");
        let tenant = "test-tenant".to_owned();
        let name = "test-name".to_owned();
        let key = "test-key".to_owned();
        let masked = "test-k************t-key".to_owned();
        let last_used = Some(gen_relative_datetime(0));
        let last_used_monitor_id = Some(gen_uuid("8c8e7032-5de9-4dcf-b267-4e66e27b8a78"));
        let last_used_monitor_name = Some("test-monitor".to_owned());

        let model = ApiKey {
            api_key_id,
            tenant: tenant.clone(),
            name: name.clone(),
            key: key.clone(),
            masked: masked.clone(),
            created: gen_relative_datetime(0),
            last_used,
            last_used_monitor_id,
            last_used_monitor_name: last_used_monitor_name.clone(),
        };

        let data = ApiKeyData {
            api_key_id,
            created_at: model.created,
            tenant,
            name,
            key,
            masked,
            last_used,
            last_used_monitor_id,
            last_used_monitor_name,
        };

        assert_eq!(ApiKey::from(&data), model);
    }
}
