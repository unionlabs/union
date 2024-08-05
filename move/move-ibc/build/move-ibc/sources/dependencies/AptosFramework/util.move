/// Utility functions used by the framework modules.
module aptos_framework::util {
    friend aptos_framework::code;
    friend aptos_framework::gas_schedule;

    /// Native function to deserialize a type T.
    ///
    /// Note that this function does not put any constraint on `T`. If code uses this function to
    /// deserialized a linear value, its their responsibility that the data they deserialize is
    /// owned.
    public(friend) native fun from_bytes<T>(bytes: vector<u8>): T;

    public fun address_from_bytes(bytes: vector<u8>): address {
        from_bytes(bytes)
    }
}
