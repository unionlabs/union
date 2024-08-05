/// The `function_info` module defines the `FunctionInfo` type which simulates a function pointer.
module aptos_framework::function_info {
    use std::error;
    use std::features;
    use std::signer;
    use std::string::{Self, String};

    friend aptos_framework::fungible_asset;
    friend aptos_framework::dispatchable_fungible_asset;

    /// String is not a valid Move identifier
    const EINVALID_IDENTIFIER: u64 = 1;
    /// Function specified in the FunctionInfo doesn't exist on chain.
    const EINVALID_FUNCTION: u64 = 2;
    /// Feature hasn't been activated yet.
    const ENOT_ACTIVATED: u64 = 3;

    /// A `String` holds a sequence of bytes which is guaranteed to be in utf8 format.
    struct FunctionInfo has copy, drop, store {
        module_address: address,
        module_name: String,
        function_name: String,
    }

    /// Creates a new function info from names.
    public fun new_function_info(
        module_signer: &signer,
        module_name: String,
        function_name: String,
    ): FunctionInfo {
        new_function_info_from_address(
            signer::address_of(module_signer),
            module_name,
            function_name,
        )
    }

    public(friend) fun new_function_info_from_address(
        module_address: address,
        module_name: String,
        function_name: String,
    ): FunctionInfo {
        assert!(
            is_identifier(string::bytes(&module_name)),
            EINVALID_IDENTIFIER
        );
        assert!(
            is_identifier(string::bytes(&function_name)),
            EINVALID_IDENTIFIER
        );
        FunctionInfo {
            module_address,
            module_name,
            function_name,
        }
    }

    /// Check if the dispatch target function meets the type requirements of the disptach entry point.
    ///
    /// framework_function is the dispatch native function defined in the aptos_framework.
    /// dispatch_target is the function passed in by the user.
    ///
    /// dispatch_target should have the same signature (same argument type, same generics constraint) except
    /// that the framework_function will have a `&FunctionInfo` in the last argument that will instruct the VM which
    /// function to jump to.
    ///
    /// dispatch_target also needs to be public so the type signature will remain unchanged.
    public(friend) fun check_dispatch_type_compatibility(
        framework_function: &FunctionInfo,
        dispatch_target: &FunctionInfo,
    ): bool {
        assert!(
            features::dispatchable_fungible_asset_enabled(),
            error::aborted(ENOT_ACTIVATED)
        );
        load_function_impl(dispatch_target);
        check_dispatch_type_compatibility_impl(framework_function, dispatch_target)
    }

    /// Load up a function into VM's loader and charge for its dependencies
    ///
    /// It is **critical** to make sure that this function is invoked before `check_dispatch_type_compatibility`
    /// or performing any other dispatching logic to ensure:
    /// 1. We properly charge gas for the function to dispatch.
    /// 2. The function is loaded in the cache so that we can perform further type checking/dispatching logic.
    ///
    /// Calling `check_dispatch_type_compatibility_impl` or dispatch without loading up the module would yield an error
    /// if such module isn't accessed previously in the transaction.
    public(friend) fun load_module_from_function(f: &FunctionInfo) {
        load_function_impl(f)
    }

    native fun check_dispatch_type_compatibility_impl(lhs: &FunctionInfo, r: &FunctionInfo): bool;
    native fun is_identifier(s: &vector<u8>): bool;
    native fun load_function_impl(f: &FunctionInfo);

    // Test only dependencies so we can invoke those friend functions.
    #[test_only]
    friend aptos_framework::function_info_tests;
}
