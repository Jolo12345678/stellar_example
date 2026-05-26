#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_1_happy_path_advance_payment() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RiceChainContract);
    let client = RiceChainContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract(token_admin);
    let token_client = token::Client::new(&env, &token_address);

    let farmer = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Mint baseline mock funds to buyer
    token_client.mint(&buyer, &10000);

    // Execute core MVP transaction flow
    client.advance_payment(&token_address, &101, &farmer, &buyer, &1500);

    // Verify on-chain results
    assert_eq!(token_client.balance(&farmer), 1500);
    assert_eq!(token_client.balance(&buyer), 8500);
}

#[test]
#[should_panic(expected = "Invoice already processed")]
fn test_2_edge_case_duplicate_invoice() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RiceChainContract);
    let client = RiceChainContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract(token_admin);
    let token_client = token::Client::new(&env, &token_address);

    let farmer = Address::generate(&env);
    let buyer = Address::generate(&env);

    token_client.mint(&buyer, &20000);

    // First payment succeeds
    client.advance_payment(&token_address, &999, &farmer, &buyer, &2000);
    
    // Second payment with exact same invoice ID must panic
    client.advance_payment(&token_address, &999, &farmer, &buyer, &2000);
}

#[test]
fn test_3_state_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RiceChainContract);
    let client = RiceChainContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract(token_admin);
    let token_client = token::Client::new(&env, &token_address);

    let farmer = Address::generate(&env);
    let buyer = Address::generate(&env);

    token_client.mint(&buyer, &5000);
    client.advance_payment(&token_address, &42, &farmer, &buyer, &1000);

    // Verify state store matches input parameters precisely
    let saved_invoice = client.get_invoice(&42).unwrap();
    assert_eq!(saved_invoice.farmer, farmer);
    assert_eq!(saved_invoice.amount, 1000);
    assert_eq!(saved_invoice.is_paid, true);
}