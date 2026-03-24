#![cfg(test)]

use soroban_sdk::{testutils::Ledger, Address, Env};

use crate::{
    HistoryFilter, SortCriterion, SortOrder, StellarEscrowContract,
    StellarEscrowContractClient, TradeFilter, TradeSortField, TradeStatus,
};

fn setup() -> (Env, StellarEscrowContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StellarEscrowContract);
    let client = StellarEscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let token = Address::generate(&env);

    client.initialize(&admin, &token, &100); // 1% fee

    (env, client, admin, seller, buyer)
}

fn no_filter(env: &Env) -> HistoryFilter {
    HistoryFilter { status: None, from_ledger: None, to_ledger: None }
}

fn empty_trade_filter() -> TradeFilter {
    TradeFilter {
        status: None,
        min_amount: None,
        max_amount: None,
        from_ledger: None,
        to_ledger: None,
        seller: None,
        buyer: None,
    }
}

fn sort_by_created_asc() -> SortCriterion {
    SortCriterion { field: TradeSortField::CreatedAt, order: SortOrder::Ascending }
}

// =============================================================================
// Existing history tests
// =============================================================================

#[test]
fn test_history_empty_for_new_address() {
    let (env, client, _, seller, _) = setup();
    let page = client.get_transaction_history(&seller, &no_filter(&env), &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 0);
    assert_eq!(page.records.len(), 0);
}

#[test]
fn test_history_shows_created_trade() {
    let (env, client, _, seller, buyer) = setup();
    let trade_id = client.create_trade(&seller, &buyer, &1000, &None, &None);
    let page = client.get_transaction_history(&seller, &no_filter(&env), &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    let record = page.records.get(0).unwrap();
    assert_eq!(record.trade_id, trade_id);
    assert_eq!(record.amount, 1000);
    assert_eq!(record.status, TradeStatus::Created);
}

#[test]
fn test_history_visible_from_buyer_address() {
    let (env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &500, &None, &None);
    let page = client.get_transaction_history(&buyer, &no_filter(&env), &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
}

#[test]
fn test_history_filter_by_status() {
    let (env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    client.cancel_trade(&1);
    let filter = HistoryFilter { status: Some(TradeStatus::Cancelled), from_ledger: None, to_ledger: None };
    let page = client.get_transaction_history(&seller, &filter, &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().status, TradeStatus::Cancelled);
}

#[test]
fn test_history_filter_by_ledger_range() {
    let (env, client, _, seller, buyer) = setup();
    env.ledger().set_sequence_number(1);
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    env.ledger().set_sequence_number(100);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    let filter = HistoryFilter { status: None, from_ledger: Some(50), to_ledger: Some(200) };
    let page = client.get_transaction_history(&seller, &filter, &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 2000);
}

#[test]
fn test_history_sort_descending() {
    let (env, client, _, seller, buyer) = setup();
    env.ledger().set_sequence_number(1);
    client.create_trade(&seller, &buyer, &100, &None, &None);
    env.ledger().set_sequence_number(10);
    client.create_trade(&seller, &buyer, &200, &None, &None);
    let page = client.get_transaction_history(&seller, &no_filter(&env), &SortOrder::Descending, &0, &10);
    assert_eq!(page.records.get(0).unwrap().amount, 200);
    assert_eq!(page.records.get(1).unwrap().amount, 100);
}

#[test]
fn test_history_pagination() {
    let (env, client, _, seller, buyer) = setup();
    for _ in 0..5 {
        client.create_trade(&seller, &buyer, &1000, &None, &None);
    }
    let page1 = client.get_transaction_history(&seller, &no_filter(&env), &SortOrder::Ascending, &0, &3);
    assert_eq!(page1.records.len(), 3);
    assert_eq!(page1.total, 5);
    let page2 = client.get_transaction_history(&seller, &no_filter(&env), &SortOrder::Ascending, &3, &3);
    assert_eq!(page2.records.len(), 2);
}

#[test]
fn test_export_csv_returns_header_and_rows() {
    let (env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    let csv = client.export_transaction_csv(&seller, &HistoryFilter { status: None, from_ledger: None, to_ledger: None });
    assert!(csv.len() > 0);
}

// =============================================================================
// Advanced filtering tests
// =============================================================================

#[test]
fn test_search_trades_empty_state() {
    let (_env, client, _, _, _) = setup();
    let page = client.search_trades(&empty_trade_filter(), &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 0);
}

#[test]
fn test_search_trades_returns_all_without_filter() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    let page = client.search_trades(&empty_trade_filter(), &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 2);
}

#[test]
fn test_search_filter_by_status() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    client.cancel_trade(&1);
    let filter = TradeFilter { status: Some(TradeStatus::Cancelled), ..empty_trade_filter() };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().status, TradeStatus::Cancelled);
}

#[test]
fn test_search_filter_by_min_amount() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &500, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    let filter = TradeFilter { min_amount: Some(1000), ..empty_trade_filter() };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 2000);
}

