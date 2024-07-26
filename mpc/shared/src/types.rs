use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Contribution {
    pub success: bool,
    pub timestamp: u64,
}
