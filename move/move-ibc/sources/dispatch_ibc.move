module ibc::sample_ibc {
    use std::string;

    use aptos_framework::function_info;

    use ibc::dynamic_dispatch_app;
    use ibc::dispatcher;
    use ibc::engine;

    #[test(publisher = @ibc)]
    fun recv_packet(publisher: &signer) {
        setup(publisher);
        let data:u64 = 321;
        // Here the recvpacket function under the ibc will be implemented
        engine::dispatch<dynamic_dispatch_app::TestProof, u64>(data); 
        // std::debug::print(&result_data);
        // assert(data == result_data);
    }

    fun setup(publisher: &signer) {
        dispatcher::init_module_for_testing(publisher);

        let cb = function_info::new_function_info(
            publisher,
            string::utf8(b"dynamic_dispatch_app"),
            string::utf8(b"on_recv_packet"),
        );
        dispatcher::register(cb, dynamic_dispatch_app::new_proof());
        std::debug::print(&string::utf8(b"Setup done"));
    }
}