use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, DepsMut, StdResult, Response, Uint128, Uint64};
use cw_storage_plus::{Item,Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Pot {
    /// target_addr is the address that will receive the pot
    pub target_addr: Addr,
    /// collected keeps information on how much is collected for this pot.
    pub collected: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractFees {
    // collected keeps information on how much fees this contract has collected.
    pub collected: Uint128,
}

/// POT_SEQ holds the last pot ID
pub const FEES: Item<ContractFees> = Item::new("fee");

/// POT_SEQ holds the last pot ID
pub const POTS: Map<&str, Pot> = Map::new("pot");

pub fn save_pot(deps: DepsMut, pot1: &Pot, pot2: &Pot,) -> StdResult<()> {
    // save pot with id
    POTS.save(deps.storage, pot1.target_addr.as_str(), pot1)?;
    POTS.save(deps.storage, pot2.target_addr.as_str(), pot2)
}
