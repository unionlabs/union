/// This module allows users to deploy, upgrade and freeze modules deployed to objects on-chain.
/// This enables users to deploy modules to an object with a unique address each time they are published.
/// This modules provides an alternative method to publish code on-chain, where code is deployed to objects rather than accounts.
/// This is encouraged as it abstracts the necessary resources needed for deploying modules,
/// along with the required authorization to upgrade and freeze modules.
///
/// The functionalities of this module are as follows.
///
/// Publishing modules flow:
/// 1. Create a new object with the address derived from the publisher address and the object seed.
/// 2. Publish the module passed in the function via `metadata_serialized` and `code` to the newly created object.
/// 3. Emits 'Publish' event with the address of the newly created object.
/// 4. Create a `ManagingRefs` which stores the extend ref of the newly created object.
/// Note: This is needed to upgrade the code as the signer must be generated to upgrade the existing code in an object.
///
/// Upgrading modules flow:
/// 1. Assert the `code_object` passed in the function is owned by the `publisher`.
/// 2. Assert the `code_object` passed in the function exists in global storage.
/// 2. Retrieve the `ExtendRef` from the `code_object` and generate the signer from this.
/// 3. Upgrade the module with the `metadata_serialized` and `code` passed in the function.
/// 4. Emits 'Upgrade' event with the address of the object with the upgraded code.
/// Note: If the modules were deployed as immutable when calling `publish`, the upgrade will fail.
///
/// Freezing modules flow:
/// 1. Assert the `code_object` passed in the function exists in global storage.
/// 2. Assert the `code_object` passed in the function is owned by the `publisher`.
/// 3. Mark all the modules in the `code_object` as immutable.
/// 4. Emits 'Freeze' event with the address of the object with the frozen code.
/// Note: There is no unfreeze function as this gives no benefit if the user can freeze/unfreeze modules at will.
///       Once modules are marked as immutable, they cannot be made mutable again.
module aptos_framework::object_code_deployment {
    use std::bcs;
    use std::error;
    use std::features;
    use std::signer;
    use std::vector;
    use aptos_framework::account;
    use aptos_framework::code;
    use aptos_framework::code::PackageRegistry;
    use aptos_framework::event;
    use aptos_framework::object;
    use aptos_framework::object::{ExtendRef, Object};

    /// Object code deployment feature not supported.
    const EOBJECT_CODE_DEPLOYMENT_NOT_SUPPORTED: u64 = 1;
    /// Not the owner of the `code_object`
    const ENOT_CODE_OBJECT_OWNER: u64 = 2;
    /// `code_object` does not exist.
    const ECODE_OBJECT_DOES_NOT_EXIST: u64 = 3;

    const OBJECT_CODE_DEPLOYMENT_DOMAIN_SEPARATOR: vector<u8> = b"aptos_framework::object_code_deployment";

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// Internal struct, attached to the object, that holds Refs we need to manage the code deployment (i.e. upgrades).
    struct ManagingRefs has key {
        /// We need to keep the extend ref to be able to generate the signer to upgrade existing code.
        extend_ref: ExtendRef,
    }

    #[event]
    /// Event emitted when code is published to an object.
    struct Publish has drop, store {
        object_address: address,
    }

    #[event]
    /// Event emitted when code in an existing object is upgraded.
    struct Upgrade has drop, store {
        object_address: address,
    }

    #[event]
    /// Event emitted when code in an existing object is made immutable.
    struct Freeze has drop, store {
        object_address: address,
    }

    /// Creates a new object with a unique address derived from the publisher address and the object seed.
    /// Publishes the code passed in the function to the newly created object.
    /// The caller must provide package metadata describing the package via `metadata_serialized` and
    /// the code to be published via `code`. This contains a vector of modules to be deployed on-chain.
    public entry fun publish(
        publisher: &signer,
        metadata_serialized: vector<u8>,
        code: vector<vector<u8>>,
    ) {
        assert!(
            features::is_object_code_deployment_enabled(),
            error::unavailable(EOBJECT_CODE_DEPLOYMENT_NOT_SUPPORTED),
        );

        let publisher_address = signer::address_of(publisher);
        let object_seed = object_seed(publisher_address);
        let constructor_ref = &object::create_named_object(publisher, object_seed);
        let code_signer = &object::generate_signer(constructor_ref);
        code::publish_package_txn(code_signer, metadata_serialized, code);

        event::emit(Publish { object_address: signer::address_of(code_signer), });

        move_to(code_signer, ManagingRefs {
            extend_ref: object::generate_extend_ref(constructor_ref),
        });
    }

    inline fun object_seed(publisher: address): vector<u8> {
        let sequence_number = account::get_sequence_number(publisher) + 1;
        let seeds = vector[];
        vector::append(&mut seeds, bcs::to_bytes(&OBJECT_CODE_DEPLOYMENT_DOMAIN_SEPARATOR));
        vector::append(&mut seeds, bcs::to_bytes(&sequence_number));
        seeds
    }

    /// Upgrades the existing modules at the `code_object` address with the new modules passed in `code`,
    /// along with the metadata `metadata_serialized`.
    /// Note: If the modules were deployed as immutable when calling `publish`, the upgrade will fail.
    /// Requires the publisher to be the owner of the `code_object`.
    public entry fun upgrade(
        publisher: &signer,
        metadata_serialized: vector<u8>,
        code: vector<vector<u8>>,
        code_object: Object<PackageRegistry>,
    ) acquires ManagingRefs {
        let publisher_address = signer::address_of(publisher);
        assert!(
            object::is_owner(code_object, publisher_address),
            error::permission_denied(ENOT_CODE_OBJECT_OWNER),
        );

        let code_object_address = object::object_address(&code_object);
        assert!(exists<ManagingRefs>(code_object_address), error::not_found(ECODE_OBJECT_DOES_NOT_EXIST));

        let extend_ref = &borrow_global<ManagingRefs>(code_object_address).extend_ref;
        let code_signer = &object::generate_signer_for_extending(extend_ref);
        code::publish_package_txn(code_signer, metadata_serialized, code);

        event::emit(Upgrade { object_address: signer::address_of(code_signer), });
    }

    /// Make an existing upgradable package immutable. Once this is called, the package cannot be made upgradable again.
    /// Each `code_object` should only have one package, as one package is deployed per object in this module.
    /// Requires the `publisher` to be the owner of the `code_object`.
    public entry fun freeze_code_object(publisher: &signer, code_object: Object<PackageRegistry>) {
        code::freeze_code_object(publisher, code_object);

        event::emit(Freeze { object_address: object::object_address(&code_object), });
    }
}
