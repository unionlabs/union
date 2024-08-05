///
/// Validator lifecycle:
/// 1. Prepare a validator node set up and call stake::initialize_validator
/// 2. Once ready to deposit stake (or have funds assigned by a staking service in exchange for ownership capability),
/// call stake::add_stake (or *_with_cap versions if called from the staking service)
/// 3. Call stake::join_validator_set (or _with_cap version) to join the active validator set. Changes are effective in
/// the next epoch.
/// 4. Validate and gain rewards. The stake will automatically be locked up for a fixed duration (set by governance) and
/// automatically renewed at expiration.
/// 5. At any point, if the validator operator wants to update the consensus key or network/fullnode addresses, they can
/// call stake::rotate_consensus_key and stake::update_network_and_fullnode_addresses. Similar to changes to stake, the
/// changes to consensus key/network/fullnode addresses are only effective in the next epoch.
/// 6. Validator can request to unlock their stake at any time. However, their stake will only become withdrawable when
/// their current lockup expires. This can be at most as long as the fixed lockup duration.
/// 7. After exiting, the validator can either explicitly leave the validator set by calling stake::leave_validator_set
/// or if their stake drops below the min required, they would get removed at the end of the epoch.
/// 8. Validator can always rejoin the validator set by going through steps 2-3 again.
/// 9. An owner can always switch operators by calling stake::set_operator.
/// 10. An owner can always switch designated voter by calling stake::set_designated_voter.
module aptos_framework::stake {
    use std::error;
    use std::features;
    use std::option::{Self, Option};
    use std::signer;
    use std::vector;
    use aptos_std::bls12381;
    use aptos_std::math64::min;
    use aptos_std::table::{Self, Table};
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::account;
    use aptos_framework::coin::{Self, Coin, MintCapability};
    use aptos_framework::event::{Self, EventHandle};
    use aptos_framework::timestamp;
    use aptos_framework::system_addresses;
    use aptos_framework::staking_config::{Self, StakingConfig, StakingRewardsConfig};
    use aptos_framework::chain_status;

    friend aptos_framework::block;
    friend aptos_framework::genesis;
    friend aptos_framework::reconfiguration;
    friend aptos_framework::reconfiguration_with_dkg;
    friend aptos_framework::transaction_fee;

    /// Validator Config not published.
    const EVALIDATOR_CONFIG: u64 = 1;
    /// Not enough stake to join validator set.
    const ESTAKE_TOO_LOW: u64 = 2;
    /// Too much stake to join validator set.
    const ESTAKE_TOO_HIGH: u64 = 3;
    /// Account is already a validator or pending validator.
    const EALREADY_ACTIVE_VALIDATOR: u64 = 4;
    /// Account is not a validator.
    const ENOT_VALIDATOR: u64 = 5;
    /// Can't remove last validator.
    const ELAST_VALIDATOR: u64 = 6;
    /// Total stake exceeds maximum allowed.
    const ESTAKE_EXCEEDS_MAX: u64 = 7;
    /// Account is already registered as a validator candidate.
    const EALREADY_REGISTERED: u64 = 8;
    /// Account does not have the right operator capability.
    const ENOT_OPERATOR: u64 = 9;
    /// Validators cannot join or leave post genesis on this test network.
    const ENO_POST_GENESIS_VALIDATOR_SET_CHANGE_ALLOWED: u64 = 10;
    /// Invalid consensus public key
    const EINVALID_PUBLIC_KEY: u64 = 11;
    /// Validator set exceeds the limit
    const EVALIDATOR_SET_TOO_LARGE: u64 = 12;
    /// Voting power increase has exceeded the limit for this current epoch.
    const EVOTING_POWER_INCREASE_EXCEEDS_LIMIT: u64 = 13;
    /// Stake pool does not exist at the provided pool address.
    const ESTAKE_POOL_DOES_NOT_EXIST: u64 = 14;
    /// Owner capability does not exist at the provided account.
    const EOWNER_CAP_NOT_FOUND: u64 = 15;
    /// An account cannot own more than one owner capability.
    const EOWNER_CAP_ALREADY_EXISTS: u64 = 16;
    /// Validator is not defined in the ACL of entities allowed to be validators
    const EINELIGIBLE_VALIDATOR: u64 = 17;
    /// Cannot update stake pool's lockup to earlier than current lockup.
    const EINVALID_LOCKUP: u64 = 18;
    /// Table to store collected transaction fees for each validator already exists.
    const EFEES_TABLE_ALREADY_EXISTS: u64 = 19;
    /// Validator set change temporarily disabled because of in-progress reconfiguration.
    const ERECONFIGURATION_IN_PROGRESS: u64 = 20;

    /// Validator status enum. We can switch to proper enum later once Move supports it.
    const VALIDATOR_STATUS_PENDING_ACTIVE: u64 = 1;
    const VALIDATOR_STATUS_ACTIVE: u64 = 2;
    const VALIDATOR_STATUS_PENDING_INACTIVE: u64 = 3;
    const VALIDATOR_STATUS_INACTIVE: u64 = 4;

    /// Limit the maximum size to u16::max, it's the current limit of the bitvec
    /// https://github.com/aptos-labs/aptos-core/blob/main/crates/aptos-bitvec/src/lib.rs#L20
    const MAX_VALIDATOR_SET_SIZE: u64 = 65536;

    /// Limit the maximum value of `rewards_rate` in order to avoid any arithmetic overflow.
    const MAX_REWARDS_RATE: u64 = 1000000;

    const MAX_U64: u128 = 18446744073709551615;

    /// Capability that represents ownership and can be used to control the validator and the associated stake pool.
    /// Having this be separate from the signer for the account that the validator resources are hosted at allows
    /// modules to have control over a validator.
    struct OwnerCapability has key, store {
        pool_address: address,
    }

    /// Each validator has a separate StakePool resource and can provide a stake.
    /// Changes in stake for an active validator:
    /// 1. If a validator calls add_stake, the newly added stake is moved to pending_active.
    /// 2. If validator calls unlock, their stake is moved to pending_inactive.
    /// 2. When the next epoch starts, any pending_inactive stake is moved to inactive and can be withdrawn.
    ///    Any pending_active stake is moved to active and adds to the validator's voting power.
    ///
    /// Changes in stake for an inactive validator:
    /// 1. If a validator calls add_stake, the newly added stake is moved directly to active.
    /// 2. If validator calls unlock, their stake is moved directly to inactive.
    /// 3. When the next epoch starts, the validator can be activated if their active stake is more than the minimum.
    struct StakePool has key {
        // active stake
        active: Coin<AptosCoin>,
        // inactive stake, can be withdrawn
        inactive: Coin<AptosCoin>,
        // pending activation for next epoch
        pending_active: Coin<AptosCoin>,
        // pending deactivation for next epoch
        pending_inactive: Coin<AptosCoin>,
        locked_until_secs: u64,
        // Track the current operator of the validator node.
        // This allows the operator to be different from the original account and allow for separation of
        // the validator operations and ownership.
        // Only the account holding OwnerCapability of the staking pool can update this.
        operator_address: address,

        // Track the current vote delegator of the staking pool.
        // Only the account holding OwnerCapability of the staking pool can update this.
        delegated_voter: address,

        // The events emitted for the entire StakePool's lifecycle.
        initialize_validator_events: EventHandle<RegisterValidatorCandidateEvent>,
        set_operator_events: EventHandle<SetOperatorEvent>,
        add_stake_events: EventHandle<AddStakeEvent>,
        reactivate_stake_events: EventHandle<ReactivateStakeEvent>,
        rotate_consensus_key_events: EventHandle<RotateConsensusKeyEvent>,
        update_network_and_fullnode_addresses_events: EventHandle<UpdateNetworkAndFullnodeAddressesEvent>,
        increase_lockup_events: EventHandle<IncreaseLockupEvent>,
        join_validator_set_events: EventHandle<JoinValidatorSetEvent>,
        distribute_rewards_events: EventHandle<DistributeRewardsEvent>,
        unlock_stake_events: EventHandle<UnlockStakeEvent>,
        withdraw_stake_events: EventHandle<WithdrawStakeEvent>,
        leave_validator_set_events: EventHandle<LeaveValidatorSetEvent>,
    }

    /// Validator info stored in validator address.
    struct ValidatorConfig has key, copy, store, drop {
        consensus_pubkey: vector<u8>,
        network_addresses: vector<u8>,
        // to make it compatible with previous definition, remove later
        fullnode_addresses: vector<u8>,
        // Index in the active set if the validator corresponding to this stake pool is active.
        validator_index: u64,
    }

    /// Consensus information per validator, stored in ValidatorSet.
    struct ValidatorInfo has copy, store, drop {
        addr: address,
        voting_power: u64,
        config: ValidatorConfig,
    }

    /// Full ValidatorSet, stored in @aptos_framework.
    /// 1. join_validator_set adds to pending_active queue.
    /// 2. leave_valdiator_set moves from active to pending_inactive queue.
    /// 3. on_new_epoch processes two pending queues and refresh ValidatorInfo from the owner's address.
    struct ValidatorSet has copy, key, drop, store {
        consensus_scheme: u8,
        // Active validators for the current epoch.
        active_validators: vector<ValidatorInfo>,
        // Pending validators to leave in next epoch (still active).
        pending_inactive: vector<ValidatorInfo>,
        // Pending validators to join in next epoch.
        pending_active: vector<ValidatorInfo>,
        // Current total voting power.
        total_voting_power: u128,
        // Total voting power waiting to join in the next epoch.
        total_joining_power: u128,
    }

    /// AptosCoin capabilities, set during genesis and stored in @CoreResource account.
    /// This allows the Stake module to mint rewards to stakers.
    struct AptosCoinCapabilities has key {
        mint_cap: MintCapability<AptosCoin>,
    }

    struct IndividualValidatorPerformance has store, drop {
        successful_proposals: u64,
        failed_proposals: u64,
    }

    struct ValidatorPerformance has key {
        validators: vector<IndividualValidatorPerformance>,
    }

    struct RegisterValidatorCandidateEvent has drop, store {
        pool_address: address,
    }

    #[event]
    struct RegisterValidatorCandidate has drop, store {
        pool_address: address,
    }

    struct SetOperatorEvent has drop, store {
        pool_address: address,
        old_operator: address,
        new_operator: address,
    }

    #[event]
    struct SetOperator has drop, store {
        pool_address: address,
        old_operator: address,
        new_operator: address,
    }

    struct AddStakeEvent has drop, store {
        pool_address: address,
        amount_added: u64,
    }

    #[event]
    struct AddStake has drop, store {
        pool_address: address,
        amount_added: u64,
    }

    struct ReactivateStakeEvent has drop, store {
        pool_address: address,
        amount: u64,
    }

    #[event]
    struct ReactivateStake has drop, store {
        pool_address: address,
        amount: u64,
    }

    struct RotateConsensusKeyEvent has drop, store {
        pool_address: address,
        old_consensus_pubkey: vector<u8>,
        new_consensus_pubkey: vector<u8>,
    }

    #[event]
    struct RotateConsensusKey has drop, store {
        pool_address: address,
        old_consensus_pubkey: vector<u8>,
        new_consensus_pubkey: vector<u8>,
    }

    struct UpdateNetworkAndFullnodeAddressesEvent has drop, store {
        pool_address: address,
        old_network_addresses: vector<u8>,
        new_network_addresses: vector<u8>,
        old_fullnode_addresses: vector<u8>,
        new_fullnode_addresses: vector<u8>,
    }

    #[event]
    struct UpdateNetworkAndFullnodeAddresses has drop, store {
        pool_address: address,
        old_network_addresses: vector<u8>,
        new_network_addresses: vector<u8>,
        old_fullnode_addresses: vector<u8>,
        new_fullnode_addresses: vector<u8>,
    }

    struct IncreaseLockupEvent has drop, store {
        pool_address: address,
        old_locked_until_secs: u64,
        new_locked_until_secs: u64,
    }

    #[event]
    struct IncreaseLockup has drop, store {
        pool_address: address,
        old_locked_until_secs: u64,
        new_locked_until_secs: u64,
    }

    struct JoinValidatorSetEvent has drop, store {
        pool_address: address,
    }

    #[event]
    struct JoinValidatorSet has drop, store {
        pool_address: address,
    }

    struct DistributeRewardsEvent has drop, store {
        pool_address: address,
        rewards_amount: u64,
    }

    #[event]
    struct DistributeRewards has drop, store {
        pool_address: address,
        rewards_amount: u64,
    }

    struct UnlockStakeEvent has drop, store {
        pool_address: address,
        amount_unlocked: u64,
    }

    #[event]
    struct UnlockStake has drop, store {
        pool_address: address,
        amount_unlocked: u64,
    }

    struct WithdrawStakeEvent has drop, store {
        pool_address: address,
        amount_withdrawn: u64,
    }

    #[event]
    struct WithdrawStake has drop, store {
        pool_address: address,
        amount_withdrawn: u64,
    }

    struct LeaveValidatorSetEvent has drop, store {
        pool_address: address,
    }

    #[event]
    struct LeaveValidatorSet has drop, store {
        pool_address: address,
    }

    /// Stores transaction fees assigned to validators. All fees are distributed to validators
    /// at the end of the epoch.
    struct ValidatorFees has key {
        fees_table: Table<address, Coin<AptosCoin>>,
    }

    /// Initializes the resource storing information about collected transaction fees per validator.
    /// Used by `transaction_fee.move` to initialize fee collection and distribution.
    public(friend) fun initialize_validator_fees(aptos_framework: &signer) {
        system_addresses::assert_aptos_framework(aptos_framework);
        assert!(
            !exists<ValidatorFees>(@aptos_framework),
            error::already_exists(EFEES_TABLE_ALREADY_EXISTS)
        );
        move_to(aptos_framework, ValidatorFees { fees_table: table::new() });
    }

    /// Stores the transaction fee collected to the specified validator address.
    public(friend) fun add_transaction_fee(validator_addr: address, fee: Coin<AptosCoin>) acquires ValidatorFees {
        let fees_table = &mut borrow_global_mut<ValidatorFees>(@aptos_framework).fees_table;
        if (table::contains(fees_table, validator_addr)) {
            let collected_fee = table::borrow_mut(fees_table, validator_addr);
            coin::merge(collected_fee, fee);
        } else {
            table::add(fees_table, validator_addr, fee);
        }
    }

    #[view]
    /// Return the lockup expiration of the stake pool at `pool_address`.
    /// This will throw an error if there's no stake pool at `pool_address`.
    public fun get_lockup_secs(pool_address: address): u64 acquires StakePool {
        assert_stake_pool_exists(pool_address);
        borrow_global<StakePool>(pool_address).locked_until_secs
    }

    #[view]
    /// Return the remaining lockup of the stake pool at `pool_address`.
    /// This will throw an error if there's no stake pool at `pool_address`.
    public fun get_remaining_lockup_secs(pool_address: address): u64 acquires StakePool {
        assert_stake_pool_exists(pool_address);
        let lockup_time = borrow_global<StakePool>(pool_address).locked_until_secs;
        if (lockup_time <= timestamp::now_seconds()) {
            0
        } else {
            lockup_time - timestamp::now_seconds()
        }
    }

    #[view]
    /// Return the different stake amounts for `pool_address` (whether the validator is active or not).
    /// The returned amounts are for (active, inactive, pending_active, pending_inactive) stake respectively.
    public fun get_stake(pool_address: address): (u64, u64, u64, u64) acquires StakePool {
        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global<StakePool>(pool_address);
        (
            coin::value(&stake_pool.active),
            coin::value(&stake_pool.inactive),
            coin::value(&stake_pool.pending_active),
            coin::value(&stake_pool.pending_inactive),
        )
    }

