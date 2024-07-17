///
/// Simple vesting contract that allows specifying how much APT coins should be vesting in each fixed-size period. The
/// vesting contract also comes with staking and allows shareholders to withdraw rewards anytime.
///
/// Vesting schedule is represented as a vector of distributions. For example, a vesting schedule of
/// [3/48, 3/48, 1/48] means that after the vesting starts:
/// 1. The first and second periods will vest 3/48 of the total original grant.
/// 2. The third period will vest 1/48.
/// 3. All subsequent periods will also vest 1/48 (last distribution in the schedule) until the original grant runs out.
///
/// Shareholder flow:
/// 1. Admin calls create_vesting_contract with a schedule of [3/48, 3/48, 1/48] with a vesting cliff of 1 year and
/// vesting period of 1 month.
/// 2. After a month, a shareholder calls unlock_rewards to request rewards. They can also call vest() which would also
/// unlocks rewards but since the 1 year cliff has not passed (vesting has not started), vest() would not release any of
/// the original grant.
/// 3. After the unlocked rewards become fully withdrawable (as it's subject to staking lockup), shareholders can call
/// distribute() to send all withdrawable funds to all shareholders based on the original grant's shares structure.
/// 4. After 1 year and 1 month, the vesting schedule now starts. Shareholders call vest() to unlock vested coins. vest()
/// checks the schedule and unlocks 3/48 of the original grant in addition to any accumulated rewards since last
/// unlock_rewards(). Once the unlocked coins become withdrawable, shareholders can call distribute().
/// 5. Assuming the shareholders forgot to call vest() for 2 months, when they call vest() again, they will unlock vested
/// tokens for the next period since last vest. This would be for the first month they missed. They can call vest() a
/// second time to unlock for the second month they missed.
///
/// Admin flow:
/// 1. After creating the vesting contract, admin cannot change the vesting schedule.
/// 2. Admin can call update_voter, update_operator, or reset_lockup at any time to update the underlying staking
/// contract.
/// 3. Admin can also call update_beneficiary for any shareholder. This would send all distributions (rewards, vested
/// coins) of that shareholder to the beneficiary account. By defalt, if a beneficiary is not set, the distributions are
/// send directly to the shareholder account.
/// 4. Admin can call terminate_vesting_contract to terminate the vesting. This would first finish any distribution but
/// will prevent any further rewards or vesting distributions from being created. Once the locked up stake becomes
/// withdrawable, admin can call admin_withdraw to withdraw all funds to the vesting contract's withdrawal address.
module aptos_framework::vesting {
    use std::bcs;
    use std::error;
    use std::fixed_point32::{Self, FixedPoint32};
    use std::signer;
    use std::string::{utf8, String};
    use std::vector;

    use aptos_std::pool_u64::{Self, Pool};
    use aptos_std::simple_map::{Self, SimpleMap};

    use aptos_framework::account::{Self, SignerCapability, new_event_handle};
    use aptos_framework::aptos_account::{Self, assert_account_is_registered_for_apt};
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::coin::{Self, Coin};
    use aptos_framework::event::{EventHandle, emit, emit_event};
    use aptos_framework::stake;
    use aptos_framework::staking_contract;
    use aptos_framework::system_addresses;
    use aptos_framework::timestamp;

    friend aptos_framework::genesis;

    const VESTING_POOL_SALT: vector<u8> = b"aptos_framework::vesting";

    /// Withdrawal address is invalid.
    const EINVALID_WITHDRAWAL_ADDRESS: u64 = 1;
    /// Vesting schedule cannot be empty.
    const EEMPTY_VESTING_SCHEDULE: u64 = 2;
    /// Vesting period cannot be 0.
    const EZERO_VESTING_SCHEDULE_PERIOD: u64 = 3;
    /// Shareholders list cannot be empty.
    const ENO_SHAREHOLDERS: u64 = 4;
    /// The length of shareholders and shares lists don't match.
    const ESHARES_LENGTH_MISMATCH: u64 = 5;
    /// Vesting cannot start before or at the current block timestamp. Has to be in the future.
    const EVESTING_START_TOO_SOON: u64 = 6;
    /// The signer is not the admin of the vesting contract.
    const ENOT_ADMIN: u64 = 7;
    /// Vesting contract needs to be in active state.
    const EVESTING_CONTRACT_NOT_ACTIVE: u64 = 8;
    /// Admin can only withdraw from an inactive (paused or terminated) vesting contract.
    const EVESTING_CONTRACT_STILL_ACTIVE: u64 = 9;
    /// No vesting contract found at provided address.
    const EVESTING_CONTRACT_NOT_FOUND: u64 = 10;
    /// Cannot terminate the vesting contract with pending active stake. Need to wait until next epoch.
    const EPENDING_STAKE_FOUND: u64 = 11;
    /// Grant amount cannot be 0.
    const EZERO_GRANT: u64 = 12;
    /// Vesting account has no other management roles beside admin.
    const EVESTING_ACCOUNT_HAS_NO_ROLES: u64 = 13;
    /// The vesting account has no such management role.
    const EROLE_NOT_FOUND: u64 = 14;
    /// Account is not admin or does not have the required role to take this action.
    const EPERMISSION_DENIED: u64 = 15;
    /// Zero items were provided to a *_many function.
    const EVEC_EMPTY_FOR_MANY_FUNCTION: u64 = 16;

    /// Maximum number of shareholders a vesting pool can support.
    const MAXIMUM_SHAREHOLDERS: u64 = 30;

    /// Vesting contract states.
    /// Vesting contract is active and distributions can be made.
    const VESTING_POOL_ACTIVE: u64 = 1;
    /// Vesting contract has been terminated and all funds have been released back to the withdrawal address.
    const VESTING_POOL_TERMINATED: u64 = 2;

    /// Roles that can manage certain aspects of the vesting account beyond the main admin.
    const ROLE_BENEFICIARY_RESETTER: vector<u8> = b"ROLE_BENEFICIARY_RESETTER";

    struct VestingSchedule has copy, drop, store {
        // The vesting schedule as a list of fractions that vest for each period. The last number is repeated until the
        // vesting amount runs out.
        // For example [1/24, 1/24, 1/48] with a period of 1 month means that after vesting starts, the first two months
        // will vest 1/24 of the original total amount. From the third month only, 1/48 will vest until the vesting fund
        // runs out.
        // u32/u32 should be sufficient to support vesting schedule fractions.
        schedule: vector<FixedPoint32>,
        // When the vesting should start.
        start_timestamp_secs: u64,
        // In seconds. How long each vesting period is. For example 1 month.
        period_duration: u64,
        // Last vesting period, 1-indexed. For example if 2 months have passed, the last vesting period, if distribution
        // was requested, would be 2. Default value is 0 which means there have been no vesting periods yet.
        last_vested_period: u64,
    }

    struct StakingInfo has store {
        // Where the vesting's stake pool is located at. Included for convenience.
        pool_address: address,
        // The currently assigned operator.
        operator: address,
        // The currently assigned voter.
        voter: address,
        // Commission paid to the operator of the stake pool.
        commission_percentage: u64,
    }

    struct VestingContract has key {
        state: u64,
        admin: address,
        grant_pool: Pool,
        beneficiaries: SimpleMap<address, address>,
        vesting_schedule: VestingSchedule,
        // Withdrawal address where all funds would be released back to if the admin ends the vesting for a specific
        // account or terminates the entire vesting contract.
        withdrawal_address: address,
        staking: StakingInfo,
        // Remaining amount in the grant. For calculating accumulated rewards.
        remaining_grant: u64,
        // Used to control staking.
        signer_cap: SignerCapability,

        // Events.
        update_operator_events: EventHandle<UpdateOperatorEvent>,
        update_voter_events: EventHandle<UpdateVoterEvent>,
        reset_lockup_events: EventHandle<ResetLockupEvent>,
        set_beneficiary_events: EventHandle<SetBeneficiaryEvent>,
        unlock_rewards_events: EventHandle<UnlockRewardsEvent>,
        vest_events: EventHandle<VestEvent>,
        distribute_events: EventHandle<DistributeEvent>,
        terminate_events: EventHandle<TerminateEvent>,
        admin_withdraw_events: EventHandle<AdminWithdrawEvent>,
    }

    struct VestingAccountManagement has key {
        roles: SimpleMap<String, address>,
    }

    struct AdminStore has key {
        vesting_contracts: vector<address>,
        // Used to create resource accounts for new vesting contracts so there's no address collision.
        nonce: u64,

        create_events: EventHandle<CreateVestingContractEvent>,
    }

    #[event]
    struct CreateVestingContract has drop, store {
        operator: address,
        voter: address,
        grant_amount: u64,
        withdrawal_address: address,
        vesting_contract_address: address,
        staking_pool_address: address,
        commission_percentage: u64,
    }

    #[event]
    struct UpdateOperator has drop, store {
        admin: address,
        vesting_contract_address: address,
        staking_pool_address: address,
        old_operator: address,
        new_operator: address,
        commission_percentage: u64,
    }

    #[event]
    struct UpdateVoter has drop, store {
        admin: address,
        vesting_contract_address: address,
        staking_pool_address: address,
        old_voter: address,
        new_voter: address,
    }

    #[event]
    struct ResetLockup has drop, store {
        admin: address,
        vesting_contract_address: address,
        staking_pool_address: address,
        new_lockup_expiration_secs: u64,
    }

    #[event]
    struct SetBeneficiary has drop, store {
        admin: address,
        vesting_contract_address: address,
        shareholder: address,
        old_beneficiary: address,
        new_beneficiary: address,
    }

    #[event]
    struct UnlockRewards has drop, store {
        admin: address,
        vesting_contract_address: address,
        staking_pool_address: address,
        amount: u64,
    }

    #[event]
    struct Vest has drop, store {
        admin: address,
        vesting_contract_address: address,
        staking_pool_address: address,
        period_vested: u64,
        amount: u64,
    }

    #[event]
    struct Distribute has drop, store {
        admin: address,
        vesting_contract_address: address,
        amount: u64,
    }

    #[event]
    struct Terminate has drop, store {
        admin: address,
        vesting_contract_address: address,
    }

    #[event]
    struct AdminWithdraw has drop, store {
        admin: address,
        vesting_contract_address: address,
        amount: u64,
    }

    struct CreateVestingContractEvent has drop, store {
        operator: address,
        voter: address,
        grant_amount: u64,
        withdrawal_address: address,
        vesting_contract_address: address,
        staking_pool_address: address,
        commission_percentage: u64,
    }

    struct UpdateOperatorEvent has drop, store {
        admin: address,
        vesting_contract_address: address,
        staking_pool_address: address,
        old_operator: address,
        new_operator: address,
        commission_percentage: u64,
    }

