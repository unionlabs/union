module ibc::sample_ibc {
    use std::string;

    use aptos_framework::function_info;

    use ibc::dynamic_dispatch_app;
    use ibc::dynamic_dispatch_app2;
    use ibc::dispatcher;
    use ibc::engine;
    use std::option;
    use std::vector;
    use ibc::packet::{Self, Packet};

    // #[test(publisher = @ibc)]
    // public entry fun recv_packet(publisher: &signer) {
    //     setup(publisher);
    //     let packet =
    //         packet::new(
    //             1,
    //             2,
    //             3,
    //             x"000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000003e8",
    //             100,
    //             200
    //         );

    //     let param =
    //         dynamic_dispatch_app::new_dynamic_dispatch_param(
    //             0,
    //             option::some(packet),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none()
    //         );
    //     engine::dispatch<dynamic_dispatch_app::TestProof, dynamic_dispatch_app::DynamicDispatchParam>(
    //         param
    //     );
    // }

    // #[test(publisher = @ibc)]
    // fun acknowledge_packet(publisher: &signer) {
    //     setup(publisher);
    //     let packet =
    //         packet::new(
    //             1,
    //             2,
    //             3,
    //             x"000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000003e8",
    //             100,
    //             200
    //         );

    //     let param =
    //         dynamic_dispatch_app::new_dynamic_dispatch_param(
    //             1,
    //             option::some(packet),
    //             option::some(b"333333"),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none(),
    //             option::none()
    //         );
    //     engine::dispatch<dynamic_dispatch_app::TestProof, dynamic_dispatch_app::DynamicDispatchParam>(
    //         param
    //     );
    // }


    #[test(publisher = @ibc)]
    #[expected_failure(abort_code = 0)]
    fun test_register(publisher: &signer) {
        setup(publisher);

        let param =
            dynamic_dispatch_app::new_dynamic_dispatch_param(
                14,
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
                option::none(),
                option::none()
            );

        engine::dispatch<dynamic_dispatch_app::PingPongProof, dynamic_dispatch_app::DynamicDispatchParam>(
            param
        );

        let param2 =
                dynamic_dispatch_app2::new_dynamic_dispatch_param(
                    14,
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
                    option::none(),
                    option::none()
                );


        engine::dispatch<dynamic_dispatch_app2::TestProof2, dynamic_dispatch_app2::DynamicDispatchParam>(
            param2
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
        dispatcher::register<dynamic_dispatch_app::PingPongProof>(cb, dynamic_dispatch_app::new_ping_pong_proof(), b"something");

        let cb =
            function_info::new_function_info(
                publisher,
                string::utf8(b"dynamic_dispatch_app2"),
                string::utf8(b"on_packet")
            );
        dispatcher::register<dynamic_dispatch_app2::TestProof2>(cb, dynamic_dispatch_app2::new_proof2(),b"something2");

        std::debug::print(&string::utf8(b"Setup is done?"));
    }
}
