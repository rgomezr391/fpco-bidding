use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ADMIN, AUCTIONS, AUCTIONS_CRANK_QUEUE};
use cosmwasm_std::{
    coins, to_json_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};
use rand::Rng;

pub type Result<T> = std::result::Result<T, ContractError>;

const CRANK_MAX_BID_ITEMS: u32 = 3;
const DENOM: &'static str = "eth";
const LIMIT: u32 = 10;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    ADMIN.save(deps.storage, &msg.admin)?;

    let initial_vec: Vec<u64> = Vec::new();
    AUCTIONS_CRANK_QUEUE.save(deps.storage, &initial_vec)?;

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
        BidItemsByAuctionId {
            auction_id,
        } => {
            let response = query::get_bid_items_by_auction_id(deps, auction_id)?;
            Ok(to_json_binary(&response)?)
        },
        BidItem {
            id
        } => {
            let response = query::get_bid_item_by_id(deps, id)?;
            Ok(to_json_binary(&response)?)
        },
        BidItems {
            start_after,
        } => {
            let response = query::get_bid_items(deps, start_after, LIMIT)?;
            Ok(to_json_binary(&response)?)
        },
        Auctions {
            start_after,
        } => {
            let response = query::get_auctions(deps, start_after, LIMIT)?;
            Ok(to_json_binary(&response)?)
        },
    }
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response> {
    use ExecuteMsg::*;

    match msg {
        CreateAuction {
            name,
            bid_items,
        } => exec::create_auction(deps, info, name, bid_items).map_err(Into::into),
        SetAuctionState{
            id,
            status,
        }  => exec::set_auction_state(deps, info, id, status).map_err(Into::into),
        AddBidItems {
            auction_id,
            bid_items,
        } => exec::add_bid_items(deps, info, auction_id, bid_items).map_err(Into::into),
        PlaceBid {
            bid_item_id,
            coins_to_bid,
        } => exec::place_bid(deps, info, env, bid_item_id, coins_to_bid).map_err(Into::into),
        AdvanceCrank {} => exec::advance_crank(deps, info, env).map_err(Into::into),
    }
}

mod exec {
    use cosmwasm_std::{Timestamp, Uint64};

    use crate::state::{Auction, AuctionStatus, Bid, BidItem, BidItemStatus, BIDS, BID_ITEMS};

    use super::*;

    pub fn create_auction(deps: DepsMut, info: MessageInfo, name: String, bid_items: Vec<String>) -> Result<Response> {
        let curr_admin = ADMIN.load(deps.storage)?;

        if curr_admin != &info.sender {
            return Err(ContractError::Unauthorized { sender: info.sender });
        }

        let auction = Auction {
            name: name,
            total_bids: Uint64::from(0 as u64),
            total_coins: 0,
            available_bid_items: Uint64::from(bid_items.len() as u64),
            current_state: AuctionStatus::Active,
        };

        let mut rng = rand::thread_rng();
        let auction_generated_id: u64 = rng.gen::<u64>();

        AUCTIONS.save(deps.storage, auction_generated_id, &auction)?;

        add_bid_items_to_auction(bid_items, auction_generated_id, deps)?;

        let resp = Response::new()
            .add_attribute("action", "create_auction")
            .add_attribute("auction_id", auction_generated_id.to_string());

        Ok(resp)
    }

    fn add_bid_items_to_auction(bid_items: Vec<String>, auction_id: u64, deps: DepsMut<'_>) -> Result<()> {
        let mut rng = rand::thread_rng();

        for bid_item in bid_items {
            let item = BidItem {
                name: bid_item,
                total_bids: Uint64::from(0 as u64),
                total_coins: 0,
                winner: None,
                auction_id: auction_id,
                current_state: BidItemStatus::Active
            };
    
            let bid_item_generated_id: u64 = rng.gen::<u64>();
    
            BID_ITEMS.save(deps.storage, bid_item_generated_id, &item)?;
        };

        Ok(())
    }
    
