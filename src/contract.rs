use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

use crate::msg::{HandleMsg, InitMsg, QueryMsg, BalanceResponse};
use cosmwasm::errors::{contract_err, unauthorized, Result};
use cosmwasm::traits::{Api, Extern, Storage, ReadonlyStorage};
use cosmwasm::types::{log, CanonicalAddr, HumanAddr, Env, Response};

use cw_storage::{singleton, Singleton, serialize, singleton_read, ReadonlySingleton, PrefixedStorage, ReadonlyPrefixedStorage};

pub const PREFIX_BALANCES: &[u8] = b"balances";



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub region: String,
    pub beneficiary: CanonicalAddr,
    pub owner: CanonicalAddr,
    pub oracle: CanonicalAddr,
    pub ecostate: i64,
    pub total_tokens: i64,
    pub released_tokens: i64,
    pub payout_start_height: Option<i64>,
    pub payout_end_height: Option<i64>,
    pub is_locked: bool,
}

impl State {
  
    fn is_started(&self, env: &Env) -> bool {
        if let Some(start_height) = self.payout_start_height {
            if start_height <= env.block.height   {
                return true;
            }
        }

        return false;
    }

    fn is_expired(&self, env: &Env) -> bool {
        if let Some(end_height) = self.payout_end_height {
            if env.block.height > end_height {
                return true;
            }
        }

        return false;
    }

    fn is_done(&self) -> bool {
        if self.released_tokens == self.total_tokens {
            return true;
        }

        return false;
    }

    const fn is_locked(&self) -> bool {
        return self.is_locked;
    }
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, b"config")
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, b"config")
}

pub fn init<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    msg: InitMsg,
) -> Result<Response> {
    let state = State {
        region: msg.region,
        beneficiary: deps.api.canonical_address(&msg.beneficiary)?,
        owner: env.message.signer.clone(),
        oracle: deps.api.canonical_address(&msg.oracle)?,
        ecostate: msg.ecostate,
        total_tokens: msg.total_tokens,
        released_tokens: 0,
        payout_start_height: msg.payout_start_height,
        payout_end_height: msg.payout_end_height,
        is_locked: false
    };

    if state.is_expired(&env) {
        contract_err("creating expired contract")
    } else {
        config(&mut deps.storage).save(&state)?;
        Ok(Response::default())
    }
}

pub fn handle<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    msg: HandleMsg,
) -> Result<Response> {
    let state = config(&mut deps.storage).load()?;

    match msg {
        HandleMsg::UpdateEcostate {ecostate} => update_ecostate(deps,env, state, ecostate),
        HandleMsg::Lock {} => lock(deps, env),
        HandleMsg::UnLock {} => unlock(deps, env),
        HandleMsg::ChangeBeneficiary {beneficiary} => chanage_beneficiary(deps, env, beneficiary),
        HandleMsg::ChangeOracle {oracle} => chanage_oracle(deps, env, oracle),
        HandleMsg::TransferOwnership {owner} => transfer_ownership(deps, env, owner),
    }
}

fn update_ecostate<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    state: State,
    new_ecostate: i64,
) -> Result<Response> {

    if state.is_started(&env) == false {
        return contract_err("The contract has not been started.");
    } 
    
    if state.is_expired(&env) {
        contract_err("The contract has expired.")
    } else if state.is_locked() {
        contract_err("The contract is locked.")
    } else if state.is_done() {
        contract_err("The contract status is DONE.")
    } else if env.message.signer != state.oracle {  // // this function is only called by oracle
        unauthorized()
    } else {
        let mut release_tokens : i64;
        let old_ecostate = state.ecostate;
        let change_amount  : i64 = new_ecostate - state.ecostate;

        if change_amount < 0 {
            release_tokens = 0;
        } else if change_amount < 100 {
            if 5000 < new_ecostate {

                let above50 : i64 = new_ecostate - 5000;
                release_tokens = (above50 * 2 + 50) / 100;      // ROUND VALUE
            } else {
                release_tokens = 0;
            }
        } else {
            release_tokens = change_amount;
        }

        if release_tokens > 0 {

            let remain_tokens = state.total_tokens - state.released_tokens;
            if release_tokens > remain_tokens {
                release_tokens = remain_tokens;
            }
        
            let mut balances_store = PrefixedStorage::new(PREFIX_BALANCES, &mut deps.storage);

            let mut from_balance = read_u128(&balances_store, state.beneficiary.as_slice())?;
            from_balance += release_tokens as u128;
            balances_store.set(state.beneficiary.as_slice(), &from_balance.to_be_bytes());
        }

        config(&mut deps.storage).update(&|mut state| {
            state.ecostate = new_ecostate;
            state.released_tokens += release_tokens;

            Ok(state)
        })?;

        let res = Response {
            messages: vec![],
            log: vec![
                    log("action", "update_ecostate"),
                    log("old_ecostate", &old_ecostate.to_string()),
                    log("new_ecostate", &new_ecostate.to_string()),
                    log("change_ecostate", &change_amount.to_string()),
                    log("release_tokens", &release_tokens.to_string()),
                ],
            data: None,
        };
    
        Ok(res)
    }
}

