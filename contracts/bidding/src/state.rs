use cosmwasm_std::{Addr, Decimal256, Timestamp, Uint64};
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

// #[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default, Debug)]
// pub struct AuctionId(pub Uint64);

// #[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default, Debug)]
// pub struct BidItemId(pub Uint64);

// #[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default, Debug)]
// pub struct BidId(pub Uint64);

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AuctionStatus {
    Active,
    Suspended,
    PendingCompletion,
    Completed,
}

#[derive(Clone, Serialize, Deserialize, JsonSchema, Debug)]
pub struct Auction {
    pub name: String,
    pub available_bid_items: Uint64,
    pub total_bids: Uint64,
    pub total_coins: Decimal256,
    pub current_state: AuctionStatus,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct BidItem {
    pub name: String,
    pub total_bids: Uint64,
    pub total_coins: Decimal256,
    pub winner: Option<Addr>,
    pub auction_id: u64,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Bid {
    pub amount: Decimal256,
    pub bidder: Addr,
    pub placed: Timestamp,
    pub bid_item_id: u64,
}

// impl<'a> PrimaryKey<'a> for AuctionId {
//     type Prefix = ();
//     type SubPrefix = ();
//     type Suffix = u64;
//     type SuperSuffix = u64;

//     #[inline]
//     fn key(&self) -> Vec<CwKey> {
//         vec![CwKey::Val64(self.0.u64().to_cw_bytes())]
//     }
// }

// impl Auction {
//     pub fn create_auction(
//         &mut self,
//         ctx: &mut Context,
//         auction: Auction,
//     ) -> Result<AuctionId> {
        
//     }
// }