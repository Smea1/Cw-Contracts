use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct Constants {             //常量信息
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}