#[test]
fn test_search_filter_by_max_amount() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &500, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);
    let filter = TradeFilter { max_amount: Some(1000), ..empty_trade_filter() };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 500);
}

#[test]
fn test_search_filter_by_amount_range() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &100, &None, &None);
    client.create_trade(&seller, &buyer, &500, &None, &None);
    client.create_trade(&seller, &buyer, &5000, &None, &None);
    let filter = TradeFilter { min_amount: Some(200), max_amount: Some(1000), ..empty_trade_filter() };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 500);
}

#[test]
fn test_search_filter_by_seller() {
    let (env, client, _, seller, buyer) = setup();
    let other_seller = Address::generate(&env);
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&other_seller, &buyer, &2000, &None, &None);
    let filter = TradeFilter { seller: Some(seller.clone()), ..empty_trade_filter() };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().seller, seller);
}

#[test]
fn test_search_filter_by_buyer() {
    let (env, client, _, seller, buyer) = setup();
    let other_buyer = Address::generate(&env);
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &other_buyer, &2000, &None, &None);
    let filter = TradeFilter { buyer: Some(buyer.clone()), ..empty_trade_filter() };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().buyer, buyer);
}

#[test]
fn test_search_multi_criteria_filter() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &500, &None, &None);
    client.create_trade(&seller, &buyer, &1500, &None, &None);
    client.cancel_trade(&1);
    // status=Cancelled AND amount>=200
    let filter = TradeFilter {
        status: Some(TradeStatus::Cancelled),
        min_amount: Some(200),
        ..empty_trade_filter()
    };
    let page = client.search_trades(&filter, &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 500);
}

#[test]
fn test_search_sort_by_amount_descending() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &300, &None, &None);
    client.create_trade(&seller, &buyer, &100, &None, &None);
    client.create_trade(&seller, &buyer, &200, &None, &None);
    let sort = SortCriterion { field: TradeSortField::Amount, order: SortOrder::Descending };
    let page = client.search_trades(&empty_trade_filter(), &sort, &0, &10).unwrap();
    assert_eq!(page.records.get(0).unwrap().amount, 300);
    assert_eq!(page.records.get(1).unwrap().amount, 200);
    assert_eq!(page.records.get(2).unwrap().amount, 100);
}

#[test]
fn test_search_sort_by_amount_ascending() {
    let (_env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &300, &None, &None);
    client.create_trade(&seller, &buyer, &100, &None, &None);
    client.create_trade(&seller, &buyer, &200, &None, &None);
    let sort = SortCriterion { field: TradeSortField::Amount, order: SortOrder::Ascending };
    let page = client.search_trades(&empty_trade_filter(), &sort, &0, &10).unwrap();
    assert_eq!(page.records.get(0).unwrap().amount, 100);
    assert_eq!(page.records.get(2).unwrap().amount, 300);
}

#[test]
fn test_search_invalid_amount_range_returns_error() {
    let (_env, client, _, _, _) = setup();
    let filter = TradeFilter { min_amount: Some(1000), max_amount: Some(100), ..empty_trade_filter() };
    assert!(client.try_search_trades(&filter, &sort_by_created_asc(), &0, &10).is_err());
}

#[test]
fn test_search_invalid_ledger_range_returns_error() {
    let (_env, client, _, _, _) = setup();
    let filter = TradeFilter { from_ledger: Some(500), to_ledger: Some(100), ..empty_trade_filter() };
    assert!(client.try_search_trades(&filter, &sort_by_created_asc(), &0, &10).is_err());
}

#[test]
fn test_search_pagination() {
    let (_env, client, _, seller, buyer) = setup();
    for _ in 0..5 {
        client.create_trade(&seller, &buyer, &1000, &None, &None);
    }
    let page1 = client.search_trades(&empty_trade_filter(), &sort_by_created_asc(), &0, &3).unwrap();
    assert_eq!(page1.records.len(), 3);
    assert_eq!(page1.total, 5);
    let page2 = client.search_trades(&empty_trade_filter(), &sort_by_created_asc(), &3, &3).unwrap();
    assert_eq!(page2.records.len(), 2);
}