    pub fn set_auction_state(deps: DepsMut, info: MessageInfo, id: u64, auction_status: AuctionStatus) -> Result<Response> {
        let curr_admin: Addr = ADMIN.load(deps.storage)?;

        if curr_admin != &info.sender {
            return Err(ContractError::Unauthorized { sender: info.sender });
        }

        let auction = AUCTIONS
            .may_load(deps.storage, id)?
            .ok_or(ContractError::InvalidAuctionId)?;

        let response = match auction.current_state {
            AuctionStatus::PendingCompletion => {
                Response::new()
                .add_attribute("action", "set_auction_state")
                .add_attribute("response", "Can't revert an auction that's already in pending completion")
            },
            AuctionStatus::Completed => {
                Response::new()
                .add_attribute("action", "set_auction_state")
                .add_attribute("response", "Can't revert an auction that's already completed.")
            },
            AuctionStatus::Suspended =>  {
                match auction_status {
                    AuctionStatus::PendingCompletion | AuctionStatus::Active  => {
                        AUCTIONS.update(deps.storage, id, |existing| -> StdResult<_> {
                            // TODO - possible exception if ID doesn't exist
                            let mut value = existing.unwrap();
                            value.current_state = auction_status;
                            Ok(value)
                        })?;

                        if auction_status == AuctionStatus::PendingCompletion {
                            let vec = AUCTIONS_CRANK_QUEUE.update(deps.storage, |mut vec| -> StdResult<_> {
                                vec.push(id);
                                Ok(vec)
                            })?;

                            let auctions_crank_queue_count = vec.len();

                            Response::new()
                            .add_attribute("action", "set_auction_state")
                            .add_attribute("response", "Auction has been transitioned to the desired state.")
                            .add_attribute("auctions_crank_queue_count", auctions_crank_queue_count.to_string())
                        }
                        else {
                            Response::new()
                            .add_attribute("action", "set_auction_state")
                            .add_attribute("response", "Auction has been transitioned to the desired state.")
                        }
                    },
                    AuctionStatus::Suspended => {
                        Response::new()
                            .add_attribute("action", "set_auction_state")
                            .add_attribute("response", "Auction is already in suspended state.")
                    },
                    AuctionStatus::Completed => {
                        Response::new()
                            .add_attribute("action", "set_auction_state")
                            .add_attribute("response", "Auction is already in completed state.")
                    },
                }
            },
            AuctionStatus::Active =>  {
                AUCTIONS.update(deps.storage, id, |existing| -> StdResult<_> {
                    // TODO - possible exception if ID doesn't exist
                    let mut value = existing.unwrap();
                    value.current_state = auction_status;
                    Ok(value)
                })?;

                if auction_status == AuctionStatus::PendingCompletion {
                    let vec = AUCTIONS_CRANK_QUEUE.update(deps.storage, |mut vec| -> StdResult<_> {
                        vec.push(id);
                        Ok(vec)
                    })?;

                    let auctions_crank_queue_count = vec.len();

                    Response::new()
                        .add_attribute("action", "set_auction_state")
                        .add_attribute("response", "Auction has been transitioned to the desired state.")
                        .add_attribute("auctions_crank_queue_count", auctions_crank_queue_count.to_string())
                }
                else {
                    Response::new()
                        .add_attribute("action", "set_auction_state")
                        .add_attribute("response", "Auction has been transitioned to the desired state.")
                }
            },
        };

        Ok(response)
    }

    pub fn add_bid_items(deps: DepsMut, info: MessageInfo, id: u64, bid_items: Vec<String>) -> Result<Response> {
        let curr_admin: Addr = ADMIN.load(deps.storage)?;

        if curr_admin != &info.sender {
            return Err(ContractError::Unauthorized { sender: info.sender });
        }

        let auction = AUCTIONS
            .may_load(deps.storage, id)?
            .ok_or(ContractError::InvalidAuctionId)?;

        match auction.current_state {
            AuctionStatus::PendingCompletion | AuctionStatus::Completed => {
                return Err(ContractError::AuctionNonUpdateable);

            },
            _ => {},
        }

        add_bid_items_to_auction(bid_items, id, deps)?;

        let response = Response::new()
                    .add_attribute("action", "add_bid_items")
                    .add_attribute("response", "Successfully added bid items to auction.");

        Ok(response)
    }

