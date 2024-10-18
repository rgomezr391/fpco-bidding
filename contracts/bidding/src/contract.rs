use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ADMINS, ADMIN, AUCTIONS};
use cosmwasm_std::{
    coins, to_json_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};
use schemars::Set;
use rand::Rng;

pub type Result<T> = std::result::Result<T, ContractError>;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // let admin = deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(deps.storage, &msg.admin)?;

    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary> {
    use QueryMsg::*;

    match msg {
        Admin {} => Ok(to_json_binary(&query::admin(deps)?)?),
        Auction {
            id
        } => {
            let response = query::get_auction(deps, id)?;
            Ok(to_json_binary(&response)?)
        },
    }
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response> {
    use ExecuteMsg::*;

    match msg {
        // Leave {} => exec::leave(deps, info).map_err(Into::into),
        // Donate {} => exec::donate(deps, info),
        CreateAuction {
            name,
            bid_items,
        } => exec::create_auction(deps, info, name, bid_items).map_err(Into::into),
    }
}

mod exec {
    use std::str::FromStr;

    use cosmwasm_std::{Decimal256, Uint64};

    use crate::state::{Auction, AuctionStatus, BidItem, BID_ITEMS};

    use super::*;

    // pub fn leave(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    //     ADMINS.remove(deps.storage, &info.sender);

    //     let resp = Response::new()
    //         .add_attribute("action", "leave")
    //         .add_attribute("sender", info.sender.as_str());

    //     Ok(resp)
    // }

    // pub fn donate(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    //     let denom = DONATION_DENOM.load(deps.storage)?;
    //     let admins: Result<Vec<_>, _> = ADMINS
    //         .keys(deps.storage, None, None, Order::Ascending)
    //         .collect();
    //     let admins = admins?;

    //     let donation = cw_utils::must_pay(&info, &denom)?.u128();

    //     let donation_per_admin = donation / (admins.len() as u128);

    //     let messages = admins.into_iter().map(|admin| BankMsg::Send {
    //         to_address: admin.to_string(),
    //         amount: coins(donation_per_admin, &denom),
    //     });

    //     let resp = Response::new()
    //         .add_messages(messages)
    //         .add_attribute("action", "donate")
    //         .add_attribute("amount", donation.to_string())
    //         .add_attribute("per_admin", donation_per_admin.to_string());

    //     Ok(resp)
    // }

    pub fn create_auction(deps: DepsMut, info: MessageInfo, name: String, bid_items: Vec<String>) -> Result<Response> {
        let curr_admin = ADMIN.load(deps.storage)?;

        if curr_admin != &info.sender {
            return Err(ContractError::Unauthorized { sender: info.sender });
        }

        let auction = Auction {
            name: name,
            total_bids: Uint64::from(0 as u64),
            total_coins: Decimal256::from_str("0").unwrap(),
            available_bid_items: Uint64::from(bid_items.len() as u64),
            current_state: AuctionStatus::Active,
        };

        let mut rng = rand::thread_rng();
        let auction_generated_id: u64 = rng.gen::<u64>();

        AUCTIONS.save(deps.storage, auction_generated_id, &auction)?;

        for bid_item in bid_items {
            let item = BidItem {
                name: bid_item,
                total_bids: Uint64::from(0 as u64),
                total_coins: Decimal256::from_str("0").unwrap(),
                winner: None,
                auction_id: auction_generated_id,
            };

            let bid_item_generated_id: u64 = rng.gen::<u64>();

            BID_ITEMS.save(deps.storage, bid_item_generated_id, &item)?;
        }

        let resp = Response::new()
            .add_attribute("action", "create_auction")
            .add_attribute("auction_id", auction_generated_id.to_string());

        Ok(resp)
    }
}

mod query {
    // use cw_multi_test::Contract;

    // use crate::msg::{AdminResp, AuctionResp};

    use crate::state::Auction;

    use super::*;

    pub fn admin(deps: Deps) -> Result<Addr> {
        let admin = ADMIN.load(deps.storage)?;
        Ok(admin)
        // let resp = AdminResp { admin };
        // Ok(resp)
    }

    pub fn get_auction(deps: Deps, id: u64) -> Result<Auction> {
        let auction = AUCTIONS
            .may_load(deps.storage, id)?
            .ok_or(ContractError::InvalidId)?;
        Ok(auction)
        // let resp = AuctionResp { auction };
        // Ok(resp)
    }

}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, StdError};
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::state::Auction;

    use super::*;

    #[test]
    fn instantiation() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let sender_address =  Addr::unchecked("owner");
        //let sender_address_str = sender_address.to_string();

        let addr = app
            .instantiate_contract(
                code_id,
                sender_address.clone(),
                &InstantiateMsg {
                    admin: sender_address.clone(),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp: Addr = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::Admin {})
            .unwrap();

        assert_eq!(resp, sender_address );
    }

    #[test]
    fn create_auction() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let sender_address =  Addr::unchecked("owner");
        //let sender_address_str = sender_address.to_string();

        let addr = app
            .instantiate_contract(
                code_id,
                sender_address.clone(),
                &InstantiateMsg {
                    admin: sender_address.clone(),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let bid_items= vec![ "My first bid item".to_string(), "My second bid item".to_string() ];

        let resp = app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::CreateAuction { name: "TestAuction #1".to_string(), bid_items: bid_items },
            &[],
        )
        .unwrap();

        let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();

        let auction_id = &wasm.attributes
                .iter()
                .find(|attr| attr.key == "auction_id")
                .unwrap()
                .value;

        // TODO - avoid using unwrap since this can fail
        let auction_id_u64 = auction_id.parse::<u64>().unwrap();

        let resp: Auction = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::Auction { id: auction_id_u64 })
            .unwrap();

        assert_eq!(resp.name, "TestAuction #1");
    }

    // #[test]
    // fn donations() {
    //     let mut app = App::new(|router, _, storage| {
    //         router
    //             .bank
    //             .init_balance(storage, &Addr::unchecked("user"), coins(5, "eth"))
    //             .unwrap()
    //     });

    //     let code = ContractWrapper::new(execute, instantiate, query);
    //     let code_id = app.store_code(Box::new(code));

    //     let addr = app
    //         .instantiate_contract(
    //             code_id,
    //             Addr::unchecked("owner"),
    //             &InstantiateMsg {
    //                 admins: vec!["admin1".to_owned(), "admin2".to_owned()],
    //                 donation_denom: "eth".to_owned(),
    //             },
    //             &[],
    //             "Contract",
    //             None,
    //         )
    //         .unwrap();

    //     app.execute_contract(
    //         Addr::unchecked("user"),
    //         addr.clone(),
    //         &ExecuteMsg::Donate {},
    //         &coins(5, "eth"),
    //     )
    //     .unwrap();

    //     assert_eq!(
    //         app.wrap()
    //             .query_balance("user", "eth")
    //             .unwrap()
    //             .amount
    //             .u128(),
    //         0
    //     );

    //     assert_eq!(
    //         app.wrap()
    //             .query_balance(&addr, "eth")
    //             .unwrap()
    //             .amount
    //             .u128(),
    //         1
    //     );

    //     assert_eq!(
    //         app.wrap()
    //             .query_balance("admin1", "eth")
    //             .unwrap()
    //             .amount
    //             .u128(),
    //         2
    //     );

    //     assert_eq!(
    //         app.wrap()
    //             .query_balance("admin2", "eth")
    //             .unwrap()
    //             .amount
    //             .u128(),
    //         2
    //     );
    // }
}