fn lock<S: Storage, A: Api>(deps: &mut Extern<S, A>, env: Env) -> Result<Response> {
    config(&mut deps.storage).update(&|mut state| {
        if env.message.signer != state.owner {
            return unauthorized();
        }
        state.is_locked = true;
        Ok(state)
    })?;

    let res = Response {
        messages: vec![],
        log: vec![
                log("action", "lock"),
            ],
        data: None,
    };

    Ok(res)
}

fn unlock<S: Storage, A: Api>(deps: &mut Extern<S, A>, env: Env) -> Result<Response> {
    config(&mut deps.storage).update(&|mut state| {
        if env.message.signer != state.owner {
            return unauthorized();
        }
        state.is_locked = false;
        Ok(state)
    })?;

    let res = Response {
        messages: vec![],
        log: vec![
                log("action", "unlock"),
            ],
        data: None,
    };

    Ok(res)
}

fn chanage_beneficiary<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    beneficiary: HumanAddr,
) -> Result<Response> {
    let api = deps.api;
    config(&mut deps.storage).update(&|mut state| {
        if env.message.signer != state.owner {
            return unauthorized();
        }
        state.beneficiary = api.canonical_address(&beneficiary)?;
        Ok(state)
    })?;

    let res = Response {
        messages: vec![],
        log: vec![
                log("action", "chanage_beneficiary"),
                log("beneficiary", beneficiary.as_str())],
        data: None,
    };

    Ok(res)
}

fn chanage_oracle<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    oracle: HumanAddr,
) -> Result<Response> {
    let api = deps.api;
    config(&mut deps.storage).update(&|mut state| {
        if env.message.signer != state.owner {
            return unauthorized();
        }
        state.oracle = api.canonical_address(&oracle)?;
        Ok(state)
    })?;

    let res = Response {
        messages: vec![],
        log: vec![
                log("action", "chanage_oracle"),
                log("oracle", oracle.as_str())],
        data: None,
    };

    Ok(res)
}

fn transfer_ownership<S: Storage, A: Api>(
    deps: &mut Extern<S, A>,
    env: Env,
    owner: HumanAddr,
) -> Result<Response> {
    let api = deps.api;
    config(&mut deps.storage).update(&|mut state| {
        if env.message.signer != state.owner {
            return unauthorized();
        }
        state.owner = api.canonical_address(&owner)?;
        Ok(state)
    })?;

    let res = Response {
        messages: vec![],
        log: vec![
                log("action", "transfer_ownership"),
                log("owner", owner.as_str())],
        data: None,
    };

    Ok(res)
}


pub fn query<S: Storage, A: Api>(
    deps: &Extern<S, A>, 
    msg: QueryMsg,
) -> Result<Vec<u8>> {
    let state = config_read(&deps.storage).load()?;

    match msg {
        QueryMsg::State {} => serialize(&state),
        QueryMsg::Balance {address} => {
            let address_key = deps.api.canonical_address(&address)?;
            let balance = read_balance(&deps.storage, &address_key)?;
            let out = serialize(&BalanceResponse {
                balance: balance.to_string(),
            })?;
            Ok(out)
        }
    }
}

// Converts 16 bytes value into u128
// Errors if data found that is not 16 bytes
pub fn bytes_to_u128(data: &[u8]) -> Result<u128> {
    match data[0..16].try_into() {
        Ok(bytes) => Ok(u128::from_be_bytes(bytes)),
        Err(_) => contract_err("Corrupted data found. 16 byte expected."),
    }
}

// Reads 16 byte storage value into u128
// Returns zero if key does not exist. Errors if data found that is not 16 bytes
pub fn read_u128<S: ReadonlyStorage>(store: &S, key: &[u8]) -> Result<u128> {
    return match store.get(key) {
        Some(data) => bytes_to_u128(&data),
        None => Ok(0u128),
    };
}

fn read_balance<S: Storage>(store: &S, owner: &CanonicalAddr) -> Result<u128> {
    let balance_store = ReadonlyPrefixedStorage::new(PREFIX_BALANCES, store);
    return read_u128(&balance_store, owner.as_slice());
}