    struct UpdateVoterEvent has drop, store {
        admin: address,
        vesting_contract_address: address,
        staking_pool_address: address,
        old_voter: address,
        new_voter: address,
    }

    struct ResetLockupEvent has drop, store {
        admin: address,
        vesting_contract_address: address,
        staking_pool_address: address,
        new_lockup_expiration_secs: u64,
    }

    struct SetBeneficiaryEvent has drop, store {
        admin: address,
        vesting_contract_address: address,
        shareholder: address,
        old_beneficiary: address,
        new_beneficiary: address,
    }

    struct UnlockRewardsEvent has drop, store {
        admin: address,
        vesting_contract_address: address,
        staking_pool_address: address,
        amount: u64,
    }

    struct VestEvent has drop, store {
        admin: address,
        vesting_contract_address: address,
        staking_pool_address: address,
        period_vested: u64,
        amount: u64,
    }

    struct DistributeEvent has drop, store {
        admin: address,
        vesting_contract_address: address,
        amount: u64,
    }

    struct TerminateEvent has drop, store {
        admin: address,
        vesting_contract_address: address,
    }

    struct AdminWithdrawEvent has drop, store {
        admin: address,
        vesting_contract_address: address,
        amount: u64,
    }

    #[view]
    /// Return the address of the underlying stake pool (separate resource account) of the vesting contract.
    ///
    /// This errors out if the vesting contract with the provided address doesn't exist.
    public fun stake_pool_address(vesting_contract_address: address): address acquires VestingContract {
        assert_vesting_contract_exists(vesting_contract_address);
        borrow_global<VestingContract>(vesting_contract_address).staking.pool_address
    }

    #[view]
    /// Return the vesting start timestamp (in seconds) of the vesting contract.
    /// Vesting will start at this time, and once a full period has passed, the first vest will become unlocked.
    ///
    /// This errors out if the vesting contract with the provided address doesn't exist.
    public fun vesting_start_secs(vesting_contract_address: address): u64 acquires VestingContract {
        assert_vesting_contract_exists(vesting_contract_address);
        borrow_global<VestingContract>(vesting_contract_address).vesting_schedule.start_timestamp_secs
    }

    #[view]
    /// Return the duration of one vesting period (in seconds).
    /// Each vest is released after one full period has started, starting from the specified start_timestamp_secs.
    ///
    /// This errors out if the vesting contract with the provided address doesn't exist.
    public fun period_duration_secs(vesting_contract_address: address): u64 acquires VestingContract {
        assert_vesting_contract_exists(vesting_contract_address);
        borrow_global<VestingContract>(vesting_contract_address).vesting_schedule.period_duration
    }

    #[view]
    /// Return the remaining grant, consisting of unvested coins that have not been distributed to shareholders.
    /// Prior to start_timestamp_secs, the remaining grant will always be equal to the original grant.
    /// Once vesting has started, and vested tokens are distributed, the remaining grant will decrease over time,
    /// according to the vesting schedule.
    ///
    /// This errors out if the vesting contract with the provided address doesn't exist.
    public fun remaining_grant(vesting_contract_address: address): u64 acquires VestingContract {
        assert_vesting_contract_exists(vesting_contract_address);
        borrow_global<VestingContract>(vesting_contract_address).remaining_grant
    }

    #[view]
    /// Return the beneficiary account of the specified shareholder in a vesting contract.
    /// This is the same as the shareholder address by default and only different if it's been explicitly set.
    ///
    /// This errors out if the vesting contract with the provided address doesn't exist.
    public fun beneficiary(vesting_contract_address: address, shareholder: address): address acquires VestingContract {
        assert_vesting_contract_exists(vesting_contract_address);
        get_beneficiary(borrow_global<VestingContract>(vesting_contract_address), shareholder)
    }

    #[view]
    /// Return the percentage of accumulated rewards that is paid to the operator as commission.
    ///
    /// This errors out if the vesting contract with the provided address doesn't exist.
    public fun operator_commission_percentage(vesting_contract_address: address): u64 acquires VestingContract {
        assert_vesting_contract_exists(vesting_contract_address);
        borrow_global<VestingContract>(vesting_contract_address).staking.commission_percentage
    }

    #[view]
    /// Return all the vesting contracts a given address is an admin of.
    public fun vesting_contracts(admin: address): vector<address> acquires AdminStore {
        if (!exists<AdminStore>(admin)) {
            vector::empty<address>()
        } else {
            borrow_global<AdminStore>(admin).vesting_contracts
        }
    }

    #[view]
    /// Return the operator who runs the validator for the vesting contract.
    ///
    /// This errors out if the vesting contract with the provided address doesn't exist.
    public fun operator(vesting_contract_address: address): address acquires VestingContract {
        assert_vesting_contract_exists(vesting_contract_address);
        borrow_global<VestingContract>(vesting_contract_address).staking.operator
    }

    #[view]
    /// Return the voter who will be voting on on-chain governance proposals on behalf of the vesting contract's stake
    /// pool.
    ///
    /// This errors out if the vesting contract with the provided address doesn't exist.
    public fun voter(vesting_contract_address: address): address acquires VestingContract {
        assert_vesting_contract_exists(vesting_contract_address);
        borrow_global<VestingContract>(vesting_contract_address).staking.voter
    }

    #[view]
    /// Return the vesting contract's vesting schedule. The core schedule is represented as a list of u64-based
    /// fractions, where the rightmmost 32 bits can be divided by 2^32 to get the fraction, and anything else is the
    /// whole number.
    ///
    /// For example 3/48, or 0.0625, will be represented as 268435456. The fractional portion would be
    /// 268435456 / 2^32 = 0.0625. Since there are fewer than 32 bits, the whole number portion is effectively 0.
    /// So 268435456 = 0.0625.
    ///
    /// This errors out if the vesting contract with the provided address doesn't exist.
    public fun vesting_schedule(vesting_contract_address: address): VestingSchedule acquires VestingContract {
        assert_vesting_contract_exists(vesting_contract_address);
        borrow_global<VestingContract>(vesting_contract_address).vesting_schedule
    }

    #[view]
    /// Return the total accumulated rewards that have not been distributed to shareholders of the vesting contract.
    /// This excludes any unpaid commission that the operator has not collected.
    ///
    /// This errors out if the vesting contract with the provided address doesn't exist.
    public fun total_accumulated_rewards(vesting_contract_address: address): u64 acquires VestingContract {
        assert_active_vesting_contract(vesting_contract_address);

        let vesting_contract = borrow_global<VestingContract>(vesting_contract_address);
        let (total_active_stake, _, commission_amount) =
            staking_contract::staking_contract_amounts(vesting_contract_address, vesting_contract.staking.operator);
        total_active_stake - vesting_contract.remaining_grant - commission_amount
    }

    #[view]
    /// Return the accumulated rewards that have not been distributed to the provided shareholder. Caller can also pass
    /// the beneficiary address instead of shareholder address.
    ///
    /// This errors out if the vesting contract with the provided address doesn't exist.
    public fun accumulated_rewards(
        vesting_contract_address: address, shareholder_or_beneficiary: address): u64 acquires VestingContract {
        assert_active_vesting_contract(vesting_contract_address);

        let total_accumulated_rewards = total_accumulated_rewards(vesting_contract_address);
        let shareholder = shareholder(vesting_contract_address, shareholder_or_beneficiary);
        let vesting_contract = borrow_global<VestingContract>(vesting_contract_address);
        let shares = pool_u64::shares(&vesting_contract.grant_pool, shareholder);
        pool_u64::shares_to_amount_with_total_coins(&vesting_contract.grant_pool, shares, total_accumulated_rewards)
    }

    #[view]
    /// Return the list of all shareholders in the vesting contract.
    public fun shareholders(vesting_contract_address: address): vector<address> acquires VestingContract {
        assert_active_vesting_contract(vesting_contract_address);

        let vesting_contract = borrow_global<VestingContract>(vesting_contract_address);
        pool_u64::shareholders(&vesting_contract.grant_pool)
    }

    #[view]
    /// Return the shareholder address given the beneficiary address in a given vesting contract. If there are multiple
    /// shareholders with the same beneficiary address, only the first shareholder is returned. If the given beneficiary
    /// address is actually a shareholder address, just return the address back.
    ///
    /// This returns 0x0 if no shareholder is found for the given beneficiary / the address is not a shareholder itself.
    public fun shareholder(
        vesting_contract_address: address,
        shareholder_or_beneficiary: address
    ): address acquires VestingContract {
        assert_active_vesting_contract(vesting_contract_address);

        let shareholders = &shareholders(vesting_contract_address);
        if (vector::contains(shareholders, &shareholder_or_beneficiary)) {
            return shareholder_or_beneficiary
        };
        let vesting_contract = borrow_global<VestingContract>(vesting_contract_address);
        let result = @0x0;
        vector::any(shareholders, |shareholder| {
            if (shareholder_or_beneficiary == get_beneficiary(vesting_contract, *shareholder)) {
                result = *shareholder;
                true
            } else {
                false
            }
        });

        result
    }

    /// Create a vesting schedule with the given schedule of distributions, a vesting start time and period duration.
    public fun create_vesting_schedule(
        schedule: vector<FixedPoint32>,
        start_timestamp_secs: u64,
        period_duration: u64,
    ): VestingSchedule {
        assert!(vector::length(&schedule) > 0, error::invalid_argument(EEMPTY_VESTING_SCHEDULE));
        assert!(period_duration > 0, error::invalid_argument(EZERO_VESTING_SCHEDULE_PERIOD));
        assert!(
            start_timestamp_secs >= timestamp::now_seconds(),
            error::invalid_argument(EVESTING_START_TOO_SOON),
        );

        VestingSchedule {
            schedule,
            start_timestamp_secs,
            period_duration,
            last_vested_period: 0,
        }
    }

