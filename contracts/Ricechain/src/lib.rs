#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Invoice(u64), // Maps invoice ID to Invoice struct
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invoice {
    pub farmer: Address,
    pub amount: i128,
    pub is_paid: bool,
}

@contract
pub struct RiceChainContract;

#[contractimpl]
impl RiceChainContract {
    /// Initializes and funds an advance payment for a specific delivery invoice.
    /// The contract pulls funds from the buyer/escrow provider and sends them directly to the farmer.
    pub fn advance_payment(
        env: Env,
        token_address: Address,
        invoice_id: u64,
        farmer: Address,
        buyer: Address,
        amount: i128,
    ) {
        // Ensure the buyer is authorizing this specific payout transaction
        buyer.require_auth();

        let key = DataKey::Invoice(invoice_id);
        
        // Ensure the invoice ID has not been used or processed previously
        if env.storage().persistent().has(&key) {
            panic!("Invoice already processed");
        }

        // Initialize token client for USDC asset movement
        let client = token::Client::new(&env, &token_address);

        // Execute on-chain cash movement: Buyer/Escrow -> Farmer
        client.transfer(&buyer, &farmer, &amount);

        // Persist the state change onto the ledger to prevent double spending
        let invoice_record = Invoice {
            farmer: farmer.clone(),
            amount,
            is_paid: true,
        };
        env.storage().persistent().set(&key, &invoice_record);
    }

    /// Helper view function to verify the state of an invoice registration
    pub fn get_invoice(env: Env, invoice_id: u64) -> Option<Invoice> {
        let key = DataKey::Invoice(invoice_id);
        env.storage().persistent().get(&key)
    }
}