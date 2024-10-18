use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Timestamp};
use schemars::Set;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Leave {},
    // Donate {},
    CreateAuction{
        name: String,
        bid_items: Set<String>,
    },
}

#[cw_serde]
pub struct AdminsListResp {
    pub admins: Vec<Addr>,
}

#[cw_serde]
pub struct AdminResp {
    pub admin: Addr,
}

#[cw_serde]
pub struct JoinTimeResp {
    pub joined: Timestamp,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminsListResp)]
    AdminsList {},
    #[returns(JoinTimeResp)]
    JoinTime { admin: String },
    #[returns(AdminResp)]
    Admin {},
}
