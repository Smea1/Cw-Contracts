use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Uint128;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct InitialBalance {         //初始余额结构体
    pub address: String,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {    //初始消息结构图，里面包含了初始余额结构体
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Vec<InitialBalance>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {   //执行消息枚举
    Approve {
        spender: String,
        amount: Uint128,
    },
    Transfer {
        recipient: String,
        amount: Uint128,
    },
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    Burn {
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {    //查询消息枚举
    Balance { address: String },
    Allowance { owner: String, spender: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct BalanceResponse {    //余额响应结构体
    pub balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AllowanceResponse {     //授权响应结构体
    pub allowance: Uint128,
}
