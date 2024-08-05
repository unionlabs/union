module aptos_framework::system_addresses {
    use std::error;
    use std::signer;

    /// The address/account did not correspond to the core resource address
    const ENOT_CORE_RESOURCE_ADDRESS: u64 = 1;
    /// The operation can only be performed by the VM
    const EVM: u64 = 2;
    /// The address/account did not correspond to the core framework address
    const ENOT_APTOS_FRAMEWORK_ADDRESS: u64 = 3;
    /// The address is not framework reserved address
    const ENOT_FRAMEWORK_RESERVED_ADDRESS: u64 = 4;

    public fun assert_core_resource(account: &signer) {
        assert_core_resource_address(signer::address_of(account))
    }

    public fun assert_core_resource_address(addr: address) {
        assert!(is_core_resource_address(addr), error::permission_denied(ENOT_CORE_RESOURCE_ADDRESS))
    }

    public fun is_core_resource_address(addr: address): bool {
        addr == @core_resources
    }

    public fun assert_aptos_framework(account: &signer) {
        assert!(
            is_aptos_framework_address(signer::address_of(account)),
            error::permission_denied(ENOT_APTOS_FRAMEWORK_ADDRESS),
        )
    }

    public fun assert_framework_reserved_address(account: &signer) {
        assert_framework_reserved(signer::address_of(account));
    }

    public fun assert_framework_reserved(addr: address) {
        assert!(
            is_framework_reserved_address(addr),
            error::permission_denied(ENOT_FRAMEWORK_RESERVED_ADDRESS),
        )
    }

    /// Return true if `addr` is 0x0 or under the on chain governance's control.
    public fun is_framework_reserved_address(addr: address): bool {
        is_aptos_framework_address(addr) ||
            addr == @0x2 ||
            addr == @0x3 ||
            addr == @0x4 ||
            addr == @0x5 ||
            addr == @0x6 ||
            addr == @0x7 ||
            addr == @0x8 ||
            addr == @0x9 ||
            addr == @0xa
    }

    /// Return true if `addr` is 0x1.
    public fun is_aptos_framework_address(addr: address): bool {
        addr == @aptos_framework
    }

    /// Assert that the signer has the VM reserved address.
    public fun assert_vm(account: &signer) {
        assert!(is_vm(account), error::permission_denied(EVM))
    }

    /// Return true if `addr` is a reserved address for the VM to call system modules.
    public fun is_vm(account: &signer): bool {
        is_vm_address(signer::address_of(account))
    }

    /// Return true if `addr` is a reserved address for the VM to call system modules.
    public fun is_vm_address(addr: address): bool {
        addr == @vm_reserved
    }

    /// Return true if `addr` is either the VM address or an Aptos Framework address.
    public fun is_reserved_address(addr: address): bool {
        is_aptos_framework_address(addr) || is_vm_address(addr)
    }
}