    #[view]
    /// Returns the validator's state.
    public fun get_validator_state(pool_address: address): u64 acquires ValidatorSet {
        let validator_set = borrow_global<ValidatorSet>(@aptos_framework);
        if (option::is_some(&find_validator(&validator_set.pending_active, pool_address))) {
            VALIDATOR_STATUS_PENDING_ACTIVE
        } else if (option::is_some(&find_validator(&validator_set.active_validators, pool_address))) {
            VALIDATOR_STATUS_ACTIVE
        } else if (option::is_some(&find_validator(&validator_set.pending_inactive, pool_address))) {
            VALIDATOR_STATUS_PENDING_INACTIVE
        } else {
            VALIDATOR_STATUS_INACTIVE
        }
    }

    #[view]
    /// Return the voting power of the validator in the current epoch.
    /// This is the same as the validator's total active and pending_inactive stake.
    public fun get_current_epoch_voting_power(pool_address: address): u64 acquires StakePool, ValidatorSet {
        assert_stake_pool_exists(pool_address);
        let validator_state = get_validator_state(pool_address);
        // Both active and pending inactive validators can still vote in the current epoch.
        if (validator_state == VALIDATOR_STATUS_ACTIVE || validator_state == VALIDATOR_STATUS_PENDING_INACTIVE) {
            let active_stake = coin::value(&borrow_global<StakePool>(pool_address).active);
            let pending_inactive_stake = coin::value(&borrow_global<StakePool>(pool_address).pending_inactive);
            active_stake + pending_inactive_stake
        } else {
            0
        }
    }

    #[view]
    /// Return the delegated voter of the validator at `pool_address`.
    public fun get_delegated_voter(pool_address: address): address acquires StakePool {
        assert_stake_pool_exists(pool_address);
        borrow_global<StakePool>(pool_address).delegated_voter
    }

    #[view]
    /// Return the operator of the validator at `pool_address`.
    public fun get_operator(pool_address: address): address acquires StakePool {
        assert_stake_pool_exists(pool_address);
        borrow_global<StakePool>(pool_address).operator_address
    }

    /// Return the pool address in `owner_cap`.
    public fun get_owned_pool_address(owner_cap: &OwnerCapability): address {
        owner_cap.pool_address
    }

    #[view]
    /// Return the validator index for `pool_address`.
    public fun get_validator_index(pool_address: address): u64 acquires ValidatorConfig {
        assert_stake_pool_exists(pool_address);
        borrow_global<ValidatorConfig>(pool_address).validator_index
    }

    #[view]
    /// Return the number of successful and failed proposals for the proposal at the given validator index.
    public fun get_current_epoch_proposal_counts(validator_index: u64): (u64, u64) acquires ValidatorPerformance {
        let validator_performances = &borrow_global<ValidatorPerformance>(@aptos_framework).validators;
        let validator_performance = vector::borrow(validator_performances, validator_index);
        (validator_performance.successful_proposals, validator_performance.failed_proposals)
    }

    #[view]
    /// Return the validator's config.
    public fun get_validator_config(
        pool_address: address
    ): (vector<u8>, vector<u8>, vector<u8>) acquires ValidatorConfig {
        assert_stake_pool_exists(pool_address);
        let validator_config = borrow_global<ValidatorConfig>(pool_address);
        (validator_config.consensus_pubkey, validator_config.network_addresses, validator_config.fullnode_addresses)
    }

    #[view]
    public fun stake_pool_exists(addr: address): bool {
        exists<StakePool>(addr)
    }

    /// Initialize validator set to the core resource account.
    public(friend) fun initialize(aptos_framework: &signer) {
        system_addresses::assert_aptos_framework(aptos_framework);

        move_to(aptos_framework, ValidatorSet {
            consensus_scheme: 0,
            active_validators: vector::empty(),
            pending_active: vector::empty(),
            pending_inactive: vector::empty(),
            total_voting_power: 0,
            total_joining_power: 0,
        });

        move_to(aptos_framework, ValidatorPerformance {
            validators: vector::empty(),
        });
    }

    /// This is only called during Genesis, which is where MintCapability<AptosCoin> can be created.
    /// Beyond genesis, no one can create AptosCoin mint/burn capabilities.
    public(friend) fun store_aptos_coin_mint_cap(aptos_framework: &signer, mint_cap: MintCapability<AptosCoin>) {
        system_addresses::assert_aptos_framework(aptos_framework);
        move_to(aptos_framework, AptosCoinCapabilities { mint_cap })
    }

    /// Allow on chain governance to remove validators from the validator set.
    public fun remove_validators(
        aptos_framework: &signer,
        validators: &vector<address>,
    ) acquires ValidatorSet {
        assert_reconfig_not_in_progress();
        system_addresses::assert_aptos_framework(aptos_framework);
        let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
        let active_validators = &mut validator_set.active_validators;
        let pending_inactive = &mut validator_set.pending_inactive;
        spec {
            update ghost_active_num = len(active_validators);
            update ghost_pending_inactive_num = len(pending_inactive);
        };
        let len_validators = vector::length(validators);
        let i = 0;
        // Remove each validator from the validator set.
        while ({
            spec {
                invariant i <= len_validators;
                invariant spec_validators_are_initialized(active_validators);
                invariant spec_validator_indices_are_valid(active_validators);
                invariant spec_validators_are_initialized(pending_inactive);
                invariant spec_validator_indices_are_valid(pending_inactive);
                invariant ghost_active_num + ghost_pending_inactive_num == len(active_validators) + len(pending_inactive);
            };
            i < len_validators
        }) {
            let validator = *vector::borrow(validators, i);
            let validator_index = find_validator(active_validators, validator);
            if (option::is_some(&validator_index)) {
                let validator_info = vector::swap_remove(active_validators, *option::borrow(&validator_index));
                vector::push_back(pending_inactive, validator_info);
                spec {
                    update ghost_active_num = ghost_active_num - 1;
                    update ghost_pending_inactive_num = ghost_pending_inactive_num + 1;
                };
            };
            i = i + 1;
        };
    }

    /// Initialize the validator account and give ownership to the signing account
    /// except it leaves the ValidatorConfig to be set by another entity.
    /// Note: this triggers setting the operator and owner, set it to the account's address
    /// to set later.
    public entry fun initialize_stake_owner(
        owner: &signer,
        initial_stake_amount: u64,
        operator: address,
        voter: address,
    ) acquires AllowedValidators, OwnerCapability, StakePool, ValidatorSet {
        initialize_owner(owner);
        move_to(owner, ValidatorConfig {
            consensus_pubkey: vector::empty(),
            network_addresses: vector::empty(),
            fullnode_addresses: vector::empty(),
            validator_index: 0,
        });

        if (initial_stake_amount > 0) {
            add_stake(owner, initial_stake_amount);
        };

        let account_address = signer::address_of(owner);
        if (account_address != operator) {
            set_operator(owner, operator)
        };
        if (account_address != voter) {
            set_delegated_voter(owner, voter)
        };
    }

    /// Initialize the validator account and give ownership to the signing account.
    public entry fun initialize_validator(
        account: &signer,
        consensus_pubkey: vector<u8>,
        proof_of_possession: vector<u8>,
        network_addresses: vector<u8>,
        fullnode_addresses: vector<u8>,
    ) acquires AllowedValidators {
        // Checks the public key has a valid proof-of-possession to prevent rogue-key attacks.
        let pubkey_from_pop = &mut bls12381::public_key_from_bytes_with_pop(
            consensus_pubkey,
            &proof_of_possession_from_bytes(proof_of_possession)
        );
        assert!(option::is_some(pubkey_from_pop), error::invalid_argument(EINVALID_PUBLIC_KEY));

        initialize_owner(account);
        move_to(account, ValidatorConfig {
            consensus_pubkey,
            network_addresses,
            fullnode_addresses,
            validator_index: 0,
        });
    }

    fun initialize_owner(owner: &signer) acquires AllowedValidators {
        let owner_address = signer::address_of(owner);
        assert!(is_allowed(owner_address), error::not_found(EINELIGIBLE_VALIDATOR));
        assert!(!stake_pool_exists(owner_address), error::already_exists(EALREADY_REGISTERED));

        move_to(owner, StakePool {
            active: coin::zero<AptosCoin>(),
            pending_active: coin::zero<AptosCoin>(),
            pending_inactive: coin::zero<AptosCoin>(),
            inactive: coin::zero<AptosCoin>(),
            locked_until_secs: 0,
            operator_address: owner_address,
            delegated_voter: owner_address,
            // Events.
            initialize_validator_events: account::new_event_handle<RegisterValidatorCandidateEvent>(owner),
            set_operator_events: account::new_event_handle<SetOperatorEvent>(owner),
            add_stake_events: account::new_event_handle<AddStakeEvent>(owner),
            reactivate_stake_events: account::new_event_handle<ReactivateStakeEvent>(owner),
            rotate_consensus_key_events: account::new_event_handle<RotateConsensusKeyEvent>(owner),
            update_network_and_fullnode_addresses_events: account::new_event_handle<UpdateNetworkAndFullnodeAddressesEvent>(
                owner
            ),
            increase_lockup_events: account::new_event_handle<IncreaseLockupEvent>(owner),
            join_validator_set_events: account::new_event_handle<JoinValidatorSetEvent>(owner),
            distribute_rewards_events: account::new_event_handle<DistributeRewardsEvent>(owner),
            unlock_stake_events: account::new_event_handle<UnlockStakeEvent>(owner),
            withdraw_stake_events: account::new_event_handle<WithdrawStakeEvent>(owner),
            leave_validator_set_events: account::new_event_handle<LeaveValidatorSetEvent>(owner),
        });

        move_to(owner, OwnerCapability { pool_address: owner_address });
    }

    /// Extract and return owner capability from the signing account.
    public fun extract_owner_cap(owner: &signer): OwnerCapability acquires OwnerCapability {
        let owner_address = signer::address_of(owner);
        assert_owner_cap_exists(owner_address);
        move_from<OwnerCapability>(owner_address)
    }

    /// Deposit `owner_cap` into `account`. This requires `account` to not already have ownership of another
    /// staking pool.
    public fun deposit_owner_cap(owner: &signer, owner_cap: OwnerCapability) {
        assert!(!exists<OwnerCapability>(signer::address_of(owner)), error::not_found(EOWNER_CAP_ALREADY_EXISTS));
        move_to(owner, owner_cap);
    }

    /// Destroy `owner_cap`.
    public fun destroy_owner_cap(owner_cap: OwnerCapability) {
        let OwnerCapability { pool_address: _ } = owner_cap;
    }

    /// Allows an owner to change the operator of the stake pool.
    public entry fun set_operator(owner: &signer, new_operator: address) acquires OwnerCapability, StakePool {
        let owner_address = signer::address_of(owner);
        assert_owner_cap_exists(owner_address);
        let ownership_cap = borrow_global<OwnerCapability>(owner_address);
        set_operator_with_cap(ownership_cap, new_operator);
    }

    /// Allows an account with ownership capability to change the operator of the stake pool.
    public fun set_operator_with_cap(owner_cap: &OwnerCapability, new_operator: address) acquires StakePool {
        let pool_address = owner_cap.pool_address;
        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        let old_operator = stake_pool.operator_address;
        stake_pool.operator_address = new_operator;

        if (std::features::module_event_migration_enabled()) {
            event::emit(
                SetOperator {
                    pool_address,
                    old_operator,
                    new_operator,
                },
            );
        };

        event::emit_event(
            &mut stake_pool.set_operator_events,
            SetOperatorEvent {
                pool_address,
                old_operator,
                new_operator,
            },
        );
    }

    /// Allows an owner to change the delegated voter of the stake pool.
    public entry fun set_delegated_voter(owner: &signer, new_voter: address) acquires OwnerCapability, StakePool {
        let owner_address = signer::address_of(owner);
        assert_owner_cap_exists(owner_address);
        let ownership_cap = borrow_global<OwnerCapability>(owner_address);
        set_delegated_voter_with_cap(ownership_cap, new_voter);
    }

    /// Allows an owner to change the delegated voter of the stake pool.
    public fun set_delegated_voter_with_cap(owner_cap: &OwnerCapability, new_voter: address) acquires StakePool {
        let pool_address = owner_cap.pool_address;
        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        stake_pool.delegated_voter = new_voter;
    }

    /// Add `amount` of coins from the `account` owning the StakePool.
    public entry fun add_stake(owner: &signer, amount: u64) acquires OwnerCapability, StakePool, ValidatorSet {
        let owner_address = signer::address_of(owner);
        assert_owner_cap_exists(owner_address);
        let ownership_cap = borrow_global<OwnerCapability>(owner_address);
        add_stake_with_cap(ownership_cap, coin::withdraw<AptosCoin>(owner, amount));
    }

    /// Add `coins` into `pool_address`. this requires the corresponding `owner_cap` to be passed in.
    public fun add_stake_with_cap(owner_cap: &OwnerCapability, coins: Coin<AptosCoin>) acquires StakePool, ValidatorSet {
        assert_reconfig_not_in_progress();
        let pool_address = owner_cap.pool_address;
        assert_stake_pool_exists(pool_address);

        let amount = coin::value(&coins);
        if (amount == 0) {
            coin::destroy_zero(coins);
            return
        };

        // Only track and validate voting power increase for active and pending_active validator.
        // Pending_inactive validator will be removed from the validator set in the next epoch.
        // Inactive validator's total stake will be tracked when they join the validator set.
        let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
        // Search directly rather using get_validator_state to save on unnecessary loops.
        if (option::is_some(&find_validator(&validator_set.active_validators, pool_address)) ||
            option::is_some(&find_validator(&validator_set.pending_active, pool_address))) {
            update_voting_power_increase(amount);
        };

        // Add to pending_active if it's a current validator because the stake is not counted until the next epoch.
        // Otherwise, the delegation can be added to active directly as the validator is also activated in the epoch.
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        if (is_current_epoch_validator(pool_address)) {
            coin::merge<AptosCoin>(&mut stake_pool.pending_active, coins);
        } else {
            coin::merge<AptosCoin>(&mut stake_pool.active, coins);
        };

        let (_, maximum_stake) = staking_config::get_required_stake(&staking_config::get());
        let voting_power = get_next_epoch_voting_power(stake_pool);
        assert!(voting_power <= maximum_stake, error::invalid_argument(ESTAKE_EXCEEDS_MAX));

        if (std::features::module_event_migration_enabled()) {
            event::emit(
                AddStake {
                    pool_address,
                    amount_added: amount,
                },
            );
        };
        event::emit_event(
            &mut stake_pool.add_stake_events,
            AddStakeEvent {
                pool_address,
                amount_added: amount,
            },
        );
    }

    /// Move `amount` of coins from pending_inactive to active.
    public entry fun reactivate_stake(owner: &signer, amount: u64) acquires OwnerCapability, StakePool {
        assert_reconfig_not_in_progress();
        let owner_address = signer::address_of(owner);
        assert_owner_cap_exists(owner_address);
        let ownership_cap = borrow_global<OwnerCapability>(owner_address);
        reactivate_stake_with_cap(ownership_cap, amount);
    }

