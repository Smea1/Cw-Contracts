use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};  //https://github.com/CosmWasm/cosmwasm/tree/main/packages/std/src 基础库引用

use crate::coin_helpers::assert_sent_sufficient_coin;  //crate理解为库，定义在src/lib.rs，引用coin_helpers文件指定函数
use crate::error::ContractError;  //引用error库的枚举ContractError
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ResolveRecordResponse}; //引用msg库的结构体和枚举
use crate::state::{config, config_read, resolver, resolver_read, Config, NameRecord}; //引入state库的函数和结构体

const MIN_NAME_LENGTH: u64 = 3;  //定义常量
const MAX_NAME_LENGTH: u64 = 64;

#[cfg_attr(not(feature = "library"), entry_point)]   //当没有设置库时等价于#[entry_point],entry_point是一个宏，用于向虚拟机表明入口点
pub fn instantiate(
    deps: DepsMut,   //DepsMut是一个结构体，由库文件引入，用于读写合约storage，自带了一些api功能，比如地址checksum
    _env: Env,        //Env结构体存了区块高度、时间戳和本合约的一些信息
    _info: MessageInfo,   //结构体包含了消息发起者、发送资金等信息 类似于msg.sender,msg.value
    msg: InstantiateMsg,   //该结构体在msg.rs定义，包含 购买费用和转账费用
) -> Result<Response, StdError> {
    let config_state = Config {
        purchase_price: msg.purchase_price,
        transfer_price: msg.transfer_price,
    };//实例化一个Config结构体

    config(deps.storage).save(&config_state)?;   //调用config函数，将config_state结构体存入storage中

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,    //该枚举类型包含register和transfer两个选项，用于确定调用那个哪个函数
) -> Result<Response, ContractError> {
    match msg {    //match关键字用于匹配传入的消息类型并执行对应的函数
        ExecuteMsg::Register { name } => execute_register(deps, env, info, name),
        ExecuteMsg::Transfer { name, to } => execute_transfer(deps, env, info, name, to),
    }
}


pub fn execute_register(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
) -> Result<Response, ContractError> {
    // we only need to check here - at point of registration
    validate_name(&name)?;    //判断传入的name是否有效
    let config_state = config(deps.storage).load()?;  //读取config状态  ？返回一个Result结构体
    assert_sent_sufficient_coin(&info.funds, config_state.purchase_price)?;   //传入资金数量和购买价格

    let key = name.as_bytes();
    let record = NameRecord { owner: info.sender };

    if (resolver(deps.storage).may_load(key)?).is_some() {     //调用resolver函数检查该名字是否被注册过
        // name is already taken
        return Err(ContractError::NameTaken { name });
    }

    // name is available
    resolver(deps.storage).save(key, &record)?;   //写入storage

    Ok(Response::default())
}

pub fn execute_transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    to: String,
) -> Result<Response, ContractError> {
    let config_state = config(deps.storage).load()?;
    assert_sent_sufficient_coin(&info.funds, config_state.transfer_price)?;

    let new_owner = deps.api.addr_validate(&to)?;
    let key = name.as_bytes();
    resolver(deps.storage).update(key, |record| {
        if let Some(mut record) = record {
            if info.sender != record.owner {
                return Err(ContractError::Unauthorized {});
            }

            record.owner = new_owner.clone();
            Ok(record)
        } else {
            Err(ContractError::NameNotExists { name: name.clone() })
        }
    })?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ResolveRecord { name } => query_resolver(deps, env, name),
        QueryMsg::Config {} => to_binary(&config_read(deps.storage).load()?),
    }
}

fn query_resolver(deps: Deps, _env: Env, name: String) -> StdResult<Binary> {
    let key = name.as_bytes();

    let address = match resolver_read(deps.storage).may_load(key)? {
        Some(record) => Some(String::from(&record.owner)),
        None => None,
    };
    let resp = ResolveRecordResponse { address };

    to_binary(&resp)
}

// let's not import a regexp library and just do these checks by hand
fn invalid_char(c: char) -> bool {         //检查字符串是否有效
    let is_valid = c.is_digit(10) || c.is_ascii_lowercase() || (c == '.' || c == '-' || c == '_');
    !is_valid
}

/// validate_name returns an error if the name is invalid
/// (we require 3-64 lowercase ascii letters, numbers, or . - _)
fn validate_name(name: &str) -> Result<(), ContractError> {
    let length = name.len() as u64;
    if (name.len() as u64) < MIN_NAME_LENGTH {
        Err(ContractError::NameTooShort {
            length,
            min_length: MIN_NAME_LENGTH,
        })
    } else if (name.len() as u64) > MAX_NAME_LENGTH {
        Err(ContractError::NameTooLong {
            length,
            max_length: MAX_NAME_LENGTH,
        })
    } else {
        match name.find(invalid_char) {   //根据结果匹配两种方向
            None => Ok(()),
            Some(bytepos_invalid_char_start) => {
                let c = name[bytepos_invalid_char_start..].chars().next().unwrap();
                Err(ContractError::InvalidCharacter { c })
            }
        }
    }
}
