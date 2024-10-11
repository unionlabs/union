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
    "artifact_name_pattern",
    "env_set",
    "env_entry",
)

load("@bazel_tools//tools/build_defs/cc:action_names.bzl", "ACTION_NAMES")

all_link_actions = [
    ACTION_NAMES.cpp_link_executable,
    ACTION_NAMES.cpp_link_dynamic_library,
    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
]

def _impl(ctx):
    toolchain_identifier = "msys_x64_mingw"
    host_system_name = "local"
    target_system_name = "local"
    target_cpu = "x64_windows"
    target_libc = "mingw"
    compiler = "mingw-gcc"
    abi_version = "local"
    abi_libc_version = "local"
    cc_target_os = None
    builtin_sysroot = None
    action_configs = []

    install = "/usr/x86_64-w64-mingw32/"
    gcc_libpath = "/usr/lib/gcc/x86_64-w64-mingw32/10-win32/"
    bin_prefix = "/usr/bin/x86_64-w64-mingw32-"


    targets_windows_feature = feature(
        name = "targets_windows",
        implies = ["copy_dynamic_libraries_to_binary"],
        enabled = True,
    )

    copy_dynamic_libraries_to_binary_feature = feature(name = "copy_dynamic_libraries_to_binary")

    gcc_env_feature = feature(
        name = "gcc_env",
        enabled = True,
        env_sets = [
            env_set(
                actions = [
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.cpp_module_codegen,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.cpp_link_executable,
                    ACTION_NAMES.cpp_link_dynamic_library,
                    ACTION_NAMES.cpp_link_nodeps_dynamic_library,
                    ACTION_NAMES.cpp_link_static_library,
                ],
                env_entries = [
                    env_entry(key = "PATH", value = "NOT_USED"),
                ],
            ),
        ],
    )

    msys_mingw_flags = [
        "-B " + install + "bin",
        "-nostdinc",
        "-U_FORTIFY_SOURCE",
        "-fstack-protector",
        "-fno-omit-frame-pointer",
        "-fcolor-diagnostics",
        "-Wall",
        "-Wthread-safety",
        "-Wself-assign",
        "-x c++",
        "-lstdc++",
        "-lpthread"
        ]

    msys_mingw_link_flags = [
        "-l:libstdc++.a",
        "-L" + install + "lib",
        "-L/usr/lib/gcc/x86_64-w64-mingw32/8.3-w32",
        "-v",
        "-lm",
        "-no-canonical-prefixes",
    ]

    default_compile_flags_feature = feature(
        name = "default_compile_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = [
                    ACTION_NAMES.assemble,
                    ACTION_NAMES.preprocess_assemble,
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.c_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.cpp_module_codegen,
                    ACTION_NAMES.lto_backend,
                    ACTION_NAMES.clif_match,
                ],
            ),
            flag_set(
               actions = [
                    ACTION_NAMES.linkstamp_compile,
                    ACTION_NAMES.cpp_compile,
                    ACTION_NAMES.cpp_header_parsing,
                    ACTION_NAMES.cpp_module_compile,
                    ACTION_NAMES.cpp_module_codegen,
                    ACTION_NAMES.lto_backend,
                    ACTION_NAMES.clif_match,
                ],
                flag_groups = ([flag_group(flags = msys_mingw_flags)] if msys_mingw_flags else []),
            ),
        ],
    )

    compiler_param_file_feature = feature(
        name = "compiler_param_file",
    )

    default_link_flags_feature = feature(
        name = "default_link_flags",
        enabled = True,
        flag_sets = [
            flag_set(
                actions = all_link_actions,
                flag_groups = ([flag_group(flags = msys_mingw_link_flags)] if msys_mingw_link_flags else []),
            ),
        ],
    )

    supports_dynamic_linker_feature = feature(name = "supports_dynamic_linker", enabled = True)

    features = [
        targets_windows_feature,
        copy_dynamic_libraries_to_binary_feature,
        gcc_env_feature,
        default_compile_flags_feature,
        compiler_param_file_feature,
        default_link_flags_feature,
        supports_dynamic_linker_feature,
    ]

    cxx_builtin_include_directories = [
        install +"include",
        gcc_libpath +"include",
        gcc_libpath +"include-fixed",
        "/usr/share/mingw-w64/include/"
    ]

    artifact_name_patterns = [
        artifact_name_pattern(
            category_name = "executable",
            prefix = "",
            extension = ".exe",
        ),
    ]

    make_variables = []
    tool_paths = [
        tool_path(name = "ld", path = bin_prefix + "ld"),
        tool_path(name = "cpp", path = bin_prefix + "cpp"),
        tool_path(name = "gcov", path = "/usr/bin/gcov"),
        tool_path(name = "nm", path = bin_prefix + "nm"),
        tool_path(name = "objcopy", path = bin_prefix + "objcopy"),
        tool_path(name = "objdump", path = bin_prefix + "objdump"),
        tool_path(name = "strip", path = bin_prefix + "strip"),
        tool_path(name = "gcc", path = bin_prefix + "gcc"),
        tool_path(name = "ar", path = bin_prefix + "ar"),
    ]

    return cc_common.create_cc_toolchain_config_info(
        ctx = ctx,
        features = features,
        action_configs = action_configs,
        artifact_name_patterns = artifact_name_patterns,
        cxx_builtin_include_directories = cxx_builtin_include_directories,
        toolchain_identifier = toolchain_identifier,
        host_system_name = host_system_name,
        target_system_name = target_system_name,
        target_cpu = target_cpu,
        target_libc = target_libc,
        compiler = compiler,
        abi_version = abi_version,
        abi_libc_version = abi_libc_version,
        tool_paths = tool_paths,
        make_variables = make_variables,
        builtin_sysroot = builtin_sysroot,
        cc_target_os = cc_target_os,
    )

windows_cc_toolchain_config = rule(
    implementation = _impl,
    attrs = {
        "target": attr.string(mandatory = True),
        "stdlib": attr.string(),
    },
    provides = [CcToolchainConfigInfo],
)

