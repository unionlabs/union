cc_test(
    name = "grpc_test",
    srcs = [
        "grpctest.cpp",
        "message_builder_test.cpp",
    ],
    copts = ["-Itests"],
    # This is required.
    linkstatic = 1,
    deps = [
        "//tests:monster_test_cc_fbs",
        "//tests:monster_test_grpc",
        "//tests:test_assert",
        "//tests:test_builder",
        "@com_github_grpc_grpc//:grpc++",
    ],
)
