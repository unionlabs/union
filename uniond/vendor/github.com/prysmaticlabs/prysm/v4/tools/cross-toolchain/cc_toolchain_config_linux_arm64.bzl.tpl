load(
    "@bazel_tools//tools/cpp:cc_toolchain_config_lib.bzl",
    "action_config",
    "feature",
    "feature_set",
    "flag_group",
    "flag_set",
    "make_variable",
    "tool",
    "tool_path",
    "with_feature_set",
)

load("@bazel_tools//tools/build_defs/cc:action_names.bzl", "ACTION_NAMES")

all_compile_actions = [
    ACTION_NAMES.c_compile,
    ACTION_NAMES.cpp_compile,
    ACTION_NAMES.linkstamp_compile,
    ACTION_NAMES.assemble,
    ACTION_NAMES.preprocess_assemble,
    ACTION_NAMES.cpp_header_parsing,
    ACTION_NAMES.cpp_module_compile,
    ACTION_NAMES.cpp_module_codegen,
    ACTION_NAMES.clif_match,
    ACTION_NAMES.lto_backend,
]

all_cpp_compile_actions = [
    ACTION_NAMES.cpp_compile,
    ACTION_NAMES.linkstamp_compile,
    ACTION_NAMES.cpp_header_parsing,
    ACTION_NAMES.cpp_module_compile,
    ACTION_NAMES.cpp_module_codegen,
    ACTION_NAMES.clif_match,
]

all_link_actions = [
    ACTION_NAMES.cpp_link_executable,
    ACTION_NAMES.cpp_link_dynamic_library,
    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
]


