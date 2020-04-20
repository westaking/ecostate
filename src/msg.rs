use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm::types::{HumanAddr};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub region: String,
    pub beneficiary: HumanAddr,
    pub oracle: HumanAddr,
    pub ecostate: i64,
    pub total_tokens: i64,
    pub payout_start_height: Option<i64>,
    pub payout_end_height: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HandleMsg {
    UpdateEcostate {ecostate: i64},
    Lock {},
    UnLock {},
    ChangeBeneficiary {beneficiary: HumanAddr},
    ChangeOracle {oracle: HumanAddr},
    TransferOwnership {owner: HumanAddr},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum QueryMsg {
    State {},
    Balance {address: HumanAddr}
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct BalanceResponse {
    pub balance: String,
}