/// Enhanced multisig account standard on Aptos. This is different from the native multisig scheme support enforced via
/// the account's auth key.
///
/// This module allows creating a flexible and powerful multisig account with seamless support for updating owners
/// without changing the auth key. Users can choose to store transaction payloads waiting for owner signatures on chain
/// or off chain (primary consideration is decentralization/transparency vs gas cost).
///
/// The multisig account is a resource account underneath. By default, it has no auth key and can only be controlled via
/// the special multisig transaction flow. However, owners can create a transaction to change the auth key to match a
/// private key off chain if so desired.
///
/// Transactions need to be executed in order of creation, similar to transactions for a normal Aptos account (enforced
/// with account nonce).
///
/// The flow is like below:
/// 1. Owners can create a new multisig account by calling create (signer is default single owner) or with
/// create_with_owners where multiple initial owner addresses can be specified. This is different (and easier) from
/// the native multisig scheme where the owners' public keys have to be specified. Here, only addresses are needed.
/// 2. Owners can be added/removed any time by calling add_owners or remove_owners. The transactions to do still need
/// to follow the k-of-n scheme specified for the multisig account.
/// 3. To create a new transaction, an owner can call create_transaction with the transaction payload. This will store
/// the full transaction payload on chain, which adds decentralization (censorship is not possible as the data is
/// available on chain) and makes it easier to fetch all transactions waiting for execution. If saving gas is desired,
/// an owner can alternatively call create_transaction_with_hash where only the payload hash is stored. Later execution
/// will be verified using the hash. Only owners can create transactions and a transaction id (incremeting id) will be
/// assigned.
/// 4. To approve or reject a transaction, other owners can call approve() or reject() with the transaction id.
/// 5. If there are enough approvals, any owner can execute the transaction using the special MultisigTransaction type
/// with the transaction id if the full payload is already stored on chain or with the transaction payload if only a
/// hash is stored. Transaction execution will first check with this module that the transaction payload has gotten
/// enough signatures. If so, it will be executed as the multisig account. The owner who executes will pay for gas.
/// 6. If there are enough rejections, any owner can finalize the rejection by calling execute_rejected_transaction().
///
/// Note that this multisig account model is not designed to use with a large number of owners. The more owners there
/// are, the more expensive voting on transactions will become. If a large number of owners is designed, such as in a
/// flat governance structure, clients are encouraged to write their own modules on top of this multisig account module
/// and implement the governance voting logic on top.
module aptos_framework::multisig_account {
    use aptos_framework::account::{Self, SignerCapability, new_event_handle, create_resource_address};
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::chain_id;
    use aptos_framework::create_signer::create_signer;
    use aptos_framework::coin;
    use aptos_framework::event::{EventHandle, emit_event, emit};
    use aptos_framework::timestamp::now_seconds;
    use aptos_std::simple_map::{Self, SimpleMap};
    use aptos_std::table::{Self, Table};
    use std::bcs::to_bytes;
    use std::error;
    use std::hash::sha3_256;
    use std::option::{Self, Option};
    use std::signer::address_of;
    use std::string::String;
    use std::vector;

    /// The salt used to create a resource account during multisig account creation.
    /// This is used to avoid conflicts with other modules that also create resource accounts with the same owner
    /// account.
    const DOMAIN_SEPARATOR: vector<u8> = b"aptos_framework::multisig_account";

    // Any error codes > 2000 can be thrown as part of transaction prologue.
    /// Owner list cannot contain the same address more than once.
    const EDUPLICATE_OWNER: u64 = 1;
    /// Specified account is not a multisig account.
    const EACCOUNT_NOT_MULTISIG: u64 = 2002;
    /// Account executing this operation is not an owner of the multisig account.
    const ENOT_OWNER: u64 = 2003;
    /// Transaction payload cannot be empty.
    const EPAYLOAD_CANNOT_BE_EMPTY: u64 = 4;
    /// Multisig account must have at least one owner.
    const ENOT_ENOUGH_OWNERS: u64 = 5;
    /// Transaction with specified id cannot be found.
    const ETRANSACTION_NOT_FOUND: u64 = 2006;
    /// Provided target function does not match the hash stored in the on-chain transaction.
    const EPAYLOAD_DOES_NOT_MATCH_HASH: u64 = 2008;
    /// Transaction has not received enough approvals to be executed.
    const ENOT_ENOUGH_APPROVALS: u64 = 2009;
    /// Provided target function does not match the payload stored in the on-chain transaction.
    const EPAYLOAD_DOES_NOT_MATCH: u64 = 2010;
    /// Transaction has not received enough rejections to be officially rejected.
    const ENOT_ENOUGH_REJECTIONS: u64 = 10;
    /// Number of signatures required must be more than zero and at most the total number of owners.
    const EINVALID_SIGNATURES_REQUIRED: u64 = 11;
    /// Payload hash must be exactly 32 bytes (sha3-256).
    const EINVALID_PAYLOAD_HASH: u64 = 12;
    /// The multisig account itself cannot be an owner.
    const EOWNER_CANNOT_BE_MULTISIG_ACCOUNT_ITSELF: u64 = 13;
    /// Multisig accounts has not been enabled on this current network yet.
    const EMULTISIG_ACCOUNTS_NOT_ENABLED_YET: u64 = 14;
    /// The number of metadata keys and values don't match.
    const ENUMBER_OF_METADATA_KEYS_AND_VALUES_DONT_MATCH: u64 = 15;
    /// The specified metadata contains duplicate attributes (keys).
    const EDUPLICATE_METADATA_KEY: u64 = 16;
    /// The sequence number provided is invalid. It must be between [1, next pending transaction - 1].
    const EINVALID_SEQUENCE_NUMBER: u64 = 17;
    /// Provided owners to remove and new owners overlap.
    const EOWNERS_TO_REMOVE_NEW_OWNERS_OVERLAP: u64 = 18;
    /// The number of pending transactions has exceeded the maximum allowed.
    const EMAX_PENDING_TRANSACTIONS_EXCEEDED: u64 = 19;
    /// The multisig v2 enhancement feature is not enabled.
    const EMULTISIG_V2_ENHANCEMENT_NOT_ENABLED: u64 = 20;


    const ZERO_AUTH_KEY: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";

    const MAX_PENDING_TRANSACTIONS: u64 = 20;

    /// Represents a multisig account's configurations and transactions.
    /// This will be stored in the multisig account (created as a resource account separate from any owner accounts).
    struct MultisigAccount has key {
        // The list of all owner addresses.
        owners: vector<address>,
        // The number of signatures required to pass a transaction (k in k-of-n).
        num_signatures_required: u64,
        // Map from transaction id (incrementing id) to transactions to execute for this multisig account.
        // Already executed transactions are deleted to save on storage but can always be accessed via events.
        transactions: Table<u64, MultisigTransaction>,
        // The sequence number assigned to the last executed or rejected transaction. Used to enforce in-order
        // executions of proposals, similar to sequence number for a normal (single-user) account.
        last_executed_sequence_number: u64,
        // The sequence number to assign to the next transaction. This is not always last_executed_sequence_number + 1
        // as there can be multiple pending transactions. The number of pending transactions should be equal to
        // next_sequence_number - (last_executed_sequence_number + 1).
        next_sequence_number: u64,
        // The signer capability controlling the multisig (resource) account. This can be exchanged for the signer.
        // Currently not used as the MultisigTransaction can validate and create a signer directly in the VM but
        // this can be useful to have for on-chain composability in the future.
        signer_cap: Option<SignerCapability>,
        // The multisig account's metadata such as name, description, etc. This can be updated through the multisig
        // transaction flow (i.e. self-update).
        // Note: Attributes can be arbitrarily set by the multisig account and thus will only be used for off-chain
        // display purposes only. They don't change any on-chain semantics of the multisig account.
        metadata: SimpleMap<String, vector<u8>>,

        // Events.
        add_owners_events: EventHandle<AddOwnersEvent>,
        remove_owners_events: EventHandle<RemoveOwnersEvent>,
        update_signature_required_events: EventHandle<UpdateSignaturesRequiredEvent>,
        create_transaction_events: EventHandle<CreateTransactionEvent>,
        vote_events: EventHandle<VoteEvent>,
        execute_rejected_transaction_events: EventHandle<ExecuteRejectedTransactionEvent>,
        execute_transaction_events: EventHandle<TransactionExecutionSucceededEvent>,
        transaction_execution_failed_events: EventHandle<TransactionExecutionFailedEvent>,
        metadata_updated_events: EventHandle<MetadataUpdatedEvent>,
    }

    /// A transaction to be executed in a multisig account.
    /// This must contain either the full transaction payload or its hash (stored as bytes).
    struct MultisigTransaction has copy, drop, store {
        payload: Option<vector<u8>>,
        payload_hash: Option<vector<u8>>,
        // Mapping from owner adress to vote (yes for approve, no for reject). Uses a simple map to deduplicate.
        votes: SimpleMap<address, bool>,
        // The owner who created this transaction.
        creator: address,
        // The timestamp in seconds when the transaction was created.
        creation_time_secs: u64,
    }

    /// Contains information about execution failure.
    struct ExecutionError has copy, drop, store {
        // The module where the error occurs.
        abort_location: String,
        // There are 3 error types, stored as strings:
        // 1. VMError. Indicates an error from the VM, e.g. out of gas, invalid auth key, etc.
        // 2. MoveAbort. Indicates an abort, e.g. assertion failure, from inside the executed Move code.
        // 3. MoveExecutionFailure. Indicates an error from Move code where the VM could not continue. For example,
        // arithmetic failures.
        error_type: String,
        // The detailed error code explaining which error occurred.
        error_code: u64,
    }

    /// Used only for verifying multisig account creation on top of existing accounts.
    struct MultisigAccountCreationMessage has copy, drop {
        // Chain id is included to prevent cross-chain replay.
        chain_id: u8,
        // Account address is included to prevent cross-account replay (when multiple accounts share the same auth key).
        account_address: address,
        // Sequence number is not needed for replay protection as the multisig account can only be created once.
        // But it's included to ensure timely execution of account creation.
        sequence_number: u64,
        // The list of owners for the multisig account.
        owners: vector<address>,
        // The number of signatures required (signature threshold).
        num_signatures_required: u64,
    }

    /// Used only for verifying multisig account creation on top of existing accounts and rotating the auth key to 0x0.
    struct MultisigAccountCreationWithAuthKeyRevocationMessage has copy, drop {
        // Chain id is included to prevent cross-chain replay.
        chain_id: u8,
        // Account address is included to prevent cross-account replay (when multiple accounts share the same auth key).
        account_address: address,
        // Sequence number is not needed for replay protection as the multisig account can only be created once.
        // But it's included to ensure timely execution of account creation.
        sequence_number: u64,
        // The list of owners for the multisig account.
        owners: vector<address>,
        // The number of signatures required (signature threshold).
        num_signatures_required: u64,
    }

    /// Event emitted when new owners are added to the multisig account.
    struct AddOwnersEvent has drop, store {
        owners_added: vector<address>,
    }

    #[event]
    struct AddOwners has drop, store {
        multisig_account: address,
        owners_added: vector<address>,
    }

    /// Event emitted when new owners are removed from the multisig account.
    struct RemoveOwnersEvent has drop, store {
        owners_removed: vector<address>,
    }