    public fun reactivate_stake_with_cap(owner_cap: &OwnerCapability, amount: u64) acquires StakePool {
        assert_reconfig_not_in_progress();
        let pool_address = owner_cap.pool_address;
        assert_stake_pool_exists(pool_address);

        // Cap the amount to reactivate by the amount in pending_inactive.
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        let total_pending_inactive = coin::value(&stake_pool.pending_inactive);
        amount = min(amount, total_pending_inactive);

        // Since this does not count as a voting power change (pending inactive still counts as voting power in the
        // current epoch), stake can be immediately moved from pending inactive to active.
        // We also don't need to check voting power increase as there's none.
        let reactivated_coins = coin::extract(&mut stake_pool.pending_inactive, amount);
        coin::merge(&mut stake_pool.active, reactivated_coins);

        if (std::features::module_event_migration_enabled()) {
            event::emit(
                ReactivateStake {
                    pool_address,
                    amount,
                },
            );
        };
        event::emit_event(
            &mut stake_pool.reactivate_stake_events,
            ReactivateStakeEvent {
                pool_address,
                amount,
            },
        );
    }

    /// Rotate the consensus key of the validator, it'll take effect in next epoch.
    public entry fun rotate_consensus_key(
        operator: &signer,
        pool_address: address,
        new_consensus_pubkey: vector<u8>,
        proof_of_possession: vector<u8>,
    ) acquires StakePool, ValidatorConfig {
        assert_reconfig_not_in_progress();
        assert_stake_pool_exists(pool_address);

        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        assert!(signer::address_of(operator) == stake_pool.operator_address, error::unauthenticated(ENOT_OPERATOR));

        assert!(exists<ValidatorConfig>(pool_address), error::not_found(EVALIDATOR_CONFIG));
        let validator_info = borrow_global_mut<ValidatorConfig>(pool_address);
        let old_consensus_pubkey = validator_info.consensus_pubkey;
        // Checks the public key has a valid proof-of-possession to prevent rogue-key attacks.
        let pubkey_from_pop = &mut bls12381::public_key_from_bytes_with_pop(
            new_consensus_pubkey,
            &proof_of_possession_from_bytes(proof_of_possession)
        );
        assert!(option::is_some(pubkey_from_pop), error::invalid_argument(EINVALID_PUBLIC_KEY));
        validator_info.consensus_pubkey = new_consensus_pubkey;

        if (std::features::module_event_migration_enabled()) {
            event::emit(
                RotateConsensusKey {
                    pool_address,
                    old_consensus_pubkey,
                    new_consensus_pubkey,
                },
            );
        };
        event::emit_event(
            &mut stake_pool.rotate_consensus_key_events,
            RotateConsensusKeyEvent {
                pool_address,
                old_consensus_pubkey,
                new_consensus_pubkey,
            },
        );
    }

    /// Update the network and full node addresses of the validator. This only takes effect in the next epoch.
    public entry fun update_network_and_fullnode_addresses(
        operator: &signer,
        pool_address: address,
        new_network_addresses: vector<u8>,
        new_fullnode_addresses: vector<u8>,
    ) acquires StakePool, ValidatorConfig {
        assert_reconfig_not_in_progress();
        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        assert!(signer::address_of(operator) == stake_pool.operator_address, error::unauthenticated(ENOT_OPERATOR));
        assert!(exists<ValidatorConfig>(pool_address), error::not_found(EVALIDATOR_CONFIG));
        let validator_info = borrow_global_mut<ValidatorConfig>(pool_address);
        let old_network_addresses = validator_info.network_addresses;
        validator_info.network_addresses = new_network_addresses;
        let old_fullnode_addresses = validator_info.fullnode_addresses;
        validator_info.fullnode_addresses = new_fullnode_addresses;

        if (std::features::module_event_migration_enabled()) {
            event::emit(
                UpdateNetworkAndFullnodeAddresses {
                    pool_address,
                    old_network_addresses,
                    new_network_addresses,
                    old_fullnode_addresses,
                    new_fullnode_addresses,
                },
            );
        };
        event::emit_event(
            &mut stake_pool.update_network_and_fullnode_addresses_events,
            UpdateNetworkAndFullnodeAddressesEvent {
                pool_address,
                old_network_addresses,
                new_network_addresses,
                old_fullnode_addresses,
                new_fullnode_addresses,
            },
        );

    }

    /// Similar to increase_lockup_with_cap but will use ownership capability from the signing account.
    public entry fun increase_lockup(owner: &signer) acquires OwnerCapability, StakePool {
        let owner_address = signer::address_of(owner);
        assert_owner_cap_exists(owner_address);
        let ownership_cap = borrow_global<OwnerCapability>(owner_address);
        increase_lockup_with_cap(ownership_cap);
    }

    /// Unlock from active delegation, it's moved to pending_inactive if locked_until_secs < current_time or
    /// directly inactive if it's not from an active validator.
    public fun increase_lockup_with_cap(owner_cap: &OwnerCapability) acquires StakePool {
        let pool_address = owner_cap.pool_address;
        assert_stake_pool_exists(pool_address);
        let config = staking_config::get();

        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        let old_locked_until_secs = stake_pool.locked_until_secs;
        let new_locked_until_secs = timestamp::now_seconds() + staking_config::get_recurring_lockup_duration(&config);
        assert!(old_locked_until_secs < new_locked_until_secs, error::invalid_argument(EINVALID_LOCKUP));
        stake_pool.locked_until_secs = new_locked_until_secs;

        if (std::features::module_event_migration_enabled()) {
            event::emit(
                IncreaseLockup {
                    pool_address,
                    old_locked_until_secs,
                    new_locked_until_secs,
                },
            );
        };
        event::emit_event(
            &mut stake_pool.increase_lockup_events,
            IncreaseLockupEvent {
                pool_address,
                old_locked_until_secs,
                new_locked_until_secs,
            },
        );
    }

    /// This can only called by the operator of the validator/staking pool.
    public entry fun join_validator_set(
        operator: &signer,
        pool_address: address
    ) acquires StakePool, ValidatorConfig, ValidatorSet {
        assert!(
            staking_config::get_allow_validator_set_change(&staking_config::get()),
            error::invalid_argument(ENO_POST_GENESIS_VALIDATOR_SET_CHANGE_ALLOWED),
        );

        join_validator_set_internal(operator, pool_address);
    }

    /// Request to have `pool_address` join the validator set. Can only be called after calling `initialize_validator`.
    /// If the validator has the required stake (more than minimum and less than maximum allowed), they will be
    /// added to the pending_active queue. All validators in this queue will be added to the active set when the next
    /// epoch starts (eligibility will be rechecked).
    ///
    /// This internal version can only be called by the Genesis module during Genesis.
    public(friend) fun join_validator_set_internal(
        operator: &signer,
        pool_address: address
    ) acquires StakePool, ValidatorConfig, ValidatorSet {
        assert_reconfig_not_in_progress();
        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        assert!(signer::address_of(operator) == stake_pool.operator_address, error::unauthenticated(ENOT_OPERATOR));
        assert!(
            get_validator_state(pool_address) == VALIDATOR_STATUS_INACTIVE,
            error::invalid_state(EALREADY_ACTIVE_VALIDATOR),
        );

        let config = staking_config::get();
        let (minimum_stake, maximum_stake) = staking_config::get_required_stake(&config);
        let voting_power = get_next_epoch_voting_power(stake_pool);
        assert!(voting_power >= minimum_stake, error::invalid_argument(ESTAKE_TOO_LOW));
        assert!(voting_power <= maximum_stake, error::invalid_argument(ESTAKE_TOO_HIGH));

        // Track and validate voting power increase.
        update_voting_power_increase(voting_power);

        // Add validator to pending_active, to be activated in the next epoch.
        let validator_config = borrow_global_mut<ValidatorConfig>(pool_address);
        assert!(!vector::is_empty(&validator_config.consensus_pubkey), error::invalid_argument(EINVALID_PUBLIC_KEY));

        // Validate the current validator set size has not exceeded the limit.
        let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
        vector::push_back(
            &mut validator_set.pending_active,
            generate_validator_info(pool_address, stake_pool, *validator_config)
        );
        let validator_set_size = vector::length(&validator_set.active_validators) + vector::length(
            &validator_set.pending_active
        );
        assert!(validator_set_size <= MAX_VALIDATOR_SET_SIZE, error::invalid_argument(EVALIDATOR_SET_TOO_LARGE));

        if (std::features::module_event_migration_enabled()) {
            event::emit(JoinValidatorSet { pool_address });
        };
        event::emit_event(
            &mut stake_pool.join_validator_set_events,
            JoinValidatorSetEvent { pool_address },
        );
    }

    /// Similar to unlock_with_cap but will use ownership capability from the signing account.
    public entry fun unlock(owner: &signer, amount: u64) acquires OwnerCapability, StakePool {
        assert_reconfig_not_in_progress();
        let owner_address = signer::address_of(owner);
        assert_owner_cap_exists(owner_address);
        let ownership_cap = borrow_global<OwnerCapability>(owner_address);
        unlock_with_cap(amount, ownership_cap);
    }

    /// Unlock `amount` from the active stake. Only possible if the lockup has expired.
    public fun unlock_with_cap(amount: u64, owner_cap: &OwnerCapability) acquires StakePool {
        assert_reconfig_not_in_progress();
        // Short-circuit if amount to unlock is 0 so we don't emit events.
        if (amount == 0) {
            return
        };

        // Unlocked coins are moved to pending_inactive. When the current lockup cycle expires, they will be moved into
        // inactive in the earliest possible epoch transition.
        let pool_address = owner_cap.pool_address;
        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        // Cap amount to unlock by maximum active stake.
        let amount = min(amount, coin::value(&stake_pool.active));
        let unlocked_stake = coin::extract(&mut stake_pool.active, amount);
        coin::merge<AptosCoin>(&mut stake_pool.pending_inactive, unlocked_stake);

        if (std::features::module_event_migration_enabled()) {
            event::emit(
                UnlockStake {
                    pool_address,
                    amount_unlocked: amount,
                },
            );
        };
        event::emit_event(
            &mut stake_pool.unlock_stake_events,
            UnlockStakeEvent {
                pool_address,
                amount_unlocked: amount,
            },
        );
    }

    /// Withdraw from `account`'s inactive stake.
    public entry fun withdraw(
        owner: &signer,
        withdraw_amount: u64
    ) acquires OwnerCapability, StakePool, ValidatorSet {
        let owner_address = signer::address_of(owner);
        assert_owner_cap_exists(owner_address);
        let ownership_cap = borrow_global<OwnerCapability>(owner_address);
        let coins = withdraw_with_cap(ownership_cap, withdraw_amount);
        coin::deposit<AptosCoin>(owner_address, coins);
    }

    /// Withdraw from `pool_address`'s inactive stake with the corresponding `owner_cap`.
    public fun withdraw_with_cap(
        owner_cap: &OwnerCapability,
        withdraw_amount: u64
    ): Coin<AptosCoin> acquires StakePool, ValidatorSet {
        assert_reconfig_not_in_progress();
        let pool_address = owner_cap.pool_address;
        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        // There's an edge case where a validator unlocks their stake and leaves the validator set before
        // the stake is fully unlocked (the current lockup cycle has not expired yet).
        // This can leave their stake stuck in pending_inactive even after the current lockup cycle expires.
        if (get_validator_state(pool_address) == VALIDATOR_STATUS_INACTIVE &&
            timestamp::now_seconds() >= stake_pool.locked_until_secs) {
            let pending_inactive_stake = coin::extract_all(&mut stake_pool.pending_inactive);
            coin::merge(&mut stake_pool.inactive, pending_inactive_stake);
        };

        // Cap withdraw amount by total inactive coins.
        withdraw_amount = min(withdraw_amount, coin::value(&stake_pool.inactive));
        if (withdraw_amount == 0) return coin::zero<AptosCoin>();

        if (std::features::module_event_migration_enabled()) {
            event::emit(
                WithdrawStake {
                    pool_address,
                    amount_withdrawn: withdraw_amount,
                },
            );
        };
        event::emit_event(
            &mut stake_pool.withdraw_stake_events,
            WithdrawStakeEvent {
                pool_address,
                amount_withdrawn: withdraw_amount,
            },
        );

        coin::extract(&mut stake_pool.inactive, withdraw_amount)
    }

    /// Request to have `pool_address` leave the validator set. The validator is only actually removed from the set when
    /// the next epoch starts.
    /// The last validator in the set cannot leave. This is an edge case that should never happen as long as the network
    /// is still operational.
    ///
    /// Can only be called by the operator of the validator/staking pool.
    public entry fun leave_validator_set(
        operator: &signer,
        pool_address: address
    ) acquires StakePool, ValidatorSet {
        assert_reconfig_not_in_progress();
        let config = staking_config::get();
        assert!(
            staking_config::get_allow_validator_set_change(&config),
            error::invalid_argument(ENO_POST_GENESIS_VALIDATOR_SET_CHANGE_ALLOWED),
        );

        assert_stake_pool_exists(pool_address);
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        // Account has to be the operator.
        assert!(signer::address_of(operator) == stake_pool.operator_address, error::unauthenticated(ENOT_OPERATOR));

        let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
        // If the validator is still pending_active, directly kick the validator out.
        let maybe_pending_active_index = find_validator(&validator_set.pending_active, pool_address);
        if (option::is_some(&maybe_pending_active_index)) {
            vector::swap_remove(
                &mut validator_set.pending_active, option::extract(&mut maybe_pending_active_index));

            // Decrease the voting power increase as the pending validator's voting power was added when they requested
            // to join. Now that they changed their mind, their voting power should not affect the joining limit of this
            // epoch.
            let validator_stake = (get_next_epoch_voting_power(stake_pool) as u128);
            // total_joining_power should be larger than validator_stake but just in case there has been a small
            // rounding error somewhere that can lead to an underflow, we still want to allow this transaction to
            // succeed.
            if (validator_set.total_joining_power > validator_stake) {
                validator_set.total_joining_power = validator_set.total_joining_power - validator_stake;
            } else {
                validator_set.total_joining_power = 0;
            };
        } else {
            // Validate that the validator is already part of the validator set.
            let maybe_active_index = find_validator(&validator_set.active_validators, pool_address);
            assert!(option::is_some(&maybe_active_index), error::invalid_state(ENOT_VALIDATOR));
            let validator_info = vector::swap_remove(
                &mut validator_set.active_validators, option::extract(&mut maybe_active_index));
            assert!(vector::length(&validator_set.active_validators) > 0, error::invalid_state(ELAST_VALIDATOR));
            vector::push_back(&mut validator_set.pending_inactive, validator_info);

            if (std::features::module_event_migration_enabled()) {
                event::emit(LeaveValidatorSet { pool_address });
            };
            event::emit_event(
                &mut stake_pool.leave_validator_set_events,
                LeaveValidatorSetEvent {
                    pool_address,
                },
            );
        };
    }

    /// Returns true if the current validator can still vote in the current epoch.
    /// This includes validators that requested to leave but are still in the pending_inactive queue and will be removed
    /// when the epoch starts.
    public fun is_current_epoch_validator(pool_address: address): bool acquires ValidatorSet {
        assert_stake_pool_exists(pool_address);
        let validator_state = get_validator_state(pool_address);
        validator_state == VALIDATOR_STATUS_ACTIVE || validator_state == VALIDATOR_STATUS_PENDING_INACTIVE
    }

