module zkgm::multiplex {
    use zkgm::ethabi;

    use std::vector;

    struct Multiplex has copy, drop, store {
        sender: vector<u8>,
        eureka: bool,
        contract_address: vector<u8>,
        contract_calldata: vector<u8>
    }

    fun encode(multiplex: &Multiplex): vector<u8> {
        let buf = vector::empty();

        let sender = vector::empty();
        let contract_address = vector::empty();
        let contract_calldata = vector::empty();
    }
}
