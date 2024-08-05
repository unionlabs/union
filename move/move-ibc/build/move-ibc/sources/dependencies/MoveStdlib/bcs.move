/// Utility for converting a Move value to its binary representation in BCS (Binary Canonical
/// Serialization). BCS is the binary encoding for Move resources and other non-module values
/// published on-chain. See https://github.com/aptos-labs/bcs#binary-canonical-serialization-bcs for more
/// details on BCS.
module std::bcs {
    /// Return the binary representation of `v` in BCS (Binary Canonical Serialization) format
    native public fun to_bytes<MoveValue>(v: &MoveValue): vector<u8>;

    // ==============================
    // Module Specification
    spec module {} // switch to module documentation context

    spec module {
        /// Native function which is defined in the prover's prelude.
        native fun serialize<MoveValue>(v: &MoveValue): vector<u8>;
    }
}
