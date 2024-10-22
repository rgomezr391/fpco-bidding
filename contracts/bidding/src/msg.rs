use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use crate::state::{Auction, AuctionStatus, BidItem};

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
        id: u64,
        status: AuctionStatus,
    },
    AddBidItems {
        auction_id: u64,
        bid_items: Vec<String>,
    },
    PlaceBid {
        bid_item_id: u64,
        coins_to_bid: u128,
    },
    AdvanceCrank {},
}

#[cw_serde]
pub struct AdminsListResp {
    pub admins: Vec<Addr>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    Admin {},
    #[returns(Auction)]
    Auction {
        id: u64
    },
    #[returns(Vec<(u64, BidItem)>)]
    BidItemsByAuctionId {
        auction_id: u64
    },
    #[returns(BidItem)]
    BidItem {
        id: u64
    },
}
