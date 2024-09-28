use std::str::FromStr;

use chrono::{Duration, NaiveDateTime, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use uuid::Uuid;

pub const RSA_MODULUS: &str =
    "r31yIMCMynfhVFbAu_oZvolDpPbUctgUtbq9pRhybGLHxSfGFYQBVJnJMRRclit_fninHBUeHpyqi_A0TUlJ8ggn73AaRY\
    UFoMdczJQvCXZX0YS4vYQm0tSjnApg5bRHIpWCElu6oOKxWplYFQdLcv6ntGK9HfJG2eDU4wDAfyqBoris_0vkKvAtfMVuS\
    wALFAnJhXLDugpmEE8x2SNMLzOkAPtVFEvK3bd8eC-0zEWHYMnd7uhyIshskUglqUDbY8V8hGLLvgvDlpMkBfUO_xyptabK\
    RnftWNM40ZIVJp3GSkho3RsZILXfB8DS_2wzXR-vyTb4rugd2xDXGwlUb0pcv0_CTkTbnxT4lfLATzr_ddespiqQ6-BgGBu\
    MPctlkeWSsvLNbB8BbWMkgkYfRnc69vDnaHvtd0eUU8fW8ct-JK2VL6C-ULh-qg3AWrznKLDmlx9XaFtfUWt-3AgEqPz8u3\
    BnNtGLvDnVSLLqOM0dqmbji-x7ZUjf2pewO67SDkUWLosR6SWhLVjudA8xhmemFZT2PP97Rn6yo48WGtHDrYjL3uusqGA4C\
    BiQOeJsMRvUrGAWVSWYSfjGHBc1jIARVC335i1jvBG9BpIryPY19_awFgIEnufAF_U2G1Ad_631nIckg6TUOW8cbQQcYh4u\
    MGJalt6dQu0YOhUoXNE";

pub const RSA_EXPONENT: &str = "AQAB";

pub fn encode_jwt<T: Serialize>(kid: &str, claims: &T) -> String {
    // WARNING: This is a valid private key but it absolutely should not be used in production.
    const DUMMY_PRIVATE_KEY: &str = "-----BEGIN RSA PRIVATE KEY-----\n\
        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\n\
        -----END RSA PRIVATE KEY-----";

    encode(
        &Header {
            alg: Algorithm::RS256,
            kid: Some(kid.to_string()),
            ..Default::default()
        },
        &claims,
        &EncodingKey::from_rsa_pem(DUMMY_PRIVATE_KEY.as_bytes()).unwrap(),
    )
    .unwrap()
}

/// Create a `Uuid` from a string.
pub fn gen_uuid(uuid: &str) -> Uuid {
    Uuid::from_str(uuid).unwrap()
}

/// Check if a string is a valid `Uuid`.
pub fn is_uuid(uuid: &str) -> bool {
    if let Ok(_) = Uuid::from_str(uuid) {
        true
    } else {
        false
    }
}

/// Create a `NaiveDateTime` from a string.
pub fn gen_datetime(ts: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.f").unwrap()
}

/// Create a `NaiveDateTime` relative to now, offset by `seconds`.
pub fn gen_relative_datetime(seconds: i64) -> NaiveDateTime {
    Utc::now().naive_utc() + Duration::seconds(seconds)
}

/// Check if a string is a valid datetime.
pub fn is_datetime(datetime: &str) -> bool {
    if let Ok(_) = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%dT%H:%M:%S%.f") {
        true
    } else {
        false
    }
}
