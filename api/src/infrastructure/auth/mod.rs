pub mod jwt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use mockall::automock;

use crate::errors::Error;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Jwt {
    pub acr: String,
    pub auth_time: u64,
    pub azp: String,
    pub name: String,
    pub tenant: String,
    pub exp: u64,
    pub iat: u64,
    pub iss: String,
    pub jti: String,
    pub sub: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait JwtAuth {
    async fn decode_jwt(&self, token: &str) -> Result<Jwt, Error>;
}
