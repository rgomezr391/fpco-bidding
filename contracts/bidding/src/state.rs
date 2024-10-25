use std::mem;

use cosmwasm_std::{Addr, StdError, StdResult, Timestamp, Uint64};
use cw_storage_plus::{IntKey, Item, Key, KeyDeserialize, Map, Prefixer, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::impl_monotonic_id;

// Auction has one to many Bid Items
// Bid Items has one to many Bids

pub const ADMIN: Item<Addr> = Item::new("admin");
// pub const DONATION_DENOM: Item<String> = Item::new("donation_denom");
pub const ADMINS: Map<&Addr, Timestamp> = Map::new("admins");
pub const AUCTIONS: Map<AuctionId, Auction> = Map::new("auctions");
pub const BID_ITEMS: Map<BidItemKey, BidItem> = Map::new("bid_items");
pub const BIDS: Map<BidKey, Bid> = Map::new("bids");
pub const AUCTIONS_CRANK_QUEUE: Item<Vec<u64>> = Item::new("auctions_crank_queue");

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

///////////////////////////
/// Keys Definitions   ///
//////////////////////////

#[inline]
fn slice_to_array<const N: usize>(slice: &[u8]) -> StdResult<[u8; N]> {
    slice
        .try_into()
        .map_err(|err: std::array::TryFromSliceError| StdError::generic_err(err.to_string()))
}

/////// Auction ID ///////

impl_monotonic_id!(
    AuctionId,
    "auction_id",
    "Id that represents an auction."
);

// #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Default, Debug)]
// pub struct AuctionId(pub u32);

// impl<'a> PrimaryKey<'a> for AuctionId {
//     type Prefix = ();
//     type SubPrefix = ();
//     type Suffix = u64;
//     type SuperSuffix = u64;

//     #[inline]
//     fn key(&self) -> Vec<Key> {
//         vec![Key::Val32(self.0.to_cw_bytes())]
//     }
// }

// impl<'a> Prefixer<'a> for AuctionId {
//     fn prefix(&self) -> Vec<Key> {
//         vec![Key::Val32(self.0.to_cw_bytes())]
//     }
// }

// impl KeyDeserialize for AuctionId {
//     type Output = Self;
//     const KEY_ELEMS: u16 = 1;

//     fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
//         Self::from_slice(value.as_slice())
//     }

//     fn from_slice(value: &[u8]) -> StdResult<Self::Output> {
//         Ok(Self(u32::from_cw_bytes(slice_to_array(value)?)))
//     }
// }

/////// Bid Item ID ///////

impl_monotonic_id!(
    BidItemId,
    "auction_id",
    "Id that represents an auction."
);

// #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Default, Debug)]
// pub struct BidItemId(pub u32);

// impl<'a> PrimaryKey<'a> for BidItemId {
//     type Prefix = ();
//     type SubPrefix = ();
//     type Suffix = u64;
//     type SuperSuffix = u64;

//     #[inline]
//     fn key(&self) -> Vec<Key> {
//         vec![Key::Val32(self.0.to_cw_bytes())]
//     }
// }

// impl<'a> Prefixer<'a> for BidItemId {
//     fn prefix(&self) -> Vec<Key> {
//         vec![Key::Val32(self.0.to_cw_bytes())]
//     }
// }

// impl KeyDeserialize for BidItemId {
//     type Output = Self;
//     const KEY_ELEMS: u16 = 1;

//     fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
//         Self::from_slice(value.as_slice())
//     }

//     fn from_slice(value: &[u8]) -> StdResult<Self::Output> {
//         Ok(Self(u32::from_cw_bytes(slice_to_array(value)?)))
//     }
// }

/////// Bid ID ///////

impl_monotonic_id!(
    BidId,
    "auction_id",
    "Id that represents an auction."
);

// #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Default, Debug)]
// pub struct BidId(pub u32);

// impl<'a> PrimaryKey<'a> for BidId {
//     type Prefix = ();
//     type SubPrefix = ();
//     type Suffix = u64;
//     type SuperSuffix = u64;

//     #[inline]
//     fn key(&self) -> Vec<Key> {
//         vec![Key::Val32(self.0.to_cw_bytes())]
//     }
// }

// impl<'a> Prefixer<'a> for BidId {
//     fn prefix(&self) -> Vec<Key> {
//         vec![Key::Val32(self.0.to_cw_bytes())]
//     }
// }

// impl KeyDeserialize for BidId {
//     type Output = Self;
//     const KEY_ELEMS: u16 = 1;

//     fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
//         Self::from_slice(value.as_slice())
//     }

//     fn from_slice(value: &[u8]) -> StdResult<Self::Output> {
//         Ok(Self(u32::from_cw_bytes(slice_to_array(value)?)))
//     }
// }

/////// Bid Item Key ///////

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema, Debug)]
pub struct BidItemKey {
    pub auction_id: AuctionId,
    pub bid_item_id: BidItemId,
}

impl<'a> PrimaryKey<'a> for BidItemKey {
    type Prefix = AuctionId;
    type SubPrefix = ();
    type Suffix = BidItemId;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        let mut keys = self.auction_id.key();
        keys.extend(self.bid_item_id.key());
        keys
    }
}

impl KeyDeserialize for BidItemKey {
    type Output = Self;
    const KEY_ELEMS: u16 = 2;

    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Self::from_slice(value.as_slice())
    }

    fn from_slice(value: &[u8]) -> StdResult<Self::Output> {
        const SIZE: usize = 2 + mem::size_of::<AuctionId>() + mem::size_of::<BidItemId>();

        if value.len() != SIZE {
            return Err(StdError::invalid_data_size(SIZE, value.len()));
        }

        let (auction, bid_item) = value.split_at(mem::size_of::<u32>() + 2);
        let auction = u32::from_cw_bytes(slice_to_array(&auction[2..]).unwrap());
        let bid_item = u32::from_cw_bytes(slice_to_array(bid_item).unwrap());

        Ok(Self {
            auction_id: AuctionId(auction),
            bid_item_id: BidItemId(bid_item),
        })
    }
}

/////// Bid Key ///////

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema, Debug)]
pub struct BidKey {
    pub bid_item_id: BidItemId,
    pub bid_id: BidId,
}

impl<'a> PrimaryKey<'a> for BidKey {
    type Prefix = BidItemId;
    type SubPrefix = ();
    type Suffix = BidId;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        let mut keys = self.bid_item_id.key();
        keys.extend(self.bid_id.key());
        keys
    }
}

impl KeyDeserialize for BidKey {
    type Output = Self;
    const KEY_ELEMS: u16 = 2;

    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Self::from_slice(value.as_slice())
    }

    fn from_slice(value: &[u8]) -> StdResult<Self::Output> {
        const SIZE: usize = 2 + mem::size_of::<BidItemId>() + mem::size_of::<BidId>();

        if value.len() != SIZE {
            return Err(StdError::invalid_data_size(SIZE, value.len()));
        }

        let (bid_item, bid) = value.split_at(mem::size_of::<u32>() + 2);
        let bid_item = u32::from_cw_bytes(slice_to_array(&bid_item[2..]).unwrap());
        let bid = u32::from_cw_bytes(slice_to_array(bid).unwrap());

        Ok(Self {
            bid_item_id: BidItemId(bid_item),
            bid_id: BidId(bid),
        })
    }
}