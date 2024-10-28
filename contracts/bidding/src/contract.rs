use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ADMIN, AUCTIONS, AUCTIONS_CRANK_QUEUE};
use cosmwasm_std::{
    coins, to_json_binary, Addr, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult,
};

pub type Result<T> = std::result::Result<T, ContractError>;

const CRANK_MAX_BID_ITEMS: u32 = 3;
const DENOM: &'static str = "eth";
const PAGINATION_LIMIT: u32 = 10;

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
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
            let response = query::get_bid_items(deps, start_after, PAGINATION_LIMIT)?;
            Ok(to_json_binary(&response)?)
        },
        Auctions {
            start_after,
        } => {
            let response = query::get_auctions(deps, start_after, PAGINATION_LIMIT)?;
            Ok(to_json_binary(&response)?)
        },
        BidItemsById { 
            bid_items_ids
        } => {
            let response = query::get_bid_items_by_id(deps, bid_items_ids)?;
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
    use cosmwasm_std::Uint64;

    use crate::state::{Auction, AuctionId, AuctionStatus, Bid, BidId, BidItem, BidItemId, BidItemKey, BidItemStatus, BidKey, BIDS, BID_ITEMS, BID_ITEMS_TO_AUCTIONS, WINNING_BIDS};

    use super::*;

    pub fn create_auction(deps: DepsMut, info: MessageInfo, name: String, bid_items: Vec<String>) -> Result<Response> {
        let curr_admin = ADMIN.load(deps.storage)?;

        if curr_admin != &info.sender {
            return Err(ContractError::Unauthorized { sender: info.sender });
        }

        let auction_id = AuctionId::next(deps.storage)?;

        let auction = Auction {
            name: name,
            total_bids: Uint64::from(0 as u64),
            total_coins: 0,
            available_bid_items: Uint64::from(bid_items.len() as u64),
            current_state: AuctionStatus::Active,
        };

        AUCTIONS.save(deps.storage, auction_id, &auction)?;

        add_bid_items_to_auction(bid_items, auction_id, deps)?;

        let resp = Response::new()
            .add_attribute("action", "create_auction")
            .add_attribute("auction_id", auction_id.to_string());

        Ok(resp)
    }

    fn add_bid_items_to_auction(bid_items: Vec<String>, auction_id: AuctionId, deps: DepsMut<'_>) -> Result<()> {

        for bid_item in bid_items {
            let bid_item_id = BidItemId::next(deps.storage)?;

            let key = BidItemKey {
                auction_id,
                bid_item_id,
            };

            let item = BidItem {
                name: bid_item,
                total_bids: Uint64::from(0 as u64),
                total_coins: 0,
                winner: None,
                current_state: BidItemStatus::Active
            };
        
            BID_ITEMS.save(deps.storage, key, &item)?;
            BID_ITEMS_TO_AUCTIONS.save(deps.storage, bid_item_id, &auction_id)?;
        };

        Ok(())
    }
    
    pub fn set_auction_state(deps: DepsMut, info: MessageInfo, id: AuctionId, auction_status: AuctionStatus) -> Result<Response> {
        let curr_admin: Addr = ADMIN.load(deps.storage)?;

        if curr_admin != &info.sender {
            return Err(ContractError::Unauthorized { sender: info.sender });
        }

        let mut auction = AUCTIONS
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
                        auction.current_state = auction_status;
                        AUCTIONS.save(deps.storage, id, &auction)?;

                        if auction_status == AuctionStatus::PendingCompletion {
                            AUCTIONS_CRANK_QUEUE.save(deps.storage, id, &())?;
                            let auctions_crank_queue_count = AUCTIONS_CRANK_QUEUE.range(deps.storage, None, None, Order::Ascending).count();

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
                            .add_attribute("response", "Only the crank can set an auction to a complete state.")
                    },
                }
            },
            AuctionStatus::Active =>  {
                auction.current_state = auction_status;
                AUCTIONS.save(deps.storage, id, &auction)?;

                if auction_status == AuctionStatus::PendingCompletion {
                    AUCTIONS_CRANK_QUEUE.save(deps.storage, id, &())?;
                    let auctions_crank_queue_count = AUCTIONS_CRANK_QUEUE.range(deps.storage, None, None, Order::Ascending).count();

                    Response::new()
                        .add_attribute("action", "set_auction_state")
                        .add_attribute("response", "Auction has been transitioned to the desired state.")
                        .add_attribute("auctions_crank_queue_count", auctions_crank_queue_count.to_string())
                }
                else if auction_status == AuctionStatus::Completed {
                    Response::new()
                        .add_attribute("action", "set_auction_state")
                        .add_attribute("response", "Only the crank can set an auction to a complete state.")
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

    pub fn add_bid_items(deps: DepsMut, info: MessageInfo, auction_id: AuctionId, bid_items: Vec<String>) -> Result<Response> {
        let curr_admin: Addr = ADMIN.load(deps.storage)?;

        if curr_admin != &info.sender {
            return Err(ContractError::Unauthorized { sender: info.sender });
        }

        let auction = AUCTIONS
            .may_load(deps.storage, auction_id)?
            .ok_or(ContractError::InvalidAuctionId)?;

        match auction.current_state {
            AuctionStatus::PendingCompletion | AuctionStatus::Completed => {
                return Err(ContractError::AuctionNonUpdateable);

            },
            _ => {},
        }

        add_bid_items_to_auction(bid_items, auction_id, deps)?;

        let response = Response::new()
                    .add_attribute("action", "add_bid_items")
                    .add_attribute("response", "Successfully added bid items to auction.");

        Ok(response)
    }

    pub fn place_bid(deps: DepsMut, info: MessageInfo, env: Env, bid_item_id: BidItemId, coins_to_bid: u128) -> Result<Response> {

        let auction_id = BID_ITEMS_TO_AUCTIONS
            .may_load(deps.storage, bid_item_id)?
            .ok_or(ContractError::InvalidBidItemId)?;

        let auction = AUCTIONS
            .may_load(deps.storage, auction_id)?
            .ok_or(ContractError::InvalidAuctionId)?;

        if auction.current_state != AuctionStatus::Active {
            return Err(ContractError::AuctionCompleted);
        }

        let bid_id = BidId::next(deps.storage)?;

        let item = Bid {
            amount: coins_to_bid,
            bidder: info.sender, 
            placed: env.block.time,
        };

        let key = BidKey {
            bid_id,
            bid_item_id, 
        };

        BIDS.save(deps.storage, key, &item)?;

        check_winning_bid(deps, bid_item_id, item, key)?;

        let response = Response::new()
                    .add_attribute("action", "place_bid")
                    .add_attribute("response", "Successfully placed bid.");

        Ok(response)
    }

    fn check_winning_bid(deps: DepsMut<'_>, bid_item_id: BidItemId, item: Bid, key: BidKey) -> Result<()> {
        Ok(match WINNING_BIDS.may_load(deps.storage, bid_item_id)? {
            Some(bid) => {  // There's an existing winning bid.
                let current_winning_bid = BIDS.load(deps.storage, bid)?;
    
                if item.amount > current_winning_bid.amount || (item.amount == current_winning_bid.amount && item.placed > current_winning_bid.placed) {
                    WINNING_BIDS.save(deps.storage, bid_item_id, &key)?;
                }
            }
            None => {   // No winning bid exists, set current one as winning bid.
                WINNING_BIDS.save(deps.storage, bid_item_id, &key)?;
            }
        })
    }
    
    pub fn advance_crank(deps: DepsMut, _info: MessageInfo, _env: Env) -> Result<Response> {
        let auction_ids_to_process = extract_auction_ids_to_process(&deps);

        let mut processed_bid_items = 0;
        let mut auctions_completed: Vec<AuctionId> = vec![];

        for auction_id in auction_ids_to_process {

            let mut bid_items = get_pending_bid_items_by_auction_id(&deps, auction_id)?;

            while processed_bid_items < CRANK_MAX_BID_ITEMS && bid_items.len() > 0 {

                let mut bid_item = bid_items.pop().unwrap();

                if bid_item.1.winner == None {  // If there's no winner then it means this bid item is still pending
                    let bids = get_bids_from_bid_item_id(&deps, bid_item.0)?;

                    let key = BidItemKey {
                        auction_id: auction_id,
                        bid_item_id: bid_item.0,
                    };

                    if bids.len() > 0 {
                        let winning_bid = WINNING_BIDS
                        .load(deps.storage, bid_item.0)?;

                        // Refund other bids & process Winning bid
                        process_bids(&deps, winning_bid.bid_id, &bids)?;

                        let bid = BIDS.load(deps.storage, winning_bid)?;

                        // Update Bid Item
                        bid_item.1.winner = Some(bid.bidder);
                    }

                    bid_item.1.current_state = BidItemStatus::Completed;
                    
                    BID_ITEMS.save(deps.storage, key, &bid_item.1)?;
    
                    processed_bid_items += 1;
                }
            }

            if bid_items.len() == 0 {    // This means that the auction has been completed, so marking it as completed to remove it below
                auctions_completed.push(auction_id);
            }
        }

        // Removing Auctions from Crank queue
        for auction_completed in auctions_completed {
            AUCTIONS_CRANK_QUEUE.remove(deps.storage, auction_completed);

            // Update Auction status as Complete
            let mut auction = AUCTIONS.load(deps.storage, auction_completed)?;
            auction.current_state = AuctionStatus::Completed;
            AUCTIONS.save(deps.storage, auction_completed, &auction)?;
        }

        let response = Response::new()
                    .add_attribute("action", "advance_crank")
                    .add_attribute("response", "Successfully advanced crank.");
                
        Ok(response)
    }

    fn extract_auction_ids_to_process (deps: &DepsMut<'_>) -> Vec<AuctionId> {
        let mut auction_ids_to_process: Vec<AuctionId> = vec![];
        let auctions_crank_queue = AUCTIONS_CRANK_QUEUE.range(deps.storage, None, None, Order::Ascending);
    
        for auction_id in auctions_crank_queue {
            auction_ids_to_process.push(auction_id.unwrap().0);
        }

        auction_ids_to_process
    }
    
    pub fn get_pending_bid_items_by_auction_id(deps: &DepsMut, auction_id: AuctionId) -> Result<Vec<(BidItemId, BidItem)>> {
        let iter = BID_ITEMS
            .prefix(auction_id)
            .range(deps.storage, None, None, Order::Ascending);

        let mut results: Vec<(BidItemId, BidItem)> = Vec::new();

        for bid_item in iter {
            let (key, value) = bid_item?;
            if value.current_state == BidItemStatus::Active {
                results.push((key, value));
            }
        }

        Ok(results)
    }

    pub fn get_bids_from_bid_item_id(deps: &DepsMut, bid_item_id: BidItemId) -> Result<Vec<(BidId, Bid)>> {
        let iter = BIDS
            .prefix(bid_item_id)
            .range(deps.storage, None, None, Order::Ascending);

        let mut results: Vec<(BidId, Bid)> = Vec::new();

        for bid in iter {
            let (key, value) = bid?;
            results.push((key, value));
        }

        Ok(results)
    }

    pub fn process_bids(deps: &DepsMut, winning_bid_id: BidId, bids: &Vec<(BidId, Bid)>) -> Result<()> {
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

    use crate::state::{Auction, AuctionId, BidItem, BidItemId, BidItemKey, BID_ITEMS, BID_ITEMS_TO_AUCTIONS};

    use super::*;

    pub fn admin(deps: Deps) -> Result<Addr> {
        let admin = ADMIN.load(deps.storage)?;
        Ok(admin)
    }

    pub fn get_auction(deps: Deps, id: AuctionId) -> Result<Auction> {
        Ok(AUCTIONS
            .may_load(deps.storage, id)?
            .ok_or(ContractError::InvalidAuctionId)?)
    }

    pub fn get_bid_item_by_id(deps: Deps, id: BidItemId) -> Result<BidItem> {
        let auction_id = BID_ITEMS_TO_AUCTIONS 
            .may_load(deps.storage, id)?
            .ok_or(ContractError::InvalidBidItemId)?;

        let key = BidItemKey {
            auction_id: auction_id,
            bid_item_id: id,
        };

        Ok(BID_ITEMS
            .may_load(deps.storage, key)?
            .ok_or(ContractError::InvalidBidItemId)?)
    }

    pub fn get_bid_items(deps: Deps, start_after: Option<BidItemKey>, limit: u32) -> Result<Vec<(BidItemKey, BidItem)>> {
        let start = start_after.map(Bound::exclusive);
        let limit = limit as usize;

        let iter = BID_ITEMS.range(deps.storage, start, None, Order::Ascending)
            .take(limit);

        let result: Vec<(BidItemKey, BidItem)> = iter
            .map(|item| {
                let (key, value) = item?;
                Ok((key, value))
            })
            .collect::<StdResult<_>>()?;

        Ok(result)
    }

    pub fn get_auctions(deps: Deps, start_after: Option<AuctionId>, limit: u32) -> Result<Vec<(AuctionId, Auction)>> {
        let start = start_after.map(Bound::exclusive);
        let limit = limit as usize;

        let iter = AUCTIONS.range(deps.storage, start, None, Order::Ascending)
            .take(limit);

        let result: Vec<(AuctionId, Auction)> = iter
            .map(|item| {
                let (key, value) = item?;
                Ok((key, value))
            })
            .collect::<StdResult<_>>()?;

        Ok(result)
    }

    pub fn get_bid_items_by_auction_id(deps: Deps, auction_id: AuctionId) -> Result<Vec<(BidItemId, BidItem)>> {
        let iter: Box<dyn Iterator<Item = std::result::Result<(BidItemId, BidItem), cosmwasm_std::StdError>>> = BID_ITEMS
            .prefix(auction_id)
            .range(deps.storage, None, None, Order::Ascending);

        let mut results: Vec<(BidItemId, BidItem)> = Vec::new();

        for bid_item in iter {
            let (key, value) = bid_item?;
            results.push((key, value));
        }

        Ok(results)
    }

    pub fn get_bid_items_by_id(deps: Deps, bid_items_ids: Vec<BidItemId>) -> Result<Vec<(BidItemId, BidItem)>> {
        let mut results: Vec<(BidItemId, BidItem)> = vec![];

        for bid_item_id in bid_items_ids {

            let bid_item = get_bid_item_by_id(deps, bid_item_id)?;
            results.push((bid_item_id, bid_item));
        }

        Ok(results)
    }

}