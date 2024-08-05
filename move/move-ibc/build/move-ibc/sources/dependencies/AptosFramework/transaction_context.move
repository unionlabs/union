module aptos_framework::transaction_context {
    use std::error;
    use std::features;
    use std::option::Option;
    use std::string::String;

    /// Transaction context is only available in the user transaction prologue, execution, or epilogue phases.
    const ETRANSACTION_CONTEXT_NOT_AVAILABLE: u64 = 1;

    /// The transaction context extension feature is not enabled.
    const ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED: u64 = 2;

    /// A wrapper denoting aptos unique identifer (AUID)
    /// for storing an address
    struct AUID has drop, store {
        unique_address: address
    }

    /// Represents the entry function payload.
    struct EntryFunctionPayload has copy, drop {
        account_address: address,
        module_name: String,
        function_name: String,
        ty_args_names: vector<String>,
        args: vector<vector<u8>>,
    }

    /// Represents the multisig payload.
    struct MultisigPayload has copy, drop {
        multisig_address: address,
        entry_function_payload: Option<EntryFunctionPayload>,
    }

    /// Returns the transaction hash of the current transaction.
    native fun get_txn_hash(): vector<u8>;

    /// Returns the transaction hash of the current transaction.
    /// Internally calls the private function `get_txn_hash`.
    /// This function is created for to feature gate the `get_txn_hash` function.
    public fun get_transaction_hash(): vector<u8> {
        get_txn_hash()
    }

    /// Returns a universally unique identifier (of type address) generated
    /// by hashing the transaction hash of this transaction and a sequence number
    /// specific to this transaction. This function can be called any
    /// number of times inside a single transaction. Each such call increments
    /// the sequence number and generates a new unique address.
    /// Uses Scheme in types/src/transaction/authenticator.rs for domain separation
    /// from other ways of generating unique addresses.
    native fun generate_unique_address(): address;

    /// Returns a aptos unique identifier. Internally calls
    /// the private function `generate_unique_address`. This function is
    /// created for to feature gate the `generate_unique_address` function.
    public fun generate_auid_address(): address {
        generate_unique_address()
    }

    /// Returns the script hash of the current entry function.
    public native fun get_script_hash(): vector<u8>;

    /// This method runs `generate_unique_address` native function and returns
    /// the generated unique address wrapped in the AUID class.
    public fun generate_auid(): AUID {
        return AUID {
            unique_address: generate_unique_address()
        }
    }

    /// Returns the unique address wrapped in the given AUID struct.
    public fun auid_address(auid: &AUID): address {
        auid.unique_address
    }

    /// Returns the sender's address for the current transaction.
    /// This function aborts if called outside of the transaction prologue, execution, or epilogue phases.
    public fun sender(): address {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        sender_internal()
    }
    native fun sender_internal(): address;

    /// Returns the list of the secondary signers for the current transaction.
    /// If the current transaction has no secondary signers, this function returns an empty vector.
    /// This function aborts if called outside of the transaction prologue, execution, or epilogue phases.
    public fun secondary_signers(): vector<address> {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        secondary_signers_internal()
    }
    native fun secondary_signers_internal(): vector<address>;

    /// Returns the gas payer address for the current transaction.
    /// It is either the sender's address if no separate gas fee payer is specified for the current transaction,
    /// or the address of the separate gas fee payer if one is specified.
    /// This function aborts if called outside of the transaction prologue, execution, or epilogue phases.
    public fun gas_payer(): address {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        gas_payer_internal()
    }
    native fun gas_payer_internal(): address;

    /// Returns the max gas amount in units which is specified for the current transaction.
    /// This function aborts if called outside of the transaction prologue, execution, or epilogue phases.
    public fun max_gas_amount(): u64 {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        max_gas_amount_internal()
    }
    native fun max_gas_amount_internal(): u64;

    /// Returns the gas unit price in Octas which is specified for the current transaction.
    /// This function aborts if called outside of the transaction prologue, execution, or epilogue phases.
    public fun gas_unit_price(): u64 {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        gas_unit_price_internal()
    }
    native fun gas_unit_price_internal(): u64;

    /// Returns the chain ID specified for the current transaction.
    /// This function aborts if called outside of the transaction prologue, execution, or epilogue phases.
    public fun chain_id(): u8 {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        chain_id_internal()
    }
    native fun chain_id_internal(): u8;

    /// Returns the entry function payload if the current transaction has such a payload. Otherwise, return `None`.
    /// This function aborts if called outside of the transaction prologue, execution, or epilogue phases.
    public fun entry_function_payload(): Option<EntryFunctionPayload> {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        entry_function_payload_internal()
    }
    native fun entry_function_payload_internal(): Option<EntryFunctionPayload>;

    /// Returns the account address of the entry function payload.
    public fun account_address(payload: &EntryFunctionPayload): address {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        payload.account_address
    }

    /// Returns the module name of the entry function payload.
    public fun module_name(payload: &EntryFunctionPayload): String {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        payload.module_name
    }

    /// Returns the function name of the entry function payload.
    public fun function_name(payload: &EntryFunctionPayload): String {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        payload.function_name
    }

    /// Returns the type arguments names of the entry function payload.
    public fun type_arg_names(payload: &EntryFunctionPayload): vector<String> {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        payload.ty_args_names
    }

    /// Returns the arguments of the entry function payload.
    public fun args(payload: &EntryFunctionPayload): vector<vector<u8>> {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        payload.args
    }

    /// Returns the multisig payload if the current transaction has such a payload. Otherwise, return `None`.
    /// This function aborts if called outside of the transaction prologue, execution, or epilogue phases.
    public fun multisig_payload(): Option<MultisigPayload> {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        multisig_payload_internal()
    }
    native fun multisig_payload_internal(): Option<MultisigPayload>;

    /// Returns the multisig account address of the multisig payload.
    public fun multisig_address(payload: &MultisigPayload): address {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        payload.multisig_address
    }

    /// Returns the inner entry function payload of the multisig payload.
    public fun inner_entry_function_payload(payload: &MultisigPayload): Option<EntryFunctionPayload> {
        assert!(features::transaction_context_extension_enabled(), error::invalid_state(ETRANSACTION_CONTEXT_EXTENSION_NOT_ENABLED));
        payload.entry_function_payload
    }

    #[test()]
    fun test_auid_uniquess() {
        use std::vector;

        let auids: vector<address> = vector<address>[];
        let i: u64 = 0;
        let count: u64 = 50;
        while (i < count) {
            i = i + 1;
            vector::push_back(&mut auids, generate_auid_address());
        };
        i = 0;
        while (i < count - 1) {
            let j: u64 = i + 1;
            while (j < count) {
                assert!(*vector::borrow(&auids, i) != *vector::borrow(&auids, j), 0);
                j = j + 1;
            };
            i = i + 1;
        };
    }

    #[test]
    #[expected_failure(abort_code=196609, location = Self)]
    fun test_call_sender() {
        // expected to fail with the error code of `invalid_state(E_TRANSACTION_CONTEXT_NOT_AVAILABLE)`
        let _sender = sender();
    }

    #[test]
    #[expected_failure(abort_code=196609, location = Self)]
    fun test_call_secondary_signers() {
        // expected to fail with the error code of `invalid_state(E_TRANSACTION_CONTEXT_NOT_AVAILABLE)`
        let _secondary_signers = secondary_signers();
    }

    #[test]
    #[expected_failure(abort_code=196609, location = Self)]
    fun test_call_gas_payer() {
        // expected to fail with the error code of `invalid_state(E_TRANSACTION_CONTEXT_NOT_AVAILABLE)`
        let _gas_payer = gas_payer();
    }

    #[test]
    #[expected_failure(abort_code=196609, location = Self)]
    fun test_call_max_gas_amount() {
        // expected to fail with the error code of `invalid_state(E_TRANSACTION_CONTEXT_NOT_AVAILABLE)`
        let _max_gas_amount = max_gas_amount();
    }

    #[test]
    #[expected_failure(abort_code=196609, location = Self)]
    fun test_call_gas_unit_price() {
        // expected to fail with the error code of `invalid_state(E_TRANSACTION_CONTEXT_NOT_AVAILABLE)`
        let _gas_unit_price = gas_unit_price();
    }

    #[test]
    #[expected_failure(abort_code=196609, location = Self)]
    fun test_call_chain_id() {
        // expected to fail with the error code of `invalid_state(E_TRANSACTION_CONTEXT_NOT_AVAILABLE)`
        let _chain_id = chain_id();
    }

    #[test]
    #[expected_failure(abort_code=196609, location = Self)]
    fun test_call_entry_function_payload() {
        // expected to fail with the error code of `invalid_state(E_TRANSACTION_CONTEXT_NOT_AVAILABLE)`
        let _entry_fun = entry_function_payload();
    }

    #[test]
    #[expected_failure(abort_code=196609, location = Self)]
    fun test_call_multisig_payload() {
        // expected to fail with the error code of `invalid_state(E_TRANSACTION_CONTEXT_NOT_AVAILABLE)`
        let _multisig = multisig_payload();
    }
}
