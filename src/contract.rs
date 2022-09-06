use std::ops::{Div, Add};

#[cfg(not(feature = "library"))]
use cosmwasm_std::{from_binary, entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Uint64};
use cw20::{Cw20Contract, Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg, PotResponse};
use crate::state::{Config, CONFIG, Pot, POTS, save_pot, FEES, ContractFees};

// You only need to import the `Percentage` struct
use percentage::Percentage;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sei-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg
        .owner
        .and_then(|addr_string| deps.api.addr_validate(addr_string.as_str()).ok())
        .unwrap_or(info.sender);

    let config = Config {
        owner: owner.clone(),
    };

    let fees = ContractFees {
        collected: Uint128::new(0)
    };
    FEES.save(deps.storage, &fees)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePot {
            target_addr_1,
            target_addr_2,
            receive_msg,
        } => execute_create_pot(deps, info, target_addr_1, target_addr_2, receive_msg),
        ExecuteMsg::WithdrawPot {
            amount
        } => execute_withdraw_pot(deps, info, amount),
    }
}

pub fn execute_withdraw_pot(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // address that requested the withdrawl
    let address_request = info.sender;
    let str_address_request = address_request.to_string();
    // Find the address in POTS. Error if not found.
    let mut p = POTS.load(deps.storage, address_request.as_str())?;
    // Verify the amount is correct
    if amount > p.collected {
        return Err(ContractError::CustomError { val:"Wrong amount to withdraw".to_string() });
    }

    let amount_fees = amount.multiply_ratio(Uint128::new(1), Uint128::new(100));
    let amount_user = amount - amount_fees;

    // Making sure address_request is equal to target address in pot.
    if address_request != p.target_addr {
        // This check is quite useless
        return Err(ContractError::CustomError { val:"Wrong address to withdraw".to_string() });
    }

    let mut res = Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("address", p.target_addr.to_string());

    // let cw20_addr = address_request;
    let cw20 = Cw20Contract(deps.api.addr_validate("usei")?);
    let msg = cw20.call(Cw20ExecuteMsg::Transfer {
        recipient: address_request.into_string(),
        amount: amount_user,
    })?;

    res = res.add_message(msg);

    POTS.remove(deps.storage, &str_address_request);
    if amount < p.collected {
        p.collected = p.collected - amount;
        POTS.save(deps.storage, p.target_addr.as_str(), &p)?;
    }

    let mut fees = FEES.load(deps.storage)?;
    fees.collected = fees.collected + amount_fees;
    FEES.save(deps.storage, &fees)?;

    Ok(res)
}

pub fn execute_create_pot(
    deps: DepsMut,
    info: MessageInfo,
    target_addr_1: String,
    target_addr_2: String,
    wrapped: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {

    // This only works for usei token
    if info.sender != "usei" {
        return Err(ContractError::CustomError { val:"Wrong token".to_string() });
    }
    if wrapped.amount == Uint128::new(0) {
        return Err(ContractError::CustomError { val:"No token sent".to_string() });
    }

    let amount_for_each_pot = wrapped.amount.div(Uint128::new(2));
    let pot1 = Pot {
        target_addr: deps.api.addr_validate(target_addr_1.as_str())?,
        collected: amount_for_each_pot,
    };
    let pot2 = Pot {
        target_addr: deps.api.addr_validate(target_addr_2.as_str())?,
        collected: amount_for_each_pot,
    };

    save_pot(deps, &pot1, &pot2)?;
    Ok(Response::new()
        .add_attribute("action", "execute_create_pot")
        .add_attribute("target_addr_1", target_addr_1)
        .add_attribute("target_addr_2", target_addr_2)
    )

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QueryOwner {} => to_binary(&query_owner(deps)),
        QueryMsg::GetPot { addr } => to_binary(&query_pot(deps, &addr)?),
        QueryMsg::GetFees {} => to_binary(&query_fees(deps)),
    }
}

fn query_fees (deps: Deps) -> ContractFees {
    let fees = FEES.load(deps.storage).unwrap();
    fees
}

fn query_owner (deps: Deps) -> Config {
    let owner = CONFIG.load(deps.storage).unwrap();
    owner
}

