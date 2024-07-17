#[test_only]
module std::string_tests {
    use std::string;

    #[test]
    fun test_valid_utf8() {
        let sparkle_heart = vector[240, 159, 146, 150];
        let s = string::utf8(sparkle_heart);
        assert!(string::length(&s) == 4, 22);
    }

    #[test]
    #[expected_failure(abort_code = string::EINVALID_UTF8)]
    fun test_invalid_utf8() {
        let no_sparkle_heart = vector[0, 159, 146, 150];
        let s = string::utf8(no_sparkle_heart);
        assert!(string::length(&s) == 1, 22);
    }

    #[test]
    fun test_sub_string() {
        let s = string::utf8(b"abcd");
        let sub = string::sub_string(&s, 2, 4);
        assert!(sub == string::utf8(b"cd"), 22)
    }

    #[test]
    #[expected_failure(abort_code = string::EINVALID_INDEX)]
    fun test_sub_string_invalid_boundary() {
        let sparkle_heart = vector[240, 159, 146, 150];
        let s = string::utf8(sparkle_heart);
        let _sub = string::sub_string(&s, 1, 4);
    }

    #[test]
    #[expected_failure(abort_code = string::EINVALID_INDEX)]
    fun test_sub_string_invalid_index() {
        let s = string::utf8(b"abcd");
        let _sub = string::sub_string(&s, 4, 5);
    }

    #[test]
    fun test_sub_string_empty() {
        let s = string::utf8(b"abcd");
        let sub = string::sub_string(&s, 4, 4);
        assert!(string::is_empty(&sub), 22)
    }

    #[test]
    fun test_index_of() {
        let s = string::utf8(b"abcd");
        let r = string::utf8(b"bc");
        let p = string::index_of(&s, &r);
        assert!(p == 1, 22)
    }

    #[test]
    fun test_index_of_fail() {
        let s = string::utf8(b"abcd");
        let r = string::utf8(b"bce");
        let p = string::index_of(&s, &r);
        assert!(p == 4, 22)
    }

    #[test]
    fun test_append() {
        let s = string::utf8(b"abcd");
        string::append(&mut s, string::utf8(b"ef"));
        assert!(s == string::utf8(b"abcdef"), 22)
    }

    #[test]
    fun test_insert() {
        let s = string::utf8(b"abcd");
        string::insert(&mut s, 1, string::utf8(b"xy"));
        assert!(s == string::utf8(b"axybcd"), 22)
    }
}
