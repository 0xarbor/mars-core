use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use mars::error::MarsError;
use mars::helpers::option_string_to_addr;

use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

// INIT

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // initialize Config
    let config = Config {
        owner: deps.api.addr_validate(&msg.owner)?,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

// HANDLERS

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, MarsError> {
    match msg {
        ExecuteMsg::ExecuteCosmosMsg(cosmos_msg) => {
            handle_execute_cosmos_msg(deps, env, info, cosmos_msg)
        }

        ExecuteMsg::UpdateConfig { owner } => execute_update_config(deps, env, info, owner),
    }
}

/// Execute Cosmos message
pub fn handle_execute_cosmos_msg(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: CosmosMsg,
) -> Result<Response, MarsError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(MarsError::Unauthorized {});
    }

    let res = Response::new()
        .add_attribute("action", "execute_cosmos_msg")
        .add_message(msg);
    Ok(res)
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Option<String>,
) -> Result<Response, MarsError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(MarsError::Unauthorized {});
    };

    config.owner = option_string_to_addr(deps.api, owner, config.owner)?;

    CONFIG.save(deps.storage, &config)?;

    let response = Response::new().add_attribute("action", "update_config");

    Ok(response)
}

// QUERIES

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner,
    })
}

// TESTS

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_info;
    use cosmwasm_std::{attr, Addr, BankMsg, Coin, CosmosMsg, SubMsg, Uint128};
    use mars::testing::{mock_dependencies, mock_env, MockEnvParams};

    #[test]
    fn test_proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            owner: String::from("owner"),
        };
        let info = mock_info("owner", &[]);

        let res =
            instantiate(deps.as_mut(), mock_env(MockEnvParams::default()), info, msg).unwrap();
        let empty_vec: Vec<SubMsg> = vec![];
        assert_eq!(empty_vec, res.messages);

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(Addr::unchecked("owner"), config.owner);
    }

    #[test]
    fn test_update_config() {
        let mut deps = mock_dependencies(&[]);

        // *
        // init config with valid params
        // *
        let msg = InstantiateMsg {
            owner: String::from("owner"),
        };
        let info = mock_info("owner", &[]);
        let _res =
            instantiate(deps.as_mut(), mock_env(MockEnvParams::default()), info, msg).unwrap();

        // *
        // non owner is not authorized
        // *
        let msg = ExecuteMsg::UpdateConfig { owner: None };
        let info = mock_info("somebody", &[]);
        let error_res =
            execute(deps.as_mut(), mock_env(MockEnvParams::default()), info, msg).unwrap_err();
        assert_eq!(error_res, MarsError::Unauthorized {});

        // *
        // update config with all new params
        // *
        let msg = ExecuteMsg::UpdateConfig {
            owner: Some(String::from("new_owner")),
        };
        let info = mock_info("owner", &[]);
        // we can just call .unwrap() to assert this was a success
        let res = execute(deps.as_mut(), mock_env(MockEnvParams::default()), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Read config from state
        let new_config = CONFIG.load(&deps.storage).unwrap();

        assert_eq!(new_config.owner, Addr::unchecked("new_owner"));
    }

    #[test]
    fn test_execute_cosmos_msg() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            owner: String::from("owner"),
        };
        let info = mock_info("owner", &[]);
        let _res =
            instantiate(deps.as_mut(), mock_env(MockEnvParams::default()), info, msg).unwrap();

        let bank = BankMsg::Send {
            to_address: "destination".to_string(),
            amount: vec![Coin {
                denom: "uluna".to_string(),
                amount: Uint128::new(123456),
            }],
        };
        let cosmos_msg = CosmosMsg::Bank(bank);
        let msg = ExecuteMsg::ExecuteCosmosMsg(cosmos_msg.clone());

        // *
        // non owner is not authorized
        // *
        let info = mock_info("somebody", &[]);
        let error_res = execute(
            deps.as_mut(),
            mock_env(MockEnvParams::default()),
            info,
            msg.clone(),
        )
        .unwrap_err();
        assert_eq!(error_res, MarsError::Unauthorized {});

        // *
        // can execute Cosmos msg
        // *
        let info = mock_info("owner", &[]);
        let res = execute(deps.as_mut(), mock_env(MockEnvParams::default()), info, msg).unwrap();
        assert_eq!(res.messages, vec![SubMsg::new(cosmos_msg)]);
        let expected_attr = vec![attr("action", "execute_cosmos_msg")];
        assert_eq!(res.attributes, expected_attr);
    }
}