    /// Create a vesting contract with a given configurations.
    public fun create_vesting_contract(
        admin: &signer,
        shareholders: &vector<address>,
        buy_ins: SimpleMap<address, Coin<AptosCoin>>,
        vesting_schedule: VestingSchedule,
        withdrawal_address: address,
        operator: address,
        voter: address,
        commission_percentage: u64,
        // Optional seed used when creating the staking contract account.
        contract_creation_seed: vector<u8>,
    ): address acquires AdminStore {
        assert!(
            !system_addresses::is_reserved_address(withdrawal_address),
            error::invalid_argument(EINVALID_WITHDRAWAL_ADDRESS),
        );
        assert_account_is_registered_for_apt(withdrawal_address);
        assert!(vector::length(shareholders) > 0, error::invalid_argument(ENO_SHAREHOLDERS));
        assert!(
            simple_map::length(&buy_ins) == vector::length(shareholders),
            error::invalid_argument(ESHARES_LENGTH_MISMATCH),
        );

        // Create a coins pool to track shareholders and shares of the grant.
        let grant = coin::zero<AptosCoin>();
        let grant_amount = 0;
        let grant_pool = pool_u64::create(MAXIMUM_SHAREHOLDERS);
        vector::for_each_ref(shareholders, |shareholder| {
            let shareholder: address = *shareholder;
            let (_, buy_in) = simple_map::remove(&mut buy_ins, &shareholder);
            let buy_in_amount = coin::value(&buy_in);
            coin::merge(&mut grant, buy_in);
            pool_u64::buy_in(
                &mut grant_pool,
                shareholder,
                buy_in_amount,
            );
            grant_amount = grant_amount + buy_in_amount;
        });
        assert!(grant_amount > 0, error::invalid_argument(EZERO_GRANT));

        // If this is the first time this admin account has created a vesting contract, initialize the admin store.
        let admin_address = signer::address_of(admin);
        if (!exists<AdminStore>(admin_address)) {
            move_to(admin, AdminStore {
                vesting_contracts: vector::empty<address>(),
                nonce: 0,
                create_events: new_event_handle<CreateVestingContractEvent>(admin),
            });
        };

        // Initialize the vesting contract in a new resource account. This allows the same admin to create multiple
        // pools.
        let (contract_signer, contract_signer_cap) = create_vesting_contract_account(admin, contract_creation_seed);
        let pool_address = staking_contract::create_staking_contract_with_coins(
            &contract_signer, operator, voter, grant, commission_percentage, contract_creation_seed);

        // Add the newly created vesting contract's address to the admin store.
        let contract_address = signer::address_of(&contract_signer);
        let admin_store = borrow_global_mut<AdminStore>(admin_address);
        vector::push_back(&mut admin_store.vesting_contracts, contract_address);
        if (std::features::module_event_migration_enabled()) {
            emit(
                CreateVestingContract {
                    operator,
                    voter,
                    withdrawal_address,
                    grant_amount,
                    vesting_contract_address: contract_address,
                    staking_pool_address: pool_address,
                    commission_percentage,
                },
            );
        };
        emit_event(
            &mut admin_store.create_events,
            CreateVestingContractEvent {
                operator,
                voter,
                withdrawal_address,
                grant_amount,
                vesting_contract_address: contract_address,
                staking_pool_address: pool_address,
                commission_percentage,
            },
        );

        move_to(&contract_signer, VestingContract {
            state: VESTING_POOL_ACTIVE,
            admin: admin_address,
            grant_pool,
            beneficiaries: simple_map::create<address, address>(),
            vesting_schedule,
            withdrawal_address,
            staking: StakingInfo { pool_address, operator, voter, commission_percentage },
            remaining_grant: grant_amount,
            signer_cap: contract_signer_cap,
            update_operator_events: new_event_handle<UpdateOperatorEvent>(&contract_signer),
            update_voter_events: new_event_handle<UpdateVoterEvent>(&contract_signer),
            reset_lockup_events: new_event_handle<ResetLockupEvent>(&contract_signer),
            set_beneficiary_events: new_event_handle<SetBeneficiaryEvent>(&contract_signer),
            unlock_rewards_events: new_event_handle<UnlockRewardsEvent>(&contract_signer),
            vest_events: new_event_handle<VestEvent>(&contract_signer),
            distribute_events: new_event_handle<DistributeEvent>(&contract_signer),
            terminate_events: new_event_handle<TerminateEvent>(&contract_signer),
            admin_withdraw_events: new_event_handle<AdminWithdrawEvent>(&contract_signer),
        });

        simple_map::destroy_empty(buy_ins);
        contract_address
    }

    /// Unlock any accumulated rewards.
    public entry fun unlock_rewards(contract_address: address) acquires VestingContract {
        let accumulated_rewards = total_accumulated_rewards(contract_address);
        let vesting_contract = borrow_global<VestingContract>(contract_address);
        unlock_stake(vesting_contract, accumulated_rewards);
    }

    /// Call `unlock_rewards` for many vesting contracts.
    public entry fun unlock_rewards_many(contract_addresses: vector<address>) acquires VestingContract {
        let len = vector::length(&contract_addresses);

        assert!(len != 0, error::invalid_argument(EVEC_EMPTY_FOR_MANY_FUNCTION));

        vector::for_each_ref(&contract_addresses, |contract_address| {
            let contract_address: address = *contract_address;
            unlock_rewards(contract_address);
        });
    }

    /// Unlock any vested portion of the grant.
    public entry fun vest(contract_address: address) acquires VestingContract {
        // Unlock all rewards first, if any.
        unlock_rewards(contract_address);

        // Unlock the vested amount. This amount will become withdrawable when the underlying stake pool's lockup
        // expires.
        let vesting_contract = borrow_global_mut<VestingContract>(contract_address);
        // Short-circuit if vesting hasn't started yet.
        if (vesting_contract.vesting_schedule.start_timestamp_secs > timestamp::now_seconds()) {
            return
        };

        // Check if the next vested period has already passed. If not, short-circuit since there's nothing to vest.
        let vesting_schedule = &mut vesting_contract.vesting_schedule;
        let last_vested_period = vesting_schedule.last_vested_period;
        let next_period_to_vest = last_vested_period + 1;
        let last_completed_period =
            (timestamp::now_seconds() - vesting_schedule.start_timestamp_secs) / vesting_schedule.period_duration;
        if (last_completed_period < next_period_to_vest) {
            return
        };

        // Calculate how much has vested, excluding rewards.
        // Index is 0-based while period is 1-based so we need to subtract 1.
        let schedule = &vesting_schedule.schedule;
        let schedule_index = next_period_to_vest - 1;
        let vesting_fraction = if (schedule_index < vector::length(schedule)) {
            *vector::borrow(schedule, schedule_index)
        } else {
            // Last vesting schedule fraction will repeat until the grant runs out.
            *vector::borrow(schedule, vector::length(schedule) - 1)
        };
        let total_grant = pool_u64::total_coins(&vesting_contract.grant_pool);
        let vested_amount = fixed_point32::multiply_u64(total_grant, vesting_fraction);
        // Cap vested amount by the remaining grant amount so we don't try to distribute more than what's remaining.
        vested_amount = min(vested_amount, vesting_contract.remaining_grant);
        vesting_contract.remaining_grant = vesting_contract.remaining_grant - vested_amount;
        vesting_schedule.last_vested_period = next_period_to_vest;
        unlock_stake(vesting_contract, vested_amount);

        if (std::features::module_event_migration_enabled()) {
            emit(
                Vest {
                    admin: vesting_contract.admin,
                    vesting_contract_address: contract_address,
                    staking_pool_address: vesting_contract.staking.pool_address,
                    period_vested: next_period_to_vest,
                    amount: vested_amount,
                },
            );
        };
        emit_event(
            &mut vesting_contract.vest_events,
            VestEvent {
                admin: vesting_contract.admin,
                vesting_contract_address: contract_address,
                staking_pool_address: vesting_contract.staking.pool_address,
                period_vested: next_period_to_vest,
                amount: vested_amount,
            },
        );
    }

    /// Call `vest` for many vesting contracts.
    public entry fun vest_many(contract_addresses: vector<address>) acquires VestingContract {
        let len = vector::length(&contract_addresses);

        assert!(len != 0, error::invalid_argument(EVEC_EMPTY_FOR_MANY_FUNCTION));

        vector::for_each_ref(&contract_addresses, |contract_address| {
            let contract_address = *contract_address;
            vest(contract_address);
        });
    }

    /// Distribute any withdrawable stake from the stake pool.
    public entry fun distribute(contract_address: address) acquires VestingContract {
        assert_active_vesting_contract(contract_address);

        let vesting_contract = borrow_global_mut<VestingContract>(contract_address);
        let coins = withdraw_stake(vesting_contract, contract_address);
        let total_distribution_amount = coin::value(&coins);
        if (total_distribution_amount == 0) {
            coin::destroy_zero(coins);
            return
        };

        // Distribute coins to all shareholders in the vesting contract.
        let grant_pool = &vesting_contract.grant_pool;
        let shareholders = &pool_u64::shareholders(grant_pool);
        vector::for_each_ref(shareholders, |shareholder| {
            let shareholder = *shareholder;
            let shares = pool_u64::shares(grant_pool, shareholder);
            let amount = pool_u64::shares_to_amount_with_total_coins(grant_pool, shares, total_distribution_amount);
            let share_of_coins = coin::extract(&mut coins, amount);
            let recipient_address = get_beneficiary(vesting_contract, shareholder);
            aptos_account::deposit_coins(recipient_address, share_of_coins);
        });

        // Send any remaining "dust" (leftover due to rounding error) to the withdrawal address.
        if (coin::value(&coins) > 0) {
            aptos_account::deposit_coins(vesting_contract.withdrawal_address, coins);
        } else {
            coin::destroy_zero(coins);
        };

        if (std::features::module_event_migration_enabled()) {
            emit(
                Distribute {
                    admin: vesting_contract.admin,
                    vesting_contract_address: contract_address,
                    amount: total_distribution_amount,
                },
            );
        };
        emit_event(
            &mut vesting_contract.distribute_events,
            DistributeEvent {
                admin: vesting_contract.admin,
                vesting_contract_address: contract_address,
                amount: total_distribution_amount,
            },
        );
    }

    /// Call `distribute` for many vesting contracts.
    public entry fun distribute_many(contract_addresses: vector<address>) acquires VestingContract {
        let len = vector::length(&contract_addresses);

        assert!(len != 0, error::invalid_argument(EVEC_EMPTY_FOR_MANY_FUNCTION));

        vector::for_each_ref(&contract_addresses, |contract_address| {
            let contract_address = *contract_address;
            distribute(contract_address);
        });
    }

    /// Terminate the vesting contract and send all funds back to the withdrawal address.
    public entry fun terminate_vesting_contract(admin: &signer, contract_address: address) acquires VestingContract {
        assert_active_vesting_contract(contract_address);

        // Distribute all withdrawable coins, which should have been from previous rewards withdrawal or vest.
        distribute(contract_address);

        let vesting_contract = borrow_global_mut<VestingContract>(contract_address);
        verify_admin(admin, vesting_contract);
        let (active_stake, _, pending_active_stake, _) = stake::get_stake(vesting_contract.staking.pool_address);
        assert!(pending_active_stake == 0, error::invalid_state(EPENDING_STAKE_FOUND));

        // Unlock all remaining active stake.
        vesting_contract.state = VESTING_POOL_TERMINATED;
        vesting_contract.remaining_grant = 0;
        unlock_stake(vesting_contract, active_stake);

        if (std::features::module_event_migration_enabled()) {
            emit(
                Terminate {
                    admin: vesting_contract.admin,
                    vesting_contract_address: contract_address,
                },
            );
        };
        emit_event(
            &mut vesting_contract.terminate_events,
            TerminateEvent {
                admin: vesting_contract.admin,
                vesting_contract_address: contract_address,
            },
        );
    }