    /// Update the validator performance (proposal statistics). This is only called by block::prologue().
    /// This function cannot abort.
    public(friend) fun update_performance_statistics(
        proposer_index: Option<u64>,
        failed_proposer_indices: vector<u64>
    ) acquires ValidatorPerformance {
        // Validator set cannot change until the end of the epoch, so the validator index in arguments should
        // match with those of the validators in ValidatorPerformance resource.
        let validator_perf = borrow_global_mut<ValidatorPerformance>(@aptos_framework);
        let validator_len = vector::length(&validator_perf.validators);

        spec {
            update ghost_valid_perf = validator_perf;
            update ghost_proposer_idx = proposer_index;
        };
        // proposer_index is an option because it can be missing (for NilBlocks)
        if (option::is_some(&proposer_index)) {
            let cur_proposer_index = option::extract(&mut proposer_index);
            // Here, and in all other vector::borrow, skip any validator indices that are out of bounds,
            // this ensures that this function doesn't abort if there are out of bounds errors.
            if (cur_proposer_index < validator_len) {
                let validator = vector::borrow_mut(&mut validator_perf.validators, cur_proposer_index);
                spec {
                    assume validator.successful_proposals + 1 <= MAX_U64;
                };
                validator.successful_proposals = validator.successful_proposals + 1;
            };
        };

        let f = 0;
        let f_len = vector::length(&failed_proposer_indices);
        while ({
            spec {
                invariant len(validator_perf.validators) == validator_len;
                invariant (option::spec_is_some(ghost_proposer_idx) && option::spec_borrow(
                    ghost_proposer_idx
                ) < validator_len) ==>
                    (validator_perf.validators[option::spec_borrow(ghost_proposer_idx)].successful_proposals ==
                        ghost_valid_perf.validators[option::spec_borrow(ghost_proposer_idx)].successful_proposals + 1);
            };
            f < f_len
        }) {
            let validator_index = *vector::borrow(&failed_proposer_indices, f);
            if (validator_index < validator_len) {
                let validator = vector::borrow_mut(&mut validator_perf.validators, validator_index);
                spec {
                    assume validator.failed_proposals + 1 <= MAX_U64;
                };
                validator.failed_proposals = validator.failed_proposals + 1;
            };
            f = f + 1;
        };
    }

    /// Triggered during a reconfiguration. This function shouldn't abort.
    ///
    /// 1. Distribute transaction fees and rewards to stake pools of active and pending inactive validators (requested
    /// to leave but not yet removed).
    /// 2. Officially move pending active stake to active and move pending inactive stake to inactive.
    /// The staking pool's voting power in this new epoch will be updated to the total active stake.
    /// 3. Add pending active validators to the active set if they satisfy requirements so they can vote and remove
    /// pending inactive validators so they no longer can vote.
    /// 4. The validator's voting power in the validator set is updated to be the corresponding staking pool's voting
    /// power.
    public(friend) fun on_new_epoch(
    ) acquires StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
        let config = staking_config::get();
        let validator_perf = borrow_global_mut<ValidatorPerformance>(@aptos_framework);

        // Process pending stake and distribute transaction fees and rewards for each currently active validator.
        vector::for_each_ref(&validator_set.active_validators, |validator| {
            let validator: &ValidatorInfo = validator;
            update_stake_pool(validator_perf, validator.addr, &config);
        });

        // Process pending stake and distribute transaction fees and rewards for each currently pending_inactive validator
        // (requested to leave but not removed yet).
        vector::for_each_ref(&validator_set.pending_inactive, |validator| {
            let validator: &ValidatorInfo = validator;
            update_stake_pool(validator_perf, validator.addr, &config);
        });

        // Activate currently pending_active validators.
        append(&mut validator_set.active_validators, &mut validator_set.pending_active);

        // Officially deactivate all pending_inactive validators. They will now no longer receive rewards.
        validator_set.pending_inactive = vector::empty();

        // Update active validator set so that network address/public key change takes effect.
        // Moreover, recalculate the total voting power, and deactivate the validator whose
        // voting power is less than the minimum required stake.
        let next_epoch_validators = vector::empty();
        let (minimum_stake, _) = staking_config::get_required_stake(&config);
        let vlen = vector::length(&validator_set.active_validators);
        let total_voting_power = 0;
        let i = 0;
        while ({
            spec {
                invariant spec_validators_are_initialized(next_epoch_validators);
                invariant i <= vlen;
            };
            i < vlen
        }) {
            let old_validator_info = vector::borrow_mut(&mut validator_set.active_validators, i);
            let pool_address = old_validator_info.addr;
            let validator_config = borrow_global_mut<ValidatorConfig>(pool_address);
            let stake_pool = borrow_global_mut<StakePool>(pool_address);
            let new_validator_info = generate_validator_info(pool_address, stake_pool, *validator_config);

            // A validator needs at least the min stake required to join the validator set.
            if (new_validator_info.voting_power >= minimum_stake) {
                spec {
                    assume total_voting_power + new_validator_info.voting_power <= MAX_U128;
                };
                total_voting_power = total_voting_power + (new_validator_info.voting_power as u128);
                vector::push_back(&mut next_epoch_validators, new_validator_info);
            };
            i = i + 1;
        };

        validator_set.active_validators = next_epoch_validators;
        validator_set.total_voting_power = total_voting_power;
        validator_set.total_joining_power = 0;

        // Update validator indices, reset performance scores, and renew lockups.
        validator_perf.validators = vector::empty();
        let recurring_lockup_duration_secs = staking_config::get_recurring_lockup_duration(&config);
        let vlen = vector::length(&validator_set.active_validators);
        let validator_index = 0;
        while ({
            spec {
                invariant spec_validators_are_initialized(validator_set.active_validators);
                invariant len(validator_set.pending_active) == 0;
                invariant len(validator_set.pending_inactive) == 0;
                invariant 0 <= validator_index && validator_index <= vlen;
                invariant vlen == len(validator_set.active_validators);
                invariant forall i in 0..validator_index:
                    global<ValidatorConfig>(validator_set.active_validators[i].addr).validator_index < validator_index;
                invariant forall i in 0..validator_index:
                    validator_set.active_validators[i].config.validator_index < validator_index;
                invariant len(validator_perf.validators) == validator_index;
            };
            validator_index < vlen
        }) {
            let validator_info = vector::borrow_mut(&mut validator_set.active_validators, validator_index);
            validator_info.config.validator_index = validator_index;
            let validator_config = borrow_global_mut<ValidatorConfig>(validator_info.addr);
            validator_config.validator_index = validator_index;

            vector::push_back(&mut validator_perf.validators, IndividualValidatorPerformance {
                successful_proposals: 0,
                failed_proposals: 0,
            });

            // Automatically renew a validator's lockup for validators that will still be in the validator set in the
            // next epoch.
            let stake_pool = borrow_global_mut<StakePool>(validator_info.addr);
            let now_secs = timestamp::now_seconds();
            let reconfig_start_secs = if (chain_status::is_operating()) {
                get_reconfig_start_time_secs()
            } else {
                now_secs
            };
            if (stake_pool.locked_until_secs <= reconfig_start_secs) {
                spec {
                    assume now_secs + recurring_lockup_duration_secs <= MAX_U64;
                };
                stake_pool.locked_until_secs = now_secs + recurring_lockup_duration_secs;
            };

            validator_index = validator_index + 1;
        };

