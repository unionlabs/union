module aptos_std::type_info {
    use std::bcs;
    use std::features;
    use std::string::{Self, String};
    use std::vector;

    //
    // Error codes
    //

    const E_NATIVE_FUN_NOT_AVAILABLE: u64 = 1;

    //
    // Structs
    //

    struct TypeInfo has copy, drop, store {
        account_address: address,
        module_name: vector<u8>,
        struct_name: vector<u8>,
    }

    //
    // Public functions
    //

    public fun account_address(type_info: &TypeInfo): address {
        type_info.account_address
    }

    public fun module_name(type_info: &TypeInfo): vector<u8> {
        type_info.module_name
    }

    public fun struct_name(type_info: &TypeInfo): vector<u8> {
        type_info.struct_name
    }

    /// Returns the current chain ID, mirroring what `aptos_framework::chain_id::get()` would return, except in `#[test]`
    /// functions, where this will always return `4u8` as the chain ID, whereas `aptos_framework::chain_id::get()` will
    /// return whichever ID was passed to `aptos_framework::chain_id::initialize_for_test()`.
    public fun chain_id(): u8 {
        if (!features::aptos_stdlib_chain_id_enabled()) {
            abort(std::error::invalid_state(E_NATIVE_FUN_NOT_AVAILABLE))
        };

        chain_id_internal()
    }

    /// Return the `TypeInfo` struct containing  for the type `T`.
    public native fun type_of<T>(): TypeInfo;

    /// Return the human readable string for the type, including the address, module name, and any type arguments.
    /// Example: 0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>
    /// Or: 0x1::table::Table<0x1::string::String, 0x1::string::String>
    public native fun type_name<T>(): String;

    native fun chain_id_internal(): u8;

    /// Return the BCS size, in bytes, of value at `val_ref`.
    ///
    /// See the [BCS spec](https://github.com/diem/bcs)
    ///
    /// See `test_size_of_val()` for an analysis of common types and
    /// nesting patterns, as well as `test_size_of_val_vectors()` for an
    /// analysis of vector size dynamism.
    public fun size_of_val<T>(val_ref: &T): u64 {
        // Return vector length of vectorized BCS representation.
        vector::length(&bcs::to_bytes(val_ref))
    }

    #[test_only]
    use aptos_std::table::Table;

    #[test]
    fun test_type_of() {
        let type_info = type_of<TypeInfo>();
        assert!(account_address(&type_info) == @aptos_std, 0);
        assert!(module_name(&type_info) == b"type_info", 1);
        assert!(struct_name(&type_info) == b"TypeInfo", 2);
    }

    #[test]
    fun test_type_of_with_type_arg() {
        let type_info = type_of<Table<String, String>>();
        assert!(account_address(&type_info) == @aptos_std, 0);
        assert!(module_name(&type_info) == b"table", 1);
        assert!(struct_name(&type_info) == b"Table<0x1::string::String, 0x1::string::String>", 2);
    }

    #[test(fx = @std)]
    fun test_chain_id(fx: signer) {
        // We need to enable the feature in order for the native call to be allowed.
        features::change_feature_flags_for_testing(&fx, vector[features::get_aptos_stdlib_chain_id_feature()], vector[]);

        // The testing environment chain ID is 4u8.
        assert!(chain_id() == 4u8, 1);
    }

    #[test]
    fun test_type_name() {


        assert!(type_name<bool>() == string::utf8(b"bool"), 0);
        assert!(type_name<u8>() == string::utf8(b"u8"), 1);
        assert!(type_name<u64>() == string::utf8(b"u64"), 2);
        assert!(type_name<u128>() == string::utf8(b"u128"), 3);
        assert!(type_name<address>() == string::utf8(b"address"), 4);
        assert!(type_name<signer>() == string::utf8(b"signer"), 5);

        // vector
        assert!(type_name<vector<u8>>() == string::utf8(b"vector<u8>"), 6);
        assert!(type_name<vector<vector<u8>>>() == string::utf8(b"vector<vector<u8>>"), 7);
        assert!(type_name<vector<vector<TypeInfo>>>() == string::utf8(b"vector<vector<0x1::type_info::TypeInfo>>"), 8);


        // struct
        assert!(type_name<TypeInfo>() == string::utf8(b"0x1::type_info::TypeInfo"), 9);
        assert!(type_name<
            Table<
                TypeInfo,
                Table<u8, vector<TypeInfo>>
            >
        >() == string::utf8(b"0x1::table::Table<0x1::type_info::TypeInfo, 0x1::table::Table<u8, vector<0x1::type_info::TypeInfo>>>"), 10);
    }

    #[verify_only]
    fun verify_type_of() {
        let type_info = type_of<TypeInfo>();
        let account_address = account_address(&type_info);
        let module_name = module_name(&type_info);
        let struct_name = struct_name(&type_info);
        spec {
            assert account_address == @aptos_std;
            assert module_name == b"type_info";
            assert struct_name == b"TypeInfo";
        };
    }

    #[verify_only]
    fun verify_type_of_generic<T>() {
        let type_info = type_of<T>();
        let account_address = account_address(&type_info);
        let module_name = module_name(&type_info);
        let struct_name = struct_name(&type_info);
        spec {
            assert account_address == type_of<T>().account_address;
            assert module_name == type_of<T>().module_name;
            assert struct_name == type_of<T>().struct_name;
        };
    }
    spec verify_type_of_generic {
        aborts_if !spec_is_struct<T>();
    }

    #[test_only]
    struct CustomType has drop {}

    #[test_only]
    struct SimpleStruct has copy, drop {
        field: u8
    }

    #[test_only]
    struct ComplexStruct<T> has copy, drop {
        field_1: bool,
        field_2: u8,
        field_3: u64,
        field_4: u128,
        field_5: SimpleStruct,
        field_6: T
    }

    #[test_only]
    struct TwoBools has drop {
        bool_1: bool,
        bool_2: bool
    }

    #[test_only]
    use std::option;

    #[test(account = @0x0)]
    /// Ensure valid returns across native types and nesting schemas.
    fun test_size_of_val(
        account: &signer
    ) {
        assert!(size_of_val(&false) == 1, 0); // Bool takes 1 byte.
        assert!(size_of_val<u8>(&0) == 1, 0); // u8 takes 1 byte.
        assert!(size_of_val<u64>(&0) == 8, 0); // u64 takes 8 bytes.
        assert!(size_of_val<u128>(&0) == 16, 0); // u128 takes 16 bytes.
        // Address is a u256.
        assert!(size_of_val(&@0x0) == 32, 0);
        assert!(size_of_val(account) == 32, 0); // Signer is an address.
        // Assert custom type without fields has size 1.
        assert!(size_of_val(&CustomType{}) == 1, 0);
        // Declare a simple struct with a 1-byte field.
        let simple_struct = SimpleStruct{field: 0};
        // Assert size is indicated as 1 byte.
        assert!(size_of_val(&simple_struct) == 1, 0);
        let complex_struct = ComplexStruct<u128>{
            field_1: false,
            field_2: 0,
            field_3: 0,
            field_4: 0,
            field_5: simple_struct,
            field_6: 0
        }; // Declare a complex struct with another nested inside.
        // Assert size is bytewise sum of components.
        assert!(size_of_val(&complex_struct) == (1 + 1 + 8 + 16 + 1 + 16), 0);
        // Declare a struct with two boolean values.
        let two_bools = TwoBools{bool_1: false, bool_2: false};
        // Assert size is two bytes.
        assert!(size_of_val(&two_bools) == 2, 0);
        // Declare an empty vector of element type u64.
        let empty_vector_u64 = vector::empty<u64>();
        // Declare an empty vector of element type u128.
        let empty_vector_u128 = vector::empty<u128>();
        // Assert size is 1 byte regardless of underlying element type.
        assert!(size_of_val(&empty_vector_u64) == 1, 0);
        // Assert size is 1 byte regardless of underlying element type.
        assert!(size_of_val(&empty_vector_u128) == 1, 0);
        // Declare a bool in a vector.
        let bool_vector = vector::singleton(false);
        // Push back another bool.
        vector::push_back(&mut bool_vector, false);
        // Assert size is 3 bytes (1 per element, 1 for base vector).
        assert!(size_of_val(&bool_vector) == 3, 0);
        // Get a some option, which is implemented as a vector.
        let u64_option = option::some(0);
        // Assert size is 9 bytes (8 per element, 1 for base vector).
        assert!(size_of_val(&u64_option) == 9, 0);
        option::extract(&mut u64_option); // Remove the value inside.
        // Assert size reduces to 1 byte.
        assert!(size_of_val(&u64_option) == 1, 0);
    }

    #[test]
    /// Verify returns for base vector size at different lengths, with
    /// different underlying fixed-size elements.
    ///
    /// For a vector of length n containing fixed-size elements, the
    /// size of the vector is b + n * s bytes, where s is the size of an
    /// element in bytes, and b is a "base size" in bytes that varies
    /// with n.
    ///
    /// The base size is an artifact of vector BCS encoding, namely,
    /// with b leading bytes that declare how many elements are in the
    /// vector. Each such leading byte has a reserved control bit (e.g.
    /// is this the last leading byte?), such that 7 bits per leading
    /// byte remain for the actual element count. Hence for a single
    /// leading byte, the maximum element count that can be described is
    /// (2 ^ 7) - 1, and for b leading bytes, the maximum element count
    /// that can be described is (2 ^ 7) ^ b - 1:
    ///
    /// * b = 1,                         n < 128
    /// * b = 2,                  128 <= n < 16384
    /// * b = 3,                16384 <= n < 2097152
    /// * ...
    /// *           (2 ^ 7) ^ (b - 1) <= n < (2 ^ 7) ^ b
    /// * ...
    /// * b = 9,    72057594037927936 <= n < 9223372036854775808
    /// * b = 10, 9223372036854775808 <= n < 18446744073709551616
    ///
    /// Note that the upper bound on n for b = 10 is 2 ^ 64, rather than
    /// (2 ^ 7) ^ 10 - 1, because the former, lower figure is the
    /// maximum number of elements that can be stored in a vector in the
    /// first place, e.g. U64_MAX.
    ///
    /// In practice b > 2 is unlikely to be encountered.
    fun test_size_of_val_vectors() {
        // Declare vector base sizes.
        let (base_size_1, base_size_2, base_size_3) = (1, 2, 3);
        // A base size of 1 applies for 127 or less elements.
        let n_elems_cutoff_1 = 127; // (2 ^ 7) ^ 1 - 1.
        // A base size of 2 applies for 128 < n <= 16384 elements.
        let n_elems_cutoff_2 = 16383; // (2 ^ 7) ^ 2 - 1.
        let vector_u64 = vector::empty<u64>(); // Declare empty vector.
        let null_element = 0; // Declare a null element.
        // Get element size.
        let element_size = size_of_val(&null_element);
        // Vector size is 1 byte when length is 0.
        assert!(size_of_val(&vector_u64) == base_size_1, 0);
        let i = 0; // Declare loop counter.
        while (i < n_elems_cutoff_1) { // Iterate until first cutoff:
            // Add an element.
            vector::push_back(&mut vector_u64, null_element);
            i = i + 1; // Increment counter.
        };
        // Vector base size is still 1 byte.
        assert!(size_of_val(&vector_u64) - element_size * i == base_size_1, 0);
        // Add another element, exceeding the cutoff.
        vector::push_back(&mut vector_u64, null_element);
        i = i + 1; // Increment counter.
        // Vector base size is now 2 bytes.
        assert!(size_of_val(&vector_u64) - element_size * i == base_size_2, 0);
        while (i < n_elems_cutoff_2) { // Iterate until second cutoff:
            // Add an element.
            vector::push_back(&mut vector_u64, null_element);
            i = i + 1; // Increment counter.
        };
        // Vector base size is still 2 bytes.
        assert!(size_of_val(&vector_u64) - element_size * i == base_size_2, 0);
        // Add another element, exceeding the cutoff.
        vector::push_back(&mut vector_u64, null_element);
        i = i + 1; // Increment counter.
        // Vector base size is now 3 bytes.
        assert!(size_of_val(&vector_u64) - element_size * i == base_size_3, 0);
        // Repeat for custom struct.
        let vector_complex = vector::empty<ComplexStruct<address>>();
        // Declare a null element.
        let null_element = ComplexStruct{
            field_1: false,
            field_2: 0,
            field_3: 0,
            field_4: 0,
            field_5: SimpleStruct{field: 0},
            field_6: @0x0
        };
        element_size = size_of_val(&null_element); // Get element size.
        // Vector size is 1 byte when length is 0.
        assert!(size_of_val(&vector_complex) == base_size_1, 0);
        i = 0; // Re-initialize loop counter.
        while (i < n_elems_cutoff_1) { // Iterate until first cutoff:
            // Add an element.
            vector::push_back(&mut vector_complex, copy null_element);
            i = i + 1; // Increment counter.
        };
        assert!( // Vector base size is still 1 byte.
            size_of_val(&vector_complex) - element_size * i == base_size_1, 0);
        // Add another element, exceeding the cutoff.
        vector::push_back(&mut vector_complex, null_element);
        i = i + 1; // Increment counter.
        assert!( // Vector base size is now 2 bytes.
            size_of_val(&vector_complex) - element_size * i == base_size_2, 0);
        while (i < n_elems_cutoff_2) { // Iterate until second cutoff:
            // Add an element.
            vector::push_back(&mut vector_complex, copy null_element);
            i = i + 1; // Increment counter.
        };
        assert!( // Vector base size is still 2 bytes.
            size_of_val(&vector_complex) - element_size * i == base_size_2, 0);
        // Add another element, exceeding the cutoff.
        vector::push_back(&mut vector_complex, null_element);
        i = i + 1; // Increment counter.
        assert!( // Vector base size is now 3 bytes.
            size_of_val(&vector_complex) - element_size * i == base_size_3, 0);
    }

}