    pub fn place_bid(deps: DepsMut, info: MessageInfo, env: Env, bid_item_id: u64, coins_to_bid: u128) -> Result<Response> {

        let bid_item = BID_ITEMS
            .may_load(deps.storage, bid_item_id)?
            .ok_or(ContractError::InvalidBidItemId)?;

        let auction = AUCTIONS
            .may_load(deps.storage, bid_item.auction_id)?
            .ok_or(ContractError::InvalidAuctionId)?;

        if auction.current_state != AuctionStatus::Active {
            return Err(ContractError::AuctionCompleted);
        }

        let mut rng = rand::thread_rng();

        let item = Bid {
            amount: coins_to_bid,
            bidder: info.sender, 
            placed: env.block.time,
            bid_item_id: bid_item_id,
        };
        
        let bid_generated_id: u64 = rng.gen::<u64>();

        BIDS.save(deps.storage, bid_generated_id, &item)?;

        let response = Response::new()
                    .add_attribute("action", "place_bid")
                    .add_attribute("response", "Successfully placed bid.");

        Ok(response)
    }

    pub fn advance_crank(deps: DepsMut, _info: MessageInfo, _env: Env) -> Result<Response> {
        let auction_ids_to_process = AUCTIONS_CRANK_QUEUE.load(deps.storage)?;

        let mut processed_bid_items = 0;
        let mut auctions_completed: Vec<u64> = vec![];

        for auction_id in auction_ids_to_process {

            let mut bid_items = get_pending_bid_items_by_auction_id(&deps, auction_id)?;

            while processed_bid_items < CRANK_MAX_BID_ITEMS && bid_items.len() > 0 {

                let bid_item = bid_items.pop().unwrap();

                if bid_item.1.winner == None {  // If there's no winner then it means this bid item is still pending
                    let bids = get_bids_from_bid_item_id(&deps, bid_item.0)?;

                    let winning_bid = get_winning_bid(&deps, bid_item.0, &bids)?;

                    // Refund other bids & process Winning bid
                    process_bids(&deps, winning_bid.0, &bids)?;

                    // Update Bid Item
                    BID_ITEMS.update(deps.storage, bid_item.0, |existing| -> StdResult<_> {
                        // TODO - possible exception if ID doesn't exist
                        let mut value = existing.unwrap();
                        value.winner = if winning_bid.0 != 0 { Some(winning_bid.1.bidder) } else { None };
                        value.current_state = BidItemStatus::Completed;

                        Ok(value)
                    })?;
    
                    processed_bid_items += 1;
                }
            }

            if bid_items.len() == 0 {    // This means that the auction has been completed, so marking it as completed to remove it below
                auctions_completed.push(auction_id);
            }
        }

        // Removing Auctions from Crank queue
        for auction_completed in auctions_completed {
            AUCTIONS_CRANK_QUEUE.update(deps.storage, |mut vec| -> StdResult<_> {
                vec.retain(|&x| x != auction_completed);
                Ok(vec)
            })?;
        }
        
        let response = Response::new()
                    .add_attribute("action", "advance_crank")
                    .add_attribute("response", "Successfully advanced crank.");
                
        Ok(response)
    }

    pub fn get_pending_bid_items_by_auction_id(deps: &DepsMut, auction_id: u64) -> Result<Vec<(u64, BidItem)>> {
        let mut result: Vec<(u64, BidItem)> = Vec::new();
        let iter = BID_ITEMS.range(deps.storage, None, None, Order::Ascending);

        for bid_item in iter {
            let (key, value) = bid_item?;
            
            if value.auction_id == auction_id && value.current_state == BidItemStatus::Active {
                result.push((key, value));
            }
        }

        Ok(result)
    }

    pub fn get_bids_from_bid_item_id(deps: &DepsMut, bid_item_id: u64) -> Result<Vec<(u64, Bid)>> {
        let mut result = Vec::new();
        let iter = BIDS.range(deps.storage, None, None, Order::Ascending);

        for bid in iter {
            let (key, value) = bid?;
            
            if value.bid_item_id == bid_item_id {
                result.push((key, value));
            }
        }

        Ok(result)
    }