def _impl(ctx):
    toolchain_identifier = "clang-linux-cross"
    compiler = "clang"
    abi_version = "clang"
    abi_libc_version = "glibc_unknown"
    target_libc = "glibc_unknown"
    target_cpu = ctx.attr.target.split("-")[0]

    if (target_cpu == "aarch64"):
        sysroot = "/usr/aarch64-linux-gnu"
        include_path_prefix = sysroot
    elif (target_cpu == "x86_64"):
        sysroot = "/"
        include_path_prefix = "/usr"
    else:
        fail("Unreachable")

    if (target_cpu == "aarch64"):
        cross_system_include_dirs = [
            include_path_prefix + "/include/c++/v1",
            include_path_prefix + "/lib/clang/12.0.0/include",
        ]
    else:
        cross_system_include_dirs = [
            include_path_prefix + "/include/c++/v1",
            include_path_prefix + "/lib/clang/12.0.0/include",
            include_path_prefix + "/include/x86_64-linux-gnu",
        ]

    cross_system_include_dirs += [
        include_path_prefix + "/include/",
        include_path_prefix + "/include/linux",
        include_path_prefix + "/include/asm",
        include_path_prefix + "/include/asm-generic",
    ]

    if (target_cpu == "aarch64"):
        cross_system_lib_dirs = [
            "/usr/" + ctx.attr.target + "/lib",
        ]
    else:
        cross_system_lib_dirs = [
            "/usr/lib/x86_64-linux-gnu/",
        ]

    cross_system_lib_dirs += [
        "/usr/lib/gcc/x86_64-linux-gnu/8",
    ]

    opt_feature = feature(name = "opt")
    dbg_feature = feature(name = "dbg")
    fastbuild_feature = feature(name = "fastbuild")
    random_seed_feature = feature(name = "random_seed", enabled = True)
    supports_pic_feature = feature(name = "supports_pic", enabled = True)
    supports_dynamic_linker_feature = feature(name = "supports_dynamic_linker", enabled = True)

    unfiltered_compile_flags_feature = feature(
        name = "unfiltered_compile_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = all_compile_actions,
                flag_groups = [
                    flag_group(
                        flags = [
                            "-no-canonical-prefixes",
                            "-Wno-builtin-macro-redefined",
                            "-D__DATE__=\"redacted\"",
                            "-D__TIMESTAMP__=\"redacted\"",
                            "-D__TIME__=\"redacted\"",
                        ],
                    ),
                ],
            ),
        ],
    )

    # explicit arch specific system includes
    system_include_flags = []
    for d in cross_system_include_dirs:
        system_include_flags += ["-idirafter", d]

    default_compile_flags_feature = feature(
        name = "default_compile_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = all_compile_actions,
                flag_groups = [
                    flag_group(
                        flags = [
                            "--target=" + ctx.attr.target,
                            "-nostdinc",
                            "-U_FORTIFY_SOURCE",
                            "-fstack-protector",
                            "-fno-omit-frame-pointer",
                            "-fcolor-diagnostics",
                            "-Wall",
                            "-Wthread-safety",
                            "-Wself-assign",
                        ] + system_include_flags,
                    ),
                ],
            ),
            flag_set(
                actions = all_compile_actions,
                flag_groups = [flag_group(flags = ["-g", "-fstandalone-debug"])],
                with_features = [with_feature_set(features = ["dbg"])],
            ),
            flag_set(
                actions = all_compile_actions,
                flag_groups = [
                    flag_group(
                        flags = [
                            "-g0",
                            "-O2",
                            "-D_FORTIFY_SOURCE=1",
                            "-DNDEBUG",
                            "-ffunction-sections",
                            "-fdata-sections",
                        ],
                    ),
                ],
                with_features = [with_feature_set(features = ["opt"])],
            ),
            flag_set(
                actions = all_cpp_compile_actions,
                flag_groups = [flag_group(flags = ["-std=c++17", "-nostdinc++"])],
            ),
        ],
    )

    additional_link_flags = [
        "-l:libc++.a",
        "-l:libc++abi.a",
        "-l:libunwind.a",
        "-lpthread",
        "-ldl",
        "-rtlib=compiler-rt",
    ]

    default_link_flags_feature = feature(
        name = "default_link_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = all_link_actions,
                flag_groups = [
                    flag_group(
                        flags = additional_link_flags + [
                            "--target=" + ctx.attr.target,
                            "-lm",
                            "-no-canonical-prefixes",
                            "-fuse-ld=lld",
                            "-Wl,--build-id=md5",
                            "-Wl,--hash-style=gnu",
                            "-Wl,-z,relro,-z,now",
                        ] + ["-L" + d for d in cross_system_lib_dirs],
                    ),
                ],
            ),
            flag_set(
                actions = all_link_actions,
                flag_groups = [flag_group(flags = ["-Wl,--gc-sections"])],
                with_features = [with_feature_set(features = ["opt"])],
            ),
        ],
    )

    objcopy_embed_flags_feature = feature(
        name = "objcopy_embed_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = ["objcopy_embed_data"],
                flag_groups = [flag_group(flags = ["-I", "binary"])],
            ),
        ],
    )

    user_compile_flags_feature = feature(
        name = "user_compile_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = all_compile_actions,
                flag_groups = [
                    flag_group(
                        expand_if_available = "user_compile_flags",
                        flags = ["%{user_compile_flags}"],
                        iterate_over = "user_compile_flags",
                    ),
                ],
            ),
        ],
    )

    sysroot_feature = feature(
        name = "sysroot",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = all_compile_actions + all_link_actions,
                flag_groups = [
                    flag_group(
                        expand_if_available = "sysroot",
                        flags = ["--sysroot=%{sysroot}"],
                    ),
                ],
            ),
        ],
    )

    coverage_feature = feature(
        name = "coverage",
        flag_sets = [
            flag_set(
                actions = all_compile_actions,
                flag_groups = [
                    flag_group(
                        flags = ["-fprofile-instr-generate", "-fcoverage-mapping"],
                    ),
                ],
            ),
            flag_set(
                actions = all_link_actions,
                flag_groups = [flag_group(flags = ["-fprofile-instr-generate"])],
            ),
        ],
        provides = ["profile"],
    )

    features = [
        opt_feature,
        fastbuild_feature,
        dbg_feature,
        random_seed_feature,
        supports_pic_feature,
        supports_dynamic_linker_feature,
        unfiltered_compile_flags_feature,
        default_link_flags_feature,
        default_compile_flags_feature,
        objcopy_embed_flags_feature,
        user_compile_flags_feature,
        sysroot_feature,
        coverage_feature,
    ]

    tool_paths = [
        tool_path(name = "ld", path = "/usr/bin/ld.lld"),
        tool_path(name = "cpp", path = "/usr/bin/clang-cpp"),
        tool_path(name = "dwp", path = "/usr/bin/llvm-dwp"),
        tool_path(name = "gcov", path = "/usr/bin/llvm-profdata"),
        tool_path(name = "nm", path = "/usr/bin/llvm-nm"),
        tool_path(name = "objcopy", path = "/usr/bin/llvm-objcopy"),
        tool_path(name = "objdump", path = "/usr/bin/llvm-objdump"),
        tool_path(name = "strip", path = "/usr/bin/strip"),
        tool_path(name = "gcc", path = "/usr/bin/clang"),
        tool_path(name = "ar", path = "/usr/bin/llvm-ar"),
    ]

    return cc_common.create_cc_toolchain_config_info(
        ctx = ctx,
        features = features,
        abi_version = abi_version,
        abi_libc_version = abi_libc_version,
        builtin_sysroot = sysroot,
        compiler = compiler,
        cxx_builtin_include_directories = cross_system_include_dirs,
        host_system_name = "x86_64-unknown-linux-gnu",
        target_cpu = target_cpu,
        target_libc = target_libc,
        target_system_name = ctx.attr.target,
        tool_paths = tool_paths,
        toolchain_identifier = toolchain_identifier,
    )

arm64_cc_toolchain_config = rule(
    implementation = _impl,
    attrs = {
        "target": attr.string(mandatory = True),
        "stdlib": attr.string(),
    },
    provides = [CcToolchainConfigInfo],
)