    #[event]
    struct RemoveOwners has drop, store {
        multisig_account: address,
        owners_removed: vector<address>,
    }

    /// Event emitted when the number of signatures required is updated.
    struct UpdateSignaturesRequiredEvent has drop, store {
        old_num_signatures_required: u64,
        new_num_signatures_required: u64,
    }

    #[event]
    struct UpdateSignaturesRequired has drop, store {
        multisig_account: address,
        old_num_signatures_required: u64,
        new_num_signatures_required: u64,
    }

    /// Event emitted when a transaction is created.
    struct CreateTransactionEvent has drop, store {
        creator: address,
        sequence_number: u64,
        transaction: MultisigTransaction,
    }

    #[event]
    struct CreateTransaction has drop, store {
        multisig_account: address,
        creator: address,
        sequence_number: u64,
        transaction: MultisigTransaction,
    }

    /// Event emitted when an owner approves or rejects a transaction.
    struct VoteEvent has drop, store {
        owner: address,
        sequence_number: u64,
        approved: bool,
    }

    #[event]
    struct Vote has drop, store {
        multisig_account: address,
        owner: address,
        sequence_number: u64,
        approved: bool,
    }

    /// Event emitted when a transaction is officially rejected because the number of rejections has reached the
    /// number of signatures required.
    struct ExecuteRejectedTransactionEvent has drop, store {
        sequence_number: u64,
        num_rejections: u64,
        executor: address,
    }

    #[event]
    struct ExecuteRejectedTransaction has drop, store {
        multisig_account: address,
        sequence_number: u64,
        num_rejections: u64,
        executor: address,
    }

    /// Event emitted when a transaction is executed.
    struct TransactionExecutionSucceededEvent has drop, store {
        executor: address,
        sequence_number: u64,
        transaction_payload: vector<u8>,
        num_approvals: u64,
    }

    #[event]
    struct TransactionExecutionSucceeded has drop, store {
        multisig_account: address,
        executor: address,
        sequence_number: u64,
        transaction_payload: vector<u8>,
        num_approvals: u64,
    }

    /// Event emitted when a transaction's execution failed.
    struct TransactionExecutionFailedEvent has drop, store {
        executor: address,
        sequence_number: u64,
        transaction_payload: vector<u8>,
        num_approvals: u64,
        execution_error: ExecutionError,
    }

    #[event]
    struct TransactionExecutionFailed has drop, store {
        multisig_account: address,
        executor: address,
        sequence_number: u64,
        transaction_payload: vector<u8>,
        num_approvals: u64,
        execution_error: ExecutionError,
    }

    /// Event emitted when a transaction's metadata is updated.
    struct MetadataUpdatedEvent has drop, store {
        old_metadata: SimpleMap<String, vector<u8>>,
        new_metadata: SimpleMap<String, vector<u8>>,
    }

    #[event]
    struct MetadataUpdated has drop, store {
        multisig_account: address,
        old_metadata: SimpleMap<String, vector<u8>>,
        new_metadata: SimpleMap<String, vector<u8>>,
    }

    ////////////////////////// View functions ///////////////////////////////

    #[view]
    /// Return the multisig account's metadata.
    public fun metadata(multisig_account: address): SimpleMap<String, vector<u8>> acquires MultisigAccount {
        borrow_global<MultisigAccount>(multisig_account).metadata
    }

    #[view]
    /// Return the number of signatures required to execute or execute-reject a transaction in the provided
    /// multisig account.
    public fun num_signatures_required(multisig_account: address): u64 acquires MultisigAccount {
        borrow_global<MultisigAccount>(multisig_account).num_signatures_required
    }

    #[view]
    /// Return a vector of all of the provided multisig account's owners.
    public fun owners(multisig_account: address): vector<address> acquires MultisigAccount {
        borrow_global<MultisigAccount>(multisig_account).owners
    }

    #[view]
    /// Return true if the provided owner is an owner of the provided multisig account.
    public fun is_owner(owner: address, multisig_account: address): bool acquires MultisigAccount {
        vector::contains(&borrow_global<MultisigAccount>(multisig_account).owners, &owner)
    }

    #[view]
    /// Return the transaction with the given transaction id.
    public fun get_transaction(
        multisig_account: address,
        sequence_number: u64,
    ): MultisigTransaction acquires MultisigAccount {
        let multisig_account_resource = borrow_global<MultisigAccount>(multisig_account);
        assert!(
            sequence_number > 0 && sequence_number < multisig_account_resource.next_sequence_number,
            error::invalid_argument(EINVALID_SEQUENCE_NUMBER),
        );
        *table::borrow(&multisig_account_resource.transactions, sequence_number)
    }

    #[view]
    /// Return all pending transactions.
    public fun get_pending_transactions(
        multisig_account: address
    ): vector<MultisigTransaction> acquires MultisigAccount {
        let pending_transactions: vector<MultisigTransaction> = vector[];
        let multisig_account = borrow_global<MultisigAccount>(multisig_account);
        let i = multisig_account.last_executed_sequence_number + 1;
        let next_sequence_number = multisig_account.next_sequence_number;
        while (i < next_sequence_number) {
            vector::push_back(&mut pending_transactions, *table::borrow(&multisig_account.transactions, i));
            i = i + 1;
        };
        pending_transactions
    }

    #[view]
    /// Return the payload for the next transaction in the queue.
    public fun get_next_transaction_payload(
        multisig_account: address, provided_payload: vector<u8>): vector<u8> acquires MultisigAccount {
        let multisig_account_resource = borrow_global<MultisigAccount>(multisig_account);
        let sequence_number = multisig_account_resource.last_executed_sequence_number + 1;
        let transaction = table::borrow(&multisig_account_resource.transactions, sequence_number);

        if (option::is_some(&transaction.payload)) {
            *option::borrow(&transaction.payload)
        } else {
            provided_payload
        }
    }

    #[view]
    /// Return true if the transaction with given transaction id can be executed now.
    public fun can_be_executed(multisig_account: address, sequence_number: u64): bool acquires MultisigAccount {
        assert_valid_sequence_number(multisig_account, sequence_number);
        let (num_approvals, _) = num_approvals_and_rejections(multisig_account, sequence_number);
        sequence_number == last_resolved_sequence_number(multisig_account) + 1 &&
            num_approvals >= num_signatures_required(multisig_account)
    }

    #[view]
    /// Return true if the owner can execute the transaction with given transaction id now.
    public fun can_execute(owner: address, multisig_account: address, sequence_number: u64): bool acquires MultisigAccount {
        assert_valid_sequence_number(multisig_account, sequence_number);
        let (num_approvals, _) = num_approvals_and_rejections(multisig_account, sequence_number);
        if (!has_voted_for_approval(multisig_account, sequence_number, owner)) {
            num_approvals = num_approvals + 1;
        };
        is_owner(owner, multisig_account) &&
            sequence_number == last_resolved_sequence_number(multisig_account) + 1 &&
            num_approvals >= num_signatures_required(multisig_account)
    }

    #[view]
    /// Return true if the transaction with given transaction id can be officially rejected.
    public fun can_be_rejected(multisig_account: address, sequence_number: u64): bool acquires MultisigAccount {
        assert_valid_sequence_number(multisig_account, sequence_number);
        let (_, num_rejections) = num_approvals_and_rejections(multisig_account, sequence_number);
        sequence_number == last_resolved_sequence_number(multisig_account) + 1 &&
            num_rejections >= num_signatures_required(multisig_account)
    }

    #[view]
    /// Return true if the owner can execute the "rejected" transaction with given transaction id now.
    public fun can_reject(owner: address, multisig_account: address, sequence_number: u64): bool acquires MultisigAccount {
        assert_valid_sequence_number(multisig_account, sequence_number);
        let (_, num_rejections) = num_approvals_and_rejections(multisig_account, sequence_number);
        if (!has_voted_for_rejection(multisig_account, sequence_number, owner)) {
            num_rejections = num_rejections + 1;
        };
        is_owner(owner, multisig_account) &&
            sequence_number == last_resolved_sequence_number(multisig_account) + 1 &&
            num_rejections >= num_signatures_required(multisig_account)
    }

    #[view]
    /// Return the predicted address for the next multisig account if created from the given creator address.
    public fun get_next_multisig_account_address(creator: address): address {
        let owner_nonce = account::get_sequence_number(creator);
        create_resource_address(&creator, create_multisig_account_seed(to_bytes(&owner_nonce)))
    }

    #[view]
    /// Return the id of the last transaction that was executed (successful or failed) or removed.
    public fun last_resolved_sequence_number(multisig_account: address): u64 acquires MultisigAccount {
        let multisig_account_resource = borrow_global_mut<MultisigAccount>(multisig_account);
        multisig_account_resource.last_executed_sequence_number
    }

    #[view]
    /// Return the id of the next transaction created.
    public fun next_sequence_number(multisig_account: address): u64 acquires MultisigAccount {
        let multisig_account_resource = borrow_global_mut<MultisigAccount>(multisig_account);
        multisig_account_resource.next_sequence_number
    }

    #[view]
    /// Return a bool tuple indicating whether an owner has voted and if so, whether they voted yes or no.
    public fun vote(
        multisig_account: address, sequence_number: u64, owner: address): (bool, bool) acquires MultisigAccount {
        let multisig_account_resource = borrow_global_mut<MultisigAccount>(multisig_account);
        assert!(
            sequence_number > 0 && sequence_number < multisig_account_resource.next_sequence_number,
            error::invalid_argument(EINVALID_SEQUENCE_NUMBER),
        );
        let transaction = table::borrow(&multisig_account_resource.transactions, sequence_number);
        let votes = &transaction.votes;
        let voted = simple_map::contains_key(votes, &owner);
        let vote = voted && *simple_map::borrow(votes, &owner);
        (voted, vote)
    }

    #[view]
    public fun available_transaction_queue_capacity(multisig_account: address): u64 acquires MultisigAccount {
        let multisig_account_resource = borrow_global_mut<MultisigAccount>(multisig_account);
        let num_pending_transactions = multisig_account_resource.next_sequence_number - multisig_account_resource.last_executed_sequence_number - 1;
        if (num_pending_transactions > MAX_PENDING_TRANSACTIONS) {
            0
        } else {
            MAX_PENDING_TRANSACTIONS - num_pending_transactions
        }
    }

    ////////////////////////// Multisig account creation functions ///////////////////////////////

