/// A module which defines the basic concept of
/// [*capabilities*](https://en.wikipedia.org/wiki/Capability-based_security) for managing access control.
///
/// EXPERIMENTAL
///
/// # Overview
///
/// A capability is a unforgeable token which testifies that a signer has authorized a certain operation.
/// The token is valid during the transaction where it is obtained. Since the type `capability::Cap` has
/// no ability to be stored in global memory, capabilities cannot leak out of a transaction. For every function
/// called within a transaction which has a capability as a parameter, it is guaranteed that the capability
/// has been obtained via a proper signer-based authorization step previously in the transaction's execution.
///
/// ## Usage
///
/// Initializing and acquiring capabilities is usually encapsulated in a module with a type
/// tag which can only be constructed by this module.
///
/// ```
/// module Pkg::Feature {
///   use std::capability::Cap;
///
///   /// A type tag used in Cap<Feature>. Only this module can create an instance,
///   /// and there is no public function other than Self::acquire which returns a value of this type.
///   /// This way, this module has full control how Cap<Feature> is given out.
///   struct Feature has drop {}
///
///   /// Initializes this module.
///   public fun initialize(s: &signer) {
///     // Create capability. This happens once at module initialization time.
///     // One needs to provide a witness for being the owner of Feature
///     // in the 2nd parameter.
///     <<additional conditions allowing to initialize this capability>>
///     capability::create<Feature>(s, &Feature{});
///   }
///
///   /// Acquires the capability to work with this feature.
///   public fun acquire(s: &signer): Cap<Feature> {
///     <<additional conditions allowing to acquire this capability>>
///     capability::acquire<Feature>(s, &Feature{});
///   }
///
///   /// Does something related to the feature. The caller must pass a Cap<Feature>.
///   public fun do_something(_cap: Cap<Feature>) { ... }
/// }
/// ```
///
/// ## Delegation
///
/// Capabilities come with the optional feature of *delegation*. Via `Self::delegate`, an owner of a capability
/// can designate another signer to be also capable of acquiring the capability. Like the original creator,
/// the delegate needs to present his signer to obtain the capability in his transactions. Delegation can
/// be revoked via `Self::revoke`, removing this access right from the delegate.
///
/// While the basic authorization mechanism for delegates is the same as with core capabilities, the
/// target of delegation might be subject of restrictions which need to be specified and verified. This can
/// be done via global invariants in the specification language. For example, in order to prevent delegation
/// all together for a capability, one can use the following invariant:
///
/// ```
///   invariant forall a: address where capability::spec_has_cap<Feature>(a):
///               len(capability::spec_delegates<Feature>(a)) == 0;
/// ```
///
/// Similarly, the following invariant would enforce that delegates, if existent, must satisfy a certain
/// predicate:
///
/// ```
///   invariant forall a: address where capability::spec_has_cap<Feature>(a):
///               forall d in capability::spec_delegates<Feature>(a):
///                  is_valid_delegate_for_feature(d);
/// ```
///
module aptos_std::capability {
    use std::error;
    use std::signer;
    use std::vector;

    /// Capability resource already exists on the specified account
    const ECAPABILITY_ALREADY_EXISTS: u64 = 1;
    /// Capability resource not found
    const ECAPABILITY_NOT_FOUND: u64 = 2;
    /// Account does not have delegated permissions
    const EDELEGATE: u64 = 3;

    /// The token representing an acquired capability. Cannot be stored in memory, but copied and dropped freely.
    struct Cap<phantom Feature> has copy, drop {
        root: address
    }

    /// A linear version of a capability token. This can be used if an acquired capability should be enforced
    /// to be used only once for an authorization.
    struct LinearCap<phantom Feature> has drop {
        root: address
    }

    /// An internal data structure for representing a configured capability.
    struct CapState<phantom Feature> has key {
        delegates: vector<address>
    }

    /// An internal data structure for representing a configured delegated capability.
    struct CapDelegateState<phantom Feature> has key {
        root: address
    }

