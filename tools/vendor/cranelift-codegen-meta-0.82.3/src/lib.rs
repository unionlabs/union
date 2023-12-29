//! This crate generates Rust sources for use by
//! [`cranelift_codegen`](../cranelift_codegen/index.html).

use std::path::Path;
#[macro_use]
mod cdsl;
mod srcgen;

pub mod error;
pub mod isa;

mod gen_inst;
mod gen_settings;
mod gen_types;

mod constant_hash;
mod shared;
mod unique_table;

/// Generate an ISA from an architecture string (e.g. "x86_64").
pub fn isa_from_arch(arch: &str) -> Result<isa::Isa, String> {
    isa::Isa::from_arch(arch).ok_or_else(|| format!("no supported isa found for arch `{}`", arch))
}

/// Generates all the Rust source files used in Cranelift from the meta-language.
pub fn generate(isas: &[isa::Isa], out_dir: &str, crate_dir: &Path) -> Result<(), error::Error> {
    // Create all the definitions:
    // - common definitions.
    let mut shared_defs = shared::define();

    gen_settings::generate(
        &shared_defs.settings,
        gen_settings::ParentGroup::None,
        "settings.rs",
        &out_dir,
    )?;
    gen_types::generate("types.rs", &out_dir)?;

    // - per ISA definitions.
    let target_isas = isa::define(isas, &mut shared_defs);

    // At this point, all definitions are done.
    let all_formats = shared_defs.verify_instruction_formats();

    // Generate all the code.
    gen_inst::generate(
        all_formats,
        &shared_defs.all_instructions,
        "opcodes.rs",
        "inst_builder.rs",
        "clif.isle",
        &out_dir,
        crate_dir,
    )?;

    for isa in target_isas {
        gen_settings::generate(
            &isa.settings,
            gen_settings::ParentGroup::Shared,
            &format!("settings-{}.rs", isa.name),
            &out_dir,
        )?;
    }

    Ok(())
}