    pub fn get_winning_bid(_deps: &DepsMut, _bid_item_id: u64, bids: &Vec<(u64, Bid)>) -> Result<(u64, Bid)> {

        let mut highest_bid_id: u64 = 0;
        let mut highest_bid = Bid {
            amount: 0,
            placed: Timestamp::from_seconds(0),
            bid_item_id: 0,
            bidder: Addr::unchecked(""),
        };

        for bid in bids {
            if bid.1.amount > highest_bid.amount {
                highest_bid.amount = bid.1.amount;
                highest_bid.placed = bid.1.placed;
                highest_bid_id = bid.0;
            } 
            else if bid.1.amount == highest_bid.amount && bid.1.placed > highest_bid.placed {
                highest_bid.amount = bid.1.amount;
                highest_bid.placed = bid.1.placed;
                highest_bid_id = bid.0;
            }
        }

        Ok((highest_bid_id, highest_bid))
    }

    pub fn process_bids(deps: &DepsMut, winning_bid_id: u64, bids: &Vec<(u64, Bid)>) -> Result<()> {
        let curr_admin: Addr = ADMIN.load(deps.storage)?;
        let curr_admin_str = curr_admin.to_string();

        let _ = bids.into_iter().map(|bid|
            BankMsg::Send {
                to_address: if bid.0 != winning_bid_id { bid.1.bidder.to_string() } else { curr_admin_str.clone() },
                amount: coins(bid.1.amount, DENOM)
            }
        );

        Ok(())
    }

}

mod query {

    use cw_storage_plus::Bound;

    use crate::state::{Auction, BidItem, BID_ITEMS};

    use super::*;

    pub fn admin(deps: Deps) -> Result<Addr> {
        let admin = ADMIN.load(deps.storage)?;
        Ok(admin)
    }

    pub fn get_auction(deps: Deps, id: u64) -> Result<Auction> {
        Ok(AUCTIONS
            .may_load(deps.storage, id)?
            .ok_or(ContractError::InvalidAuctionId)?)
    }

    pub fn get_bid_item_by_id(deps: Deps, id: u64) -> Result<BidItem> {
        Ok(BID_ITEMS
            .may_load(deps.storage, id)?
            .ok_or(ContractError::InvalidBidItemId)?)
    }

    pub fn get_bid_items(deps: Deps, start_after: Option<u64>, limit: u32) -> Result<Vec<(u64, BidItem)>> {
        let start = Bound::inclusive(start_after.unwrap());
        let limit = limit as usize;

        let iter = BID_ITEMS.range(deps.storage, Some(start), None, Order::Ascending)
            .take(limit);

        let result: Vec<(u64, BidItem)> = iter
            .map(|item| {
                let (key, value) = item?;
                Ok((key, value))
            })
            .collect::<StdResult<_>>()?;

        Ok(result)
    }

    pub fn get_auctions(deps: Deps, start_after: Option<u64>, limit: u32) -> Result<Vec<(u64, Auction)>> {
        let start = Bound::inclusive(start_after.unwrap_or(0));
        let limit = limit as usize;

        let iter = AUCTIONS.range(deps.storage, Some(start), None, Order::Ascending)
            .take(limit);

        let result: Vec<(u64, Auction)> = iter
            .map(|item| {
                let (key, value) = item?;
                Ok((key, value))
            })
            .collect::<StdResult<_>>()?;

        Ok(result)
    }

