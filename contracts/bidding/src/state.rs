use cosmwasm_std::{Addr, Timestamp, Uint64};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Auction has one to many Bid Items
// Bid Items has one to many Bids

pub const ADMIN: Item<Addr> = Item::new("admin");
// pub const DONATION_DENOM: Item<String> = Item::new("donation_denom");
pub const ADMINS: Map<&Addr, Timestamp> = Map::new("admins");
pub const AUCTIONS: Map<u64, Auction> = Map::new("auctions");
pub const BID_ITEMS: Map<u64, BidItem> = Map::new("bid_items");
pub const BIDS: Map<u64, Bid> = Map::new("bids");
pub const AUCTIONS_CRANK_QUEUE: Item<Vec<u64>> = Item::new("auctions_crank_queue");

// #[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default, Debug)]
// pub struct AuctionId(pub Uint64);

// #[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default, Debug)]
// pub struct BidItemId(pub Uint64);

// #[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default, Debug)]
// pub struct BidId(pub Uint64);

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AuctionStatus {
    Active,
    Suspended,
    PendingCompletion,
    Completed,
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum BidItemStatus {
    Active,
    Completed,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug)]
pub struct Auction {
    pub name: String,
    pub available_bid_items: Uint64,
    pub total_bids: Uint64,
    pub total_coins: u128,
    pub current_state: AuctionStatus,
}

#[derive(PartialEq, Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct BidItem {
    pub name: String,
    pub total_bids: Uint64,
    pub total_coins: u128,
    pub winner: Option<Addr>,
    pub auction_id: u64,
    pub current_state: BidItemStatus,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Bid {
    pub amount: u128,
    pub bidder: Addr,
    pub placed: Timestamp,
    pub bid_item_id: u64,
}