use crate::cdsl::isa::TargetIsa;
use crate::cdsl::settings::{PredicateNode, SettingGroup, SettingGroupBuilder};

use crate::shared::Definitions as SharedDefinitions;

pub(crate) fn define(shared_defs: &mut SharedDefinitions) -> TargetIsa {
    let settings = define_settings(&shared_defs.settings);

    TargetIsa::new("x86", settings)
}

fn define_settings(shared: &SettingGroup) -> SettingGroup {
    let mut settings = SettingGroupBuilder::new("x86");

    // CPUID.01H:ECX
    let has_sse3 = settings.add_bool(
        "has_sse3",
        "Has support for SSE3.",
        "SSE3: CPUID.01H:ECX.SSE3[bit 0]",
        // Needed for default `enable_simd` setting.
        true,
    );
    let has_ssse3 = settings.add_bool(
        "has_ssse3",
        "Has support for SSSE3.",
        "SSSE3: CPUID.01H:ECX.SSSE3[bit 9]",
        // Needed for default `enable_simd` setting.
        true,
    );
    let has_sse41 = settings.add_bool(
        "has_sse41",
        "Has support for SSE4.1.",
        "SSE4.1: CPUID.01H:ECX.SSE4_1[bit 19]",
        // Needed for default `enable_simd` setting.
        true,
    );
    let has_sse42 = settings.add_bool(
        "has_sse42",
        "Has support for SSE4.2.",
        "SSE4.2: CPUID.01H:ECX.SSE4_2[bit 20]",
        true,
    );
    let has_avx = settings.add_bool(
        "has_avx",
        "Has support for AVX.",
        "AVX: CPUID.01H:ECX.AVX[bit 28]",
        false,
    );
    let has_avx2 = settings.add_bool(
        "has_avx2",
        "Has support for AVX2.",
        "AVX2: CPUID.07H:EBX.AVX2[bit 5]",
        false,
    );
    let has_avx512bitalg = settings.add_bool(
        "has_avx512bitalg",
        "Has support for AVX512BITALG.",
        "AVX512BITALG: CPUID.07H:ECX.AVX512BITALG[bit 12]",
        false,
    );
    let has_avx512dq = settings.add_bool(
        "has_avx512dq",
        "Has support for AVX512DQ.",
        "AVX512DQ: CPUID.07H:EBX.AVX512DQ[bit 17]",
        false,
    );
    let has_avx512vl = settings.add_bool(
        "has_avx512vl",
        "Has support for AVX512VL.",
        "AVX512VL: CPUID.07H:EBX.AVX512VL[bit 31]",
        false,
    );
    let has_avx512vbmi = settings.add_bool(
        "has_avx512vbmi",
        "Has support for AVX512VMBI.",
        "AVX512VBMI: CPUID.07H:ECX.AVX512VBMI[bit 1]",
        false,
    );
    let has_avx512f = settings.add_bool(
        "has_avx512f",
        "Has support for AVX512F.",
        "AVX512F: CPUID.07H:EBX.AVX512F[bit 16]",
        false,
    );
    let has_popcnt = settings.add_bool(
        "has_popcnt",
        "Has support for POPCNT.",
        "POPCNT: CPUID.01H:ECX.POPCNT[bit 23]",
        false,
    );

    // CPUID.(EAX=07H, ECX=0H):EBX
    let has_bmi1 = settings.add_bool(
        "has_bmi1",
        "Has support for BMI1.",
        "BMI1: CPUID.(EAX=07H, ECX=0H):EBX.BMI1[bit 3]",
        false,
    );
    let has_bmi2 = settings.add_bool(
        "has_bmi2",
        "Has support for BMI2.",
        "BMI2: CPUID.(EAX=07H, ECX=0H):EBX.BMI2[bit 8]",
        false,
    );

    // CPUID.EAX=80000001H:ECX
    let has_lzcnt = settings.add_bool(
        "has_lzcnt",
        "Has support for LZCNT.",
        "LZCNT: CPUID.EAX=80000001H:ECX.LZCNT[bit 5]",
        false,
    );

    let shared_enable_simd = shared.get_bool("enable_simd");

    settings.add_predicate("use_ssse3", predicate!(has_ssse3));
    settings.add_predicate("use_sse41", predicate!(has_sse41));
    settings.add_predicate("use_sse42", predicate!(has_sse41 && has_sse42));

    settings.add_predicate(
        "use_ssse3_simd",
        predicate!(shared_enable_simd && has_ssse3),
    );
    settings.add_predicate(
        "use_sse41_simd",
        predicate!(shared_enable_simd && has_sse41),
    );
    settings.add_predicate(
        "use_sse42_simd",
        predicate!(shared_enable_simd && has_sse41 && has_sse42),
    );

    settings.add_predicate("use_avx_simd", predicate!(shared_enable_simd && has_avx));
    settings.add_predicate("use_avx2_simd", predicate!(shared_enable_simd && has_avx2));
    settings.add_predicate(
        "use_avx512bitalg_simd",
        predicate!(shared_enable_simd && has_avx512bitalg),
    );
    settings.add_predicate(
        "use_avx512dq_simd",
        predicate!(shared_enable_simd && has_avx512dq),
    );
    settings.add_predicate(
        "use_avx512vl_simd",
        predicate!(shared_enable_simd && has_avx512vl),
    );
    settings.add_predicate(
        "use_avx512vbmi_simd",
        predicate!(shared_enable_simd && has_avx512vbmi),
    );
    settings.add_predicate(
        "use_avx512f_simd",
        predicate!(shared_enable_simd && has_avx512f),
    );

    settings.add_predicate("use_popcnt", predicate!(has_popcnt && has_sse42));
    settings.add_predicate("use_bmi1", predicate!(has_bmi1));
    settings.add_predicate("use_lzcnt", predicate!(has_lzcnt));

    // Some shared boolean values are used in x86 instruction predicates, so we need to group them
    // in the same TargetIsa, for compatibility with code generated by meta-python.
    // TODO Once all the meta generation code has been migrated from Python to Rust, we can put it
    // back in the shared SettingGroup, and use it in x86 instruction predicates.

    let is_pic = shared.get_bool("is_pic");
    let emit_all_ones_funcaddrs = shared.get_bool("emit_all_ones_funcaddrs");
    settings.add_predicate("is_pic", predicate!(is_pic));
    settings.add_predicate("not_is_pic", predicate!(!is_pic));
    settings.add_predicate(
        "all_ones_funcaddrs_and_not_is_pic",
        predicate!(emit_all_ones_funcaddrs && !is_pic),
    );
    settings.add_predicate(
        "not_all_ones_funcaddrs_and_not_is_pic",
        predicate!(!emit_all_ones_funcaddrs && !is_pic),
    );

    // Presets corresponding to x86 CPUs.

    settings.add_preset(
        "baseline",
        "A baseline preset with no extensions enabled.",
        preset!(),
    );
    let nehalem = settings.add_preset(
        "nehalem",
        "Nehalem microarchitecture.",
        preset!(has_sse3 && has_ssse3 && has_sse41 && has_sse42 && has_popcnt),
    );
    let haswell = settings.add_preset(
        "haswell",
        "Haswell microarchitecture.",
        preset!(nehalem && has_bmi1 && has_bmi2 && has_lzcnt),
    );
    let broadwell = settings.add_preset(
        "broadwell",
        "Broadwell microarchitecture.",
        preset!(haswell),
    );
    let skylake = settings.add_preset("skylake", "Skylake microarchitecture.", preset!(broadwell));
    let cannonlake = settings.add_preset(
        "cannonlake",
        "Canon Lake microarchitecture.",
        preset!(skylake),
    );
    settings.add_preset(
        "icelake",
        "Ice Lake microarchitecture.",
        preset!(cannonlake),
    );
    settings.add_preset(
        "znver1",
        "Zen (first generation) microarchitecture.",
        preset!(
            has_sse3
                && has_ssse3
                && has_sse41
                && has_sse42
                && has_popcnt
                && has_bmi1
                && has_bmi2
                && has_lzcnt
        ),
    );

    settings.build()
}
