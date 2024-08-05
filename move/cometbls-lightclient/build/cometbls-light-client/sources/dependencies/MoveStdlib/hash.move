/// Module which defines SHA hashes for byte vectors.
///
/// The functions in this module are natively declared both in the Move runtime
/// as in the Move prover's prelude.
module std::hash {
    native public fun sha2_256(data: vector<u8>): vector<u8>;
    native public fun sha3_256(data: vector<u8>): vector<u8>;
}
