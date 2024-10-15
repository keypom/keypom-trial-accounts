use near_sdk::json_types::U128;

use crate::*;

#[derive(Clone)]
#[near(serializers = [json, borsh])]
pub enum Action {
    NEAR(NearAction),
    EVM(EvmAction),
}

#[derive(Clone)]
#[near(serializers = [json, borsh])]
pub struct NearAction {
    pub method_name: String,
    pub contract_id: AccountId,
    pub gas_attached: Gas,
    pub deposit_attached: NearToken,
}

#[derive(Clone)]
#[near(serializers = [json, borsh])]
pub struct EvmAction {
    pub method_name: String,
    pub contract_address: [u8; 20], // Ethereum address
    pub gas_limit: u128,
    pub value: U128, // Value in wei
}