fn query_pot(deps: Deps, addr: &str) -> StdResult<PotResponse> {
    let pot = POTS.load(deps.storage, addr)?;
    Ok(PotResponse {
        target_addr: pot.target_addr.into_string(),
        collected: pot.collected.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};

    #[test]
    fn test_withdraw() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let mut info = mock_info("creator", &[]);

        let msg = InstantiateMsg { owner: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // should create pot
        let msg = ExecuteMsg::CreatePot {
            target_addr_1: String::from("alice"),
            target_addr_2: String::from("bob"),
            receive_msg: Cw20ReceiveMsg {
                sender: String::from("cw20"),
                amount: Uint128::new(1000),
                msg: to_binary(&ReceiveMsg::Send { id: Uint64::new(1) }).unwrap(),
            }
        };
        info.sender = Addr::unchecked("usei");
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // query pot
        let msg = QueryMsg::GetPot { addr: "alice".to_string() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let pot: Pot = from_binary(&res).unwrap();
        assert_eq!(
            pot,
            Pot {
                target_addr: Addr::unchecked("alice"),
                collected: Uint128::new(500)
            }
        );

        // Withdraw pot
        let msg = ExecuteMsg::WithdrawPot { amount: Uint128::new(100) };
        info.sender = Addr::unchecked("alice");
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query pot
        let msg = QueryMsg::GetPot { addr: "alice".to_string() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let pot: Pot = from_binary(&res).unwrap();
        assert_eq!(
            pot,
            Pot {
                target_addr: Addr::unchecked("alice"),
                collected: Uint128::new(400)
            }
        );

        // query fees
        let msg = QueryMsg::GetFees {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let fees: ContractFees = from_binary(&res).unwrap();
        assert_eq!(
            fees,
            ContractFees {
                collected: Uint128::new(1)
            }
        );

        // Withdraw pot
        let msg = ExecuteMsg::WithdrawPot { amount: Uint128::new(100) };
        let info = mock_info("bob", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query pot
        let msg = QueryMsg::GetPot { addr: "bob".to_string() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let pot: Pot = from_binary(&res).unwrap();
        assert_eq!(
            pot,
            Pot {
                target_addr: Addr::unchecked("bob"),
                collected: Uint128::new(400)
            }
        );

        // query fees
        let msg = QueryMsg::GetFees {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let fees: ContractFees = from_binary(&res).unwrap();
        assert_eq!(
            fees,
            ContractFees {
                collected: Uint128::new(2)
            }
        );
    }

    #[test]
    fn test_create_pot() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let mut info = mock_info("creator", &[]);

        let msg = InstantiateMsg { owner: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // should create pot
        let msg = ExecuteMsg::CreatePot {
            target_addr_1: String::from("alice"),
            target_addr_2: String::from("bob"),
            receive_msg: Cw20ReceiveMsg {
                sender: String::from("cw20"),
                amount: Uint128::new(100),
                msg: to_binary(&ReceiveMsg::Send { id: Uint64::new(1) }).unwrap(),
            }
        };
        info.sender = Addr::unchecked("usei");
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // should create pot
        let msg = ExecuteMsg::CreatePot {
            target_addr_1: String::from("max"),
            target_addr_2: String::from("jane"),
            receive_msg: Cw20ReceiveMsg {
                sender: String::from("cw20"),
                amount: Uint128::new(100),
                msg: to_binary(&ReceiveMsg::Send { id: Uint64::new(1) }).unwrap(),
            }
        };
        info.sender = Addr::unchecked("usei");
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // should create pot
        let msg = ExecuteMsg::CreatePot {
            target_addr_1: String::from("karren"),
            target_addr_2: String::from("john"),
            receive_msg: Cw20ReceiveMsg {
                sender: String::from("cw20"),
                amount: Uint128::new(100),
                msg: to_binary(&ReceiveMsg::Send { id: Uint64::new(1) }).unwrap(),
            }
        };
        info.sender = Addr::unchecked("usei");
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // query pot
        let msg = QueryMsg::GetPot { addr: "alice".to_string() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let pot: Pot = from_binary(&res).unwrap();
        assert_eq!(
            pot,
            Pot {
                target_addr: Addr::unchecked("alice"),
                collected: Uint128::new(50)
            }
        );

        // query pot
        let msg = QueryMsg::GetPot { addr: "bob".to_string() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let pot: Pot = from_binary(&res).unwrap();
        assert_eq!(
            pot,
            Pot {
                target_addr: Addr::unchecked("bob"),
                collected: Uint128::new(50)
            }
        );

        // query pot
        let msg = QueryMsg::GetPot { addr: "karren".to_string() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let pot: Pot = from_binary(&res).unwrap();
        assert_eq!(
            pot,
            Pot {
                target_addr: Addr::unchecked("karren"),
                collected: Uint128::new(50)
            }
        );
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &[]);

        //no owner specified in the instantiation message
        let msg = InstantiateMsg { owner: None };
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let state = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(
            state,
            Config {
                owner: Addr::unchecked("creator".to_string()),
            }
        );

        //specifying an owner address in the instantiation message
        let msg = InstantiateMsg {
            owner: Some("specified_owner".to_string()),
        };

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let state = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(
            state,
            Config {
                owner: Addr::unchecked("specified_owner".to_string()),
            }
        );

        let res = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::QueryOwner {  },
        ).unwrap();
        let config: Config = from_binary(&res).unwrap();
        assert_eq!(
            config.owner.to_string(),
            "specified_owner"
        );
        assert_ne!(
            config.owner.to_string(),
            "not_owner"
        );

    }
}