#[test]
fn test_search_trades_for_address() {
    let (env, client, _, seller, buyer) = setup();
    let other = Address::generate(&env);
    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&other, &buyer, &2000, &None, &None);
    // seller's index only has trade 1
    let page = client.search_trades_for_address(&seller, &empty_trade_filter(), &sort_by_created_asc(), &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 1000);
}

// =============================================================================
// Filter preset tests
// =============================================================================

#[test]
fn test_save_and_retrieve_preset() {
    let (env, client, _, seller, _) = setup();
    let name = soroban_sdk::String::from_str(&env, "my preset");
    let preset_id = client.save_filter_preset(
        &seller, &name, &empty_trade_filter(), &sort_by_created_asc(),
    ).unwrap();
    let preset = client.get_filter_preset(&preset_id).unwrap();
    assert_eq!(preset.id, preset_id);
    assert_eq!(preset.owner, seller);
}

#[test]
fn test_list_presets_for_user() {
    let (env, client, _, seller, _) = setup();
    let name1 = soroban_sdk::String::from_str(&env, "preset one");
    let name2 = soroban_sdk::String::from_str(&env, "preset two");
    client.save_filter_preset(&seller, &name1, &empty_trade_filter(), &sort_by_created_asc()).unwrap();
    client.save_filter_preset(&seller, &name2, &empty_trade_filter(), &sort_by_created_asc()).unwrap();
    let presets = client.list_filter_presets(&seller);
    assert_eq!(presets.len(), 2);
}

#[test]
fn test_update_preset() {
    let (env, client, _, seller, _) = setup();
    let name = soroban_sdk::String::from_str(&env, "original");
    let preset_id = client.save_filter_preset(&seller, &name, &empty_trade_filter(), &sort_by_created_asc()).unwrap();
    let new_name = soroban_sdk::String::from_str(&env, "updated");
    let new_filter = TradeFilter { min_amount: Some(100), ..empty_trade_filter() };
    client.update_filter_preset(&seller, &preset_id, &new_name, &new_filter, &sort_by_created_asc()).unwrap();
    let preset = client.get_filter_preset(&preset_id).unwrap();
    assert_eq!(preset.filter.min_amount, Some(100));
}

#[test]
fn test_delete_preset() {
    let (env, client, _, seller, _) = setup();
    let name = soroban_sdk::String::from_str(&env, "to delete");
    let preset_id = client.save_filter_preset(&seller, &name, &empty_trade_filter(), &sort_by_created_asc()).unwrap();
    client.delete_filter_preset(&seller, &preset_id).unwrap();
    assert!(client.try_get_filter_preset(&preset_id).is_err());
}

#[test]
fn test_delete_preset_removes_from_list() {
    let (env, client, _, seller, _) = setup();
    let name = soroban_sdk::String::from_str(&env, "p1");
    let preset_id = client.save_filter_preset(&seller, &name, &empty_trade_filter(), &sort_by_created_asc()).unwrap();
    client.delete_filter_preset(&seller, &preset_id).unwrap();
    let presets = client.list_filter_presets(&seller);
    assert_eq!(presets.len(), 0);
}

#[test]
fn test_unauthorized_delete_fails() {
    let (env, client, _, seller, buyer) = setup();
    let name = soroban_sdk::String::from_str(&env, "mine");
    let preset_id = client.save_filter_preset(&seller, &name, &empty_trade_filter(), &sort_by_created_asc()).unwrap();
    assert!(client.try_delete_filter_preset(&buyer, &preset_id).is_err());
}

#[test]
fn test_search_with_preset() {
    let (env, client, _, seller, buyer) = setup();
    client.create_trade(&seller, &buyer, &500, &None, &None);
    client.create_trade(&seller, &buyer, &1500, &None, &None);
    let filter = TradeFilter { min_amount: Some(1000), ..empty_trade_filter() };
    let name = soroban_sdk::String::from_str(&env, "big trades");
    let preset_id = client.save_filter_preset(&seller, &name, &filter, &sort_by_created_asc()).unwrap();
    let page = client.search_with_preset(&preset_id, &0, &10).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 1500);
}

#[test]
fn test_preset_persistence_across_calls() {
    let (env, client, _, seller, _) = setup();
    let name = soroban_sdk::String::from_str(&env, "persistent");
    let filter = TradeFilter { status: Some(TradeStatus::Created), ..empty_trade_filter() };
    let preset_id = client.save_filter_preset(&seller, &name, &filter, &sort_by_created_asc()).unwrap();
    // Retrieve in a separate call — verifies persistent storage
    let preset = client.get_filter_preset(&preset_id).unwrap();
    assert_eq!(preset.filter.status, Some(TradeStatus::Created));
}
