use serde::Serialize;

#[derive(Serialize)]
pub struct Paging {
    pub total: u32,
}
