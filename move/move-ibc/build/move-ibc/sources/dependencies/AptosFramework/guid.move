/// A module for generating globally unique identifiers
module aptos_framework::guid {
    friend aptos_framework::account;
    friend aptos_framework::object;

    /// A globally unique identifier derived from the sender's address and a counter
    struct GUID has drop, store {
        id: ID
    }

    /// A non-privileged identifier that can be freely created by anyone. Useful for looking up GUID's.
    struct ID has copy, drop, store {
        /// If creation_num is `i`, this is the `i+1`th GUID created by `addr`
        creation_num: u64,
        /// Address that created the GUID
        addr: address
    }

    /// GUID generator must be published ahead of first usage of `create_with_capability` function.
    const EGUID_GENERATOR_NOT_PUBLISHED: u64 = 0;

    /// Create and return a new GUID from a trusted module.
    public(friend) fun create(addr: address, creation_num_ref: &mut u64): GUID {
        let creation_num = *creation_num_ref;
        *creation_num_ref = creation_num + 1;
        GUID {
            id: ID {
                creation_num,
                addr,
            }
        }
    }

    /// Create a non-privileged id from `addr` and `creation_num`
    public fun create_id(addr: address, creation_num: u64): ID {
        ID { creation_num, addr }
    }

    /// Get the non-privileged ID associated with a GUID
    public fun id(guid: &GUID): ID {
        guid.id
    }

    /// Return the account address that created the GUID
    public fun creator_address(guid: &GUID): address {
        guid.id.addr
    }

    /// Return the account address that created the guid::ID
    public fun id_creator_address(id: &ID): address {
        id.addr
    }

    /// Return the creation number associated with the GUID
    public fun creation_num(guid: &GUID): u64 {
        guid.id.creation_num
    }

    /// Return the creation number associated with the guid::ID
    public fun id_creation_num(id: &ID): u64 {
        id.creation_num
    }

    /// Return true if the GUID's ID is `id`
    public fun eq_id(guid: &GUID, id: &ID): bool {
        &guid.id == id
    }
}
