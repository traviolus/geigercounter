#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, attr, Attribute, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, to_json_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg};
use crate::state::{Config, CONFIG, RADIOACTIVITY, SECONDS_IN_DAY, UserState};

const CONTRACT_NAME: &str = "crates.io:geigercounter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &Config {
        owner: info.sender,
    })?;

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "instantiate"),
        ])
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Measure {} => try_measure(deps, env, info),
        ExecuteMsg::Reset {} => try_reset(deps, info),
    }
}

fn try_measure(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let now = env.block.time.seconds();
    let attrs: Vec<Attribute> = vec![
        attr("action", "try_measure"),
        attr("current_time", now.to_string())
    ];

    match RADIOACTIVITY.may_load(deps.storage, &info.sender)? {
        Some(mut state) => {
            let time_diff = now - state.last_interaction;

            if time_diff < SECONDS_IN_DAY {
                return Err(ContractError::Limit {});
            }

            if time_diff > SECONDS_IN_DAY * 2 {
                state.radioactivity = 1;
            } else {
                state.radioactivity += 1;
            }

            state.last_interaction = now;
            RADIOACTIVITY.save(deps.storage, &info.sender, &state)?;
        }
        None => {
            RADIOACTIVITY.save(deps.storage, &info.sender, &UserState {
                last_interaction: now,
                radioactivity: 1u64,
            })?;
        }
    };

    Ok(Response::new().add_attributes(attrs))
}

fn try_reset(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let keys: Vec<StdResult<Addr>> = RADIOACTIVITY.keys(deps.storage, None, None, Order::Ascending).collect();
    for key in keys {
        match key {
            Ok(key) => {
                let mut state = RADIOACTIVITY.load(deps.storage, &key)?;
                state.radioactivity = 0;
                RADIOACTIVITY.save(deps.storage, &key, &state)?;
            }
            Err(err) => {
                return Err(ContractError::Std(err));
            }
        }
    }

    Ok(Response::new().add_attribute("method", "try_reset"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Radioactivity { address } => to_json_binary(&query_radioactivity(deps, address)?),
        QueryMsg::Leaderboard {} => to_json_binary(&query_leaderboard(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

fn query_radioactivity(deps: Deps, address: String) -> StdResult<u64> {
    let query_addr = deps.api.addr_validate(&address)?;
    let state = RADIOACTIVITY.load(deps.storage, &query_addr)?;
    Ok(state.radioactivity)
}

fn query_leaderboard(deps: Deps) -> StdResult<Vec<(Addr, u64)>> {
    let mut leaderboard: Vec<(Addr, u64)> = RADIOACTIVITY
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| {
            let (addr, state) = item?;
            Ok((addr, state.radioactivity))
        })
        .collect::<StdResult<Vec<_>>>()?;

    leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(leaderboard)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::new())
}
