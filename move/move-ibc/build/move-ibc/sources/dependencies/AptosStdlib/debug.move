/// Module providing debug functionality.
module aptos_std::debug {
    use std::string::String;

    public fun print<T>(x: &T) {
        native_print(format(x));
    }

    public fun print_stack_trace() {
        native_print(native_stack_trace());
    }

    inline fun format<T>(x: &T): String {
        aptos_std::string_utils::debug_string(x)
    }

    native fun native_print(x: String);
    native fun native_stack_trace(): String;

    #[test_only]
    use std::vector;

    #[test_only]
    struct Foo has drop {}
    #[test_only]
    struct Bar has drop { x: u128, y: Foo, z: bool }
    #[test_only]
    struct Box<T> has drop { x: T }

    #[test_only]
    struct GenericStruct<phantom T> has drop {
        val: u64,
    }

    #[test_only]
    struct TestInner has drop {
        val: u128,
        vec: vector<u128>,
        msgs: vector<vector<u8>>
    }

    #[test_only]
    struct TestStruct has drop {
        addr: address,
        number: u8,
        bytes: vector<u8>,
        name: String,
        vec: vector<TestInner>,
    }

    #[test_only]
    fun assert_equal<T>(x: &T, expected: vector<u8>) {
        if (std::string::bytes(&format(x)) != &expected) {
            print(&format(x));
            print(&std::string::utf8(expected));
            assert!(false, 1);
        };
    }

    #[test_only]
    fun assert_string_equal(x: vector<u8>, expected: vector<u8>) {
        assert!(std::string::bytes(&format(&std::string::utf8(x))) == &expected, 1);
    }

    #[test]
    public fun test()  {
        let x = 42;
        assert_equal(&x, b"42");

        let v = vector::empty();
        vector::push_back(&mut v, 100);
        vector::push_back(&mut v, 200);
        vector::push_back(&mut v, 300);
        assert_equal(&v, b"[ 100, 200, 300 ]");

        let foo = Foo {};
        assert_equal(&foo, b"0x1::debug::Foo {\n  dummy_field: false\n}");

        let bar = Bar { x: 404, y: Foo {}, z: true };
        assert_equal(&bar, b"0x1::debug::Bar {\n  x: 404,\n  y: 0x1::debug::Foo {\n    dummy_field: false\n  },\n  z: true\n}");

        let box = Box { x: Foo {} };
        assert_equal(&box, b"0x1::debug::Box<0x1::debug::Foo> {\n  x: 0x1::debug::Foo {\n    dummy_field: false\n  }\n}");
    }

    #[test]
    fun test_print_string() {
        let str_bytes = b"Hello, sane Move debugging!";

        assert_equal(&str_bytes, b"0x48656c6c6f2c2073616e65204d6f766520646562756767696e6721");

        let str = std::string::utf8(str_bytes);
        assert_equal(&str, b"\"Hello, sane Move debugging!\"");
    }

    #[test]
    fun test_print_quoted_string() {
        let str_bytes = b"Can you say \"Hel\\lo\"?";

        let str = std::string::utf8(str_bytes);
        assert_equal(&str, b"\"Can you say \\\"Hel\\\\lo\\\"?\"");
    }


    #[test_only]
    use std::features;
    #[test(s = @0x123)]
    fun test_print_primitive_types(s: signer) {
        let u8 = 255u8;
        assert_equal(&u8, b"255");

        let u16 = 65535u16;
        assert_equal(&u16, b"65535");

        let u32 = 4294967295u32;
        assert_equal(&u32, b"4294967295");

        let u64 = 18446744073709551615u64;
        assert_equal(&u64, b"18446744073709551615");

        let u128 = 340282366920938463463374607431768211455u128;
        assert_equal(&u128, b"340282366920938463463374607431768211455");

        let u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935u256;
        assert_equal(&u256, b"115792089237316195423570985008687907853269984665640564039457584007913129639935");

        let bool = false;
        assert_equal(&bool, b"false");

        let bool = true;
        assert_equal(&bool, b"true");

        let a = @0x1234c0ffee;
        assert_equal(&a, b"@0x1234c0ffee");

        if (features::signer_native_format_fix_enabled()) {
            let signer = s;
            assert_equal(&signer, b"signer(@0x123)");
        }
    }

    const MSG_1 : vector<u8> = b"abcdef";
    const MSG_2 : vector<u8> = b"123456";

    #[test]
    fun test_print_struct() {
        let obj = TestInner {
            val: 100,
            vec: vector[200u128, 400u128],
            msgs: vector[MSG_1, MSG_2],
        };

        assert_equal(&obj, b"0x1::debug::TestInner {\n  val: 100,\n  vec: [ 200, 400 ],\n  msgs: [\n    0x616263646566,\n    0x313233343536\n  ]\n}");

        let obj = TestInner {
            val: 10,
            vec: vector[],
            msgs: vector[],
        };

        assert_equal(&obj, b"0x1::debug::TestInner {\n  val: 10,\n  vec: [],\n  msgs: []\n}");
    }