    /// Creates a new multisig account on top of an existing account.
    ///
    /// This offers a migration path for an existing account with a multi-ed25519 auth key (native multisig account).
    /// In order to ensure a malicious module cannot obtain backdoor control over an existing account, a signed message
    /// with a valid signature from the account's auth key is required.
    ///
    /// Note that this does not revoke auth key-based control over the account. Owners should separately rotate the auth
    /// key after they are fully migrated to the new multisig account. Alternatively, they can call
    /// create_with_existing_account_and_revoke_auth_key instead.
    public entry fun create_with_existing_account(
        multisig_address: address,
        owners: vector<address>,
        num_signatures_required: u64,
        account_scheme: u8,
        account_public_key: vector<u8>,
        create_multisig_account_signed_message: vector<u8>,
        metadata_keys: vector<String>,
        metadata_values: vector<vector<u8>>,
    ) acquires MultisigAccount {
        // Verify that the `MultisigAccountCreationMessage` has the right information and is signed by the account
        // owner's key.
        let proof_challenge = MultisigAccountCreationMessage {
            chain_id: chain_id::get(),
            account_address: multisig_address,
            sequence_number: account::get_sequence_number(multisig_address),
            owners,
            num_signatures_required,
        };
        account::verify_signed_message(
            multisig_address,
            account_scheme,
            account_public_key,
            create_multisig_account_signed_message,
            proof_challenge,
        );

        // We create the signer for the multisig account here since this is required to add the MultisigAccount resource
        // This should be safe and authorized because we have verified the signed message from the existing account
        // that authorizes creating a multisig account with the specified owners and signature threshold.
        let multisig_account = &create_signer(multisig_address);
        create_with_owners_internal(
            multisig_account,
            owners,
            num_signatures_required,
            option::none<SignerCapability>(),
            metadata_keys,
            metadata_values,
        );
    }

    /// Creates a new multisig account on top of an existing account and immediately rotate the origin auth key to 0x0.
    ///
    /// Note: If the original account is a resource account, this does not revoke all control over it as if any
    /// SignerCapability of the resource account still exists, it can still be used to generate the signer for the
    /// account.
    public entry fun create_with_existing_account_and_revoke_auth_key(
        multisig_address: address,
        owners: vector<address>,
        num_signatures_required: u64,
        account_scheme: u8,
        account_public_key: vector<u8>,
        create_multisig_account_signed_message: vector<u8>,
        metadata_keys: vector<String>,
        metadata_values: vector<vector<u8>>,
    ) acquires MultisigAccount {
        // Verify that the `MultisigAccountCreationMessage` has the right information and is signed by the account
        // owner's key.
        let proof_challenge = MultisigAccountCreationWithAuthKeyRevocationMessage {
            chain_id: chain_id::get(),
            account_address: multisig_address,
            sequence_number: account::get_sequence_number(multisig_address),
            owners,
            num_signatures_required,
        };
        account::verify_signed_message(
            multisig_address,
            account_scheme,
            account_public_key,
            create_multisig_account_signed_message,
            proof_challenge,
        );

        // We create the signer for the multisig account here since this is required to add the MultisigAccount resource
        // This should be safe and authorized because we have verified the signed message from the existing account
        // that authorizes creating a multisig account with the specified owners and signature threshold.
        let multisig_account = &create_signer(multisig_address);
        create_with_owners_internal(
            multisig_account,
            owners,
            num_signatures_required,
            option::none<SignerCapability>(),
            metadata_keys,
            metadata_values,
        );

        // Rotate the account's auth key to 0x0, which effectively revokes control via auth key.
        let multisig_address = address_of(multisig_account);
        account::rotate_authentication_key_internal(multisig_account, ZERO_AUTH_KEY);
        // This also needs to revoke any signer capability or rotation capability that exists for the account to
        // completely remove all access to the account.
        if (account::is_signer_capability_offered(multisig_address)) {
            account::revoke_any_signer_capability(multisig_account);
        };
        if (account::is_rotation_capability_offered(multisig_address)) {
            account::revoke_any_rotation_capability(multisig_account);
        };
    }

    /// Creates a new multisig account and add the signer as a single owner.
    public entry fun create(
        owner: &signer,
        num_signatures_required: u64,
        metadata_keys: vector<String>,
        metadata_values: vector<vector<u8>>,
    ) acquires MultisigAccount {
        create_with_owners(owner, vector[], num_signatures_required, metadata_keys, metadata_values);
    }

    /// Creates a new multisig account with the specified additional owner list and signatures required.
    ///
    /// @param additional_owners The owner account who calls this function cannot be in the additional_owners and there
    /// cannot be any duplicate owners in the list.
    /// @param num_signatures_required The number of signatures required to execute a transaction. Must be at least 1 and
    /// at most the total number of owners.
    public entry fun create_with_owners(
        owner: &signer,
        additional_owners: vector<address>,
        num_signatures_required: u64,
        metadata_keys: vector<String>,
        metadata_values: vector<vector<u8>>,
    ) acquires MultisigAccount {
        let (multisig_account, multisig_signer_cap) = create_multisig_account(owner);
        vector::push_back(&mut additional_owners, address_of(owner));
        create_with_owners_internal(
            &multisig_account,
            additional_owners,
            num_signatures_required,
            option::some(multisig_signer_cap),
            metadata_keys,
            metadata_values,
        );
    }

    /// Like `create_with_owners`, but removes the calling account after creation.
    ///
    /// This is for creating a vanity multisig account from a bootstrapping account that should not
    /// be an owner after the vanity multisig address has been secured.
    public entry fun create_with_owners_then_remove_bootstrapper(
        bootstrapper: &signer,
        owners: vector<address>,
        num_signatures_required: u64,
        metadata_keys: vector<String>,
        metadata_values: vector<vector<u8>>,
    ) acquires MultisigAccount {
        let bootstrapper_address = address_of(bootstrapper);
        create_with_owners(
            bootstrapper,
            owners,
            num_signatures_required,
            metadata_keys,
            metadata_values
        );
        update_owner_schema(
            get_next_multisig_account_address(bootstrapper_address),
            vector[],
            vector[bootstrapper_address],
            option::none()
        );
    }

    fun create_with_owners_internal(
        multisig_account: &signer,
        owners: vector<address>,
        num_signatures_required: u64,
        multisig_account_signer_cap: Option<SignerCapability>,
        metadata_keys: vector<String>,
        metadata_values: vector<vector<u8>>,
    ) acquires MultisigAccount {
        assert!(features::multisig_accounts_enabled(), error::unavailable(EMULTISIG_ACCOUNTS_NOT_ENABLED_YET));
        assert!(
            num_signatures_required > 0 && num_signatures_required <= vector::length(&owners),
            error::invalid_argument(EINVALID_SIGNATURES_REQUIRED),
        );

        let multisig_address = address_of(multisig_account);
        validate_owners(&owners, multisig_address);
        move_to(multisig_account, MultisigAccount {
            owners,
            num_signatures_required,
            transactions: table::new<u64, MultisigTransaction>(),
            metadata: simple_map::create<String, vector<u8>>(),
            // First transaction will start at id 1 instead of 0.
            last_executed_sequence_number: 0,
            next_sequence_number: 1,
            signer_cap: multisig_account_signer_cap,
            add_owners_events: new_event_handle<AddOwnersEvent>(multisig_account),
            remove_owners_events: new_event_handle<RemoveOwnersEvent>(multisig_account),
            update_signature_required_events: new_event_handle<UpdateSignaturesRequiredEvent>(multisig_account),
            create_transaction_events: new_event_handle<CreateTransactionEvent>(multisig_account),
            vote_events: new_event_handle<VoteEvent>(multisig_account),
            execute_rejected_transaction_events: new_event_handle<ExecuteRejectedTransactionEvent>(multisig_account),
            execute_transaction_events: new_event_handle<TransactionExecutionSucceededEvent>(multisig_account),
            transaction_execution_failed_events: new_event_handle<TransactionExecutionFailedEvent>(multisig_account),
            metadata_updated_events: new_event_handle<MetadataUpdatedEvent>(multisig_account),
        });

        update_metadata_internal(multisig_account, metadata_keys, metadata_values, false);
    }

    ////////////////////////// Self-updates ///////////////////////////////

    /// Similar to add_owners, but only allow adding one owner.
    entry fun add_owner(multisig_account: &signer, new_owner: address) acquires MultisigAccount {
        add_owners(multisig_account, vector[new_owner]);
    }

    /// Add new owners to the multisig account. This can only be invoked by the multisig account itself, through the
    /// proposal flow.
    ///
    /// Note that this function is not public so it can only be invoked directly instead of via a module or script. This
    /// ensures that a multisig transaction cannot lead to another module obtaining the multisig signer and using it to
    /// maliciously alter the owners list.
    entry fun add_owners(
        multisig_account: &signer, new_owners: vector<address>) acquires MultisigAccount {
        update_owner_schema(
            address_of(multisig_account),
            new_owners,
            vector[],
            option::none()
        );
    }

    /// Add owners then update number of signatures required, in a single operation.
    entry fun add_owners_and_update_signatures_required(
        multisig_account: &signer,
        new_owners: vector<address>,
        new_num_signatures_required: u64
    ) acquires MultisigAccount {
        update_owner_schema(
            address_of(multisig_account),
            new_owners,
            vector[],
            option::some(new_num_signatures_required)
        );
    }

    /// Similar to remove_owners, but only allow removing one owner.
    entry fun remove_owner(
        multisig_account: &signer, owner_to_remove: address) acquires MultisigAccount {
        remove_owners(multisig_account, vector[owner_to_remove]);
    }

    /// Remove owners from the multisig account. This can only be invoked by the multisig account itself, through the
    /// proposal flow.
    ///
    /// This function skips any owners who are not in the multisig account's list of owners.
    /// Note that this function is not public so it can only be invoked directly instead of via a module or script. This
    /// ensures that a multisig transaction cannot lead to another module obtaining the multisig signer and using it to
    /// maliciously alter the owners list.
    entry fun remove_owners(
        multisig_account: &signer, owners_to_remove: vector<address>) acquires MultisigAccount {
        update_owner_schema(
            address_of(multisig_account),
            vector[],
            owners_to_remove,
            option::none()
        );
    }

    /// Swap an owner in for an old one, without changing required signatures.
    entry fun swap_owner(
        multisig_account: &signer,
        to_swap_in: address,
        to_swap_out: address
    ) acquires MultisigAccount {
        update_owner_schema(
            address_of(multisig_account),
            vector[to_swap_in],
            vector[to_swap_out],
            option::none()
        );
    }

    /// Swap owners in and out, without changing required signatures.
    entry fun swap_owners(
        multisig_account: &signer,
        to_swap_in: vector<address>,
        to_swap_out: vector<address>
    ) acquires MultisigAccount {
        update_owner_schema(
            address_of(multisig_account),
            to_swap_in,
            to_swap_out,
            option::none()
        );
    }

    /// Swap owners in and out, updating number of required signatures.
    entry fun swap_owners_and_update_signatures_required(
        multisig_account: &signer,
        new_owners: vector<address>,
        owners_to_remove: vector<address>,
        new_num_signatures_required: u64
    ) acquires MultisigAccount {
        update_owner_schema(
            address_of(multisig_account),
            new_owners,
            owners_to_remove,
            option::some(new_num_signatures_required)
        );
    }

    /// Update the number of signatures required to execute transaction in the specified multisig account.
    ///
    /// This can only be invoked by the multisig account itself, through the proposal flow.
    /// Note that this function is not public so it can only be invoked directly instead of via a module or script. This
    /// ensures that a multisig transaction cannot lead to another module obtaining the multisig signer and using it to
    /// maliciously alter the number of signatures required.
    entry fun update_signatures_required(
        multisig_account: &signer, new_num_signatures_required: u64) acquires MultisigAccount {
        update_owner_schema(
            address_of(multisig_account),
            vector[],
            vector[],
            option::some(new_num_signatures_required)
        );
    }

