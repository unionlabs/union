//! Measure instruction encoding latency using various approaches; the
//! benchmarking is feature-gated on `x86` since it only measures the encoding
//! mechanism of that backend.

#[cfg(feature = "x86")]
mod x86 {
    use cranelift_codegen::isa::x64::encoding::{
        evex::{EvexInstruction, EvexVectorLength, Register},
        rex::{LegacyPrefixes, OpcodeMap},
    };
    use criterion::{criterion_group, Criterion};

    // Define the benchmarks.
    fn x64_evex_encoding_benchmarks(c: &mut Criterion) {
        let mut group = c.benchmark_group("x64 EVEX encoding");
        let rax = Register::from(0);
        let rdx = Register::from(2);

        group.bench_function("EvexInstruction (builder pattern)", |b| {
            let mut sink = vec![];
            b.iter(|| {
                sink.clear();
                EvexInstruction::new()
                    .prefix(LegacyPrefixes::_66)
                    .map(OpcodeMap::_0F38)
                    .w(true)
                    .opcode(0x1F)
                    .reg(rax)
                    .rm(rdx)
                    .length(EvexVectorLength::V128)
                    .encode(&mut sink);
            });
        });
    }
    criterion_group!(benches, x64_evex_encoding_benchmarks);

    /// Using an inner module to feature-gate the benchmarks means that we must
    /// manually specify how to run the benchmarks (see `criterion_main!`).
    pub fn run_benchmarks() {
        criterion::__warn_about_html_reports_feature();
        criterion::__warn_about_cargo_bench_support_feature();
        benches();
        Criterion::default().configure_from_args().final_summary();
    }
}

fn main() {
    #[cfg(feature = "x86")]
    x86::run_benchmarks();

    #[cfg(not(feature = "x86"))]
    println!(
        "Unable to run the x64-evex-encoding benchmark; the `x86` feature must be enabled in Cargo.",
    );
}
