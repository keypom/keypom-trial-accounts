// usage_tracking.rs
use crate::*;

#[near]
impl Contract {
    /// Internal method to assert if a key is allowed to perform an action.
    fn assert_action_allowed(
        &mut self,
        method_name: &str,
        contract_id: &AccountId,
        gas_attached: Gas,
        deposit_attached: U128,
    ) -> (TrialData, KeyUsage) {
        let public_key = env::signer_account_pk();

        // Fetch KeyUsage
        let key_usage = self
            .key_usage_by_pk
            .get_mut(&public_key)
            .expect("Access denied");

        // Fetch TrialData
        let trial_data = self
            .trial_data_by_id
            .get(&key_usage.trial_id)
            .cloned()
            .expect("Trial data not found");

        // Check expiration time
        if let Some(expiration_time) = trial_data.expiration_time {
            assert!(
                env::block_timestamp() < expiration_time,
                "Trial period has expired"
            );
        }

        // Check allowed methods
        if !trial_data
            .allowed_methods
            .contains(&method_name.to_string())
        {
            env::panic_str("Method not allowed");
        }

        // Check allowed contracts
        if !trial_data.allowed_contracts.contains(contract_id) {
            env::panic_str("Contract not allowed");
        }

        // Check gas and deposit limits
        if let Some(max_gas) = trial_data.max_gas {
            assert!(
                gas_attached.as_gas() <= max_gas,
                "Attached gas exceeds maximum allowed"
            );
        }
        if let Some(max_deposit) = trial_data.max_deposit {
            assert!(
                deposit_attached.0 <= max_deposit.0,
                "Attached deposit exceeds maximum allowed"
            );
        }

        // Update usage statistics
        key_usage.usage_stats.total_interactions += 1;
        key_usage.usage_stats.gas_used += gas_attached.as_gas();
        key_usage.usage_stats.deposit_used.0 += deposit_attached.0;

        (trial_data.clone(), key_usage.clone())
    }

    /// Public method to perform an action after all validations.
    pub fn perform_action(
        &mut self,
        contract_id: AccountId,
        method_name: String,
        args: Vec<u8>,
        gas: Gas,
        deposit: U128,
    ) -> Promise {
        let (trial_data, key_usage) =
            self.assert_action_allowed(&method_name, &contract_id, gas, deposit);

        // Check exit conditions if any
        if let Some(exit_conditions) = &trial_data.exit_conditions {
            // Check transaction limit
            if let Some(transaction_limit) = exit_conditions.transaction_limit {
                if key_usage.usage_stats.total_interactions > transaction_limit {
                    env::panic_str("Transaction limit reached");
                }
            }
            // Additional exit conditions can be checked here
        }

        // All checks passed, proceed to call the MPC contract
        self.call_mpc_contract(
            contract_id,
            method_name,
            args,
            gas,
            deposit,
            env::signer_account_pk(),
            trial_data.chain_id,
        )
        .as_return()
    }

    /// Internal method to call the MPC contract.
    fn call_mpc_contract(
        &self,
        contract_id: AccountId,
        method_name: String,
        args: Vec<u8>,
        gas: Gas,
        deposit: U128,
        public_key: PublicKey,
        chain_id: u64,
    ) -> Promise {
        let signer_pk = env::signer_account_pk();
        env::log_str(&format!(
            "Signer public key length: {}",
            signer_pk.as_bytes().len()
        ));
        // Build the NEAR transaction
        let tx = TransactionBuilder::new::<NEAR>()
            .signer_id(env::current_account_id().to_string())
            .signer_public_key(convert_pk_to_omni(env::signer_account_pk()))
            .nonce(0) // Replace with appropriate nonce
            .receiver_id(contract_id.clone().to_string())
            .block_hash(OmniBlockHash([0u8; 32]))
            .actions(vec![OmniAction::FunctionCall(Box::new(
                OmniFunctionCallAction {
                    method_name: method_name.clone(),
                    args: args.clone(),
                    gas: OmniU64(gas.as_gas()),
                    deposit: OmniU128(deposit.into()),
                },
            ))])
            .build();

        let request_payload = create_sign_request_from_transaction(tx, public_key); // Call the helper

        // Call the MPC contract to get a signature
        Promise::new(self.mpc_contract.clone())
            .function_call_weight(
                "sign".to_string(),
                near_sdk::serde_json::to_vec(&request_payload).unwrap(), // Serialize with the correct structure
                NearToken::from_near(1),
                Gas::from_tgas(30),
                GasWeight(1),
            )
            .as_return()
    }
}
