use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use crate::state::{Auction, AuctionId, AuctionStatus, BidItem, BidItemId, BidItemKey};

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Leave {},
    // Donate {},
    CreateAuction {
        name: String,
        bid_items: Vec<String>,
    },
    SetAuctionState {
        id: AuctionId,
        status: AuctionStatus,
    },
    AddBidItems {
        auction_id: AuctionId,
        bid_items: Vec<String>,
    },
    PlaceBid {
        bid_item_id: BidItemId,
        coins_to_bid: u128,
    },
    AdvanceCrank {},
}

#[cw_serde]
pub struct AdminsListResp {
    pub admins: Vec<Addr>,
}

#[cw_serde]
pub struct BidItemsByIdResp {
    pub bid_item_id: u64,
    pub data: BidItem,
    pub auction_id: u64,
    pub bid_state: AuctionStatus,
}

// #[derive(Clone, Serialize, Deserialize, JsonSchema, Debug)]
// pub struct Market {
//     pub key: Key,
//     pub data: MarketData,
// }

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    Admin {},
    #[returns(Auction)]
    Auction {
        id: AuctionId
    },
    #[returns(Vec<(BidItemId, BidItem)>)]
    BidItemsByAuctionId {
        auction_id: AuctionId
    },
    #[returns(BidItem)]
    BidItem {
        id: BidItemId
    },
    #[returns(Vec<(BidItemKey, BidItem)>)]
    BidItems {
        start_after: Option<BidItemKey>,
    },
    #[returns(Vec<(AuctionId, Auction)>)]
    Auctions {
        start_after: Option<AuctionId>,
    },
    #[returns(Vec<(BidItemId, BidItem)>)]
    BidItemsById {
        bid_items_ids: Vec<BidItemId>,
    }
}
