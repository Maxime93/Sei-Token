use schemars::JsonSchema;
use cw20::Cw20ReceiveMsg;
use cosmwasm_std::{Uint128, Uint64};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreatePot {
        /// target_addr will receive tokens when token amount threshold is met.
        target_addr_1: String,
        /// target_addr will receive tokens when token amount threshold is met.
        target_addr_2: String,
        receive_msg: Cw20ReceiveMsg,
    },
    WithdrawPot {
        // The amount you want to withdraw
        amount: Uint128,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    // Send sends token to an id with defined pot
    Send { id: Uint64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    QueryOwner {},
    GetPot { addr: String },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PotResponse {
    /// target_addr is the address that will receive the pot
    pub target_addr: String,
    /// collected keeps information on how much is collected for this pot.
    pub collected: String,
}