    pub fn get_bid_items_by_auction_id(deps: Deps, auction_id: u64) -> Result<Vec<(u64, BidItem)>> {
        let mut result = Vec::new();
        let iter = BID_ITEMS.range(deps.storage, None, None, Order::Ascending);

        for bid_item in iter {
            let (key, value) = bid_item?;
            
            if value.auction_id == auction_id {
                result.push((key, value));
            }
        }

        Ok(result)
    }

}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::state::{Auction, AuctionStatus, BidItem};

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

    #[test]
    fn set_auction_state() {
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

        let resp = app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::SetAuctionState { id: auction_id_u64, status: AuctionStatus::PendingCompletion },
            &[],
        )
        .unwrap();

        let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();

        let auctions_crank_queue_count = &wasm.attributes
            .iter()
            .find(|attr| attr.key == "auctions_crank_queue_count")
            .unwrap()
            .value;
        

        assert_eq!(auctions_crank_queue_count, "1");
    }

    #[test]
    fn advance_crank() {
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

        let bid_items= vec![ 
            "TA1 1st bid item".to_string(),
            "TA1 2nd bid item".to_string(),
            "TA1 3rd bid item".to_string(),
            "TA1 4th bid item".to_string(),
            "TA1 5th bid item".to_string(),
        ];

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
        let auction_id_u64_first = auction_id.parse::<u64>().unwrap();

        let bid_items= vec![ 
            "TA1 6th bid item".to_string(),
            "TA1 7th bid item".to_string(),
        ];

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::AddBidItems { auction_id: auction_id_u64_first, bid_items: bid_items },
            &[],
        )
        .unwrap();

        let resp: Vec<(u64, BidItem)> = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::BidItemsByAuctionId { auction_id: auction_id_u64_first })
            .unwrap();

        for bid_item in resp {
            if bid_item.1.name == "TA1 2nd bid item" {
                app.execute_contract(
                    Addr::unchecked("user"),
                    addr.clone(),
                    &ExecuteMsg::PlaceBid { bid_item_id: bid_item.0, coins_to_bid: 4 },
                    &[],
                )
                .unwrap();

                app.execute_contract(
                    Addr::unchecked("user"),
                    addr.clone(),
                    &ExecuteMsg::PlaceBid { bid_item_id: bid_item.0, coins_to_bid: 1 },
                    &[],
                )
                .unwrap();
            } 
            else if bid_item.1.name == "TA1 4th bid item" {
                app.execute_contract(
                    Addr::unchecked("user"),
                    addr.clone(),
                    &ExecuteMsg::PlaceBid { bid_item_id: bid_item.0, coins_to_bid: 8 },
                    &[],
                )
                .unwrap();

                app.execute_contract(
                    Addr::unchecked("user"),
                    addr.clone(),
                    &ExecuteMsg::PlaceBid { bid_item_id: bid_item.0, coins_to_bid: 10 },
                    &[],
                )
                .unwrap();

                app.execute_contract(
                    Addr::unchecked("user"),
                    addr.clone(),
                    &ExecuteMsg::PlaceBid { bid_item_id: bid_item.0, coins_to_bid: 16 },
                    &[],
                )
                .unwrap();
            }
            else if bid_item.1.name == "TA1 6th bid item" {
                app.execute_contract(
                    Addr::unchecked("user"),
                    addr.clone(),
                    &ExecuteMsg::PlaceBid { bid_item_id: bid_item.0, coins_to_bid: 36 },
                    &[],
                )
                .unwrap();

                app.execute_contract(
                    Addr::unchecked("user"),
                    addr.clone(),
                    &ExecuteMsg::PlaceBid { bid_item_id: bid_item.0, coins_to_bid: 35 },
                    &[],
                )
                .unwrap();

                app.execute_contract(
                    Addr::unchecked("user"),
                    addr.clone(),
                    &ExecuteMsg::PlaceBid { bid_item_id: bid_item.0, coins_to_bid: 5 },
                    &[],
                )
                .unwrap();
            };
        }
        
        let resp = app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::SetAuctionState { id: auction_id_u64_first, status: AuctionStatus::PendingCompletion },
            &[],
        )
        .unwrap();

        let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();

        let auctions_crank_queue_count = &wasm.attributes
            .iter()
            .find(|attr| attr.key == "auctions_crank_queue_count")
            .unwrap()
            .value;

        assert_eq!(auctions_crank_queue_count, "1");

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::AdvanceCrank {  },
            &[],
        )
        .unwrap();

       app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::AdvanceCrank {  },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::AdvanceCrank {  },
            &[],
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::AdvanceCrank {  },
            &[],
        )
        .unwrap();

    }

    #[test]
    fn get_auctions() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let sender_address =  Addr::unchecked("owner");

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

        let total_auctions = 15;
        let mut auction_ids: Vec<u64> = vec![];

        for num in 0..total_auctions { // change it to get range
            let bid_items= vec![ "My first bid item".to_string(), "My second bid item".to_string() ];

            let auction_name = match num {
                n => format!("TestAuction #{n}"),
            };

            let resp = app.execute_contract(
                Addr::unchecked("owner"),
                addr.clone(),
                &ExecuteMsg::CreateAuction { name: auction_name, bid_items: bid_items },
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
            auction_ids.push(auction_id_u64);
        }

        let resp: Vec<(u64, Auction)> = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::Auctions { start_after: None } )
            .unwrap();

        assert_eq!(resp.len(), 10);

        let resp: Vec<(u64, Auction)> = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::Auctions { start_after: Some(resp[8].0) } )
            .unwrap();

        assert_eq!(resp.len(), 7);
    }
}