    /// Allow the multisig account to update its own metadata. Note that this overrides the entire existing metadata.
    /// If any attributes are not specified in the metadata, they will be removed!
    ///
    /// This can only be invoked by the multisig account itself, through the proposal flow.
    /// Note that this function is not public so it can only be invoked directly instead of via a module or script. This
    /// ensures that a multisig transaction cannot lead to another module obtaining the multisig signer and using it to
    /// maliciously alter the number of signatures required.
    entry fun update_metadata(
        multisig_account: &signer, keys: vector<String>, values: vector<vector<u8>>) acquires MultisigAccount {
        update_metadata_internal(multisig_account, keys, values, true);
    }

    fun update_metadata_internal(
        multisig_account: &signer,
        keys: vector<String>,
        values: vector<vector<u8>>,
        emit_event: bool,
    ) acquires MultisigAccount {
        let num_attributes = vector::length(&keys);
        assert!(
            num_attributes == vector::length(&values),
            error::invalid_argument(ENUMBER_OF_METADATA_KEYS_AND_VALUES_DONT_MATCH),
        );

        let multisig_address = address_of(multisig_account);
        assert_multisig_account_exists(multisig_address);
        let multisig_account_resource = borrow_global_mut<MultisigAccount>(multisig_address);
        let old_metadata = multisig_account_resource.metadata;
        multisig_account_resource.metadata = simple_map::create<String, vector<u8>>();
        let metadata = &mut multisig_account_resource.metadata;
        let i = 0;
        while (i < num_attributes) {
            let key = *vector::borrow(&keys, i);
            let value = *vector::borrow(&values, i);
            assert!(
                !simple_map::contains_key(metadata, &key),
                error::invalid_argument(EDUPLICATE_METADATA_KEY),
            );

            simple_map::add(metadata, key, value);
            i = i + 1;
        };

        if (emit_event) {
            if (std::features::module_event_migration_enabled()) {
                emit(
                    MetadataUpdated {
                        multisig_account: multisig_address,
                        old_metadata,
                        new_metadata: multisig_account_resource.metadata,
                    }
                )
            };
            emit_event(
                &mut multisig_account_resource.metadata_updated_events,
                MetadataUpdatedEvent {
                    old_metadata,
                    new_metadata: multisig_account_resource.metadata,
                }
            );
        };
    }

    ////////////////////////// Multisig transaction flow ///////////////////////////////

    /// Create a multisig transaction, which will have one approval initially (from the creator).
    public entry fun create_transaction(
        owner: &signer,
        multisig_account: address,
        payload: vector<u8>,
    ) acquires MultisigAccount {
        assert!(vector::length(&payload) > 0, error::invalid_argument(EPAYLOAD_CANNOT_BE_EMPTY));

        assert_multisig_account_exists(multisig_account);
        assert_is_owner(owner, multisig_account);

        let creator = address_of(owner);
        let transaction = MultisigTransaction {
            payload: option::some(payload),
            payload_hash: option::none<vector<u8>>(),
            votes: simple_map::create<address, bool>(),
            creator,
            creation_time_secs: now_seconds(),
        };
        add_transaction(creator, multisig_account, transaction);
    }

    /// Create a multisig transaction with a transaction hash instead of the full payload.
    /// This means the payload will be stored off chain for gas saving. Later, during execution, the executor will need
    /// to provide the full payload, which will be validated against the hash stored on-chain.
    public entry fun create_transaction_with_hash(
        owner: &signer,
        multisig_account: address,
        payload_hash: vector<u8>,
    ) acquires MultisigAccount {
        // Payload hash is a sha3-256 hash, so it must be exactly 32 bytes.
        assert!(vector::length(&payload_hash) == 32, error::invalid_argument(EINVALID_PAYLOAD_HASH));

        assert_multisig_account_exists(multisig_account);
        assert_is_owner(owner, multisig_account);

        let creator = address_of(owner);
        let transaction = MultisigTransaction {
            payload: option::none<vector<u8>>(),
            payload_hash: option::some(payload_hash),
            votes: simple_map::create<address, bool>(),
            creator,
            creation_time_secs: now_seconds(),
        };
        add_transaction(creator, multisig_account, transaction);
    }

    /// Approve a multisig transaction.
    public entry fun approve_transaction(
        owner: &signer, multisig_account: address, sequence_number: u64) acquires MultisigAccount {
        vote_transanction(owner, multisig_account, sequence_number, true);
    }

    /// Reject a multisig transaction.
    public entry fun reject_transaction(
        owner: &signer, multisig_account: address, sequence_number: u64) acquires MultisigAccount {
        vote_transanction(owner, multisig_account, sequence_number, false);
    }

    /// Generic function that can be used to either approve or reject a multisig transaction
    /// Retained for backward compatibility: the function with the typographical error in its name
    /// will continue to be an accessible entry point.
    public entry fun vote_transanction(
        owner: &signer, multisig_account: address, sequence_number: u64, approved: bool) acquires MultisigAccount {
        assert_multisig_account_exists(multisig_account);
        let multisig_account_resource = borrow_global_mut<MultisigAccount>(multisig_account);
        assert_is_owner_internal(owner, multisig_account_resource);

        assert!(
            table::contains(&multisig_account_resource.transactions, sequence_number),
            error::not_found(ETRANSACTION_NOT_FOUND),
        );
        let transaction = table::borrow_mut(&mut multisig_account_resource.transactions, sequence_number);
        let votes = &mut transaction.votes;
        let owner_addr = address_of(owner);

        if (simple_map::contains_key(votes, &owner_addr)) {
            *simple_map::borrow_mut(votes, &owner_addr) = approved;
        } else {
            simple_map::add(votes, owner_addr, approved);
        };

        if (std::features::module_event_migration_enabled()) {
            emit(
                Vote {
                    multisig_account,
                    owner: owner_addr,
                    sequence_number,
                    approved,
                }
            );
        };
        emit_event(
            &mut multisig_account_resource.vote_events,
            VoteEvent {
                owner: owner_addr,
                sequence_number,
                approved,
            }
        );
    }

    /// Generic function that can be used to either approve or reject a multisig transaction
    public entry fun vote_transaction(
        owner: &signer, multisig_account: address, sequence_number: u64, approved: bool) acquires MultisigAccount {
        assert!(features::multisig_v2_enhancement_feature_enabled(), error::invalid_state(EMULTISIG_V2_ENHANCEMENT_NOT_ENABLED));
        vote_transanction(owner, multisig_account, sequence_number, approved);
    }

    /// Generic function that can be used to either approve or reject a batch of transactions within a specified range.
    public entry fun vote_transactions(
        owner: &signer, multisig_account: address, starting_sequence_number: u64, final_sequence_number: u64, approved: bool) acquires MultisigAccount {
        assert!(features::multisig_v2_enhancement_feature_enabled(), error::invalid_state(EMULTISIG_V2_ENHANCEMENT_NOT_ENABLED));
        let sequence_number = starting_sequence_number;
        while(sequence_number <= final_sequence_number) {
            vote_transanction(owner, multisig_account, sequence_number, approved);
            sequence_number = sequence_number + 1;
        }
    }

    /// Remove the next transaction if it has sufficient owner rejections.
    public entry fun execute_rejected_transaction(
        owner: &signer,
        multisig_account: address,
    ) acquires MultisigAccount {
        assert_multisig_account_exists(multisig_account);
        assert_is_owner(owner, multisig_account);

        let sequence_number = last_resolved_sequence_number(multisig_account) + 1;
        let owner_addr = address_of(owner);
        if (features::multisig_v2_enhancement_feature_enabled()) {
            // Implicitly vote for rejection if the owner has not voted for rejection yet.
            if (!has_voted_for_rejection(multisig_account, sequence_number, owner_addr)) {
                reject_transaction(owner, multisig_account, sequence_number);
            }
        };

        let multisig_account_resource = borrow_global_mut<MultisigAccount>(multisig_account);
        let (_, num_rejections) = remove_executed_transaction(multisig_account_resource);
        assert!(
            num_rejections >= multisig_account_resource.num_signatures_required,
            error::invalid_state(ENOT_ENOUGH_REJECTIONS),
        );

        if (std::features::module_event_migration_enabled()) {
            emit(
                ExecuteRejectedTransaction {
                    multisig_account,
                    sequence_number,
                    num_rejections,
                    executor: address_of(owner),
                }
            );
        };
        emit_event(
            &mut multisig_account_resource.execute_rejected_transaction_events,
            ExecuteRejectedTransactionEvent {
                sequence_number,
                num_rejections,
                executor: owner_addr,
            }
        );
    }

    /// Remove the next transactions until the final_sequence_number if they have sufficient owner rejections.
    public entry fun execute_rejected_transactions(
        owner: &signer,
        multisig_account: address,
        final_sequence_number: u64,
    ) acquires MultisigAccount {
        assert!(features::multisig_v2_enhancement_feature_enabled(), error::invalid_state(EMULTISIG_V2_ENHANCEMENT_NOT_ENABLED));
        assert!(last_resolved_sequence_number(multisig_account) < final_sequence_number, error::invalid_argument(EINVALID_SEQUENCE_NUMBER));
        assert!(final_sequence_number < next_sequence_number(multisig_account), error::invalid_argument(EINVALID_SEQUENCE_NUMBER));
        while(last_resolved_sequence_number(multisig_account) < final_sequence_number) {
            execute_rejected_transaction(owner, multisig_account);
        }
    }

    ////////////////////////// To be called by VM only ///////////////////////////////

    /// Called by the VM as part of transaction prologue, which is invoked during mempool transaction validation and as
    /// the first step of transaction execution.
    ///
    /// Transaction payload is optional if it's already stored on chain for the transaction.
    fun validate_multisig_transaction(
        owner: &signer, multisig_account: address, payload: vector<u8>) acquires MultisigAccount {
        assert_multisig_account_exists(multisig_account);
        assert_is_owner(owner, multisig_account);
        let sequence_number = last_resolved_sequence_number(multisig_account) + 1;
        assert_transaction_exists(multisig_account, sequence_number);

        if (features::multisig_v2_enhancement_feature_enabled()) {
            assert!(
                can_execute(address_of(owner), multisig_account, sequence_number),
                error::invalid_argument(ENOT_ENOUGH_APPROVALS),
            );
        }
        else {
            assert!(
                can_be_executed(multisig_account, sequence_number),
                error::invalid_argument(ENOT_ENOUGH_APPROVALS),
            );
        };

        // If the transaction payload is not stored on chain, verify that the provided payload matches the hashes stored
        // on chain.
        let multisig_account_resource = borrow_global<MultisigAccount>(multisig_account);
        let transaction = table::borrow(&multisig_account_resource.transactions, sequence_number);
        if (option::is_some(&transaction.payload_hash)) {
            let payload_hash = option::borrow(&transaction.payload_hash);
            assert!(
                sha3_256(payload) == *payload_hash,
                error::invalid_argument(EPAYLOAD_DOES_NOT_MATCH_HASH),
            );
        };

        // If the transaction payload is stored on chain and there is a provided payload,
        // verify that the provided payload matches the stored payload.
        if (features::abort_if_multisig_payload_mismatch_enabled()
            && option::is_some(&transaction.payload)
            && !vector::is_empty(&payload)
        ) {
            let stored_payload = option::borrow(&transaction.payload);
            assert!(
                payload == *stored_payload,
                error::invalid_argument(EPAYLOAD_DOES_NOT_MATCH),
            );
        }
    }

