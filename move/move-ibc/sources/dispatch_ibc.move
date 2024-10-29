module ibc::sample_ibc {
    use std::string;

    use aptos_framework::function_info;

    use ibc::dynamic_dispatch_app;
    use ibc::dispatcher;
    use ibc::engine;
    use std::option;
    use std::vector;
    use ibc::packet::{Self, Packet};

    #[test(publisher = @ibc)]
    public entry fun recv_packet(publisher: &signer) {
        setup(publisher);
        let packet = packet::new(1, 2, 3, x"010203", 100, 200);

        let param =
            dynamic_dispatch_app::new_dynamic_dispatch_param(
                0,
                option::some(packet),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none()
            );
        engine::dispatch<dynamic_dispatch_app::TestProof, dynamic_dispatch_app::DynamicDispatchParam>(
            param
        );
    }

    #[test(publisher = @ibc)]
    fun acknowledge_packet(publisher: &signer) {
        setup(publisher);
        let packet = packet::new(1, 2, 3, x"010203", 100, 200);

        let param =
            dynamic_dispatch_app::new_dynamic_dispatch_param(
                1,
                option::some(packet),
                option::some(b"333333"),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none(),
                option::none()
            );
        engine::dispatch<dynamic_dispatch_app::TestProof, dynamic_dispatch_app::DynamicDispatchParam>(
            param
        );
    }

    fun setup(publisher: &signer) {
        dispatcher::init_module_for_testing(publisher);

        let cb =
            function_info::new_function_info(
                publisher,
                string::utf8(b"dynamic_dispatch_app"),
                string::utf8(b"on_packet")
            );
        dispatcher::register(cb, dynamic_dispatch_app::new_proof());

    }
}