    /// Withdraw all funds to the preset vesting contract's withdrawal address. This can only be called if the contract
    /// has already been terminated.
    public entry fun admin_withdraw(admin: &signer, contract_address: address) acquires VestingContract {
        let vesting_contract = borrow_global<VestingContract>(contract_address);
        assert!(
            vesting_contract.state == VESTING_POOL_TERMINATED,
            error::invalid_state(EVESTING_CONTRACT_STILL_ACTIVE)
        );

        let vesting_contract = borrow_global_mut<VestingContract>(contract_address);
        verify_admin(admin, vesting_contract);
        let coins = withdraw_stake(vesting_contract, contract_address);
        let amount = coin::value(&coins);
        if (amount == 0) {
            coin::destroy_zero(coins);
            return
        };
        aptos_account::deposit_coins(vesting_contract.withdrawal_address, coins);

        if (std::features::module_event_migration_enabled()) {
            emit(
                AdminWithdraw {
                    admin: vesting_contract.admin,
                    vesting_contract_address: contract_address,
                    amount,
                },
            );
        };
        emit_event(
            &mut vesting_contract.admin_withdraw_events,
            AdminWithdrawEvent {
                admin: vesting_contract.admin,
                vesting_contract_address: contract_address,
                amount,
            },
        );
    }

    public entry fun update_operator(
        admin: &signer,
        contract_address: address,
        new_operator: address,
        commission_percentage: u64,
    ) acquires VestingContract {
        let vesting_contract = borrow_global_mut<VestingContract>(contract_address);
        verify_admin(admin, vesting_contract);
        let contract_signer = &get_vesting_account_signer_internal(vesting_contract);
        let old_operator = vesting_contract.staking.operator;
        staking_contract::switch_operator(contract_signer, old_operator, new_operator, commission_percentage);
        vesting_contract.staking.operator = new_operator;
        vesting_contract.staking.commission_percentage = commission_percentage;

        if (std::features::module_event_migration_enabled()) {
            emit(
                UpdateOperator {
                    admin: vesting_contract.admin,
                    vesting_contract_address: contract_address,
                    staking_pool_address: vesting_contract.staking.pool_address,
                    old_operator,
                    new_operator,
                    commission_percentage,
                },
            );
        };
        emit_event(
            &mut vesting_contract.update_operator_events,
            UpdateOperatorEvent {
                admin: vesting_contract.admin,
                vesting_contract_address: contract_address,
                staking_pool_address: vesting_contract.staking.pool_address,
                old_operator,
                new_operator,
                commission_percentage,
            },
        );
    }

    public entry fun update_operator_with_same_commission(
        admin: &signer,
        contract_address: address,
        new_operator: address,
    ) acquires VestingContract {
        let commission_percentage = operator_commission_percentage(contract_address);
        update_operator(admin, contract_address, new_operator, commission_percentage);
    }

    public entry fun update_commission_percentage(
        admin: &signer,
        contract_address: address,
        new_commission_percentage: u64,
    ) acquires VestingContract {
        let operator = operator(contract_address);
        let vesting_contract = borrow_global_mut<VestingContract>(contract_address);
        verify_admin(admin, vesting_contract);
        let contract_signer = &get_vesting_account_signer_internal(vesting_contract);
        staking_contract::update_commision(contract_signer, operator, new_commission_percentage);
        vesting_contract.staking.commission_percentage = new_commission_percentage;
        // This function does not emit an event. Instead, `staking_contract::update_commission_percentage`
        // emits the event for this commission percentage update.
    }

    public entry fun update_voter(
        admin: &signer,
        contract_address: address,
        new_voter: address,
    ) acquires VestingContract {
        let vesting_contract = borrow_global_mut<VestingContract>(contract_address);
        verify_admin(admin, vesting_contract);
        let contract_signer = &get_vesting_account_signer_internal(vesting_contract);
        let old_voter = vesting_contract.staking.voter;
        staking_contract::update_voter(contract_signer, vesting_contract.staking.operator, new_voter);
        vesting_contract.staking.voter = new_voter;

        if (std::features::module_event_migration_enabled()) {
            emit(
                UpdateVoter {
                    admin: vesting_contract.admin,
                    vesting_contract_address: contract_address,
                    staking_pool_address: vesting_contract.staking.pool_address,
                    old_voter,
                    new_voter,
                },
            );
        };
        emit_event(
            &mut vesting_contract.update_voter_events,
            UpdateVoterEvent {
                admin: vesting_contract.admin,
                vesting_contract_address: contract_address,
                staking_pool_address: vesting_contract.staking.pool_address,
                old_voter,
                new_voter,
            },
        );
    }

    public entry fun reset_lockup(
        admin: &signer,
        contract_address: address,
    ) acquires VestingContract {
        let vesting_contract = borrow_global_mut<VestingContract>(contract_address);
        verify_admin(admin, vesting_contract);
        let contract_signer = &get_vesting_account_signer_internal(vesting_contract);
        staking_contract::reset_lockup(contract_signer, vesting_contract.staking.operator);

        if (std::features::module_event_migration_enabled()) {
            emit(
                ResetLockup {
                    admin: vesting_contract.admin,
                    vesting_contract_address: contract_address,
                    staking_pool_address: vesting_contract.staking.pool_address,
                    new_lockup_expiration_secs: stake::get_lockup_secs(vesting_contract.staking.pool_address),
                },
            );
        };
        emit_event(
            &mut vesting_contract.reset_lockup_events,
            ResetLockupEvent {
                admin: vesting_contract.admin,
                vesting_contract_address: contract_address,
                staking_pool_address: vesting_contract.staking.pool_address,
                new_lockup_expiration_secs: stake::get_lockup_secs(vesting_contract.staking.pool_address),
            },
        );
    }

    public entry fun set_beneficiary(
        admin: &signer,
        contract_address: address,
        shareholder: address,
        new_beneficiary: address,
    ) acquires VestingContract {
        // Verify that the beneficiary account is set up to receive APT. This is a requirement so distribute() wouldn't
        // fail and block all other accounts from receiving APT if one beneficiary is not registered.
        assert_account_is_registered_for_apt(new_beneficiary);

        let vesting_contract = borrow_global_mut<VestingContract>(contract_address);
        verify_admin(admin, vesting_contract);

        let old_beneficiary = get_beneficiary(vesting_contract, shareholder);
        let beneficiaries = &mut vesting_contract.beneficiaries;
        if (simple_map::contains_key(beneficiaries, &shareholder)) {
            let beneficiary = simple_map::borrow_mut(beneficiaries, &shareholder);
            *beneficiary = new_beneficiary;
        } else {
            simple_map::add(beneficiaries, shareholder, new_beneficiary);
        };

        if (std::features::module_event_migration_enabled()) {
            emit(
                SetBeneficiary {
                    admin: vesting_contract.admin,
                    vesting_contract_address: contract_address,
                    shareholder,
                    old_beneficiary,
                    new_beneficiary,
                },
            );
        };
        emit_event(
            &mut vesting_contract.set_beneficiary_events,
            SetBeneficiaryEvent {
                admin: vesting_contract.admin,
                vesting_contract_address: contract_address,
                shareholder,
                old_beneficiary,
                new_beneficiary,
            },
        );
    }

    /// Remove the beneficiary for the given shareholder. All distributions will sent directly to the shareholder
    /// account.
    public entry fun reset_beneficiary(
        account: &signer,
        contract_address: address,
        shareholder: address,
    ) acquires VestingAccountManagement, VestingContract {
        let vesting_contract = borrow_global_mut<VestingContract>(contract_address);
        let addr = signer::address_of(account);
        assert!(
            addr == vesting_contract.admin ||
                addr == get_role_holder(contract_address, utf8(ROLE_BENEFICIARY_RESETTER)),
            error::permission_denied(EPERMISSION_DENIED),
        );

        let beneficiaries = &mut vesting_contract.beneficiaries;
        if (simple_map::contains_key(beneficiaries, &shareholder)) {
            simple_map::remove(beneficiaries, &shareholder);
        };
    }

    public entry fun set_management_role(
        admin: &signer,
        contract_address: address,
        role: String,
        role_holder: address,
    ) acquires VestingAccountManagement, VestingContract {
        let vesting_contract = borrow_global_mut<VestingContract>(contract_address);
        verify_admin(admin, vesting_contract);

        if (!exists<VestingAccountManagement>(contract_address)) {
            let contract_signer = &get_vesting_account_signer_internal(vesting_contract);
            move_to(contract_signer, VestingAccountManagement {
                roles: simple_map::create<String, address>(),
            })
        };
        let roles = &mut borrow_global_mut<VestingAccountManagement>(contract_address).roles;
        if (simple_map::contains_key(roles, &role)) {
            *simple_map::borrow_mut(roles, &role) = role_holder;
        } else {
            simple_map::add(roles, role, role_holder);
        };
    }

    public entry fun set_beneficiary_resetter(
        admin: &signer,
        contract_address: address,
        beneficiary_resetter: address,
    ) acquires VestingAccountManagement, VestingContract {
        set_management_role(admin, contract_address, utf8(ROLE_BENEFICIARY_RESETTER), beneficiary_resetter);
    }

    /// Set the beneficiary for the operator.
    public entry fun set_beneficiary_for_operator(
        operator: &signer,
        new_beneficiary: address,
    ) {
        staking_contract::set_beneficiary_for_operator(operator, new_beneficiary);
    }

    public fun get_role_holder(contract_address: address, role: String): address acquires VestingAccountManagement {
        assert!(exists<VestingAccountManagement>(contract_address), error::not_found(EVESTING_ACCOUNT_HAS_NO_ROLES));
        let roles = &borrow_global<VestingAccountManagement>(contract_address).roles;
        assert!(simple_map::contains_key(roles, &role), error::not_found(EROLE_NOT_FOUND));
        *simple_map::borrow(roles, &role)
    }

    /// For emergency use in case the admin needs emergency control of vesting contract account.
    /// This doesn't give the admin total power as the admin would still need to follow the rules set by
    /// staking_contract and stake modules.
    public fun get_vesting_account_signer(admin: &signer, contract_address: address): signer acquires VestingContract {
        let vesting_contract = borrow_global_mut<VestingContract>(contract_address);
        verify_admin(admin, vesting_contract);
        get_vesting_account_signer_internal(vesting_contract)
    }

    fun get_vesting_account_signer_internal(vesting_contract: &VestingContract): signer {
        account::create_signer_with_capability(&vesting_contract.signer_cap)
    }

