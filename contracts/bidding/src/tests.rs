#[cfg(test)]
mod tests {
    use cosmwasm_std::Addr;
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::{msg::{ExecuteMsg, InstantiateMsg, QueryMsg}, state::{Auction, AuctionId, AuctionStatus, BidItem, BidItemId, BidItemKey}};
    use crate::contract::{execute, instantiate, query};

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
        let auction_id_u32 = auction_id.parse::<u32>().unwrap();

        let resp: Auction = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::Auction { id: AuctionId(auction_id_u32) })
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
        let auction_id_u32 = auction_id.parse::<u32>().unwrap();

        let resp = app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::SetAuctionState { id: AuctionId(auction_id_u32), status: AuctionStatus::PendingCompletion },
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
        let auction_id_u32_first = auction_id.parse::<u32>().unwrap();

        let bid_items= vec![ 
            "TA1 6th bid item".to_string(),
            "TA1 7th bid item".to_string(),
        ];

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::AddBidItems { auction_id: AuctionId(auction_id_u32_first), bid_items: bid_items },
            &[],
        )
        .unwrap();

        let resp: Vec<(BidItemId, BidItem)> = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::BidItemsByAuctionId { auction_id: AuctionId(auction_id_u32_first) })
            .unwrap();

        assert_eq!(resp.len(), 7);

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
            &ExecuteMsg::SetAuctionState { id: AuctionId(auction_id_u32_first), status: AuctionStatus::PendingCompletion },
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

        // Total Bids -> TestAuction #1 { BidItem1, BidItem2, BidItem3, BidItem4, BidItem5, BidItem6, BidItem7 }

        // Will Process TestAuction #1 { BidItem1, BidItem2, BidItem3 }
        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::AdvanceCrank {  },
            &[],
        )
        .unwrap();

        // TestAuction #1 { BidItem4, BidItem5, BidItem6, BidItem7 }
        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::AdvanceCrank {  },
            &[],
        )
        .unwrap();

        // TestAuction #1 { BidItem7 }
        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::AdvanceCrank {  },
            &[],
        )
        .unwrap();

        // Will process nothing0
        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::AdvanceCrank {  },
            &[],
        )
        .unwrap();

    }

    #[test]
    fn get_paginated_auctions() {
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

        // let resp: Vec<(AuctionId, Auction)> = app
        //     .wrap()
        //     .query_wasm_smart(&addr, &QueryMsg::Auctions { start_after: Some(resp[8].0) } )
        //     .unwrap();

        // assert_eq!(resp.len(), 7);
    }

    #[test]
    fn get_bid_items_by_ids() {
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

        let bid_items= vec![ "My first bid item".to_string(), "My second bid item".to_string() ];

        app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::CreateAuction { name: "TestAuction #1".to_string(), bid_items: bid_items },
            &[],
        )
        .unwrap();

        let resp: Vec<(BidItemKey, BidItem)> = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::BidItems { start_after: None })
            .unwrap();

        let mut bid_items_ids: Vec<BidItemId> = vec![];
        bid_items_ids.push(resp[0].0.bid_item_id);
        bid_items_ids.push(resp[1].0.bid_item_id);

        let resp: Vec<(BidItemId, BidItem)> = app
            .wrap()
            .query_wasm_smart(&addr, &QueryMsg::BidItemsById { bid_items_ids: bid_items_ids } )
            .unwrap();

        assert_eq!(resp[0].1.name, "My first bid item".to_string());
        assert_eq!(resp[1].1.name, "My second bid item".to_string());
    }
}