    #[test(s1 = @0x123, s2 = @0x456)]
    fun test_print_vectors(s1: signer, s2: signer) {
        let v_u8 = x"ffabcdef";
        assert_equal(&v_u8, b"0xffabcdef");

        let v_u16 = vector[16u16, 17u16, 18u16, 19u16];
        assert_equal(&v_u16, b"[ 16, 17, 18, 19 ]");

        let v_u32 = vector[32u32, 33u32, 34u32, 35u32];
        assert_equal(&v_u32, b"[ 32, 33, 34, 35 ]");

        let v_u64 = vector[64u64, 65u64, 66u64, 67u64];
        assert_equal(&v_u64, b"[ 64, 65, 66, 67 ]");

        let v_u128 = vector[128u128, 129u128, 130u128, 131u128];
        assert_equal(&v_u128, b"[ 128, 129, 130, 131 ]");

        let v_u256 = vector[256u256, 257u256, 258u256, 259u256];
        assert_equal(&v_u256, b"[ 256, 257, 258, 259 ]");

        let v_bool = vector[true, false];
        assert_equal(&v_bool, b"[ true, false ]");

        let v_addr = vector[@0x1234, @0x5678, @0xabcdef];
        assert_equal(&v_addr, b"[ @0x1234, @0x5678, @0xabcdef ]");

        if (features::signer_native_format_fix_enabled()) {
            let v_signer = vector[s1, s2];
            assert_equal(&v_signer, b"[ signer(@0x123), signer(@0x456) ]");
        };

        let v = vector[
            TestInner {
                val: 4u128,
                vec: vector[127u128, 128u128],
                msgs: vector[x"00ff", x"abcd"],
            },
            TestInner {
                val: 8u128 ,
                vec: vector[128u128, 129u128],
                msgs: vector[x"0000"],
            }
        ];
        assert_equal(&v, b"[\n  0x1::debug::TestInner {\n    val: 4,\n    vec: [ 127, 128 ],\n    msgs: [\n      0x00ff,\n      0xabcd\n    ]\n  },\n  0x1::debug::TestInner {\n    val: 8,\n    vec: [ 128, 129 ],\n    msgs: [\n      0x0000\n    ]\n  }\n]");
    }

    #[test(s1 = @0x123, s2 = @0x456)]
    fun test_print_vector_of_vectors(s1: signer, s2: signer) {
        let v_u8 = vector[x"ffab", x"cdef"];
        assert_equal(&v_u8, b"[\n  0xffab,\n  0xcdef\n]");

        let v_u16 = vector[vector[16u16, 17u16], vector[18u16, 19u16]];
        assert_equal(&v_u16, b"[\n  [ 16, 17 ],\n  [ 18, 19 ]\n]");

        let v_u32 = vector[vector[32u32, 33u32], vector[34u32, 35u32]];
        assert_equal(&v_u32, b"[\n  [ 32, 33 ],\n  [ 34, 35 ]\n]");

        let v_u64 = vector[vector[64u64, 65u64], vector[66u64, 67u64]];
        assert_equal(&v_u64, b"[\n  [ 64, 65 ],\n  [ 66, 67 ]\n]");

        let v_u128 = vector[vector[128u128, 129u128], vector[130u128, 131u128]];
        assert_equal(&v_u128, b"[\n  [ 128, 129 ],\n  [ 130, 131 ]\n]");

        let v_u256 = vector[vector[256u256, 257u256], vector[258u256, 259u256]];
        assert_equal(&v_u256, b"[\n  [ 256, 257 ],\n  [ 258, 259 ]\n]");

        let v_bool = vector[vector[true, false], vector[false, true]];
        assert_equal(&v_bool, b"[\n  [ true, false ],\n  [ false, true ]\n]");

        let v_addr = vector[vector[@0x1234, @0x5678], vector[@0xabcdef, @0x9999]];
        assert_equal(&v_addr, b"[\n  [ @0x1234, @0x5678 ],\n  [ @0xabcdef, @0x9999 ]\n]");

        if (features::signer_native_format_fix_enabled()) {
            let v_signer = vector[vector[s1], vector[s2]];
            assert_equal(&v_signer, b"[\n  [ signer(@0x123) ],\n  [ signer(@0x456) ]\n]");
        };

        let v = vector[
            vector[
                TestInner { val: 4u128, vec: vector[127u128, 128u128], msgs: vector[] },
                TestInner { val: 8u128 , vec: vector[128u128, 129u128], msgs: vector[] }
            ],
            vector[
                TestInner { val: 4u128, vec: vector[127u128, 128u128], msgs: vector[] },
                TestInner { val: 8u128 , vec: vector[128u128, 129u128], msgs: vector[] }
            ]
        ];
        assert_equal(&v, b"[\n  [\n    0x1::debug::TestInner {\n      val: 4,\n      vec: [ 127, 128 ],\n      msgs: []\n    },\n    0x1::debug::TestInner {\n      val: 8,\n      vec: [ 128, 129 ],\n      msgs: []\n    }\n  ],\n  [\n    0x1::debug::TestInner {\n      val: 4,\n      vec: [ 127, 128 ],\n      msgs: []\n    },\n    0x1::debug::TestInner {\n      val: 8,\n      vec: [ 128, 129 ],\n      msgs: []\n    }\n  ]\n]");
    }

    #[test]
    fun test_print_nested_struct() {
        let obj = TestStruct {
            addr: @0x1,
            number: 255u8,
            bytes: x"c0ffee",
            name: std::string::utf8(b"He\"llo"),
            vec: vector[
                TestInner { val: 1, vec: vector[130u128, 131u128], msgs: vector[] },
                TestInner { val: 2, vec: vector[132u128, 133u128], msgs: vector[] }
            ],
        };

        assert_equal(&obj, b"0x1::debug::TestStruct {\n  addr: @0x1,\n  number: 255,\n  bytes: 0xc0ffee,\n  name: \"He\\\"llo\",\n  vec: [\n    0x1::debug::TestInner {\n      val: 1,\n      vec: [ 130, 131 ],\n      msgs: []\n    },\n    0x1::debug::TestInner {\n      val: 2,\n      vec: [ 132, 133 ],\n      msgs: []\n    }\n  ]\n}");
    }

    #[test]
    fun test_print_generic_struct() {
        let obj = GenericStruct<Foo> {
            val: 60u64,
        };

        assert_equal(&obj, b"0x1::debug::GenericStruct<0x1::debug::Foo> {\n  val: 60\n}");
    }
}