    /// Create a salt for generating the resource accounts that will be holding the VestingContract.
    /// This address should be deterministic for the same admin and vesting contract creation nonce.
    fun create_vesting_contract_account(
        admin: &signer,
        contract_creation_seed: vector<u8>,
    ): (signer, SignerCapability) acquires AdminStore {
        let admin_store = borrow_global_mut<AdminStore>(signer::address_of(admin));
        let seed = bcs::to_bytes(&signer::address_of(admin));
        vector::append(&mut seed, bcs::to_bytes(&admin_store.nonce));
        admin_store.nonce = admin_store.nonce + 1;

        // Include a salt to avoid conflicts with any other modules out there that might also generate
        // deterministic resource accounts for the same admin address + nonce.
        vector::append(&mut seed, VESTING_POOL_SALT);
        vector::append(&mut seed, contract_creation_seed);

        let (account_signer, signer_cap) = account::create_resource_account(admin, seed);
        // Register the vesting contract account to receive APT as it'll be sent to it when claiming unlocked stake from
        // the underlying staking contract.
        coin::register<AptosCoin>(&account_signer);

        (account_signer, signer_cap)
    }

    fun verify_admin(admin: &signer, vesting_contract: &VestingContract) {
        assert!(signer::address_of(admin) == vesting_contract.admin, error::unauthenticated(ENOT_ADMIN));
    }

    fun assert_vesting_contract_exists(contract_address: address) {
        assert!(exists<VestingContract>(contract_address), error::not_found(EVESTING_CONTRACT_NOT_FOUND));
    }

    fun assert_active_vesting_contract(contract_address: address) acquires VestingContract {
        assert_vesting_contract_exists(contract_address);
        let vesting_contract = borrow_global<VestingContract>(contract_address);
        assert!(vesting_contract.state == VESTING_POOL_ACTIVE, error::invalid_state(EVESTING_CONTRACT_NOT_ACTIVE));
    }

    fun unlock_stake(vesting_contract: &VestingContract, amount: u64) {
        let contract_signer = &get_vesting_account_signer_internal(vesting_contract);
        staking_contract::unlock_stake(contract_signer, vesting_contract.staking.operator, amount);
    }

    fun withdraw_stake(vesting_contract: &VestingContract, contract_address: address): Coin<AptosCoin> {
        // Claim any withdrawable distribution from the staking contract. The withdrawn coins will be sent directly to
        // the vesting contract's account.
        staking_contract::distribute(contract_address, vesting_contract.staking.operator);
        let withdrawn_coins = coin::balance<AptosCoin>(contract_address);
        let contract_signer = &get_vesting_account_signer_internal(vesting_contract);
        coin::withdraw<AptosCoin>(contract_signer, withdrawn_coins)
    }

    fun get_beneficiary(contract: &VestingContract, shareholder: address): address {
        if (simple_map::contains_key(&contract.beneficiaries, &shareholder)) {
            *simple_map::borrow(&contract.beneficiaries, &shareholder)
        } else {
            shareholder
        }
    }

    #[test_only]
    use aptos_framework::stake::with_rewards;

    #[test_only]
    use aptos_framework::account::create_account_for_test;
    use aptos_std::math64::min;

    #[test_only]
    const MIN_STAKE: u64 = 100000000000000; // 1M APT coins with 8 decimals.

    #[test_only]
    const GRANT_AMOUNT: u64 = 20000000000000000; // 200M APT coins with 8 decimals.

    #[test_only]
    const VESTING_SCHEDULE_CLIFF: u64 = 31536000; // 1 year

    #[test_only]
    const VESTING_PERIOD: u64 = 2592000; // 30 days

    #[test_only]
    const VALIDATOR_STATUS_PENDING_ACTIVE: u64 = 1;
    #[test_only]
    const VALIDATOR_STATUS_ACTIVE: u64 = 2;
    #[test_only]
    const VALIDATOR_STATUS_INACTIVE: u64 = 4;

    #[test_only]
    const MODULE_EVENT: u64 = 26;

    #[test_only]
    const OPERATOR_BENEFICIARY_CHANGE: u64 = 39;

    #[test_only]
    public fun setup(aptos_framework: &signer, accounts: &vector<address>) {
        use aptos_framework::aptos_account::create_account;

        stake::initialize_for_test_custom(
            aptos_framework,
            MIN_STAKE,
            GRANT_AMOUNT * 10,
            3600,
            true,
            10,
            10000,
            1000000
        );

        vector::for_each_ref(accounts, |addr| {
            let addr: address = *addr;
            if (!account::exists_at(addr)) {
                create_account(addr);
            };
        });

        std::features::change_feature_flags_for_testing(aptos_framework, vector[MODULE_EVENT, OPERATOR_BENEFICIARY_CHANGE], vector[]);
    }

    #[test_only]
    public fun setup_vesting_contract(
        admin: &signer,
        shareholders: &vector<address>,
        shares: &vector<u64>,
        withdrawal_address: address,
        commission_percentage: u64,
    ): address acquires AdminStore {
        setup_vesting_contract_with_schedule(
            admin,
            shareholders,
            shares,
            withdrawal_address,
            commission_percentage,
            &vector[3, 2, 1],
            48,
        )
    }

    #[test_only]
    public fun setup_vesting_contract_with_schedule(
        admin: &signer,
        shareholders: &vector<address>,
        shares: &vector<u64>,
        withdrawal_address: address,
        commission_percentage: u64,
        vesting_numerators: &vector<u64>,
        vesting_denominator: u64,
    ): address acquires AdminStore {
        let schedule = vector::empty<FixedPoint32>();
        vector::for_each_ref(vesting_numerators, |num| {
            vector::push_back(&mut schedule, fixed_point32::create_from_rational(*num, vesting_denominator));
        });
        let vesting_schedule = create_vesting_schedule(
            schedule,
            timestamp::now_seconds() + VESTING_SCHEDULE_CLIFF,
            VESTING_PERIOD,
        );

        let admin_address = signer::address_of(admin);
        let buy_ins = simple_map::create<address, Coin<AptosCoin>>();
        vector::enumerate_ref(shares, |i, share| {
            let shareholder = *vector::borrow(shareholders, i);
            simple_map::add(&mut buy_ins, shareholder, stake::mint_coins(*share));
        });

        create_vesting_contract(
            admin,
            shareholders,
            buy_ins,
            vesting_schedule,
            withdrawal_address,
            admin_address,
            admin_address,
            commission_percentage,
            vector[],
        )
    }

    #[test(aptos_framework = @0x1, admin = @0x123, shareholder_1 = @0x234, shareholder_2 = @0x345, withdrawal = @111)]
    public entry fun test_end_to_end(
        aptos_framework: &signer,
        admin: &signer,
        shareholder_1: &signer,
        shareholder_2: &signer,
        withdrawal: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let withdrawal_address = signer::address_of(withdrawal);
        let shareholder_1_address = signer::address_of(shareholder_1);
        let shareholder_2_address = signer::address_of(shareholder_2);
        let shareholders = &vector[shareholder_1_address, shareholder_2_address];
        let shareholder_1_share = GRANT_AMOUNT / 4;
        let shareholder_2_share = GRANT_AMOUNT * 3 / 4;
        let shares = &vector[shareholder_1_share, shareholder_2_share];

        // Create the vesting contract.
        setup(
            aptos_framework, &vector[admin_address, withdrawal_address, shareholder_1_address, shareholder_2_address]);
        let contract_address = setup_vesting_contract(admin, shareholders, shares, withdrawal_address, 0);
        assert!(vector::length(&borrow_global<AdminStore>(admin_address).vesting_contracts) == 1, 0);
        let stake_pool_address = stake_pool_address(contract_address);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, 0);