    /// Post-execution cleanup for a successful multisig transaction execution.
    /// This function is private so no other code can call this beside the VM itself as part of MultisigTransaction.
    fun successful_transaction_execution_cleanup(
        executor: address,
        multisig_account: address,
        transaction_payload: vector<u8>,
    ) acquires MultisigAccount {
        let num_approvals = transaction_execution_cleanup_common(executor, multisig_account);
        let multisig_account_resource = borrow_global_mut<MultisigAccount>(multisig_account);
        if (std::features::module_event_migration_enabled()) {
            emit(
                TransactionExecutionSucceeded {
                    multisig_account,
                    sequence_number: multisig_account_resource.last_executed_sequence_number,
                    transaction_payload,
                    num_approvals,
                    executor,
                }
            );
        };
        emit_event(
            &mut multisig_account_resource.execute_transaction_events,
            TransactionExecutionSucceededEvent {
                sequence_number: multisig_account_resource.last_executed_sequence_number,
                transaction_payload,
                num_approvals,
                executor,
            }
        );
    }

    /// Post-execution cleanup for a failed multisig transaction execution.
    /// This function is private so no other code can call this beside the VM itself as part of MultisigTransaction.
    fun failed_transaction_execution_cleanup(
        executor: address,
        multisig_account: address,
        transaction_payload: vector<u8>,
        execution_error: ExecutionError,
    ) acquires MultisigAccount {
        let num_approvals = transaction_execution_cleanup_common(executor, multisig_account);
        let multisig_account_resource = borrow_global_mut<MultisigAccount>(multisig_account);
        if (std::features::module_event_migration_enabled()) {
            emit(
                TransactionExecutionFailed {
                    multisig_account,
                    executor,
                    sequence_number: multisig_account_resource.last_executed_sequence_number,
                    transaction_payload,
                    num_approvals,
                    execution_error,
                }
            );
        };
        emit_event(
            &mut multisig_account_resource.transaction_execution_failed_events,
            TransactionExecutionFailedEvent {
                executor,
                sequence_number: multisig_account_resource.last_executed_sequence_number,
                transaction_payload,
                num_approvals,
                execution_error,
            }
        );
    }

    ////////////////////////// Private functions ///////////////////////////////

    inline fun transaction_execution_cleanup_common(executor: address, multisig_account: address): u64 acquires MultisigAccount {
        let sequence_number = last_resolved_sequence_number(multisig_account) + 1;
        let implicit_approval = !has_voted_for_approval(multisig_account, sequence_number, executor);

        let multisig_account_resource = borrow_global_mut<MultisigAccount>(multisig_account);
        let (num_approvals, _) = remove_executed_transaction(multisig_account_resource);

        if (features::multisig_v2_enhancement_feature_enabled() && implicit_approval) {
            if (std::features::module_event_migration_enabled()) {
                emit(
                    Vote {
                        multisig_account,
                        owner: executor,
                        sequence_number,
                        approved: true,
                    }
                );
            };
            num_approvals = num_approvals + 1;
            emit_event(
                &mut multisig_account_resource.vote_events,
                VoteEvent {
                    owner: executor,
                    sequence_number,
                    approved: true,
                }
            );
        };

        num_approvals
    }

    // Remove the next transaction in the queue as it's been executed and return the number of approvals it had.
    fun remove_executed_transaction(multisig_account_resource: &mut MultisigAccount): (u64, u64) {
        let sequence_number = multisig_account_resource.last_executed_sequence_number + 1;
        let transaction = table::remove(&mut multisig_account_resource.transactions, sequence_number);
        multisig_account_resource.last_executed_sequence_number = sequence_number;
        num_approvals_and_rejections_internal(&multisig_account_resource.owners, &transaction)
    }

    inline fun add_transaction(
        creator: address,
        multisig_account: address,
        transaction: MultisigTransaction
    ) {
        if (features::multisig_v2_enhancement_feature_enabled()) {
            assert!(
                available_transaction_queue_capacity(multisig_account) > 0,
                error::invalid_state(EMAX_PENDING_TRANSACTIONS_EXCEEDED)
            );
        };

        let multisig_account_resource = borrow_global_mut<MultisigAccount>(multisig_account);

        // The transaction creator also automatically votes for the transaction.
        simple_map::add(&mut transaction.votes, creator, true);

        let sequence_number = multisig_account_resource.next_sequence_number;
        multisig_account_resource.next_sequence_number = sequence_number + 1;
        table::add(&mut multisig_account_resource.transactions, sequence_number, transaction);
        if (std::features::module_event_migration_enabled()) {
            emit(
                CreateTransaction { multisig_account: multisig_account, creator, sequence_number, transaction }
            );
        };
        emit_event(
            &mut multisig_account_resource.create_transaction_events,
            CreateTransactionEvent { creator, sequence_number, transaction },
        );
    }

    fun create_multisig_account(owner: &signer): (signer, SignerCapability) {
        let owner_nonce = account::get_sequence_number(address_of(owner));
        let (multisig_signer, multisig_signer_cap) =
            account::create_resource_account(owner, create_multisig_account_seed(to_bytes(&owner_nonce)));
        // Register the account to receive APT as this is not done by default as part of the resource account creation
        // flow.
        if (!coin::is_account_registered<AptosCoin>(address_of(&multisig_signer))) {
            coin::register<AptosCoin>(&multisig_signer);
        };

        (multisig_signer, multisig_signer_cap)
    }

    fun create_multisig_account_seed(seed: vector<u8>): vector<u8> {
        // Generate a seed that will be used to create the resource account that hosts the multisig account.
        let multisig_account_seed = vector::empty<u8>();
        vector::append(&mut multisig_account_seed, DOMAIN_SEPARATOR);
        vector::append(&mut multisig_account_seed, seed);

        multisig_account_seed
    }

    fun validate_owners(owners: &vector<address>, multisig_account: address) {
        let distinct_owners: vector<address> = vector[];
        vector::for_each_ref(owners, |owner| {
            let owner = *owner;
            assert!(owner != multisig_account, error::invalid_argument(EOWNER_CANNOT_BE_MULTISIG_ACCOUNT_ITSELF));
            let (found, _) = vector::index_of(&distinct_owners, &owner);
            assert!(!found, error::invalid_argument(EDUPLICATE_OWNER));
            vector::push_back(&mut distinct_owners, owner);
        });
    }

    inline fun assert_is_owner_internal(owner: &signer, multisig_account: &MultisigAccount) {
        assert!(
            vector::contains(&multisig_account.owners, &address_of(owner)),
            error::permission_denied(ENOT_OWNER),
        );
    }

    inline fun assert_is_owner(owner: &signer, multisig_account: address) acquires MultisigAccount {
        let multisig_account_resource = borrow_global<MultisigAccount>(multisig_account);
        assert_is_owner_internal(owner, multisig_account_resource);
    }

    inline fun num_approvals_and_rejections_internal(owners: &vector<address>, transaction: &MultisigTransaction): (u64, u64) {
        let num_approvals = 0;
        let num_rejections = 0;

        let votes = &transaction.votes;
        vector::for_each_ref(owners, |owner| {
            if (simple_map::contains_key(votes, owner)) {
                if (*simple_map::borrow(votes, owner)) {
                    num_approvals = num_approvals + 1;
                } else {
                    num_rejections = num_rejections + 1;
                };
            }
        });

        (num_approvals, num_rejections)
    }

    inline fun num_approvals_and_rejections(multisig_account: address, sequence_number: u64): (u64, u64) acquires MultisigAccount {
        let multisig_account_resource = borrow_global<MultisigAccount>(multisig_account);
        let transaction = table::borrow(&multisig_account_resource.transactions, sequence_number);
        num_approvals_and_rejections_internal(&multisig_account_resource.owners, transaction)
    }

    inline fun has_voted_for_approval(multisig_account: address, sequence_number: u64, owner: address): bool acquires MultisigAccount {
        let (voted, vote) = vote(multisig_account, sequence_number, owner);
        voted && vote
    }

    inline fun has_voted_for_rejection(multisig_account: address, sequence_number: u64, owner: address): bool acquires MultisigAccount {
        let (voted, vote) = vote(multisig_account, sequence_number, owner);
        voted && !vote
    }

    inline fun assert_multisig_account_exists(multisig_account: address) {
        assert!(exists<MultisigAccount>(multisig_account), error::invalid_state(EACCOUNT_NOT_MULTISIG));
    }

    inline fun assert_valid_sequence_number(multisig_account: address, sequence_number: u64) acquires MultisigAccount {
        let multisig_account_resource = borrow_global<MultisigAccount>(multisig_account);
        assert!(
            sequence_number > 0 && sequence_number < multisig_account_resource.next_sequence_number,
            error::invalid_argument(EINVALID_SEQUENCE_NUMBER),
        );
    }

    inline fun assert_transaction_exists(multisig_account: address, sequence_number: u64) acquires MultisigAccount {
        let multisig_account_resource = borrow_global<MultisigAccount>(multisig_account);
        assert!(
            table::contains(&multisig_account_resource.transactions, sequence_number),
            error::not_found(ETRANSACTION_NOT_FOUND),
        );
    }

    /// Add new owners, remove owners to remove, update signatures required.
    fun update_owner_schema(
        multisig_address: address,
        new_owners: vector<address>,
        owners_to_remove: vector<address>,
        optional_new_num_signatures_required: Option<u64>,
    ) acquires MultisigAccount {
        assert_multisig_account_exists(multisig_address);
        let multisig_account_ref_mut =
            borrow_global_mut<MultisigAccount>(multisig_address);
        // Verify no overlap between new owners and owners to remove.
        vector::for_each_ref(&new_owners, |new_owner_ref| {
            assert!(
                !vector::contains(&owners_to_remove, new_owner_ref),
                error::invalid_argument(EOWNERS_TO_REMOVE_NEW_OWNERS_OVERLAP)
            )
        });
        // If new owners provided, try to add them and emit an event.
        if (vector::length(&new_owners) > 0) {
            vector::append(&mut multisig_account_ref_mut.owners, new_owners);
            validate_owners(
                &multisig_account_ref_mut.owners,
                multisig_address
            );
            if (std::features::module_event_migration_enabled()) {
                emit(AddOwners { multisig_account: multisig_address, owners_added: new_owners });
            };
            emit_event(
                &mut multisig_account_ref_mut.add_owners_events,
                AddOwnersEvent { owners_added: new_owners }
            );
        };
        // If owners to remove provided, try to remove them.
        if (vector::length(&owners_to_remove) > 0) {
            let owners_ref_mut = &mut multisig_account_ref_mut.owners;
            let owners_removed = vector[];
            vector::for_each_ref(&owners_to_remove, |owner_to_remove_ref| {
                let (found, index) =
                    vector::index_of(owners_ref_mut, owner_to_remove_ref);
                if (found) {
                    vector::push_back(
                        &mut owners_removed,
                        vector::swap_remove(owners_ref_mut, index)
                    );
                }
            });
            // Only emit event if owner(s) actually removed.
            if (vector::length(&owners_removed) > 0) {
                if (std::features::module_event_migration_enabled()) {
                    emit(
                        RemoveOwners { multisig_account: multisig_address, owners_removed }
                    );
                };
                emit_event(
                    &mut multisig_account_ref_mut.remove_owners_events,
                    RemoveOwnersEvent { owners_removed }
                );
            }
        };
        // If new signature count provided, try to update count.
        if (option::is_some(&optional_new_num_signatures_required)) {
            let new_num_signatures_required =
                option::extract(&mut optional_new_num_signatures_required);
            assert!(
                new_num_signatures_required > 0,
                error::invalid_argument(EINVALID_SIGNATURES_REQUIRED)
            );
            let old_num_signatures_required =
                multisig_account_ref_mut.num_signatures_required;
            // Only apply update and emit event if a change indicated.
            if (new_num_signatures_required != old_num_signatures_required) {
                multisig_account_ref_mut.num_signatures_required =
                    new_num_signatures_required;
                if (std::features::module_event_migration_enabled()) {
                    emit(
                        UpdateSignaturesRequired {
                            multisig_account: multisig_address,
                            old_num_signatures_required,
                            new_num_signatures_required,
                        }
                    );
                };
                emit_event(
                    &mut multisig_account_ref_mut.update_signature_required_events,
                    UpdateSignaturesRequiredEvent {
                        old_num_signatures_required,
                        new_num_signatures_required,
                    }
                );
            }
        };
        // Verify number of owners.
        let num_owners = vector::length(&multisig_account_ref_mut.owners);
        assert!(
            num_owners >= multisig_account_ref_mut.num_signatures_required,
            error::invalid_state(ENOT_ENOUGH_OWNERS)
        );
    }