    /// Creates a new capability class, owned by the passed signer. A caller must pass a witness that
    /// they own the `Feature` type parameter.
    public fun create<Feature>(owner: &signer, _feature_witness: &Feature) {
        let addr = signer::address_of(owner);
        assert!(!exists<CapState<Feature>>(addr), error::already_exists(ECAPABILITY_ALREADY_EXISTS));
        move_to<CapState<Feature>>(owner, CapState { delegates: vector::empty() });
    }

    /// Acquires a capability token. Only the owner of the capability class, or an authorized delegate,
    /// can succeed with this operation. A caller must pass a witness that they own the `Feature` type
    /// parameter.
    public fun acquire<Feature>(requester: &signer, _feature_witness: &Feature): Cap<Feature>
    acquires CapState, CapDelegateState {
        Cap<Feature> { root: validate_acquire<Feature>(requester) }
    }

    /// Acquires a linear capability token. It is up to the module which owns `Feature` to decide
    /// whether to expose a linear or non-linear capability.
    public fun acquire_linear<Feature>(requester: &signer, _feature_witness: &Feature): LinearCap<Feature>
    acquires CapState, CapDelegateState {
        LinearCap<Feature> { root: validate_acquire<Feature>(requester) }
    }

    /// Helper to validate an acquire. Returns the root address of the capability.
    fun validate_acquire<Feature>(requester: &signer): address
    acquires CapState, CapDelegateState {
        let addr = signer::address_of(requester);
        if (exists<CapDelegateState<Feature>>(addr)) {
            let root_addr = borrow_global<CapDelegateState<Feature>>(addr).root;
            // double check that requester is actually registered as a delegate
            assert!(exists<CapState<Feature>>(root_addr), error::invalid_state(EDELEGATE));
            assert!(vector::contains(&borrow_global<CapState<Feature>>(root_addr).delegates, &addr),
                error::invalid_state(EDELEGATE));
            root_addr
        } else {
            assert!(exists<CapState<Feature>>(addr), error::not_found(ECAPABILITY_NOT_FOUND));
            addr
        }
    }

    /// Returns the root address associated with the given capability token. Only the owner
    /// of the feature can do this.
    public fun root_addr<Feature>(cap: Cap<Feature>, _feature_witness: &Feature): address {
        cap.root
    }

    /// Returns the root address associated with the given linear capability token.
    public fun linear_root_addr<Feature>(cap: LinearCap<Feature>, _feature_witness: &Feature): address {
        cap.root
    }

    /// Registers a delegation relation. If the relation already exists, this function does
    /// nothing.
    // TODO: explore whether this should be idempotent like now or abort
    public fun delegate<Feature>(cap: Cap<Feature>, _feature_witness: &Feature, to: &signer)
    acquires CapState {
        let addr = signer::address_of(to);
        if (exists<CapDelegateState<Feature>>(addr)) return;
        move_to(to, CapDelegateState<Feature> { root: cap.root });
        add_element(&mut borrow_global_mut<CapState<Feature>>(cap.root).delegates, addr);
    }

    /// Revokes a delegation relation. If no relation exists, this function does nothing.
    // TODO: explore whether this should be idempotent like now or abort
    public fun revoke<Feature>(cap: Cap<Feature>, _feature_witness: &Feature, from: address)
    acquires CapState, CapDelegateState
    {
        if (!exists<CapDelegateState<Feature>>(from)) return;
        let CapDelegateState { root: _root } = move_from<CapDelegateState<Feature>>(from);
        remove_element(&mut borrow_global_mut<CapState<Feature>>(cap.root).delegates, &from);
    }

    /// Helper to remove an element from a vector.
    fun remove_element<E: drop>(v: &mut vector<E>, x: &E) {
        let (found, index) = vector::index_of(v, x);
        if (found) {
            vector::remove(v, index);
        }
    }

    /// Helper to add an element to a vector.
    fun add_element<E: drop>(v: &mut vector<E>, x: E) {
        if (!vector::contains(v, &x)) {
            vector::push_back(v, x)
        }
    }
}