        if (features::periodical_reward_rate_decrease_enabled()) {
            // Update rewards rate after reward distribution.
            staking_config::calculate_and_save_latest_epoch_rewards_rate();
        };
    }

    /// Return the `ValidatorConsensusInfo` of each current validator, sorted by current validator index.
    public fun cur_validator_consensus_infos(): vector<ValidatorConsensusInfo> acquires ValidatorSet {
        let validator_set = borrow_global<ValidatorSet>(@aptos_framework);
        validator_consensus_infos_from_validator_set(validator_set)
    }


    public fun next_validator_consensus_infos(): vector<ValidatorConsensusInfo> acquires ValidatorSet, ValidatorPerformance, StakePool, ValidatorFees, ValidatorConfig {
        // Init.
        let cur_validator_set = borrow_global<ValidatorSet>(@aptos_framework);
        let staking_config = staking_config::get();
        let validator_perf = borrow_global<ValidatorPerformance>(@aptos_framework);
        let (minimum_stake, _) = staking_config::get_required_stake(&staking_config);
        let (rewards_rate, rewards_rate_denominator) = staking_config::get_reward_rate(&staking_config);

        // Compute new validator set.
        let new_active_validators = vector[];
        let num_new_actives = 0;
        let candidate_idx = 0;
        let new_total_power = 0;
        let num_cur_actives = vector::length(&cur_validator_set.active_validators);
        let num_cur_pending_actives = vector::length(&cur_validator_set.pending_active);
        spec {
            assume num_cur_actives + num_cur_pending_actives <= MAX_U64;
        };
        let num_candidates = num_cur_actives + num_cur_pending_actives;
        while ({
            spec {
                invariant candidate_idx <= num_candidates;
                invariant spec_validators_are_initialized(new_active_validators);
                invariant len(new_active_validators) == num_new_actives;
                invariant forall i in 0..len(new_active_validators):
                    new_active_validators[i].config.validator_index == i;
                invariant num_new_actives <= candidate_idx;
                invariant spec_validators_are_initialized(new_active_validators);
            };
            candidate_idx < num_candidates
        }) {
            let candidate_in_current_validator_set = candidate_idx < num_cur_actives;
            let candidate = if (candidate_idx < num_cur_actives) {
                vector::borrow(&cur_validator_set.active_validators, candidate_idx)
            } else {
                vector::borrow(&cur_validator_set.pending_active, candidate_idx - num_cur_actives)
            };
            let stake_pool = borrow_global<StakePool>(candidate.addr);
            let cur_active = coin::value(&stake_pool.active);
            let cur_pending_active = coin::value(&stake_pool.pending_active);
            let cur_pending_inactive = coin::value(&stake_pool.pending_inactive);

            let cur_reward = if (candidate_in_current_validator_set && cur_active > 0) {
                spec {
                    assert candidate.config.validator_index < len(validator_perf.validators);
                };
                let cur_perf = vector::borrow(&validator_perf.validators, candidate.config.validator_index);
                spec {
                    assume cur_perf.successful_proposals + cur_perf.failed_proposals <= MAX_U64;
                };
                calculate_rewards_amount(cur_active, cur_perf.successful_proposals, cur_perf.successful_proposals + cur_perf.failed_proposals, rewards_rate, rewards_rate_denominator)
            } else {
                0
            };

            let cur_fee = 0;
            if (features::collect_and_distribute_gas_fees()) {
                let fees_table = &borrow_global<ValidatorFees>(@aptos_framework).fees_table;
                if (table::contains(fees_table, candidate.addr)) {
                    let fee_coin = table::borrow(fees_table, candidate.addr);
                    cur_fee = coin::value(fee_coin);
                }
            };

            let lockup_expired = get_reconfig_start_time_secs() >= stake_pool.locked_until_secs;
            spec {
                assume cur_active + cur_pending_active + cur_reward + cur_fee <= MAX_U64;
                assume cur_active + cur_pending_inactive + cur_pending_active + cur_reward + cur_fee <= MAX_U64;
            };
            let new_voting_power =
                cur_active
                + if (lockup_expired) { 0 } else { cur_pending_inactive }
                + cur_pending_active
                + cur_reward + cur_fee;

            if (new_voting_power >= minimum_stake) {
                let config = *borrow_global<ValidatorConfig>(candidate.addr);
                config.validator_index = num_new_actives;
                let new_validator_info = ValidatorInfo {
                    addr: candidate.addr,
                    voting_power: new_voting_power,
                    config,
                };

                // Update ValidatorSet.
                spec {
                    assume new_total_power + new_voting_power <= MAX_U128;
                };
                new_total_power = new_total_power + (new_voting_power as u128);
                vector::push_back(&mut new_active_validators, new_validator_info);
                num_new_actives = num_new_actives + 1;

            };
            candidate_idx = candidate_idx + 1;
        };

        let new_validator_set = ValidatorSet {
            consensus_scheme: cur_validator_set.consensus_scheme,
            active_validators: new_active_validators,
            pending_inactive: vector[],
            pending_active: vector[],
            total_voting_power: new_total_power,
            total_joining_power: 0,
        };

        validator_consensus_infos_from_validator_set(&new_validator_set)
    }

    fun validator_consensus_infos_from_validator_set(validator_set: &ValidatorSet): vector<ValidatorConsensusInfo> {
        let validator_consensus_infos = vector[];

        let num_active = vector::length(&validator_set.active_validators);
        let num_pending_inactive = vector::length(&validator_set.pending_inactive);
        spec {
            assume num_active + num_pending_inactive <= MAX_U64;
        };
        let total = num_active + num_pending_inactive;

        // Pre-fill the return value with dummy values.
        let idx = 0;
        while ({
            spec {
                invariant idx <= len(validator_set.active_validators) + len(validator_set.pending_inactive);
                invariant len(validator_consensus_infos) == idx;
                invariant len(validator_consensus_infos) <= len(validator_set.active_validators) + len(validator_set.pending_inactive);
            };
            idx < total
        }) {
            vector::push_back(&mut validator_consensus_infos, validator_consensus_info::default());
            idx = idx + 1;
        };
        spec {
            assert len(validator_consensus_infos) == len(validator_set.active_validators) + len(validator_set.pending_inactive);
            assert spec_validator_indices_are_valid_config(validator_set.active_validators,
                len(validator_set.active_validators) + len(validator_set.pending_inactive));
        };

        vector::for_each_ref(&validator_set.active_validators, |obj| {
            let vi: &ValidatorInfo = obj;
            spec {
                assume len(validator_consensus_infos) == len(validator_set.active_validators) + len(validator_set.pending_inactive);
                assert vi.config.validator_index < len(validator_consensus_infos);
            };
            let vci = vector::borrow_mut(&mut validator_consensus_infos, vi.config.validator_index);
            *vci = validator_consensus_info::new(
                vi.addr,
                vi.config.consensus_pubkey,
                vi.voting_power
            );
            spec {
                assert len(validator_consensus_infos) == len(validator_set.active_validators) + len(validator_set.pending_inactive);
            };
        });

        vector::for_each_ref(&validator_set.pending_inactive, |obj| {
            let vi: &ValidatorInfo = obj;
            spec {
                assume len(validator_consensus_infos) == len(validator_set.active_validators) + len(validator_set.pending_inactive);
                assert vi.config.validator_index < len(validator_consensus_infos);
            };
            let vci = vector::borrow_mut(&mut validator_consensus_infos, vi.config.validator_index);
            *vci = validator_consensus_info::new(
                vi.addr,
                vi.config.consensus_pubkey,
                vi.voting_power
            );
            spec {
                assert len(validator_consensus_infos) == len(validator_set.active_validators) + len(validator_set.pending_inactive);
            };
        });

        validator_consensus_infos
    }

    fun addresses_from_validator_infos(infos: &vector<ValidatorInfo>): vector<address> {
        vector::map_ref(infos, |obj| {
            let info: &ValidatorInfo = obj;
            info.addr
        })
    }

    /// Calculate the stake amount of a stake pool for the next epoch.
    /// Update individual validator's stake pool if `commit == true`.
    ///
    /// 1. distribute transaction fees to active/pending_inactive delegations
    /// 2. distribute rewards to active/pending_inactive delegations
    /// 3. process pending_active, pending_inactive correspondingly
    /// This function shouldn't abort.
    fun update_stake_pool(
        validator_perf: &ValidatorPerformance,
        pool_address: address,
        staking_config: &StakingConfig,
    ) acquires StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorFees {
        let stake_pool = borrow_global_mut<StakePool>(pool_address);
        let validator_config = borrow_global<ValidatorConfig>(pool_address);
        let cur_validator_perf = vector::borrow(&validator_perf.validators, validator_config.validator_index);
        let num_successful_proposals = cur_validator_perf.successful_proposals;
        spec {
            // The following addition should not overflow because `num_total_proposals` cannot be larger than 86400,
            // the maximum number of proposals in a day (1 proposal per second).
            assume cur_validator_perf.successful_proposals + cur_validator_perf.failed_proposals <= MAX_U64;
        };
        let num_total_proposals = cur_validator_perf.successful_proposals + cur_validator_perf.failed_proposals;
        let (rewards_rate, rewards_rate_denominator) = staking_config::get_reward_rate(staking_config);
        let rewards_active = distribute_rewards(
            &mut stake_pool.active,
            num_successful_proposals,
            num_total_proposals,
            rewards_rate,
            rewards_rate_denominator
        );
        let rewards_pending_inactive = distribute_rewards(
            &mut stake_pool.pending_inactive,
            num_successful_proposals,
            num_total_proposals,
            rewards_rate,
            rewards_rate_denominator
        );
        spec {
            assume rewards_active + rewards_pending_inactive <= MAX_U64;
        };
        let rewards_amount = rewards_active + rewards_pending_inactive;
        // Pending active stake can now be active.
        coin::merge(&mut stake_pool.active, coin::extract_all(&mut stake_pool.pending_active));

        // Additionally, distribute transaction fees.
        if (features::collect_and_distribute_gas_fees()) {
            let fees_table = &mut borrow_global_mut<ValidatorFees>(@aptos_framework).fees_table;
            if (table::contains(fees_table, pool_address)) {
                let coin = table::remove(fees_table, pool_address);
                coin::merge(&mut stake_pool.active, coin);
            };
        };

        // Pending inactive stake is only fully unlocked and moved into inactive if the current lockup cycle has expired
        let current_lockup_expiration = stake_pool.locked_until_secs;
        if (get_reconfig_start_time_secs() >= current_lockup_expiration) {
            coin::merge(
                &mut stake_pool.inactive,
                coin::extract_all(&mut stake_pool.pending_inactive),
            );
        };

        if (std::features::module_event_migration_enabled()) {
            event::emit(DistributeRewards { pool_address, rewards_amount });
        };
        event::emit_event(
            &mut stake_pool.distribute_rewards_events,
            DistributeRewardsEvent {
                pool_address,
                rewards_amount,
            },
        );
    }

    /// Assuming we are in a middle of a reconfiguration (no matter it is immediate or async), get its start time.
    fun get_reconfig_start_time_secs(): u64 {
        if (reconfiguration_state::is_initialized()) {
            reconfiguration_state::start_time_secs()
        } else {
            timestamp::now_seconds()
        }
    }

    /// Calculate the rewards amount.
    fun calculate_rewards_amount(
        stake_amount: u64,
        num_successful_proposals: u64,
        num_total_proposals: u64,
        rewards_rate: u64,
        rewards_rate_denominator: u64,
    ): u64 {
        spec {
            // The following condition must hold because
            // (1) num_successful_proposals <= num_total_proposals, and
            // (2) `num_total_proposals` cannot be larger than 86400, the maximum number of proposals
            //     in a day (1 proposal per second), and `num_total_proposals` is reset to 0 every epoch.
            assume num_successful_proposals * MAX_REWARDS_RATE <= MAX_U64;
        };
        // The rewards amount is equal to (stake amount * rewards rate * performance multiplier).
        // We do multiplication in u128 before division to avoid the overflow and minimize the rounding error.
        let rewards_numerator = (stake_amount as u128) * (rewards_rate as u128) * (num_successful_proposals as u128);
        let rewards_denominator = (rewards_rate_denominator as u128) * (num_total_proposals as u128);
        if (rewards_denominator > 0) {
            ((rewards_numerator / rewards_denominator) as u64)
        } else {
            0
        }
    }

    /// Mint rewards corresponding to current epoch's `stake` and `num_successful_votes`.
    fun distribute_rewards(
        stake: &mut Coin<AptosCoin>,
        num_successful_proposals: u64,
        num_total_proposals: u64,
        rewards_rate: u64,
        rewards_rate_denominator: u64,
    ): u64 acquires AptosCoinCapabilities {
        let stake_amount = coin::value(stake);
        let rewards_amount = if (stake_amount > 0) {
            calculate_rewards_amount(
                stake_amount,
                num_successful_proposals,
                num_total_proposals,
                rewards_rate,
                rewards_rate_denominator
            )
        } else {
            0
        };
        if (rewards_amount > 0) {
            let mint_cap = &borrow_global<AptosCoinCapabilities>(@aptos_framework).mint_cap;
            let rewards = coin::mint(rewards_amount, mint_cap);
            coin::merge(stake, rewards);
        };
        rewards_amount
    }

    fun append<T>(v1: &mut vector<T>, v2: &mut vector<T>) {
        while (!vector::is_empty(v2)) {
            vector::push_back(v1, vector::pop_back(v2));
        }
    }

    fun find_validator(v: &vector<ValidatorInfo>, addr: address): Option<u64> {
        let i = 0;
        let len = vector::length(v);
        while ({
            spec {
                invariant !(exists j in 0..i: v[j].addr == addr);
            };
            i < len
        }) {
            if (vector::borrow(v, i).addr == addr) {
                return option::some(i)
            };
            i = i + 1;
        };
        option::none()
    }

    fun generate_validator_info(addr: address, stake_pool: &StakePool, config: ValidatorConfig): ValidatorInfo {
        let voting_power = get_next_epoch_voting_power(stake_pool);
        ValidatorInfo {
            addr,
            voting_power,
            config,
        }
    }

    /// Returns validator's next epoch voting power, including pending_active, active, and pending_inactive stake.
    fun get_next_epoch_voting_power(stake_pool: &StakePool): u64 {
        let value_pending_active = coin::value(&stake_pool.pending_active);
        let value_active = coin::value(&stake_pool.active);
        let value_pending_inactive = coin::value(&stake_pool.pending_inactive);
        spec {
            assume value_pending_active + value_active + value_pending_inactive <= MAX_U64;
        };
        value_pending_active + value_active + value_pending_inactive
    }

    fun update_voting_power_increase(increase_amount: u64) acquires ValidatorSet {
        let validator_set = borrow_global_mut<ValidatorSet>(@aptos_framework);
        let voting_power_increase_limit =
            (staking_config::get_voting_power_increase_limit(&staking_config::get()) as u128);
        validator_set.total_joining_power = validator_set.total_joining_power + (increase_amount as u128);

        // Only validator voting power increase if the current validator set's voting power > 0.
        if (validator_set.total_voting_power > 0) {
            assert!(
                validator_set.total_joining_power <= validator_set.total_voting_power * voting_power_increase_limit / 100,
                error::invalid_argument(EVOTING_POWER_INCREASE_EXCEEDS_LIMIT),
            );
        }
    }

    fun assert_stake_pool_exists(pool_address: address) {
        assert!(stake_pool_exists(pool_address), error::invalid_argument(ESTAKE_POOL_DOES_NOT_EXIST));
    }

    /// This provides an ACL for Testnet purposes. In testnet, everyone is a whale, a whale can be a validator.
    /// This allows a testnet to bring additional entities into the validator set without compromising the
    /// security of the testnet. This will NOT be enabled in Mainnet.
    struct AllowedValidators has key {
        accounts: vector<address>,
    }

    public fun configure_allowed_validators(
        aptos_framework: &signer,
        accounts: vector<address>
    ) acquires AllowedValidators {
        let aptos_framework_address = signer::address_of(aptos_framework);
        system_addresses::assert_aptos_framework(aptos_framework);
        if (!exists<AllowedValidators>(aptos_framework_address)) {
            move_to(aptos_framework, AllowedValidators { accounts });
        } else {
            let allowed = borrow_global_mut<AllowedValidators>(aptos_framework_address);
            allowed.accounts = accounts;
        }
    }

    fun is_allowed(account: address): bool acquires AllowedValidators {
        if (!exists<AllowedValidators>(@aptos_framework)) {
            true
        } else {
            let allowed = borrow_global<AllowedValidators>(@aptos_framework);
            vector::contains(&allowed.accounts, &account)
        }
    }

    fun assert_owner_cap_exists(owner: address) {
        assert!(exists<OwnerCapability>(owner), error::not_found(EOWNER_CAP_NOT_FOUND));
    }

    fun assert_reconfig_not_in_progress() {
        assert!(!reconfiguration_state::is_in_progress(), error::invalid_state(ERECONFIGURATION_IN_PROGRESS));
    }

    #[test_only]
    use aptos_framework::aptos_coin;
    use aptos_std::bls12381::proof_of_possession_from_bytes;
    use aptos_framework::reconfiguration_state;
    use aptos_framework::validator_consensus_info;
    use aptos_framework::validator_consensus_info::ValidatorConsensusInfo;
    #[test_only]
    use aptos_std::fixed_point64;

    #[test_only]
    const EPOCH_DURATION: u64 = 60;

    #[test_only]
    const LOCKUP_CYCLE_SECONDS: u64 = 3600;

    #[test_only]
    public fun initialize_for_test(aptos_framework: &signer) {
        reconfiguration_state::initialize(aptos_framework);
        initialize_for_test_custom(aptos_framework, 100, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 100, 1000000);
    }

    #[test_only]
    public fun join_validator_set_for_test(
        pk: &bls12381::PublicKey,
        pop: &bls12381::ProofOfPossession,
        operator: &signer,
        pool_address: address,
        should_end_epoch: bool,
    ) acquires AptosCoinCapabilities, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        let pk_bytes = bls12381::public_key_to_bytes(pk);
        let pop_bytes = bls12381::proof_of_possession_to_bytes(pop);
        rotate_consensus_key(operator, pool_address, pk_bytes, pop_bytes);
        join_validator_set(operator, pool_address);
        if (should_end_epoch) {
            end_epoch();
        }
    }

    #[test_only]
    public fun fast_forward_to_unlock(pool_address: address)
    acquires AptosCoinCapabilities, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        let expiration_time = get_lockup_secs(pool_address);
        timestamp::update_global_time_for_test_secs(expiration_time);
        end_epoch();
    }

    // Convenient function for setting up all required stake initializations.
    #[test_only]
    public fun initialize_for_test_custom(
        aptos_framework: &signer,
        minimum_stake: u64,
        maximum_stake: u64,
        recurring_lockup_secs: u64,
        allow_validator_set_change: bool,
        rewards_rate_numerator: u64,
        rewards_rate_denominator: u64,
        voting_power_increase_limit: u64,
    ) {
        timestamp::set_time_has_started_for_testing(aptos_framework);
        reconfiguration_state::initialize(aptos_framework);
        if (!exists<ValidatorSet>(@aptos_framework)) {
            initialize(aptos_framework);
        };
        staking_config::initialize_for_test(
            aptos_framework,
            minimum_stake,
            maximum_stake,
            recurring_lockup_secs,
            allow_validator_set_change,
            rewards_rate_numerator,
            rewards_rate_denominator,
            voting_power_increase_limit,
        );

        if (!exists<AptosCoinCapabilities>(@aptos_framework)) {
            let (burn_cap, mint_cap) = aptos_coin::initialize_for_test(aptos_framework);
            store_aptos_coin_mint_cap(aptos_framework, mint_cap);
            coin::destroy_burn_cap<AptosCoin>(burn_cap);
        };
    }

    // This function assumes the stake module already the capability to mint aptos coins.
    #[test_only]
    public fun mint_coins(amount: u64): Coin<AptosCoin> acquires AptosCoinCapabilities {
        let mint_cap = &borrow_global<AptosCoinCapabilities>(@aptos_framework).mint_cap;
        coin::mint(amount, mint_cap)
    }

    #[test_only]
    public fun mint(account: &signer, amount: u64) acquires AptosCoinCapabilities {
        coin::register<AptosCoin>(account);
        coin::deposit(signer::address_of(account), mint_coins(amount));
    }

    #[test_only]
    public fun mint_and_add_stake(
        account: &signer, amount: u64) acquires AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorSet {
        mint(account, amount);
        add_stake(account, amount);
    }

    #[test_only]
    public fun initialize_test_validator(
        public_key: &bls12381::PublicKey,
        proof_of_possession: &bls12381::ProofOfPossession,
        validator: &signer,
        amount: u64,
        should_join_validator_set: bool,
        should_end_epoch: bool,
    ) acquires AllowedValidators, AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        let validator_address = signer::address_of(validator);
        if (!account::exists_at(signer::address_of(validator))) {
            account::create_account_for_test(validator_address);
        };

        let pk_bytes = bls12381::public_key_to_bytes(public_key);
        let pop_bytes = bls12381::proof_of_possession_to_bytes(proof_of_possession);
        initialize_validator(validator, pk_bytes, pop_bytes, vector::empty(), vector::empty());

        if (amount > 0) {
            mint_and_add_stake(validator, amount);
        };

        if (should_join_validator_set) {
            join_validator_set(validator, validator_address);
        };
        if (should_end_epoch) {
            end_epoch();
        };
    }

    #[test_only]
    public fun create_validator_set(
        aptos_framework: &signer,
        active_validator_addresses: vector<address>,
        public_keys: vector<bls12381::PublicKey>,
    ) {
        let active_validators = vector::empty<ValidatorInfo>();
        let i = 0;
        while (i < vector::length(&active_validator_addresses)) {
            let validator_address = vector::borrow(&active_validator_addresses, i);
            let pk = vector::borrow(&public_keys, i);
            vector::push_back(&mut active_validators, ValidatorInfo {
                addr: *validator_address,
                voting_power: 0,
                config: ValidatorConfig {
                    consensus_pubkey: bls12381::public_key_to_bytes(pk),
                    network_addresses: b"",
                    fullnode_addresses: b"",
                    validator_index: 0,
                }
            });
            i = i + 1;
        };

        move_to(aptos_framework, ValidatorSet {
            consensus_scheme: 0,
            // active validators for the current epoch
            active_validators,
            // pending validators to leave in next epoch (still active)
            pending_inactive: vector::empty<ValidatorInfo>(),
            // pending validators to join in next epoch
            pending_active: vector::empty<ValidatorInfo>(),
            total_voting_power: 0,
            total_joining_power: 0,
        });
    }

    #[test_only]
    public fun create_stake_pool(
        account: &signer,
        active: Coin<AptosCoin>,
        pending_inactive: Coin<AptosCoin>,
        locked_until_secs: u64,
    ) acquires AllowedValidators, OwnerCapability, StakePool, ValidatorSet {
        let account_address = signer::address_of(account);
        initialize_stake_owner(account, 0, account_address, account_address);
        let stake_pool = borrow_global_mut<StakePool>(account_address);
        coin::merge(&mut stake_pool.active, active);
        coin::merge(&mut stake_pool.pending_inactive, pending_inactive);
        stake_pool.locked_until_secs = locked_until_secs;
    }

    // Allows unit tests to set custom validator performances.
    #[test_only]
    public fun update_validator_performances_for_test(
        proposer_index: Option<u64>,
        failed_proposer_indices: vector<u64>,
    ) acquires ValidatorPerformance {
        update_performance_statistics(proposer_index, failed_proposer_indices);
    }

    #[test_only]
    public fun generate_identity(): (bls12381::SecretKey, bls12381::PublicKey, bls12381::ProofOfPossession) {
        let (sk, pkpop) = bls12381::generate_keys();
        let pop = bls12381::generate_proof_of_possession(&sk);
        let unvalidated_pk = bls12381::public_key_with_pop_to_normal(&pkpop);
        (sk, unvalidated_pk, pop)
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x10007, location = Self)]
    public entry fun test_inactive_validator_can_add_stake_if_exceeding_max_allowed(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, false, false);

        // Add more stake to exceed max. This should fail.
        mint_and_add_stake(validator, 9901);
    }

    #[test(aptos_framework = @0x1, validator_1 = @0x123, validator_2 = @0x234)]
    #[expected_failure(abort_code = 0x10007, location = Self)]
    public entry fun test_pending_active_validator_cannot_add_stake_if_exceeding_max_allowed(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) acquires AllowedValidators, AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test_custom(aptos_framework, 50, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 10, 100000);
        // Have one validator join the set to ensure the validator set is not empty when main validator joins.
        let (_sk_1, pk_1, pop_1) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100, true, true);

        // Validator 2 joins validator set but epoch has not ended so validator is in pending_active state.
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, true, false);

        // Add more stake to exceed max. This should fail.
        mint_and_add_stake(validator_2, 9901);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x10007, location = Self)]
    public entry fun test_active_validator_cannot_add_stake_if_exceeding_max_allowed(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        // Validator joins validator set and waits for epoch end so it's in the validator set.
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);

        // Add more stake to exceed max. This should fail.
        mint_and_add_stake(validator, 9901);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x10007, location = Self)]
    public entry fun test_active_validator_with_pending_inactive_stake_cannot_add_stake_if_exceeding_max_allowed(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        // Validator joins validator set and waits for epoch end so it's in the validator set.
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);

        // Request to unlock 50 coins, which go to pending_inactive. Validator has 50 remaining in active.
        unlock(validator, 50);
        assert_validator_state(signer::address_of(validator), 50, 0, 0, 50, 0);

        // Add 9901 more. Total stake is 50 (active) + 50 (pending_inactive) + 9901 > 10000 so still exceeding max.
        mint_and_add_stake(validator, 9901);
    }

    #[test(aptos_framework = @aptos_framework, validator_1 = @0x123, validator_2 = @0x234)]
    #[expected_failure(abort_code = 0x10007, location = Self)]
    public entry fun test_pending_inactive_cannot_add_stake_if_exceeding_max_allowed(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) acquires AllowedValidators, AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100, true, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, true, true);

        // Leave validator set so validator is in pending_inactive state.
        leave_validator_set(validator_1, signer::address_of(validator_1));

        // Add 9901 more. Total stake is 50 (active) + 50 (pending_inactive) + 9901 > 10000 so still exceeding max.
        mint_and_add_stake(validator_1, 9901);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_end_to_end(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);

        // Validator has a lockup now that they've joined the validator set.
        let validator_address = signer::address_of(validator);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 1);

        // Validator adds more stake while already being active.
        // The added stake should go to pending_active to wait for activation when next epoch starts.
        mint(validator, 900);
        add_stake(validator, 100);
        assert!(coin::balance<AptosCoin>(validator_address) == 800, 2);
        assert_validator_state(validator_address, 100, 0, 100, 0, 0);

        // Pending_active stake is activated in the new epoch.
        // Rewards of 1 coin are also distributed for the existing active stake of 100 coins.
        end_epoch();
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 3);
        assert_validator_state(validator_address, 201, 0, 0, 0, 0);

        // Request unlock of 100 coins. These 100 coins are moved to pending_inactive and will be unlocked when the
        // current lockup expires.
        unlock(validator, 100);
        assert_validator_state(validator_address, 101, 0, 0, 100, 0);

        // Enough time has passed so the current lockup cycle should have ended.
        // The first epoch after the lockup cycle ended should automatically move unlocked (pending_inactive) stake
        // to inactive.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();
        // Rewards were also minted to pending_inactive, which got all moved to inactive.
        assert_validator_state(validator_address, 102, 101, 0, 0, 0);
        // Lockup is renewed and validator is still active.
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 4);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 5);

        // Validator withdraws from inactive stake multiple times.
        withdraw(validator, 50);
        assert!(coin::balance<AptosCoin>(validator_address) == 850, 6);
        assert_validator_state(validator_address, 102, 51, 0, 0, 0);
        withdraw(validator, 51);
        assert!(coin::balance<AptosCoin>(validator_address) == 901, 7);
        assert_validator_state(validator_address, 102, 0, 0, 0, 0);

        // Enough time has passed again and the validator's lockup is renewed once more. Validator is still active.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 8);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 9);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_inactive_validator_with_existing_lockup_join_validator_set(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, false, false);

        // Validator sets lockup before even joining the set and lets half of lockup pass by.
        increase_lockup(validator);
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS / 2);
        let validator_address = signer::address_of(validator);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS / 2, 1);

        // Join the validator set with an existing lockup
        join_validator_set(validator, validator_address);

        // Validator is added to the set but lockup time shouldn't have changed.
        end_epoch();
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 2);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS / 2 - EPOCH_DURATION, 3);
        assert_validator_state(validator_address, 100, 0, 0, 0, 0);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x10012, location = Self)]
    public entry fun test_cannot_reduce_lockup(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, false, false);

        // Increase lockup.
        increase_lockup(validator);
        // Reduce recurring lockup to 0.
        staking_config::update_recurring_lockup_duration_secs(aptos_framework, 1);
        // INcrease lockup should now fail because the new lockup < old lockup.
        increase_lockup(validator);
    }

    #[test(aptos_framework = @aptos_framework, validator_1 = @0x123, validator_2 = @0x234)]
    #[expected_failure(abort_code = 0x1000D, location = Self)]
    public entry fun test_inactive_validator_cannot_join_if_exceed_increase_limit(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        // Only 50% voting power increase is allowed in each epoch.
        initialize_for_test_custom(aptos_framework, 50, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 10, 50);
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100, false, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, false, false);

        // Validator 1 needs to be in the set so validator 2's added stake counts against the limit.
        join_validator_set(validator_1, signer::address_of(validator_1));
        end_epoch();

        // Validator 2 joins the validator set but their stake would lead to exceeding the voting power increase limit.
        // Therefore, this should fail.
        join_validator_set(validator_2, signer::address_of(validator_2));
    }

    #[test(aptos_framework = @aptos_framework, validator_1 = @0x123, validator_2 = @0x234)]
    public entry fun test_pending_active_validator_can_add_more_stake(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test_custom(aptos_framework, 50, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 10, 10000);
        // Need 1 validator to be in the active validator set so joining limit works.
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100, false, true);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, false, false);

        // Add more stake while still pending_active.
        let validator_2_address = signer::address_of(validator_2);
        join_validator_set(validator_2, validator_2_address);
        assert!(get_validator_state(validator_2_address) == VALIDATOR_STATUS_PENDING_ACTIVE, 0);
        mint_and_add_stake(validator_2, 100);
        assert_validator_state(validator_2_address, 200, 0, 0, 0, 0);
    }

    #[test(aptos_framework = @aptos_framework, validator_1 = @0x123, validator_2 = @0x234)]
    #[expected_failure(abort_code = 0x1000D, location = Self)]
    public entry fun test_pending_active_validator_cannot_add_more_stake_than_limit(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        // 100% voting power increase is allowed in each epoch.
        initialize_for_test_custom(aptos_framework, 50, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 10, 100);
        // Need 1 validator to be in the active validator set so joining limit works.
        let (_sk_1, pk_1, pop_1) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100, true, true);

        // Validator 2 joins the validator set but epoch has not ended so they're still pending_active.
        // Current voting power increase is already 100%. This is not failing yet.
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, true, false);

        // Add more stake, which now exceeds the 100% limit. This should fail.
        mint_and_add_stake(validator_2, 1);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_pending_active_validator_leaves_validator_set(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        // Validator joins but epoch hasn't ended, so the validator is still pending_active.
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, false);
        let validator_address = signer::address_of(validator);
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_PENDING_ACTIVE, 0);

        // Check that voting power increase is tracked.
        assert!(borrow_global<ValidatorSet>(@aptos_framework).total_joining_power == 100, 0);

        // Leave the validator set immediately.
        leave_validator_set(validator, validator_address);
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_INACTIVE, 1);

        // Check that voting power increase has been decreased when the pending active validator leaves.
        assert!(borrow_global<ValidatorSet>(@aptos_framework).total_joining_power == 0, 1);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x1000D, location = Self)]
    public entry fun test_active_validator_cannot_add_more_stake_than_limit_in_multiple_epochs(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        // Only 50% voting power increase is allowed in each epoch.
        initialize_for_test_custom(aptos_framework, 50, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 10, 50);
        // Add initial stake and join the validator set.
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);

        let validator_address = signer::address_of(validator);
        assert_validator_state(validator_address, 100, 0, 0, 0, 0);
        end_epoch();
        assert_validator_state(validator_address, 110, 0, 0, 0, 0);
        end_epoch();
        assert_validator_state(validator_address, 121, 0, 0, 0, 0);
        // Add more than 50% limit. The following line should fail.
        mint_and_add_stake(validator, 99);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x1000D, location = Self)]
    public entry fun test_active_validator_cannot_add_more_stake_than_limit(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        // Only 50% voting power increase is allowed in each epoch.
        initialize_for_test_custom(aptos_framework, 50, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 10, 50);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);

        // Add more than 50% limit. This should fail.
        mint_and_add_stake(validator, 51);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_unlock_partial_stake(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        // Reward rate = 10%.
        initialize_for_test_custom(aptos_framework, 50, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 10, 100);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);

        // Unlock half of the coins.
        let validator_address = signer::address_of(validator);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 1);
        unlock(validator, 50);
        assert_validator_state(validator_address, 50, 0, 0, 50, 0);

        // Enough time has passed so the current lockup cycle should have ended.
        // 50 coins should have unlocked while the remaining 51 (50 + rewards) should stay locked for another cycle.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 2);
        // Validator received rewards in both active and pending inactive.
        assert_validator_state(validator_address, 55, 55, 0, 0, 0);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 3);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_can_withdraw_all_stake_and_rewards_at_once(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);
        let validator_address = signer::address_of(validator);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 0);

        // One more epoch passes to generate rewards.
        end_epoch();
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 1);
        assert_validator_state(validator_address, 101, 0, 0, 0, 0);

        // Unlock all coins while still having a lockup.
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS - EPOCH_DURATION, 2);
        unlock(validator, 101);
        assert_validator_state(validator_address, 0, 0, 0, 101, 0);

        // One more epoch passes while the current lockup cycle (3600 secs) has not ended.
        timestamp::fast_forward_seconds(1000);
        end_epoch();
        // Validator should not be removed from the validator set since their 100 coins in pending_inactive state should
        // still count toward voting power.
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 3);
        assert_validator_state(validator_address, 0, 0, 0, 102, 0);

        // Enough time has passed so the current lockup cycle should have ended. Funds are now fully unlocked.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();
        assert_validator_state(validator_address, 0, 103, 0, 0, 0);
        // Validator ahs been kicked out of the validator set as their stake is 0 now.
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_INACTIVE, 4);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_unlocking_more_than_available_stake_should_cap(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, false, false);

        // Validator unlocks more stake than they have active. This should limit the unlock to 100.
        unlock(validator, 200);
        assert_validator_state(signer::address_of(validator), 0, 0, 0, 100, 0);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_withdraw_should_cap_by_inactive_stake(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        // Initial balance = 900 (idle) + 100 (staked) = 1000.
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);
        mint(validator, 900);

        // Validator unlocks stake.
        unlock(validator, 100);
        // Enough time has passed so the stake is fully unlocked.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();

        // Validator can only withdraw a max of 100 unlocked coins even if they request to withdraw more than 100.
        withdraw(validator, 200);
        let validator_address = signer::address_of(validator);
        // Receive back all coins with an extra 1 for rewards.
        assert!(coin::balance<AptosCoin>(validator_address) == 1001, 2);
        assert_validator_state(validator_address, 0, 0, 0, 0, 0);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_can_reactivate_pending_inactive_stake(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);

        // Validator unlocks stake, which gets moved into pending_inactive.
        unlock(validator, 50);
        let validator_address = signer::address_of(validator);
        assert_validator_state(validator_address, 50, 0, 0, 50, 0);

        // Validator can reactivate pending_inactive stake.
        reactivate_stake(validator, 50);
        assert_validator_state(validator_address, 100, 0, 0, 0, 0);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_reactivate_more_than_available_pending_inactive_stake_should_cap(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);

        // Validator tries to reactivate more than available pending_inactive stake, which should limit to 50.
        unlock(validator, 50);
        let validator_address = signer::address_of(validator);
        assert_validator_state(validator_address, 50, 0, 0, 50, 0);
        reactivate_stake(validator, 51);
        assert_validator_state(validator_address, 100, 0, 0, 0, 0);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_active_validator_having_insufficient_remaining_stake_after_withdrawal_gets_kicked(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);

        // Unlock enough coins that the remaining is not enough to meet the min required.
        let validator_address = signer::address_of(validator);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 1);
        unlock(validator, 50);
        assert_validator_state(validator_address, 50, 0, 0, 50, 0);

        // Enough time has passed so the current lockup cycle should have ended.
        // 50 coins should have unlocked while the remaining 51 (50 + rewards) is not enough so the validator is kicked
        // from the validator set.
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 2);
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_INACTIVE, 2);
        assert_validator_state(validator_address, 50, 50, 0, 0, 0);
        // Lockup is no longer renewed since the validator is no longer a part of the validator set.
        assert!(get_remaining_lockup_secs(validator_address) == 0, 3);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123, validator_2 = @0x234)]
    public entry fun test_active_validator_leaves_staking_but_still_has_a_lockup(
        aptos_framework: &signer,
        validator: &signer,
        validator_2: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator, 100, true, false);
        // We need a second validator here just so the first validator can leave.
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, true, true);

        // Leave the validator set while still having a lockup.
        let validator_address = signer::address_of(validator);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 0);
        leave_validator_set(validator, validator_address);
        // Validator is in pending_inactive state but is technically still part of the validator set.
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_PENDING_INACTIVE, 2);
        assert_validator_state(validator_address, 100, 0, 0, 0, 1);
        end_epoch();

        // Epoch has ended so validator is no longer part of the validator set.
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_INACTIVE, 3);
        // However, their stake, including rewards, should still subject to the existing lockup.
        assert_validator_state(validator_address, 101, 0, 0, 0, 1);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS - EPOCH_DURATION, 4);

        // If they try to unlock, their stake is moved to pending_inactive and would only be withdrawable after the
        // lockup has expired.
        unlock(validator, 50);
        assert_validator_state(validator_address, 51, 0, 0, 50, 1);
        // A couple of epochs passed but lockup has not expired so the validator's stake remains the same.
        end_epoch();
        end_epoch();
        end_epoch();
        assert_validator_state(validator_address, 51, 0, 0, 50, 1);
        // Fast forward enough so the lockup expires. Now the validator can just call withdraw directly to withdraw
        // pending_inactive stakes.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        withdraw(validator, 50);
        assert_validator_state(validator_address, 51, 0, 0, 0, 1);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123, validator_2 = @0x234)]
    public entry fun test_active_validator_leaves_staking_and_rejoins_with_expired_lockup_should_be_renewed(
        aptos_framework: &signer,
        validator: &signer,
        validator_2: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator, 100, true, false);
        // We need a second validator here just so the first validator can leave.
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, true, true);

        // Leave the validator set while still having a lockup.
        let validator_address = signer::address_of(validator);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 0);
        leave_validator_set(validator, validator_address);
        end_epoch();

        // Fast forward enough so the lockup expires.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        assert!(get_remaining_lockup_secs(validator_address) == 0, 1);

        // Validator rejoins the validator set. Once the current epoch ends, their lockup should be automatically
        // renewed.
        join_validator_set(validator, validator_address);
        end_epoch();
        assert!(get_validator_state(validator_address) == VALIDATOR_STATUS_ACTIVE, 2);
        assert!(get_remaining_lockup_secs(validator_address) == LOCKUP_CYCLE_SECONDS, 2);
    }

    #[test(aptos_framework = @aptos_framework, validator_1 = @0x123, validator_2 = @0x234)]
    public entry fun test_pending_inactive_validator_does_not_count_in_increase_limit(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        // Only 50% voting power increase is allowed in each epoch.
        initialize_for_test_custom(aptos_framework, 50, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 10, 50);
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100, true, false);
        // We need a second validator here just so the first validator can leave.
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, true, true);

        // Validator 1 leaves the validator set. Epoch has not ended so they're still pending_inactive.
        leave_validator_set(validator_1, signer::address_of(validator_1));
        // Validator 1 adds more stake. This should not succeed as it should not count as a voting power increase.
        mint_and_add_stake(validator_1, 51);
    }

    #[test(aptos_framework = @0x1, validator_1 = @0x123, validator_2 = @0x234, validator_3 = @0x345)]
    public entry fun test_multiple_validators_join_and_leave(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
        validator_3: &signer
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        let validator_1_address = signer::address_of(validator_1);
        let validator_2_address = signer::address_of(validator_2);
        let validator_3_address = signer::address_of(validator_3);

        initialize_for_test_custom(aptos_framework, 100, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 100, 100);
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let pk_1_bytes = bls12381::public_key_to_bytes(&pk_1);
        let (_sk_2, pk_2, pop_2) = generate_identity();
        let (_sk_3, pk_3, pop_3) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100, false, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, false, false);
        initialize_test_validator(&pk_3, &pop_3, validator_3, 100, false, false);

        // Validator 1 and 2 join the validator set.
        join_validator_set(validator_2, validator_2_address);
        join_validator_set(validator_1, validator_1_address);
        end_epoch();
        assert!(get_validator_state(validator_1_address) == VALIDATOR_STATUS_ACTIVE, 0);
        assert!(get_validator_state(validator_2_address) == VALIDATOR_STATUS_ACTIVE, 1);

        // Validator indices is the reverse order of the joining order.
        assert_validator_state(validator_1_address, 100, 0, 0, 0, 0);
        assert_validator_state(validator_2_address, 100, 0, 0, 0, 1);
        let validator_set = borrow_global<ValidatorSet>(@aptos_framework);
        let validator_config_1 = vector::borrow(&validator_set.active_validators, 0);
        assert!(validator_config_1.addr == validator_1_address, 2);
        assert!(validator_config_1.config.validator_index == 0, 3);
        let validator_config_2 = vector::borrow(&validator_set.active_validators, 1);
        assert!(validator_config_2.addr == validator_2_address, 4);
        assert!(validator_config_2.config.validator_index == 1, 5);

        // Validator 1 rotates consensus key. Validator 2 leaves. Validator 3 joins.
        let (_sk_1b, pk_1b, pop_1b) = generate_identity();
        let pk_1b_bytes = bls12381::public_key_to_bytes(&pk_1b);
        let pop_1b_bytes = bls12381::proof_of_possession_to_bytes(&pop_1b);
        rotate_consensus_key(validator_1, validator_1_address, pk_1b_bytes, pop_1b_bytes);
        leave_validator_set(validator_2, validator_2_address);
        join_validator_set(validator_3, validator_3_address);
        // Validator 2 is not effectively removed until next epoch.
        assert!(get_validator_state(validator_2_address) == VALIDATOR_STATUS_PENDING_INACTIVE, 6);
        assert!(
            vector::borrow(
                &borrow_global<ValidatorSet>(@aptos_framework).pending_inactive,
                0
            ).addr == validator_2_address,
            0
        );
        // Validator 3 is not effectively added until next epoch.
        assert!(get_validator_state(validator_3_address) == VALIDATOR_STATUS_PENDING_ACTIVE, 7);
        assert!(
            vector::borrow(
                &borrow_global<ValidatorSet>(@aptos_framework).pending_active,
                0
            ).addr == validator_3_address,
            0
        );
        assert!(
            vector::borrow(
                &borrow_global<ValidatorSet>(@aptos_framework).active_validators,
                0
            ).config.consensus_pubkey == pk_1_bytes,
            0
        );

        // Changes applied after new epoch
        end_epoch();
        assert!(get_validator_state(validator_1_address) == VALIDATOR_STATUS_ACTIVE, 8);
        assert_validator_state(validator_1_address, 101, 0, 0, 0, 0);
        assert!(get_validator_state(validator_2_address) == VALIDATOR_STATUS_INACTIVE, 9);
        // The validator index of validator 2 stays the same but this doesn't matter as the next time they rejoin the
        // validator set, their index will get set correctly.
        assert_validator_state(validator_2_address, 101, 0, 0, 0, 1);
        assert!(get_validator_state(validator_3_address) == VALIDATOR_STATUS_ACTIVE, 10);
        assert_validator_state(validator_3_address, 100, 0, 0, 0, 1);
        assert!(
            vector::borrow(
                &borrow_global<ValidatorSet>(@aptos_framework).active_validators,
                0
            ).config.consensus_pubkey == pk_1b_bytes,
            0
        );

        // Validators without enough stake will be removed.
        unlock(validator_1, 50);
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();
        assert!(get_validator_state(validator_1_address) == VALIDATOR_STATUS_INACTIVE, 11);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_delegated_staking_with_owner_cap(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test_custom(aptos_framework, 100, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 100, 100);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 0, false, false);
        let owner_cap = extract_owner_cap(validator);

        // Add stake when the validator is not yet activated.
        add_stake_with_cap(&owner_cap, mint_coins(100));
        let pool_address = signer::address_of(validator);
        assert_validator_state(pool_address, 100, 0, 0, 0, 0);

        // Join the validator set with enough stake.
        join_validator_set(validator, pool_address);
        end_epoch();
        assert!(get_validator_state(pool_address) == VALIDATOR_STATUS_ACTIVE, 0);

        // Unlock the entire stake.
        unlock_with_cap(100, &owner_cap);
        assert_validator_state(pool_address, 0, 0, 0, 100, 0);
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();

        // Withdraw stake + rewards.
        assert_validator_state(pool_address, 0, 101, 0, 0, 0);
        let coins = withdraw_with_cap(&owner_cap, 101);
        assert!(coin::value(&coins) == 101, 1);
        assert_validator_state(pool_address, 0, 0, 0, 0, 0);

        // Operator can separately rotate consensus key.
        let (_sk_new, pk_new, pop_new) = generate_identity();
        let pk_new_bytes = bls12381::public_key_to_bytes(&pk_new);
        let pop_new_bytes = bls12381::proof_of_possession_to_bytes(&pop_new);
        rotate_consensus_key(validator, pool_address, pk_new_bytes, pop_new_bytes);
        let validator_config = borrow_global<ValidatorConfig>(pool_address);
        assert!(validator_config.consensus_pubkey == pk_new_bytes, 2);

        // Operator can update network and fullnode addresses.
        update_network_and_fullnode_addresses(validator, pool_address, b"1", b"2");
        let validator_config = borrow_global<ValidatorConfig>(pool_address);
        assert!(validator_config.network_addresses == b"1", 3);
        assert!(validator_config.fullnode_addresses == b"2", 4);

        // Cleanups.
        coin::register<AptosCoin>(validator);
        coin::deposit(pool_address, coins);
        deposit_owner_cap(validator, owner_cap);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x1000A, location = Self)]
    public entry fun test_validator_cannot_join_post_genesis(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test_custom(aptos_framework, 100, 10000, LOCKUP_CYCLE_SECONDS, false, 1, 100, 100);

        // Joining the validator set should fail as post genesis validator set change is not allowed.
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x1000E, location = Self)]
    public entry fun test_invalid_pool_address(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);
        join_validator_set(validator, @0x234);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x1000A, location = Self)]
    public entry fun test_validator_cannot_leave_post_genesis(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test_custom(aptos_framework, 100, 10000, LOCKUP_CYCLE_SECONDS, false, 1, 100, 100);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, false, false);

        // Bypass the check to join. This is the same function called during Genesis.
        let validator_address = signer::address_of(validator);
        join_validator_set_internal(validator, validator_address);
        end_epoch();

        // Leaving the validator set should fail as post genesis validator set change is not allowed.
        leave_validator_set(validator, validator_address);
    }

    #[test(
        aptos_framework = @aptos_framework,
        validator_1 = @aptos_framework,
        validator_2 = @0x2,
        validator_3 = @0x3,
        validator_4 = @0x4,
        validator_5 = @0x5
    )]
    fun test_validator_consensus_infos_from_validator_set(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
        validator_3: &signer,
        validator_4: &signer,
        validator_5: &signer,
    ) acquires AllowedValidators, AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        let v1_addr = signer::address_of(validator_1);
        let v2_addr = signer::address_of(validator_2);
        let v3_addr = signer::address_of(validator_3);
        let v4_addr = signer::address_of(validator_4);
        let v5_addr = signer::address_of(validator_5);

        initialize_for_test(aptos_framework);

        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        let (_sk_3, pk_3, pop_3) = generate_identity();
        let (_sk_4, pk_4, pop_4) = generate_identity();
        let (_sk_5, pk_5, pop_5) = generate_identity();
        let pk_1_bytes = bls12381::public_key_to_bytes(&pk_1);
        let pk_3_bytes = bls12381::public_key_to_bytes(&pk_3);
        let pk_5_bytes = bls12381::public_key_to_bytes(&pk_5);

        initialize_test_validator(&pk_1, &pop_1, validator_1, 101, false, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 102, false, false);
        initialize_test_validator(&pk_3, &pop_3, validator_3, 103, false, false);
        initialize_test_validator(&pk_4, &pop_4, validator_4, 104, false, false);
        initialize_test_validator(&pk_5, &pop_5, validator_5, 105, false, false);

        join_validator_set(validator_3, v3_addr);
        join_validator_set(validator_1, v1_addr);
        join_validator_set(validator_5, v5_addr);
        end_epoch();
        let vci_vec_0 = validator_consensus_infos_from_validator_set(borrow_global<ValidatorSet>(@aptos_framework));
        let vci_addrs = vector::map_ref(&vci_vec_0, |obj|{
            let vci: &ValidatorConsensusInfo = obj;
            validator_consensus_info::get_addr(vci)
        });
        let vci_pks = vector::map_ref(&vci_vec_0, |obj|{
            let vci: &ValidatorConsensusInfo = obj;
            validator_consensus_info::get_pk_bytes(vci)
        });
        let vci_voting_powers = vector::map_ref(&vci_vec_0, |obj|{
            let vci: &ValidatorConsensusInfo = obj;
            validator_consensus_info::get_voting_power(vci)
        });
        assert!(vector[@0x5, @aptos_framework, @0x3] == vci_addrs, 1);
        assert!(vector[pk_5_bytes, pk_1_bytes, pk_3_bytes] == vci_pks, 2);
        assert!(vector[105, 101, 103] == vci_voting_powers, 3);
        leave_validator_set(validator_3, v3_addr);
        let vci_vec_1 = validator_consensus_infos_from_validator_set(borrow_global<ValidatorSet>(@aptos_framework));
        assert!(vci_vec_0 == vci_vec_1, 11);
        join_validator_set(validator_2, v2_addr);
        let vci_vec_2 = validator_consensus_infos_from_validator_set(borrow_global<ValidatorSet>(@aptos_framework));
        assert!(vci_vec_0 == vci_vec_2, 12);
        leave_validator_set(validator_1, v1_addr);
        let vci_vec_3 = validator_consensus_infos_from_validator_set(borrow_global<ValidatorSet>(@aptos_framework));
        assert!(vci_vec_0 == vci_vec_3, 13);
        join_validator_set(validator_4, v4_addr);
        let vci_vec_4 = validator_consensus_infos_from_validator_set(borrow_global<ValidatorSet>(@aptos_framework));
        assert!(vci_vec_0 == vci_vec_4, 14);
    }

    #[test(
        aptos_framework = @aptos_framework,
        validator_1 = @aptos_framework,
        validator_2 = @0x2,
        validator_3 = @0x3,
        validator_4 = @0x4,
        validator_5 = @0x5
    )]
    public entry fun test_staking_validator_index(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
        validator_3: &signer,
        validator_4: &signer,
        validator_5: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        let v1_addr = signer::address_of(validator_1);
        let v2_addr = signer::address_of(validator_2);
        let v3_addr = signer::address_of(validator_3);
        let v4_addr = signer::address_of(validator_4);
        let v5_addr = signer::address_of(validator_5);

        initialize_for_test(aptos_framework);

        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        let (_sk_3, pk_3, pop_3) = generate_identity();
        let (_sk_4, pk_4, pop_4) = generate_identity();
        let (_sk_5, pk_5, pop_5) = generate_identity();

        initialize_test_validator(&pk_1, &pop_1, validator_1, 100, false, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, false, false);
        initialize_test_validator(&pk_3, &pop_3, validator_3, 100, false, false);
        initialize_test_validator(&pk_4, &pop_4, validator_4, 100, false, false);
        initialize_test_validator(&pk_5, &pop_5, validator_5, 100, false, false);

        join_validator_set(validator_3, v3_addr);
        end_epoch();
        assert!(get_validator_index(v3_addr) == 0, 0);

        join_validator_set(validator_4, v4_addr);
        end_epoch();
        assert!(get_validator_index(v3_addr) == 0, 1);
        assert!(get_validator_index(v4_addr) == 1, 2);

        join_validator_set(validator_1, v1_addr);
        join_validator_set(validator_2, v2_addr);
        // pending_inactive is appended in reverse order
        end_epoch();
        assert!(get_validator_index(v3_addr) == 0, 6);
        assert!(get_validator_index(v4_addr) == 1, 7);
        assert!(get_validator_index(v2_addr) == 2, 8);
        assert!(get_validator_index(v1_addr) == 3, 9);

        join_validator_set(validator_5, v5_addr);
        end_epoch();
        assert!(get_validator_index(v3_addr) == 0, 10);
        assert!(get_validator_index(v4_addr) == 1, 11);
        assert!(get_validator_index(v2_addr) == 2, 12);
        assert!(get_validator_index(v1_addr) == 3, 13);
        assert!(get_validator_index(v5_addr) == 4, 14);

        // after swap remove, it's 3,4,2,5
        leave_validator_set(validator_1, v1_addr);
        // after swap remove, it's 5,4,2
        leave_validator_set(validator_3, v3_addr);
        end_epoch();

        assert!(get_validator_index(v5_addr) == 0, 15);
        assert!(get_validator_index(v4_addr) == 1, 16);
        assert!(get_validator_index(v2_addr) == 2, 17);
    }

    #[test(aptos_framework = @aptos_framework, validator_1 = @0x123, validator_2 = @0x234)]
    public entry fun test_validator_rewards_are_performance_based(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);

        let validator_1_address = signer::address_of(validator_1);
        let validator_2_address = signer::address_of(validator_2);

        // Both validators join the set.
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100, true, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, true, true);

        // Validator 2 failed proposal.
        let failed_proposer_indices = vector::empty<u64>();
        let validator_1_index = borrow_global<ValidatorConfig>(validator_1_address).validator_index;
        let validator_2_index = borrow_global<ValidatorConfig>(validator_2_address).validator_index;
        vector::push_back(&mut failed_proposer_indices, validator_2_index);
        let proposer_indices = option::some(validator_1_index);
        update_performance_statistics(proposer_indices, failed_proposer_indices);
        end_epoch();

        // Validator 2 received no rewards. Validator 1 didn't fail proposals, so it still receives rewards.
        assert_validator_state(validator_1_address, 101, 0, 0, 0, 1);
        assert_validator_state(validator_2_address, 100, 0, 0, 0, 0);

        // Validator 2 decides to leave. Both validators failed proposals.
        unlock(validator_2, 100);
        leave_validator_set(validator_2, validator_2_address);
        let failed_proposer_indices = vector::empty<u64>();
        let validator_1_index = borrow_global<ValidatorConfig>(validator_1_address).validator_index;
        let validator_2_index = borrow_global<ValidatorConfig>(validator_2_address).validator_index;
        vector::push_back(&mut failed_proposer_indices, validator_1_index);
        vector::push_back(&mut failed_proposer_indices, validator_2_index);
        update_performance_statistics(option::none(), failed_proposer_indices);
        // Fast forward so validator 2's stake is fully unlocked.
        timestamp::fast_forward_seconds(LOCKUP_CYCLE_SECONDS);
        end_epoch();

        // Validator 1 and 2 received no additional rewards due to failed proposals
        assert_validator_state(validator_1_address, 101, 0, 0, 0, 0);
        assert_validator_state(validator_2_address, 0, 100, 0, 0, 0);
    }

    #[test(aptos_framework = @aptos_framework, validator_1 = @0x123, validator_2 = @0x234)]
    public entry fun test_validator_rewards_rate_decrease_over_time(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);

        let genesis_time_in_secs = timestamp::now_seconds();

        let validator_1_address = signer::address_of(validator_1);
        let validator_2_address = signer::address_of(validator_2);

        // Both validators join the set.
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 1000, true, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 10000, true, true);

        // One epoch passed. Validator 1 and validator 2 should receive rewards at rewards rate = 1% every epoch.
        end_epoch();
        assert_validator_state(validator_1_address, 1010, 0, 0, 0, 1);
        assert_validator_state(validator_2_address, 10100, 0, 0, 0, 0);

        // Enable rewards rate decrease. Initially rewards rate is still 1% every epoch. Rewards rate halves every year.
        let one_year_in_secs: u64 = 31536000;
        staking_config::initialize_rewards(
            aptos_framework,
            fixed_point64::create_from_rational(1, 100),
            fixed_point64::create_from_rational(3, 1000),
            one_year_in_secs,
            genesis_time_in_secs,
            fixed_point64::create_from_rational(50, 100),
        );
        features::change_feature_flags_for_testing(aptos_framework, vector[features::get_periodical_reward_rate_decrease_feature()], vector[]);

        // For some reason, this epoch is very long. It has been 1 year since genesis when the epoch ends.
        timestamp::fast_forward_seconds(one_year_in_secs - EPOCH_DURATION * 3);
        end_epoch();
        // Validator 1 and validator 2 should still receive rewards at rewards rate = 1% every epoch. Rewards rate has halved after this epoch.
        assert_validator_state(validator_1_address, 1020, 0, 0, 0, 1);
        assert_validator_state(validator_2_address, 10200, 0, 0, 0, 0);

        // For some reason, this epoch is also very long. One year passed.
        timestamp::fast_forward_seconds(one_year_in_secs - EPOCH_DURATION);
        end_epoch();
        // Validator 1 and validator 2 should still receive rewards at rewards rate = 0.5% every epoch. Rewards rate has halved after this epoch.
        assert_validator_state(validator_1_address, 1025, 0, 0, 0, 1);
        assert_validator_state(validator_2_address, 10250, 0, 0, 0, 0);

        end_epoch();
        // Rewards rate has halved but cannot become lower than min_rewards_rate.
        // Validator 1 and validator 2 should receive rewards at rewards rate = 0.3% every epoch.
        assert_validator_state(validator_1_address, 1028, 0, 0, 0, 1);
        assert_validator_state(validator_2_address, 10280, 0, 0, 0, 0);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_update_performance_statistics_should_not_fail_due_to_out_of_bounds(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);

        let validator_address = signer::address_of(validator);
        let (_sk, pk, pop) = generate_identity();
        initialize_test_validator(&pk, &pop, validator, 100, true, true);

        let valid_validator_index = borrow_global<ValidatorConfig>(validator_address).validator_index;
        let out_of_bounds_index = valid_validator_index + 100;

        // Invalid validator index in the failed proposers vector should not lead to abort.
        let failed_proposer_indices = vector::empty<u64>();
        vector::push_back(&mut failed_proposer_indices, valid_validator_index);
        vector::push_back(&mut failed_proposer_indices, out_of_bounds_index);
        update_performance_statistics(option::none(), failed_proposer_indices);
        end_epoch();

        // Validator received no rewards due to failing to propose.
        assert_validator_state(validator_address, 100, 0, 0, 0, 0);

        // Invalid validator index in the proposer should not lead to abort.
        let proposer_index_optional = option::some(out_of_bounds_index);
        update_performance_statistics(proposer_index_optional, vector::empty<u64>());
        end_epoch();
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    #[expected_failure(abort_code = 0x1000B, location = Self)]
    public entry fun test_invalid_config(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorConfig, ValidatorSet {
        initialize_for_test_custom(aptos_framework, 50, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 100, 100);

        // Call initialize_stake_owner, which only initializes the stake pool but not validator config.
        let validator_address = signer::address_of(validator);
        account::create_account_for_test(validator_address);
        initialize_stake_owner(validator, 0, validator_address, validator_address);
        mint_and_add_stake(validator, 100);

        // Join the validator set with enough stake. This should fail because the validator didn't initialize validator
        // config.
        join_validator_set(validator, validator_address);
    }

    #[test(aptos_framework = @aptos_framework, validator = @0x123)]
    public entry fun test_valid_config(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorConfig, ValidatorSet {
        initialize_for_test_custom(aptos_framework, 50, 10000, LOCKUP_CYCLE_SECONDS, true, 1, 100, 100);

        // Call initialize_stake_owner, which only initializes the stake pool but not validator config.
        let validator_address = signer::address_of(validator);
        account::create_account_for_test(validator_address);
        initialize_stake_owner(validator, 0, validator_address, validator_address);
        mint_and_add_stake(validator, 100);

        // Initialize validator config.
        let validator_address = signer::address_of(validator);
        let (_sk_new, pk_new, pop_new) = generate_identity();
        let pk_new_bytes = bls12381::public_key_to_bytes(&pk_new);
        let pop_new_bytes = bls12381::proof_of_possession_to_bytes(&pop_new);
        rotate_consensus_key(validator, validator_address, pk_new_bytes, pop_new_bytes);

        // Join the validator set with enough stake. This now wouldn't fail since the validator config already exists.
        join_validator_set(validator, validator_address);
    }

    #[test]
    public entry fun test_rewards_calculation() {
        let stake_amount = 2000;
        let num_successful_proposals = 199;
        let num_total_proposals = 200;
        let rewards_rate = 700;
        let rewards_rate_denominator = 777;
        let rewards_amount = calculate_rewards_amount(
            stake_amount,
            num_successful_proposals,
            num_total_proposals,
            rewards_rate,
            rewards_rate_denominator
        );
        // Consider `amount_imprecise` and `amount_precise` defined as follows:
        // amount_imprecise = (stake_amount * rewards_rate / rewards_rate_denominator) * num_successful_proposals / num_total_proposals
        // amount_precise = stake_amount * rewards_rate * num_successful_proposals / (rewards_rate_denominator * num_total_proposals)
        // Although they are equivalent in the real arithmetic, they are not in the integer arithmetic due to a rounding error.
        // With the test parameters above, `amount_imprecise` is equal to 1791 because of an unbounded rounding error
        // while `amount_precise` is equal to 1792. We expect the output of `calculate_rewards_amount` to be 1792.
        assert!(rewards_amount == 1792, 0);

        let stake_amount = 100000000000000000;
        let num_successful_proposals = 9999;
        let num_total_proposals = 10000;
        let rewards_rate = 3141592;
        let rewards_rate_denominator = 10000000;
        // This should not abort due to an arithmetic overflow.
        let rewards_amount = calculate_rewards_amount(
            stake_amount,
            num_successful_proposals,
            num_total_proposals,
            rewards_rate,
            rewards_rate_denominator
        );
        assert!(rewards_amount == 31412778408000000, 0);
    }

    #[test_only]
    public fun set_validator_perf_at_least_one_block() acquires ValidatorPerformance {
        let validator_perf = borrow_global_mut<ValidatorPerformance>(@aptos_framework);
        vector::for_each_mut(&mut validator_perf.validators, |validator|{
            let validator: &mut IndividualValidatorPerformance = validator;
            if (validator.successful_proposals + validator.failed_proposals < 1) {
                validator.successful_proposals = 1;
            };
        });
    }

    #[test(aptos_framework = @0x1, validator_1 = @0x123, validator_2 = @0x234)]
    public entry fun test_removing_validator_from_active_set(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        initialize_for_test(aptos_framework);
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100, true, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, true, true);
        assert!(vector::length(&borrow_global<ValidatorSet>(@aptos_framework).active_validators) == 2, 0);

        // Remove validator 1 from the active validator set. Only validator 2 remains.
        let validator_to_remove = signer::address_of(validator_1);
        remove_validators(aptos_framework, &vector[validator_to_remove]);
        assert!(vector::length(&borrow_global<ValidatorSet>(@aptos_framework).active_validators) == 1, 0);
        assert!(get_validator_state(validator_to_remove) == VALIDATOR_STATUS_PENDING_INACTIVE, 1);
    }

    #[test_only]
    public fun end_epoch(
    ) acquires StakePool, AptosCoinCapabilities, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        // Set the number of blocks to 1, to give out rewards to non-failing validators.
        set_validator_perf_at_least_one_block();
        timestamp::fast_forward_seconds(EPOCH_DURATION);
        reconfiguration_state::on_reconfig_start();
        on_new_epoch();
        reconfiguration_state::on_reconfig_finish();
    }

    #[test_only]
    public fun assert_stake_pool(
        pool_address: address,
        active_stake: u64,
        inactive_stake: u64,
        pending_active_stake: u64,
        pending_inactive_stake: u64,
    ) acquires StakePool {
        let stake_pool = borrow_global<StakePool>(pool_address);
        let actual_active_stake = coin::value(&stake_pool.active);
        assert!(actual_active_stake == active_stake, actual_active_stake);
        let actual_inactive_stake = coin::value(&stake_pool.inactive);
        assert!(actual_inactive_stake == inactive_stake, actual_inactive_stake);
        let actual_pending_active_stake = coin::value(&stake_pool.pending_active);
        assert!(actual_pending_active_stake == pending_active_stake, actual_pending_active_stake);
        let actual_pending_inactive_stake = coin::value(&stake_pool.pending_inactive);
        assert!(actual_pending_inactive_stake == pending_inactive_stake, actual_pending_inactive_stake);
    }

    #[test_only]
    public fun assert_validator_state(
        pool_address: address,
        active_stake: u64,
        inactive_stake: u64,
        pending_active_stake: u64,
        pending_inactive_stake: u64,
        validator_index: u64,
    ) acquires StakePool, ValidatorConfig {
        assert_stake_pool(pool_address, active_stake, inactive_stake, pending_active_stake, pending_inactive_stake);
        let validator_config = borrow_global<ValidatorConfig>(pool_address);
        assert!(validator_config.validator_index == validator_index, validator_config.validator_index);
    }

    #[test(aptos_framework = @0x1, validator = @0x123)]
    public entry fun test_allowed_validators(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, ValidatorSet {
        let addr = signer::address_of(validator);
        let (burn, mint) = aptos_coin::initialize_for_test(aptos_framework);
        configure_allowed_validators(aptos_framework, vector[addr]);

        account::create_account_for_test(addr);
        coin::register<AptosCoin>(validator);
        initialize_stake_owner(validator, 0, addr, addr);
        coin::destroy_burn_cap(burn);
        coin::destroy_mint_cap(mint);
    }

    #[test(aptos_framework = @0x1, validator = @0x123)]
    #[expected_failure(abort_code = 0x60011, location = Self)]
    public entry fun test_not_allowed_validators(
        aptos_framework: &signer,
        validator: &signer,
    ) acquires AllowedValidators, OwnerCapability, StakePool, ValidatorSet {
        configure_allowed_validators(aptos_framework, vector[]);
        let (burn, mint) = aptos_coin::initialize_for_test(aptos_framework);

        let addr = signer::address_of(validator);
        account::create_account_for_test(addr);
        coin::register<AptosCoin>(validator);
        initialize_stake_owner(validator, 0, addr, addr);
        coin::destroy_burn_cap(burn);
        coin::destroy_mint_cap(mint);
    }

    #[test_only]
    public fun with_rewards(amount: u64): u64 {
        let (numerator, denominator) = staking_config::get_reward_rate(&staking_config::get());
        amount + amount * numerator / denominator
    }

    #[test_only]
    public fun get_validator_fee(validator_addr: address): u64 acquires ValidatorFees {
        let fees_table = &borrow_global<ValidatorFees>(@aptos_framework).fees_table;
        let coin = table::borrow(fees_table, validator_addr);
        coin::value(coin)
    }

    #[test_only]
    public fun assert_no_fees_for_validator(validator_addr: address) acquires ValidatorFees {
        let fees_table = &borrow_global<ValidatorFees>(@aptos_framework).fees_table;
        assert!(!table::contains(fees_table, validator_addr), 0);
    }

    #[test_only]
    const COLLECT_AND_DISTRIBUTE_GAS_FEES: u64 = 6;

    #[test(aptos_framework = @0x1, validator_1 = @0x123, validator_2 = @0x234, validator_3 = @0x345)]
    fun test_distribute_validator_fees(
        aptos_framework: &signer,
        validator_1: &signer,
        validator_2: &signer,
        validator_3: &signer,
    ) acquires AllowedValidators, AptosCoinCapabilities, OwnerCapability, StakePool, ValidatorConfig, ValidatorPerformance, ValidatorSet, ValidatorFees {
        // Make sure that fees collection and distribution is enabled.
        features::change_feature_flags_for_testing(aptos_framework, vector[COLLECT_AND_DISTRIBUTE_GAS_FEES], vector[]);
        assert!(features::collect_and_distribute_gas_fees(), 0);

        // Initialize staking and validator fees table.
        initialize_for_test(aptos_framework);
        initialize_validator_fees(aptos_framework);

        let validator_1_address = signer::address_of(validator_1);
        let validator_2_address = signer::address_of(validator_2);
        let validator_3_address = signer::address_of(validator_3);

        // Validators join the set and epoch ends.
        let (_sk_1, pk_1, pop_1) = generate_identity();
        let (_sk_2, pk_2, pop_2) = generate_identity();
        let (_sk_3, pk_3, pop_3) = generate_identity();
        initialize_test_validator(&pk_1, &pop_1, validator_1, 100, true, false);
        initialize_test_validator(&pk_2, &pop_2, validator_2, 100, true, false);
        initialize_test_validator(&pk_3, &pop_3, validator_3, 100, true, true);

        // Next, simulate fees collection during three blocks, where proposers are
        // validators 1, 2, and 1 again.
        add_transaction_fee(validator_1_address, mint_coins(100));
        add_transaction_fee(validator_2_address, mint_coins(500));
        add_transaction_fee(validator_1_address, mint_coins(200));

        // Fess have to be assigned to the right validators, but not
        // distributed yet.
        assert!(get_validator_fee(validator_1_address) == 300, 0);
        assert!(get_validator_fee(validator_2_address) == 500, 0);
        assert_no_fees_for_validator(validator_3_address);
        assert_validator_state(validator_1_address, 100, 0, 0, 0, 2);
        assert_validator_state(validator_2_address, 100, 0, 0, 0, 1);
        assert_validator_state(validator_3_address, 100, 0, 0, 0, 0);

        end_epoch();

        // Epoch ended. Validators must have recieved their rewards and, most importantly,
        // their fees.
        assert_no_fees_for_validator(validator_1_address);
        assert_no_fees_for_validator(validator_2_address);
        assert_no_fees_for_validator(validator_3_address);
        assert_validator_state(validator_1_address, 401, 0, 0, 0, 2);
        assert_validator_state(validator_2_address, 601, 0, 0, 0, 1);
        assert_validator_state(validator_3_address, 101, 0, 0, 0, 0);
    }
}