    ////////////////////////// Tests ///////////////////////////////

    #[test_only]
    use aptos_framework::aptos_account::create_account;
    #[test_only]
    use aptos_framework::timestamp;
    #[test_only]
    use aptos_std::from_bcs;
    #[test_only]
    use aptos_std::multi_ed25519;
    #[test_only]
    use std::string::utf8;
    use std::features;
    #[test_only]
    use aptos_framework::aptos_coin;
    #[test_only]
    use aptos_framework::coin::{destroy_mint_cap, destroy_burn_cap};

    #[test_only]
    const PAYLOAD: vector<u8> = vector[1, 2, 3];
    #[test_only]
    const ERROR_TYPE: vector<u8> = b"MoveAbort";
    #[test_only]
    const ABORT_LOCATION: vector<u8> = b"abort_location";
    #[test_only]
    const ERROR_CODE: u64 = 10;

    #[test_only]
    fun execution_error(): ExecutionError {
        ExecutionError {
            abort_location: utf8(ABORT_LOCATION),
            error_type: utf8(ERROR_TYPE),
            error_code: ERROR_CODE,
        }
    }

    #[test_only]
    fun setup() {
        let framework_signer = &create_signer(@0x1);
        features::change_feature_flags_for_testing(
            framework_signer, vector[features::get_multisig_accounts_feature(), features::get_multisig_v2_enhancement_feature(), features::get_abort_if_multisig_payload_mismatch_feature()], vector[]);
        timestamp::set_time_has_started_for_testing(framework_signer);
        chain_id::initialize_for_test(framework_signer, 1);
        let (burn, mint) = aptos_coin::initialize_for_test(framework_signer);
        destroy_mint_cap(mint);
        destroy_burn_cap(burn);
    }

    #[test_only]
    fun setup_disabled() {
        let framework_signer = &create_signer(@0x1);
        features::change_feature_flags_for_testing(
            framework_signer, vector[], vector[features::get_multisig_accounts_feature()]);
        timestamp::set_time_has_started_for_testing(framework_signer);
        chain_id::initialize_for_test(framework_signer, 1);
        let (burn, mint) = aptos_coin::initialize_for_test(framework_signer);
        destroy_mint_cap(mint);
        destroy_burn_cap(burn);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_end_to_end(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 2, vector[], vector[]);

        // Create three transactions.
        create_transaction(owner_1, multisig_account, PAYLOAD);
        create_transaction(owner_2, multisig_account, PAYLOAD);
        create_transaction_with_hash(owner_3, multisig_account, sha3_256(PAYLOAD));
        assert!(get_pending_transactions(multisig_account) == vector[
            get_transaction(multisig_account, 1),
            get_transaction(multisig_account, 2),
            get_transaction(multisig_account, 3),
        ], 0);

        // Owner 3 doesn't need to explicitly approve as they created the transaction.
        approve_transaction(owner_1, multisig_account, 3);
        // Third transaction has 2 approvals but cannot be executed out-of-order.
        assert!(!can_be_executed(multisig_account, 3), 0);

        // Owner 1 doesn't need to explicitly approve as they created the transaction.
        approve_transaction(owner_2, multisig_account, 1);
        // First transaction has 2 approvals so it can be executed.
        assert!(can_be_executed(multisig_account, 1), 1);
        // First transaction was executed successfully.
        successful_transaction_execution_cleanup(owner_2_addr, multisig_account, vector[]);
        assert!(get_pending_transactions(multisig_account) == vector[
            get_transaction(multisig_account, 2),
            get_transaction(multisig_account, 3),
        ], 0);

        reject_transaction(owner_1, multisig_account, 2);
        reject_transaction(owner_3, multisig_account, 2);
        // Second transaction has 1 approval (owner 3) and 2 rejections (owners 1 & 2) and thus can be removed.
        assert!(can_be_rejected(multisig_account, 2), 2);
        execute_rejected_transaction(owner_1, multisig_account);
        assert!(get_pending_transactions(multisig_account) == vector[
            get_transaction(multisig_account, 3),
        ], 0);

        // Third transaction can be executed now but execution fails.
        failed_transaction_execution_cleanup(owner_3_addr, multisig_account, PAYLOAD, execution_error());
        assert!(get_pending_transactions(multisig_account) == vector[], 0);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_end_to_end_with_implicit_votes(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 2, vector[], vector[]);

        // Create three transactions.
        create_transaction(owner_1, multisig_account, PAYLOAD);
        create_transaction(owner_2, multisig_account, PAYLOAD);
        assert!(get_pending_transactions(multisig_account) == vector[
            get_transaction(multisig_account, 1),
            get_transaction(multisig_account, 2),
        ], 0);

        reject_transaction(owner_2, multisig_account, 1);
        // Owner 2 can execute the transaction, implicitly voting to approve it,
        // which overrides their previous vote for rejection.
        assert!(can_execute(owner_2_addr, multisig_account, 1), 1);
        // First transaction was executed successfully.
        successful_transaction_execution_cleanup(owner_2_addr, multisig_account,vector[]);
        assert!(get_pending_transactions(multisig_account) == vector[
            get_transaction(multisig_account, 2),
        ], 0);

        reject_transaction(owner_1, multisig_account, 2);
        // Owner 3 can execute-reject the transaction, implicitly voting to reject it.
        assert!(can_reject(owner_3_addr, multisig_account, 2), 2);
        execute_rejected_transaction(owner_3, multisig_account);
        assert!(get_pending_transactions(multisig_account) == vector[], 0);
    }