        // The stake pool is still in pending active stake, so unlock_rewards and vest shouldn't do anything.
        let (_sk, pk, pop) = stake::generate_identity();
        stake::join_validator_set_for_test(&pk, &pop, admin, stake_pool_address, false);
        assert!(stake::get_validator_state(stake_pool_address) == VALIDATOR_STATUS_PENDING_ACTIVE, 1);
        unlock_rewards(contract_address);
        vest(contract_address);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, 0);

        // Wait for the validator to join the validator set. No rewards are earnt yet so unlock_rewards and vest should
        // still do nothing.
        stake::end_epoch();
        assert!(stake::get_validator_state(stake_pool_address) == VALIDATOR_STATUS_ACTIVE, 2);
        unlock_rewards(contract_address);
        vest(contract_address);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, 0);

        // Stake pool earns some rewards. unlock_rewards should unlock the right amount.
        stake::end_epoch();
        let rewards = get_accumulated_rewards(contract_address);
        unlock_rewards(contract_address);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, rewards);
        assert!(remaining_grant(contract_address) == GRANT_AMOUNT, 0);

        // Stake pool earns more rewards. vest should unlock the rewards but no vested tokens as vesting hasn't started.
        stake::end_epoch();
        rewards = with_rewards(rewards); // Pending inactive stake still earns rewards.
        rewards = rewards + get_accumulated_rewards(contract_address);
        vest(contract_address);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, rewards);
        assert!(remaining_grant(contract_address) == GRANT_AMOUNT, 0);

        // Fast forward to stake lockup expiration so rewards are fully unlocked.
        // In the mean time, rewards still earn rewards.
        // Calling distribute() should send rewards to the shareholders.
        stake::fast_forward_to_unlock(stake_pool_address);
        rewards = with_rewards(rewards);
        distribute(contract_address);
        let shareholder_1_bal = coin::balance<AptosCoin>(shareholder_1_address);
        let shareholder_2_bal = coin::balance<AptosCoin>(shareholder_2_address);
        // Distribution goes by the shares of the vesting contract.
        assert!(shareholder_1_bal == rewards / 4, shareholder_1_bal);
        assert!(shareholder_2_bal == rewards * 3 / 4, shareholder_2_bal);

        // Fast forward time to the vesting start.
        timestamp::update_global_time_for_test_secs(vesting_start_secs(contract_address));
        // Calling vest only unlocks rewards but not any vested token as the first vesting period hasn't passed yet.
        rewards = get_accumulated_rewards(contract_address);
        vest(contract_address);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, rewards);
        assert!(remaining_grant(contract_address) == GRANT_AMOUNT, 0);

        // Fast forward to the end of the first period. vest() should now unlock 3/48 of the tokens.
        timestamp::fast_forward_seconds(VESTING_PERIOD);
        vest(contract_address);
        let vested_amount = fraction(GRANT_AMOUNT, 3, 48);
        let remaining_grant = GRANT_AMOUNT - vested_amount;
        let pending_distribution = rewards + vested_amount;
        assert!(remaining_grant(contract_address) == remaining_grant, remaining_grant(contract_address));
        stake::assert_stake_pool(stake_pool_address, remaining_grant, 0, 0, pending_distribution);

        // Fast forward to the end of the fourth period. We can call vest() 3 times to vest the last 3 periods.
        timestamp::fast_forward_seconds(VESTING_PERIOD * 3);
        vest(contract_address);
        vested_amount = fraction(GRANT_AMOUNT, 2, 48);
        remaining_grant = remaining_grant - vested_amount;
        pending_distribution = pending_distribution + vested_amount;
        stake::assert_stake_pool(stake_pool_address, remaining_grant, 0, 0, pending_distribution);
        vest(contract_address);
        vested_amount = fraction(GRANT_AMOUNT, 1, 48);
        remaining_grant = remaining_grant - vested_amount;
        pending_distribution = pending_distribution + vested_amount;
        stake::assert_stake_pool(stake_pool_address, remaining_grant, 0, 0, pending_distribution);
        // The last vesting fraction (1/48) is repeated beyond the first 3 periods.
        vest(contract_address);
        remaining_grant = remaining_grant - vested_amount;
        pending_distribution = pending_distribution + vested_amount;
        stake::assert_stake_pool(stake_pool_address, remaining_grant, 0, 0, pending_distribution);
        assert!(remaining_grant(contract_address) == remaining_grant, 0);

        stake::end_epoch();
        let total_active = with_rewards(remaining_grant);
        pending_distribution = with_rewards(pending_distribution);
        distribute(contract_address);
        stake::assert_stake_pool(stake_pool_address, total_active, 0, 0, 0);
        assert!(coin::balance<AptosCoin>(shareholder_1_address) == shareholder_1_bal + pending_distribution / 4, 0);
        assert!(coin::balance<AptosCoin>(shareholder_2_address) == shareholder_2_bal + pending_distribution * 3 / 4, 1);
        // Withdrawal address receives the left-over dust of 1 coin due to rounding error.
        assert!(coin::balance<AptosCoin>(withdrawal_address) == 1, 0);

        // Admin terminates the vesting contract.
        terminate_vesting_contract(admin, contract_address);
        stake::assert_stake_pool(stake_pool_address, 0, 0, 0, total_active);
        assert!(remaining_grant(contract_address) == 0, 0);
        stake::fast_forward_to_unlock(stake_pool_address);
        let withdrawn_amount = with_rewards(total_active);
        stake::assert_stake_pool(stake_pool_address, 0, withdrawn_amount, 0, 0);
        let previous_bal = coin::balance<AptosCoin>(withdrawal_address);
        admin_withdraw(admin, contract_address);
        assert!(coin::balance<AptosCoin>(withdrawal_address) == previous_bal + withdrawn_amount, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    #[expected_failure(abort_code = 0x1000C, location = Self)]
    public entry fun test_create_vesting_contract_with_zero_grant_should_fail(
        aptos_framework: &signer,
        admin: &signer,
    ) acquires AdminStore {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address]);
        setup_vesting_contract(admin, &vector[@1], &vector[0], admin_address, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    #[expected_failure(abort_code = 0x10004, location = Self)]
    public entry fun test_create_vesting_contract_with_no_shareholders_should_fail(
        aptos_framework: &signer,
        admin: &signer,
    ) acquires AdminStore {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address]);
        setup_vesting_contract(admin, &vector[], &vector[], admin_address, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    #[expected_failure(abort_code = 0x10005, location = Self)]
    public entry fun test_create_vesting_contract_with_mistmaching_shareholders_should_fail(
        aptos_framework: &signer,
        admin: &signer,
    ) acquires AdminStore {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address]);
        setup_vesting_contract(admin, &vector[@1, @2], &vector[1], admin_address, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    #[expected_failure(abort_code = 0x60001, location = aptos_framework::aptos_account)]
    public entry fun test_create_vesting_contract_with_invalid_withdrawal_address_should_fail(
        aptos_framework: &signer,
        admin: &signer,
    ) acquires AdminStore {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address]);
        setup_vesting_contract(admin, &vector[@1, @2], &vector[1], @5, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    #[expected_failure(abort_code = 0x60001, location = aptos_framework::aptos_account)]
    public entry fun test_create_vesting_contract_with_missing_withdrawal_account_should_fail(
        aptos_framework: &signer,
        admin: &signer,
    ) acquires AdminStore {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address]);
        setup_vesting_contract(admin, &vector[@1, @2], &vector[1], @11, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    #[expected_failure(abort_code = 0x60002, location = aptos_framework::aptos_account)]
    public entry fun test_create_vesting_contract_with_unregistered_withdrawal_account_should_fail(
        aptos_framework: &signer,
        admin: &signer,
    ) acquires AdminStore {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address]);
        create_account_for_test(@11);
        setup_vesting_contract(admin, &vector[@1, @2], &vector[1], @11, 0);
    }

    #[test(aptos_framework = @0x1)]
    #[expected_failure(abort_code = 0x10002, location = Self)]
    public entry fun test_create_empty_vesting_schedule_should_fail(aptos_framework: &signer) {
        setup(aptos_framework, &vector[]);
        create_vesting_schedule(vector[], 1, 1);
    }

    #[test(aptos_framework = @0x1)]
    #[expected_failure(abort_code = 0x10003, location = Self)]
    public entry fun test_create_vesting_schedule_with_zero_period_duration_should_fail(aptos_framework: &signer) {
        setup(aptos_framework, &vector[]);
        create_vesting_schedule(vector[fixed_point32::create_from_rational(1, 1)], 1, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    #[expected_failure(abort_code = 0x10006, location = Self)]
    public entry fun test_create_vesting_schedule_with_invalid_vesting_start_should_fail(aptos_framework: &signer) {
        setup(aptos_framework, &vector[]);
        timestamp::update_global_time_for_test_secs(1000);
        create_vesting_schedule(
            vector[fixed_point32::create_from_rational(1, 1)],
            900,
            1);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, shareholder = @0x234)]
    public entry fun test_vest_twice_should_not_double_count(
        aptos_framework: &signer,
        admin: &signer,
        shareholder: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let shareholder_address = signer::address_of(shareholder);
        setup(aptos_framework, &vector[admin_address, shareholder_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[shareholder_address], &vector[GRANT_AMOUNT], admin_address, 0);

        // Operator needs to join the validator set for the stake pool to earn rewards.
        let stake_pool_address = stake_pool_address(contract_address);
        let (_sk, pk, pop) = stake::generate_identity();
        stake::join_validator_set_for_test(&pk, &pop, admin, stake_pool_address, true);

        // Fast forward to the end of the first period. vest() should now unlock 3/48 of the tokens.
        timestamp::update_global_time_for_test_secs(vesting_start_secs(contract_address) + VESTING_PERIOD);
        vest(contract_address);
        let vested_amount = fraction(GRANT_AMOUNT, 3, 48);
        let remaining_grant = GRANT_AMOUNT - vested_amount;
        stake::assert_stake_pool(stake_pool_address, remaining_grant, 0, 0, vested_amount);
        assert!(remaining_grant(contract_address) == remaining_grant, 0);

        // Calling vest() a second time shouldn't change anything.
        vest(contract_address);
        stake::assert_stake_pool(stake_pool_address, remaining_grant, 0, 0, vested_amount);
        assert!(remaining_grant(contract_address) == remaining_grant, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, shareholder = @0x234)]
    public entry fun test_unlock_rewards_twice_should_not_double_count(
        aptos_framework: &signer,
        admin: &signer,
        shareholder: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let shareholder_address = signer::address_of(shareholder);
        setup(aptos_framework, &vector[admin_address, shareholder_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[shareholder_address], &vector[GRANT_AMOUNT], admin_address, 0);

        // Operator needs to join the validator set for the stake pool to earn rewards.
        let stake_pool_address = stake_pool_address(contract_address);
        let (_sk, pk, pop) = stake::generate_identity();
        stake::join_validator_set_for_test(&pk, &pop, admin, stake_pool_address, true);

        // Stake pool earns some rewards. unlock_rewards should unlock the right amount.
        stake::end_epoch();
        let rewards = get_accumulated_rewards(contract_address);
        unlock_rewards(contract_address);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, rewards);
        assert!(remaining_grant(contract_address) == GRANT_AMOUNT, 0);

        // Calling unlock_rewards a second time shouldn't change anything as no new rewards has accumulated.
        unlock_rewards(contract_address);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, rewards);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, shareholder = @0x234, operator = @0x345)]
    public entry fun test_unlock_rewards_should_pay_commission_first(
        aptos_framework: &signer,
        admin: &signer,
        shareholder: &signer,
        operator: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let operator_address = signer::address_of(operator);
        let shareholder_address = signer::address_of(shareholder);
        setup(aptos_framework, &vector[admin_address, shareholder_address, operator_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[shareholder_address], &vector[GRANT_AMOUNT], admin_address, 0);
        assert!(operator_commission_percentage(contract_address) == 0, 0);

        // 10% commission will be paid to the operator.
        update_operator(admin, contract_address, operator_address, 10);
        assert!(operator_commission_percentage(contract_address) == 10, 0);

        // Operator needs to join the validator set for the stake pool to earn rewards.
        let stake_pool_address = stake_pool_address(contract_address);
        let (_sk, pk, pop) = stake::generate_identity();
        stake::join_validator_set_for_test(&pk, &pop, operator, stake_pool_address, true);

        // Stake pool earns some rewards. unlock_rewards should unlock the right amount.
        stake::end_epoch();
        let accumulated_rewards = get_accumulated_rewards(contract_address);
        let commission = accumulated_rewards / 10; // 10%.
        let staker_rewards = accumulated_rewards - commission;
        unlock_rewards(contract_address);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, accumulated_rewards);
        assert!(remaining_grant(contract_address) == GRANT_AMOUNT, 0);

        // Distribution should pay commission to operator first and remaining amount to shareholders.
        stake::fast_forward_to_unlock(stake_pool_address);
        stake::assert_stake_pool(
            stake_pool_address,
            with_rewards(GRANT_AMOUNT),
            with_rewards(accumulated_rewards),
            0,
            0
        );
        // Operator also earns more commission from the rewards earnt on the withdrawn rewards.
        let commission_on_staker_rewards = (with_rewards(staker_rewards) - staker_rewards) / 10;
        staker_rewards = with_rewards(staker_rewards) - commission_on_staker_rewards;
        commission = with_rewards(commission) + commission_on_staker_rewards;
        distribute(contract_address);
        // Rounding error leads to a dust amount of 1 transferred to the staker.
        assert!(coin::balance<AptosCoin>(shareholder_address) == staker_rewards + 1, 0);
        assert!(coin::balance<AptosCoin>(operator_address) == commission - 1, 1);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, shareholder = @0x234, operator = @0x345)]
    public entry fun test_request_commission_should_not_lock_rewards_for_shareholders(
        aptos_framework: &signer,
        admin: &signer,
        shareholder: &signer,
        operator: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let operator_address = signer::address_of(operator);
        let shareholder_address = signer::address_of(shareholder);
        setup(aptos_framework, &vector[admin_address, shareholder_address, operator_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[shareholder_address], &vector[GRANT_AMOUNT], admin_address, 0);
        assert!(operator_commission_percentage(contract_address) == 0, 0);

        // 10% commission will be paid to the operator.
        update_operator(admin, contract_address, operator_address, 10);
        assert!(operator_commission_percentage(contract_address) == 10, 0);

        // Operator needs to join the validator set for the stake pool to earn rewards.
        let stake_pool_address = stake_pool_address(contract_address);
        let (_sk, pk, pop) = stake::generate_identity();
        stake::join_validator_set_for_test(&pk, &pop, operator, stake_pool_address, true);

        // Stake pool earns some rewards.
        stake::end_epoch();

        // Operator requests commission directly with staking_contract first.
        let accumulated_rewards = get_accumulated_rewards(contract_address);
        let commission = accumulated_rewards / 10; // 10%.
        let staker_rewards = accumulated_rewards - commission;
        staking_contract::request_commission(operator, contract_address, operator_address);

        // Unlock vesting rewards. This should still pay out the accumulated rewards to shareholders.
        unlock_rewards(contract_address);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, accumulated_rewards);
        assert!(remaining_grant(contract_address) == GRANT_AMOUNT, 0);

        // Distribution should pay commission to operator first and remaining amount to shareholders.
        stake::fast_forward_to_unlock(stake_pool_address);
        stake::assert_stake_pool(
            stake_pool_address,
            with_rewards(GRANT_AMOUNT),
            with_rewards(accumulated_rewards),
            0,
            0
        );
        // Operator also earns more commission from the rewards earnt on the withdrawn rewards.
        let commission_on_staker_rewards = (with_rewards(staker_rewards) - staker_rewards) / 10;
        staker_rewards = with_rewards(staker_rewards) - commission_on_staker_rewards;
        commission = with_rewards(commission) + commission_on_staker_rewards;
        distribute(contract_address);
        // Rounding error leads to a dust amount of 1 transferred to the staker.
        assert!(coin::balance<AptosCoin>(shareholder_address) == staker_rewards + 1, 0);
        assert!(coin::balance<AptosCoin>(operator_address) == commission - 1, 1);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, operator = @0x345)]
    public entry fun test_update_operator_with_same_commission(
        aptos_framework: &signer,
        admin: &signer,
        operator: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let operator_address = signer::address_of(operator);
        setup(aptos_framework, &vector[admin_address, @11, operator_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[@11], &vector[GRANT_AMOUNT], admin_address, 10);

        update_operator_with_same_commission(admin, contract_address, operator_address);
        assert!(operator_commission_percentage(contract_address) == 10, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, shareholder = @0x234, operator = @0x345)]
    public entry fun test_commission_percentage_change(
        aptos_framework: &signer,
        admin: &signer,
        shareholder: &signer,
        operator: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let operator_address = signer::address_of(operator);
        let shareholder_address = signer::address_of(shareholder);
        setup(aptos_framework, &vector[admin_address, shareholder_address, operator_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[shareholder_address], &vector[GRANT_AMOUNT], admin_address, 0);
        assert!(operator_commission_percentage(contract_address) == 0, 0);
        let stake_pool_address = stake_pool_address(contract_address);

        // 10% commission will be paid to the operator.
        update_operator(admin, contract_address, operator_address, 10);

        // Operator needs to join the validator set for the stake pool to earn rewards.
        let (_sk, pk, pop) = stake::generate_identity();
        stake::join_validator_set_for_test(&pk, &pop, operator, stake_pool_address, true);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, 0);
        assert!(get_accumulated_rewards(contract_address) == 0, 0);
        assert!(remaining_grant(contract_address) == GRANT_AMOUNT, 0);

        // Stake pool earns some rewards.
        stake::end_epoch();
        let (_, accumulated_rewards, _) = staking_contract::staking_contract_amounts(
            contract_address,
            operator_address
        );

        // Update commission percentage to 20%. This also immediately requests commission.
        update_commission_percentage(admin, contract_address, 20);
        // Assert that the operator is still the same, and the commission percentage is updated to 20%.
        assert!(operator(contract_address) == operator_address, 0);
        assert!(operator_commission_percentage(contract_address) == 20, 0);

        // Commission is calculated using the previous commission percentage which is 10%.
        let expected_commission = accumulated_rewards / 10;

        // Stake pool earns some more rewards.
        stake::end_epoch();
        let (_, accumulated_rewards, _) = staking_contract::staking_contract_amounts(
            contract_address,
            operator_address
        );

        // Request commission again.
        staking_contract::request_commission(operator, contract_address, operator_address);
        // The commission is calculated using the current commission percentage which is 20%.
        expected_commission = with_rewards(expected_commission) + (accumulated_rewards / 5);

        // Unlocks the commission.
        stake::fast_forward_to_unlock(stake_pool_address);
        expected_commission = with_rewards(expected_commission);

        // Distribute the commission to the operator.
        distribute(contract_address);

        // Assert that the operator receives the expected commission.
        assert!(coin::balance<AptosCoin>(operator_address) == expected_commission, 1);
    }

    #[test(
        aptos_framework = @0x1,
        admin = @0x123,
        shareholder = @0x234,
        operator1 = @0x345,
        beneficiary = @0x456,
        operator2 = @0x567
    )]
    public entry fun test_set_beneficiary_for_operator(
        aptos_framework: &signer,
        admin: &signer,
        shareholder: &signer,
        operator1: &signer,
        beneficiary: &signer,
        operator2: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let operator_address1 = signer::address_of(operator1);
        let operator_address2 = signer::address_of(operator2);
        let shareholder_address = signer::address_of(shareholder);
        let beneficiary_address = signer::address_of(beneficiary);
        setup(aptos_framework, &vector[admin_address, shareholder_address, operator_address1, beneficiary_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[shareholder_address], &vector[GRANT_AMOUNT], admin_address, 0);
        assert!(operator_commission_percentage(contract_address) == 0, 0);
        let stake_pool_address = stake_pool_address(contract_address);
        // 10% commission will be paid to the operator.
        update_operator(admin, contract_address, operator_address1, 10);
        assert!(staking_contract::beneficiary_for_operator(operator_address1) == operator_address1, 0);
        set_beneficiary_for_operator(operator1, beneficiary_address);
        assert!(staking_contract::beneficiary_for_operator(operator_address1) == beneficiary_address, 0);

        // Operator needs to join the validator set for the stake pool to earn rewards.
        let (_sk, pk, pop) = stake::generate_identity();
        stake::join_validator_set_for_test(&pk, &pop, operator1, stake_pool_address, true);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, 0);
        assert!(get_accumulated_rewards(contract_address) == 0, 0);
        assert!(remaining_grant(contract_address) == GRANT_AMOUNT, 0);

        // Stake pool earns some rewards.
        stake::end_epoch();
        let (_, accumulated_rewards, _) = staking_contract::staking_contract_amounts(contract_address,
            operator_address1
        );
        // Commission is calculated using the previous commission percentage which is 10%.
        let expected_commission = accumulated_rewards / 10;

        // Request commission.
        staking_contract::request_commission(operator1, contract_address, operator_address1);
        // Unlocks the commission.
        stake::fast_forward_to_unlock(stake_pool_address);
        expected_commission = with_rewards(expected_commission);

        // Distribute the commission to the operator.
        distribute(contract_address);

        // Assert that the beneficiary receives the expected commission.
        assert!(coin::balance<AptosCoin>(operator_address1) == 0, 1);
        assert!(coin::balance<AptosCoin>(beneficiary_address) == expected_commission, 1);
        let old_beneficiay_balance = coin::balance<AptosCoin>(beneficiary_address);

        // switch operator to operator2. The rewards should go to operator2 not to the beneficiay of operator1.
        update_operator(admin, contract_address, operator_address2, 10);

        stake::end_epoch();
        let (_, accumulated_rewards, _) = staking_contract::staking_contract_amounts(contract_address,
            operator_address2
        );

        let expected_commission = accumulated_rewards / 10;

        // Request commission.
        staking_contract::request_commission(operator2, contract_address, operator_address2);
        // Unlocks the commission.
        stake::fast_forward_to_unlock(stake_pool_address);
        expected_commission = with_rewards(expected_commission);

        // Distribute the commission to the operator.
        distribute(contract_address);

        // Assert that the rewards go to operator2, and the balance of the operator1's beneficiay remains the same.
        assert!(coin::balance<AptosCoin>(operator_address2) >= expected_commission, 1);
        assert!(coin::balance<AptosCoin>(beneficiary_address) == old_beneficiay_balance, 1);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, shareholder = @0x234)]
    #[expected_failure(abort_code = 0x30008, location = Self)]
    public entry fun test_cannot_unlock_rewards_after_contract_is_terminated(
        aptos_framework: &signer,
        admin: &signer,
        shareholder: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let shareholder_address = signer::address_of(shareholder);
        setup(aptos_framework, &vector[admin_address, shareholder_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[shareholder_address], &vector[GRANT_AMOUNT], admin_address, 0);

        // Immediately terminate. Calling unlock_rewards should now fail.
        terminate_vesting_contract(admin, contract_address);
        unlock_rewards(contract_address);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, shareholder = @0x234)]
    public entry fun test_vesting_contract_with_zero_vestings(
        aptos_framework: &signer,
        admin: &signer,
        shareholder: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let shareholder_address = signer::address_of(shareholder);
        setup(aptos_framework, &vector[admin_address, shareholder_address]);
        let contract_address = setup_vesting_contract_with_schedule(
            admin,
            &vector[shareholder_address],
            &vector[GRANT_AMOUNT],
            admin_address,
            0,
            &vector[0, 3, 0, 2],
            48,
        );
        let stake_pool_address = stake_pool_address(contract_address);

        // First vest() should unlock 0 according to schedule.
        timestamp::update_global_time_for_test_secs(vesting_start_secs(contract_address) + VESTING_PERIOD);
        vest(contract_address);
        stake::assert_stake_pool(stake_pool_address, GRANT_AMOUNT, 0, 0, 0);
        assert!(remaining_grant(contract_address) == GRANT_AMOUNT, 0);

        // Next period should vest 3/48.
        timestamp::fast_forward_seconds(VESTING_PERIOD);
        vest(contract_address);
        let vested_amount = fraction(GRANT_AMOUNT, 3, 48);
        let remaining_grant = GRANT_AMOUNT - vested_amount;
        stake::assert_stake_pool(stake_pool_address, remaining_grant, 0, 0, vested_amount);
        assert!(remaining_grant(contract_address) == remaining_grant, 0);

        timestamp::fast_forward_seconds(VESTING_PERIOD);
        // Distribute the previous vested amount.
        distribute(contract_address);
        // Next period should vest 0 again.
        vest(contract_address);
        stake::assert_stake_pool(stake_pool_address, remaining_grant, 0, 0, 0);
        assert!(remaining_grant(contract_address) == remaining_grant, 0);

        // Next period should vest 2/48.
        timestamp::fast_forward_seconds(VESTING_PERIOD);
        vest(contract_address);
        let vested_amount = fraction(GRANT_AMOUNT, 2, 48);
        remaining_grant = remaining_grant - vested_amount;
        stake::assert_stake_pool(stake_pool_address, remaining_grant, 0, 0, vested_amount);
        assert!(remaining_grant(contract_address) == remaining_grant, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, shareholder = @0x234)]
    public entry fun test_last_vest_should_distribute_remaining_amount(
        aptos_framework: &signer,
        admin: &signer,
        shareholder: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let shareholder_address = signer::address_of(shareholder);
        setup(aptos_framework, &vector[admin_address, shareholder_address]);
        let contract_address = setup_vesting_contract_with_schedule(
            admin,
            &vector[shareholder_address],
            &vector[GRANT_AMOUNT],
            admin_address,
            0,
            // First vest = 3/4 but last vest should only be for the remaining 1/4.
            &vector[3],
            4,
        );
        let stake_pool_address = stake_pool_address(contract_address);

        // First vest is 3/48
        timestamp::update_global_time_for_test_secs(vesting_start_secs(contract_address) + VESTING_PERIOD);
        vest(contract_address);
        let vested_amount = fraction(GRANT_AMOUNT, 3, 4);
        let remaining_grant = GRANT_AMOUNT - vested_amount;
        stake::assert_stake_pool(stake_pool_address, remaining_grant, 0, 0, vested_amount);
        assert!(remaining_grant(contract_address) == remaining_grant, 0);

        timestamp::fast_forward_seconds(VESTING_PERIOD);
        // Distribute the previous vested amount.
        distribute(contract_address);
        // Last vest should be the remaining amount (1/4).
        vest(contract_address);
        let vested_amount = remaining_grant;
        remaining_grant = 0;
        stake::assert_stake_pool(stake_pool_address, remaining_grant, 0, 0, vested_amount);
        assert!(remaining_grant(contract_address) == remaining_grant, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, shareholder = @0x234)]
    #[expected_failure(abort_code = 0x30008, location = Self)]
    public entry fun test_cannot_vest_after_contract_is_terminated(
        aptos_framework: &signer,
        admin: &signer,
        shareholder: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let shareholder_address = signer::address_of(shareholder);
        setup(aptos_framework, &vector[admin_address, shareholder_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[shareholder_address], &vector[GRANT_AMOUNT], admin_address, 0);

        // Immediately terminate. Calling vest should now fail.
        terminate_vesting_contract(admin, contract_address);
        vest(contract_address);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, shareholder = @0x234)]
    #[expected_failure(abort_code = 0x30008, location = Self)]
    public entry fun test_cannot_terminate_twice(
        aptos_framework: &signer,
        admin: &signer,
        shareholder: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let shareholder_address = signer::address_of(shareholder);
        setup(aptos_framework, &vector[admin_address, shareholder_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[shareholder_address], &vector[GRANT_AMOUNT], admin_address, 0);

        // Call terminate_vesting_contract twice should fail.
        terminate_vesting_contract(admin, contract_address);
        terminate_vesting_contract(admin, contract_address);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, shareholder = @0x234)]
    #[expected_failure(abort_code = 0x30009, location = Self)]
    public entry fun test_cannot_call_admin_withdraw_if_contract_is_not_terminated(
        aptos_framework: &signer,
        admin: &signer,
        shareholder: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        let shareholder_address = signer::address_of(shareholder);
        setup(aptos_framework, &vector[admin_address, shareholder_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[shareholder_address], &vector[GRANT_AMOUNT], admin_address, 0);

        // Calling admin_withdraw should fail as contract has not been terminated.
        admin_withdraw(admin, contract_address);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    #[expected_failure(abort_code = 0x60001, location = aptos_framework::aptos_account)]
    public entry fun test_set_beneficiary_with_missing_account_should_fail(
        aptos_framework: &signer,
        admin: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[@1, @2], &vector[GRANT_AMOUNT, GRANT_AMOUNT], admin_address, 0);
        set_beneficiary(admin, contract_address, @1, @11);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    #[expected_failure(abort_code = 0x60002, location = aptos_framework::aptos_account)]
    public entry fun test_set_beneficiary_with_unregistered_account_should_fail(
        aptos_framework: &signer,
        admin: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[@1, @2], &vector[GRANT_AMOUNT, GRANT_AMOUNT], admin_address, 0);
        create_account_for_test(@11);
        set_beneficiary(admin, contract_address, @1, @11);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    public entry fun test_set_beneficiary_should_send_distribution(
        aptos_framework: &signer,
        admin: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address, @11]);
        let contract_address = setup_vesting_contract(
            admin, &vector[@1], &vector[GRANT_AMOUNT], admin_address, 0);
        set_beneficiary(admin, contract_address, @1, @11);
        assert!(beneficiary(contract_address, @1) == @11, 0);

        // Fast forward to the end of the first period. vest() should now unlock 3/48 of the tokens.
        timestamp::update_global_time_for_test_secs(vesting_start_secs(contract_address) + VESTING_PERIOD);
        vest(contract_address);

        // Distribution should go to the beneficiary account.
        stake::end_epoch();
        // No rewards as validator never joined the validator set.
        let vested_amount = fraction(GRANT_AMOUNT, 3, 48);
        distribute(contract_address);
        let balance = coin::balance<AptosCoin>(@11);
        assert!(balance == vested_amount, balance);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    public entry fun test_set_management_role(
        aptos_framework: &signer,
        admin: &signer,
    ) acquires AdminStore, VestingAccountManagement, VestingContract {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address]);
        let contract_address = setup_vesting_contract(
            admin, &vector[@11], &vector[GRANT_AMOUNT], admin_address, 0);
        let role = utf8(b"RANDOM");
        set_management_role(admin, contract_address, role, @12);
        assert!(get_role_holder(contract_address, role) == @12, 0);
        set_management_role(admin, contract_address, role, @13);
        assert!(get_role_holder(contract_address, role) == @13, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123)]
    public entry fun test_reset_beneficiary(
        aptos_framework: &signer,
        admin: &signer,
    ) acquires AdminStore, VestingAccountManagement, VestingContract {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address, @11, @12]);
        let contract_address = setup_vesting_contract(
            admin, &vector[@11], &vector[GRANT_AMOUNT], admin_address, 0);
        set_beneficiary(admin, contract_address, @11, @12);
        assert!(beneficiary(contract_address, @11) == @12, 0);

        // Fast forward to the end of the first period. vest() should now unlock 3/48 of the tokens.
        timestamp::update_global_time_for_test_secs(vesting_start_secs(contract_address) + VESTING_PERIOD);
        vest(contract_address);

        // Reset the beneficiary.
        reset_beneficiary(admin, contract_address, @11);

        // Distribution should go to the original account.
        stake::end_epoch();
        // No rewards as validator never joined the validator set.
        let vested_amount = fraction(GRANT_AMOUNT, 3, 48);
        distribute(contract_address);
        assert!(coin::balance<AptosCoin>(@11) == vested_amount, 0);
        assert!(coin::balance<AptosCoin>(@12) == 0, 1);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, resetter = @0x234)]
    public entry fun test_reset_beneficiary_with_resetter_role(
        aptos_framework: &signer,
        admin: &signer,
        resetter: &signer,
    ) acquires AdminStore, VestingAccountManagement, VestingContract {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address, @11, @12]);
        let contract_address = setup_vesting_contract(
            admin, &vector[@11], &vector[GRANT_AMOUNT], admin_address, 0);
        set_beneficiary(admin, contract_address, @11, @12);
        assert!(beneficiary(contract_address, @11) == @12, 0);

        // Reset the beneficiary with the resetter role.
        let resetter_address = signer::address_of(resetter);
        set_beneficiary_resetter(admin, contract_address, resetter_address);
        assert!(simple_map::length(&borrow_global<VestingAccountManagement>(contract_address).roles) == 1, 0);
        reset_beneficiary(resetter, contract_address, @11);
        assert!(beneficiary(contract_address, @11) == @11, 0);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, resetter = @0x234, random = @0x345)]
    #[expected_failure(abort_code = 0x5000F, location = Self)]
    public entry fun test_reset_beneficiary_with_unauthorized(
        aptos_framework: &signer,
        admin: &signer,
        resetter: &signer,
        random: &signer,
    ) acquires AdminStore, VestingAccountManagement, VestingContract {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address, @11]);
        let contract_address = setup_vesting_contract(
            admin, &vector[@11], &vector[GRANT_AMOUNT], admin_address, 0);

        // Reset the beneficiary with a random account. This should failed.
        set_beneficiary_resetter(admin, contract_address, signer::address_of(resetter));
        reset_beneficiary(random, contract_address, @11);
    }

    #[test(aptos_framework = @0x1, admin = @0x123, resetter = @0x234, random = @0x345)]
    public entry fun test_shareholder(
        aptos_framework: &signer,
        admin: &signer,
    ) acquires AdminStore, VestingContract {
        let admin_address = signer::address_of(admin);
        setup(aptos_framework, &vector[admin_address, @11, @12]);
        let contract_address = setup_vesting_contract(
            admin, &vector[@11], &vector[GRANT_AMOUNT], admin_address, 0);

        // Confirm that the lookup returns the same address when a shareholder is
        // passed for which there is no beneficiary.
        assert!(shareholder(contract_address, @11) == @11, 0);

        // Set a beneficiary for @11.
        set_beneficiary(admin, contract_address, @11, @12);
        assert!(beneficiary(contract_address, @11) == @12, 0);

        // Confirm that lookup from beneficiary to shareholder works when a beneficiary
        // is set.
        assert!(shareholder(contract_address, @12) == @11, 0);

        // Confirm that it returns 0x0 when the address is not in the map.
        assert!(shareholder(contract_address, @33) == @0x0, 0);
    }

    #[test_only]
    fun get_accumulated_rewards(contract_address: address): u64 acquires VestingContract {
        let vesting_contract = borrow_global<VestingContract>(contract_address);
        let (active_stake, _, _, _) = stake::get_stake(vesting_contract.staking.pool_address);
        active_stake - vesting_contract.remaining_grant
    }

    #[test_only]
    fun fraction(total: u64, numerator: u64, denominator: u64): u64 {
        fixed_point32::multiply_u64(total, fixed_point32::create_from_rational(numerator, denominator))
    }
}