    #[test(owner = @0x123)]
    public entry fun test_create_with_single_owner(owner: &signer) acquires MultisigAccount {
        setup();
        let owner_addr = address_of(owner);
        create_account(owner_addr);
        create(owner, 1, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(owner_addr);
        assert_multisig_account_exists(multisig_account);
        assert!(owners(multisig_account) == vector[owner_addr], 0);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_create_with_as_many_sigs_required_as_num_owners(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        create_account(owner_1_addr);
        create_with_owners(owner_1, vector[address_of(owner_2), address_of(owner_3)], 3, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        assert_multisig_account_exists(multisig_account);
    }

    #[test(owner = @0x123)]
    #[expected_failure(abort_code = 0x1000B, location = Self)]
    public entry fun test_create_with_zero_signatures_required_should_fail(
        owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        create(owner, 0, vector[], vector[]);
    }

    #[test(owner = @0x123)]
    #[expected_failure(abort_code = 0x1000B, location = Self)]
    public entry fun test_create_with_too_many_signatures_required_should_fail(
        owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        create(owner, 2, vector[], vector[]);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    #[expected_failure(abort_code = 0x10001, location = Self)]
    public entry fun test_create_with_duplicate_owners_should_fail(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner_1));
        create_with_owners(
            owner_1,
            vector[
                // Duplicate owner 2 addresses.
                address_of(owner_2),
                address_of(owner_3),
                address_of(owner_2),
            ],
            2,
            vector[],
            vector[]);
    }

    #[test(owner = @0x123)]
    #[expected_failure(abort_code = 0xD000E, location = Self)]
    public entry fun test_create_with_without_feature_flag_enabled_should_fail(
        owner: &signer) acquires MultisigAccount {
        setup_disabled();
        create_account(address_of(owner));
        create(owner, 2, vector[], vector[]);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    #[expected_failure(abort_code = 0x10001, location = Self)]
    public entry fun test_create_with_creator_in_additional_owners_list_should_fail(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner_1));
        create_with_owners(owner_1, vector[
            // Duplicate owner 1 addresses.
            address_of(owner_1),
            address_of(owner_2),
            address_of(owner_3),
        ], 2,
            vector[],
            vector[],
        );
    }

    #[test]
    public entry fun test_create_multisig_account_on_top_of_existing_multi_ed25519_account()
    acquires MultisigAccount {
        setup();
        let (curr_sk, curr_pk) = multi_ed25519::generate_keys(2, 3);
        let pk_unvalidated = multi_ed25519::public_key_to_unvalidated(&curr_pk);
        let auth_key = multi_ed25519::unvalidated_public_key_to_authentication_key(&pk_unvalidated);
        let multisig_address = from_bcs::to_address(auth_key);
        create_account(multisig_address);

        let expected_owners = vector[@0x123, @0x124, @0x125];
        let proof = MultisigAccountCreationMessage {
            chain_id: chain_id::get(),
            account_address: multisig_address,
            sequence_number: account::get_sequence_number(multisig_address),
            owners: expected_owners,
            num_signatures_required: 2,
        };
        let signed_proof = multi_ed25519::sign_struct(&curr_sk, proof);
        create_with_existing_account(
            multisig_address,
            expected_owners,
            2,
            1, // MULTI_ED25519_SCHEME
            multi_ed25519::unvalidated_public_key_to_bytes(&pk_unvalidated),
            multi_ed25519::signature_to_bytes(&signed_proof),
            vector[],
            vector[],
        );
        assert_multisig_account_exists(multisig_address);
        assert!(owners(multisig_address) == expected_owners, 0);
    }

    #[test]
    public entry fun test_create_multisig_account_on_top_of_existing_multi_ed25519_account_and_revoke_auth_key()
    acquires MultisigAccount {
        setup();
        let (curr_sk, curr_pk) = multi_ed25519::generate_keys(2, 3);
        let pk_unvalidated = multi_ed25519::public_key_to_unvalidated(&curr_pk);
        let auth_key = multi_ed25519::unvalidated_public_key_to_authentication_key(&pk_unvalidated);
        let multisig_address = from_bcs::to_address(auth_key);
        create_account(multisig_address);

        // Create both a signer capability and rotation capability offers
        account::set_rotation_capability_offer(multisig_address, @0x123);
        account::set_signer_capability_offer(multisig_address, @0x123);

        let expected_owners = vector[@0x123, @0x124, @0x125];
        let proof = MultisigAccountCreationWithAuthKeyRevocationMessage {
            chain_id: chain_id::get(),
            account_address: multisig_address,
            sequence_number: account::get_sequence_number(multisig_address),
            owners: expected_owners,
            num_signatures_required: 2,
        };
        let signed_proof = multi_ed25519::sign_struct(&curr_sk, proof);
        create_with_existing_account_and_revoke_auth_key(
            multisig_address,
            expected_owners,
            2,
            1, // MULTI_ED25519_SCHEME
            multi_ed25519::unvalidated_public_key_to_bytes(&pk_unvalidated),
            multi_ed25519::signature_to_bytes(&signed_proof),
            vector[],
            vector[],
        );
        assert_multisig_account_exists(multisig_address);
        assert!(owners(multisig_address) == expected_owners, 0);
        assert!(account::get_authentication_key(multisig_address) == ZERO_AUTH_KEY, 1);
        // Verify that all capability offers have been wiped.
        assert!(!account::is_rotation_capability_offered(multisig_address), 2);
        assert!(!account::is_signer_capability_offered(multisig_address), 3);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_update_signatures_required(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        create_account(owner_1_addr);
        create_with_owners(owner_1, vector[address_of(owner_2), address_of(owner_3)], 1, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        assert!(num_signatures_required(multisig_account) == 1, 0);
        update_signatures_required(&create_signer(multisig_account), 2);
        assert!(num_signatures_required(multisig_account) == 2, 1);
        // As many signatures required as number of owners (3).
        update_signatures_required(&create_signer(multisig_account), 3);
        assert!(num_signatures_required(multisig_account) == 3, 2);
    }

    #[test(owner = @0x123)]
    public entry fun test_update_metadata(owner: &signer) acquires MultisigAccount {
        setup();
        let owner_addr = address_of(owner);
        create_account(owner_addr);
        create(owner, 1, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(owner_addr);
        update_metadata(
            &create_signer(multisig_account),
            vector[utf8(b"key1"), utf8(b"key2")],
            vector[vector[1], vector[2]],
        );
        let updated_metadata = metadata(multisig_account);
        assert!(simple_map::length(&updated_metadata) == 2, 0);
        assert!(simple_map::borrow(&updated_metadata, &utf8(b"key1")) == &vector[1], 0);
        assert!(simple_map::borrow(&updated_metadata, &utf8(b"key2")) == &vector[2], 0);
    }

    #[test(owner = @0x123)]
    #[expected_failure(abort_code = 0x1000B, location = Self)]
    public entry fun test_update_with_zero_signatures_required_should_fail(
        owner: & signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        create(owner, 1, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        update_signatures_required(&create_signer(multisig_account), 0);
    }

    #[test(owner = @0x123)]
    #[expected_failure(abort_code = 0x30005, location = Self)]
    public entry fun test_update_with_too_many_signatures_required_should_fail(
        owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        create(owner, 1, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        update_signatures_required(&create_signer(multisig_account), 2);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_add_owners(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner_1));
        create(owner_1, 1, vector[], vector[]);
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        let multisig_signer = &create_signer(multisig_account);
        assert!(owners(multisig_account) == vector[owner_1_addr], 0);
        // Adding an empty vector of new owners should be no-op.
        add_owners(multisig_signer, vector[]);
        assert!(owners(multisig_account) == vector[owner_1_addr], 1);
        add_owners(multisig_signer, vector[owner_2_addr, owner_3_addr]);
        assert!(owners(multisig_account) == vector[owner_1_addr, owner_2_addr, owner_3_addr], 2);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_remove_owners(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 1, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        let multisig_signer = &create_signer(multisig_account);
        assert!(owners(multisig_account) == vector[owner_2_addr, owner_3_addr, owner_1_addr], 0);
        // Removing an empty vector of owners should be no-op.
        remove_owners(multisig_signer, vector[]);
        assert!(owners(multisig_account) == vector[owner_2_addr, owner_3_addr, owner_1_addr], 1);
        remove_owners(multisig_signer, vector[owner_2_addr]);
        assert!(owners(multisig_account) == vector[owner_1_addr, owner_3_addr], 2);
        // Removing owners that don't exist should be no-op.
        remove_owners(multisig_signer, vector[@0x130]);
        assert!(owners(multisig_account) == vector[owner_1_addr, owner_3_addr], 3);
        // Removing with duplicate owners should still work.
        remove_owners(multisig_signer, vector[owner_3_addr, owner_3_addr, owner_3_addr]);
        assert!(owners(multisig_account) == vector[owner_1_addr], 4);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    #[expected_failure(abort_code = 0x30005, location = Self)]
    public entry fun test_remove_all_owners_should_fail(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 1, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        assert!(owners(multisig_account) == vector[owner_2_addr, owner_3_addr, owner_1_addr], 0);
        let multisig_signer = &create_signer(multisig_account);
        remove_owners(multisig_signer, vector[owner_1_addr, owner_2_addr, owner_3_addr]);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    #[expected_failure(abort_code = 0x30005, location = Self)]
    public entry fun test_remove_owners_with_fewer_remaining_than_signature_threshold_should_fail(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 2, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        let multisig_signer = &create_signer(multisig_account);
        // Remove 2 owners so there's one left, which is less than the signature threshold of 2.
        remove_owners(multisig_signer, vector[owner_2_addr, owner_3_addr]);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_create_transaction(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 2, vector[], vector[]);

        create_transaction(owner_1, multisig_account, PAYLOAD);
        let transaction = get_transaction(multisig_account, 1);
        assert!(transaction.creator == owner_1_addr, 0);
        assert!(option::is_some(&transaction.payload), 1);
        assert!(option::is_none(&transaction.payload_hash), 2);
        let payload = option::extract(&mut transaction.payload);
        assert!(payload == PAYLOAD, 4);
        // Automatic yes vote from creator.
        assert!(simple_map::length(&transaction.votes) == 1, 5);
        assert!(*simple_map::borrow(&transaction.votes, &owner_1_addr), 5);
    }

    #[test(owner = @0x123)]
    #[expected_failure(abort_code = 0x10004, location = Self)]
    public entry fun test_create_transaction_with_empty_payload_should_fail(
        owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        create(owner, 1, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create_transaction(owner, multisig_account, vector[]);
    }

    #[test(owner = @0x123, non_owner = @0x124)]
    #[expected_failure(abort_code = 0x507D3, location = Self)]
    public entry fun test_create_transaction_with_non_owner_should_fail(
        owner: &signer, non_owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        create(owner, 1, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create_transaction(non_owner, multisig_account, PAYLOAD);
    }

    #[test(owner = @0x123)]
    public entry fun test_create_transaction_with_hashes(
        owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        create(owner, 1, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create_transaction_with_hash(owner, multisig_account, sha3_256(PAYLOAD));
    }

    #[test(owner = @0x123)]
    #[expected_failure(abort_code = 0x1000C, location = Self)]
    public entry fun test_create_transaction_with_empty_hash_should_fail(
        owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        create(owner, 1, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create_transaction_with_hash(owner, multisig_account, vector[]);
    }

    #[test(owner = @0x123, non_owner = @0x124)]
    #[expected_failure(abort_code = 0x507D3, location = Self)]
    public entry fun test_create_transaction_with_hashes_and_non_owner_should_fail(
        owner: &signer, non_owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        create(owner, 1, vector[], vector[]);
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create_transaction_with_hash(non_owner, multisig_account, sha3_256(PAYLOAD));
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_approve_transaction(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 2, vector[], vector[]);

        create_transaction(owner_1, multisig_account, PAYLOAD);
        approve_transaction(owner_2, multisig_account, 1);
        approve_transaction(owner_3, multisig_account, 1);
        let transaction = get_transaction(multisig_account, 1);
        assert!(simple_map::length(&transaction.votes) == 3, 0);
        assert!(*simple_map::borrow(&transaction.votes, &owner_1_addr), 1);
        assert!(*simple_map::borrow(&transaction.votes, &owner_2_addr), 2);
        assert!(*simple_map::borrow(&transaction.votes, &owner_3_addr), 3);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_validate_transaction_should_not_consider_removed_owners(
        owner_1: &signer, owner_2: &signer, owner_3: & signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 2, vector[], vector[]);

        // Owner 1 and 2 approved but then owner 1 got removed.
        create_transaction(owner_1, multisig_account, PAYLOAD);
        approve_transaction(owner_2, multisig_account, 1);
        // Before owner 1 is removed, the transaction technically has sufficient approvals.
        assert!(can_be_executed(multisig_account, 1), 0);
        let multisig_signer = &create_signer(multisig_account);
        remove_owners(multisig_signer, vector[owner_1_addr]);
        // Now that owner 1 is removed, their approval should be invalidated and the transaction no longer
        // has enough approvals to be executed.
        assert!(!can_be_executed(multisig_account, 1), 1);
    }

    #[test(owner = @0x123)]
    #[expected_failure(abort_code = 0x607D6, location = Self)]
    public entry fun test_approve_transaction_with_invalid_sequence_number_should_fail(
        owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);
        // Transaction is created with id 1.
        create_transaction(owner, multisig_account, PAYLOAD);
        approve_transaction(owner, multisig_account, 2);
    }

    #[test(owner = @0x123, non_owner = @0x124)]
    #[expected_failure(abort_code = 0x507D3, location = Self)]
    public entry fun test_approve_transaction_with_non_owner_should_fail(
        owner: &signer, non_owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);
        // Transaction is created with id 1.
        create_transaction(owner, multisig_account, PAYLOAD);
        approve_transaction(non_owner, multisig_account, 1);
    }

    #[test(owner = @0x123)]
    public entry fun test_approval_transaction_after_rejecting(
        owner: &signer) acquires MultisigAccount {
        setup();
        let owner_addr = address_of(owner);
        create_account(owner_addr);
        let multisig_account = get_next_multisig_account_address(owner_addr);
        create(owner, 1, vector[], vector[]);

        create_transaction(owner, multisig_account, PAYLOAD);
        reject_transaction(owner, multisig_account, 1);
        approve_transaction(owner, multisig_account, 1);
        let transaction = get_transaction(multisig_account, 1);
        assert!(*simple_map::borrow(&transaction.votes, &owner_addr), 1);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_reject_transaction(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 2, vector[], vector[]);

        create_transaction(owner_1, multisig_account, PAYLOAD);
        reject_transaction(owner_1, multisig_account, 1);
        reject_transaction(owner_2, multisig_account, 1);
        reject_transaction(owner_3, multisig_account, 1);
        let transaction = get_transaction(multisig_account, 1);
        assert!(simple_map::length(&transaction.votes) == 3, 0);
        assert!(!*simple_map::borrow(&transaction.votes, &owner_1_addr), 1);
        assert!(!*simple_map::borrow(&transaction.votes, &owner_2_addr), 2);
        assert!(!*simple_map::borrow(&transaction.votes, &owner_3_addr), 3);
    }

    #[test(owner = @0x123)]
    public entry fun test_reject_transaction_after_approving(
        owner: &signer) acquires MultisigAccount {
        setup();
        let owner_addr = address_of(owner);
        create_account(owner_addr);
        let multisig_account = get_next_multisig_account_address(owner_addr);
        create(owner, 1, vector[], vector[]);

        create_transaction(owner, multisig_account, PAYLOAD);
        reject_transaction(owner, multisig_account, 1);
        let transaction = get_transaction(multisig_account, 1);
        assert!(!*simple_map::borrow(&transaction.votes, &owner_addr), 1);
    }

    #[test(owner = @0x123)]
    #[expected_failure(abort_code = 0x607D6, location = Self)]
    public entry fun test_reject_transaction_with_invalid_sequence_number_should_fail(
        owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);
        // Transaction is created with id 1.
        create_transaction(owner, multisig_account, PAYLOAD);
        reject_transaction(owner, multisig_account, 2);
    }

    #[test(owner = @0x123, non_owner = @0x124)]
    #[expected_failure(abort_code = 0x507D3, location = Self)]
    public entry fun test_reject_transaction_with_non_owner_should_fail(
        owner: &signer, non_owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);
        reject_transaction(non_owner, multisig_account, 1);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_execute_transaction_successful(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 2, vector[], vector[]);

        create_transaction(owner_1, multisig_account, PAYLOAD);
        // Owner 1 doesn't need to explicitly approve as they created the transaction.
        approve_transaction(owner_2, multisig_account, 1);
        assert!(can_be_executed(multisig_account, 1), 1);
        assert!(table::contains(&borrow_global<MultisigAccount>(multisig_account).transactions, 1), 0);
        successful_transaction_execution_cleanup(owner_3_addr, multisig_account, vector[]);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_execute_transaction_failed(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 2, vector[], vector[]);

        create_transaction(owner_1, multisig_account, PAYLOAD);
        // Owner 1 doesn't need to explicitly approve as they created the transaction.
        approve_transaction(owner_2, multisig_account, 1);
        assert!(can_be_executed(multisig_account, 1), 1);
        assert!(table::contains(&borrow_global<MultisigAccount>(multisig_account).transactions, 1), 0);
        failed_transaction_execution_cleanup(owner_3_addr, multisig_account, vector[], execution_error());
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_execute_transaction_with_full_payload(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 2, vector[], vector[]);

        create_transaction_with_hash(owner_3, multisig_account, sha3_256(PAYLOAD));
        // Owner 3 doesn't need to explicitly approve as they created the transaction.
        approve_transaction(owner_1, multisig_account, 1);
        assert!(can_be_executed(multisig_account, 1), 1);
        assert!(table::contains(&borrow_global<MultisigAccount>(multisig_account).transactions, 1), 0);
        successful_transaction_execution_cleanup(owner_3_addr, multisig_account, PAYLOAD);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    public entry fun test_execute_rejected_transaction(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 2, vector[], vector[]);

        create_transaction(owner_1, multisig_account, PAYLOAD);
        reject_transaction(owner_2, multisig_account, 1);
        reject_transaction(owner_3, multisig_account, 1);
        assert!(can_be_rejected(multisig_account, 1), 1);
        assert!(table::contains(&borrow_global<MultisigAccount>(multisig_account).transactions, 1), 0);
        execute_rejected_transaction(owner_3, multisig_account);
    }

    #[test(owner = @0x123, non_owner = @0x124)]
    #[expected_failure(abort_code = 0x507D3, location = Self)]
    public entry fun test_execute_rejected_transaction_with_non_owner_should_fail(
        owner: &signer, non_owner: &signer) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);

        create_transaction(owner, multisig_account, PAYLOAD);
        reject_transaction(owner, multisig_account, 1);
        execute_rejected_transaction(non_owner, multisig_account);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    #[expected_failure(abort_code = 0x3000A, location = Self)]
    public entry fun test_execute_rejected_transaction_without_sufficient_rejections_should_fail(
        owner_1: &signer, owner_2: &signer, owner_3: &signer) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_account = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(owner_1, vector[owner_2_addr, owner_3_addr], 2, vector[], vector[]);

        create_transaction(owner_1, multisig_account, PAYLOAD);
        approve_transaction(owner_2, multisig_account, 1);
        execute_rejected_transaction(owner_3, multisig_account);
    }

    #[test(
        owner_1 = @0x123,
        owner_2 = @0x124,
        owner_3 = @0x125
    )]
    #[expected_failure(abort_code = 0x10012, location = Self)]
    fun test_update_owner_schema_overlap_should_fail(
        owner_1: &signer,
        owner_2: &signer,
        owner_3: &signer
    ) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_address = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(
            owner_1,
            vector[owner_2_addr, owner_3_addr],
            2,
            vector[],
            vector[]
        );
        update_owner_schema(
            multisig_address,
            vector[owner_1_addr],
            vector[owner_1_addr],
            option::none()
        );
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    #[expected_failure(abort_code = 196627, location = Self)]
    fun test_max_pending_transaction_limit_should_fail(
        owner_1: &signer,
        owner_2: &signer,
        owner_3: &signer
    ) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_address = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(
            owner_1,
            vector[owner_2_addr, owner_3_addr],
            2,
            vector[],
            vector[]
        );

        let remaining_iterations = MAX_PENDING_TRANSACTIONS + 1;
        while (remaining_iterations > 0) {
            create_transaction(owner_1, multisig_address, PAYLOAD);
            remaining_iterations = remaining_iterations - 1;
        }
    }

    #[test_only]
    fun create_transaction_with_eviction(
        owner: &signer,
        multisig_account: address,
        payload: vector<u8>,
    ) acquires MultisigAccount {
        while(available_transaction_queue_capacity(multisig_account) == 0) {
            execute_rejected_transaction(owner, multisig_account)
        };
        create_transaction(owner, multisig_account, payload);
    }

    #[test_only]
    fun vote_all_transactions(
        owner: &signer, multisig_account: address, approved: bool) acquires MultisigAccount {
        let starting_sequence_number = last_resolved_sequence_number(multisig_account) + 1;
        let final_sequence_number = next_sequence_number(multisig_account) - 1;
        vote_transactions(owner, multisig_account, starting_sequence_number, final_sequence_number, approved);
    }

    #[test(owner_1 = @0x123, owner_2 = @0x124, owner_3 = @0x125)]
    fun test_dos_mitigation_end_to_end(
        owner_1: &signer,
        owner_2: &signer,
        owner_3: &signer
    ) acquires MultisigAccount {
        setup();
        let owner_1_addr = address_of(owner_1);
        let owner_2_addr = address_of(owner_2);
        let owner_3_addr = address_of(owner_3);
        create_account(owner_1_addr);
        let multisig_address = get_next_multisig_account_address(owner_1_addr);
        create_with_owners(
            owner_1,
            vector[owner_2_addr, owner_3_addr],
            2,
            vector[],
            vector[]
        );

        // owner_3 is compromised and creates a bunch of bogus transactions.
        let remaining_iterations = MAX_PENDING_TRANSACTIONS;
        while (remaining_iterations > 0) {
            create_transaction(owner_3, multisig_address, PAYLOAD);
            remaining_iterations = remaining_iterations - 1;
        };

        // No one can create a transaction anymore because the transaction queue is full.
        assert!(available_transaction_queue_capacity(multisig_address) == 0, 0);

        // owner_1 and owner_2 vote "no" on all transactions.
        vote_all_transactions(owner_1, multisig_address, false);
        vote_all_transactions(owner_2, multisig_address, false);

        // owner_1 evicts a transaction and creates a transaction to remove the compromised owner.
        // Note that `PAYLOAD` is a placeholder and is not actually executed in this unit test.
        create_transaction_with_eviction(owner_1, multisig_address, PAYLOAD);

        // owner_2 approves the eviction transaction.
        approve_transaction(owner_2, multisig_address, 11);

        // owner_1 flushes the transaction queue except for the eviction transaction.
        execute_rejected_transactions(owner_1, multisig_address, 10);

        // execute the eviction transaction to remove the compromised owner.
        assert!(can_be_executed(multisig_address, 11), 0);
    }

    #[test(owner = @0x123, non_owner = @0x234)]
    #[expected_failure(abort_code = 329683, location = Self)]
    public entry fun test_create_transaction_should_fail_if_not_owner(
        owner: &signer,
        non_owner: &signer
    ) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);
        // Transaction is created with id 1.
        create_transaction(non_owner, multisig_account, PAYLOAD);
    }

    #[test(owner = @0x123, non_owner = @0x234)]
    #[expected_failure(abort_code = 329683, location = Self)]
    public entry fun test_create_transaction_with_hash_should_fail_if_not_owner(
        owner: &signer,
        non_owner: &signer
    ) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);
        // Transaction is created with id 1.
        create_transaction_with_hash(non_owner, multisig_account, sha3_256(PAYLOAD));
    }

    #[test(owner = @0x123, non_owner = @0x234)]
    #[expected_failure(abort_code = 329683, location = Self)]
    public entry fun test_reject_transaction_should_fail_if_not_owner(
        owner: &signer,
        non_owner: &signer
    ) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);
        // Transaction is created with id 1.
        create_transaction(owner, multisig_account, PAYLOAD);
        reject_transaction(non_owner, multisig_account, 1);
    }


    #[test(owner = @0x123, non_owner = @0x234)]
    #[expected_failure(abort_code = 329683, location = Self)]
    public entry fun test_approve_transaction_should_fail_if_not_owner(
        owner: &signer,
        non_owner: &signer
    ) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);
        // Transaction is created with id 1.
        create_transaction(owner, multisig_account, PAYLOAD);
        approve_transaction(non_owner, multisig_account, 1);
    }

    #[test(owner = @0x123, non_owner = @0x234)]
    #[expected_failure(abort_code = 329683, location = Self)]
    public entry fun test_vote_transaction_should_fail_if_not_owner(
        owner: &signer,
        non_owner: &signer
    ) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);
        // Transaction is created with id 1.
        create_transaction(owner, multisig_account, PAYLOAD);
        vote_transaction(non_owner, multisig_account, 1, true);
    }

    #[test(owner = @0x123, non_owner = @0x234)]
    #[expected_failure(abort_code = 329683, location = Self)]
    public entry fun test_vote_transactions_should_fail_if_not_owner(
        owner: &signer,
        non_owner: &signer
    ) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);
        // Transaction is created with id 1.
        create_transaction(owner, multisig_account, PAYLOAD);
        vote_transactions(non_owner, multisig_account, 1, 1, true);
    }

    #[test(owner = @0x123, non_owner = @0x234)]
    #[expected_failure(abort_code = 329683, location = Self)]
    public entry fun test_execute_rejected_transaction_should_fail_if_not_owner(
        owner: &signer,
        non_owner: &signer
    ) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);
        // Transaction is created with id 1.
        create_transaction(owner, multisig_account, PAYLOAD);
        reject_transaction(owner, multisig_account, 1);
        execute_rejected_transaction(non_owner, multisig_account);
    }


    #[test(owner = @0x123, non_owner = @0x234)]
    #[expected_failure(abort_code = 329683, location = Self)]
    public entry fun test_execute_rejected_transactions_should_fail_if_not_owner(
        owner: &signer,
        non_owner: &signer
    ) acquires MultisigAccount {
        setup();
        create_account(address_of(owner));
        let multisig_account = get_next_multisig_account_address(address_of(owner));
        create(owner, 1, vector[], vector[]);
        // Transaction is created with id 1.
        create_transaction(owner, multisig_account, PAYLOAD);
        reject_transaction(owner, multisig_account, 1);
        execute_rejected_transactions(non_owner, multisig_account, 1);
    }
}
