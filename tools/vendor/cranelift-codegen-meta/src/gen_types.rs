//! Generate sources with type info.
//!
//! This generates a `types.rs` file which is included in
//! `cranelift-codegen/ir/types.rs`. The file provides constant definitions for the
//! most commonly used types, including all of the scalar types.
//!
//! This ensures that the metaprogram and the generated program see the same
//! type numbering.

use crate::cdsl::types as cdsl_types;
use crate::error;
use crate::srcgen;

/// Emit a constant definition of a single value type.
fn emit_type(ty: &cdsl_types::ValueType, fmt: &mut srcgen::Formatter) {
    let name = ty.to_string().to_uppercase();
    let number = ty.number();

    fmt.doc_comment(&ty.doc());
    fmtln!(fmt, "pub const {}: Type = Type({:#x});\n", name, number);
}

/// Emit definition for all vector types with `bits` total size.
fn emit_vectors(bits: u64, fmt: &mut srcgen::Formatter) {
    let vec_size: u64 = bits / 8;
    for vec in cdsl_types::ValueType::all_lane_types()
        .map(|ty| (ty, cdsl_types::ValueType::from(ty).membytes()))
        .filter(|&(_, lane_size)| lane_size != 0 && lane_size < vec_size)
        .map(|(ty, lane_size)| (ty, vec_size / lane_size))
        .map(|(ty, lanes)| cdsl_types::VectorType::new(ty, lanes))
    {
        emit_type(&cdsl_types::ValueType::from(vec), fmt);
    }
}

/// Emit definition for all dynamic vector types with `bits` total size.
fn emit_dynamic_vectors(bits: u64, fmt: &mut srcgen::Formatter) {
    let vec_size: u64 = bits / 8;
    for vec in cdsl_types::ValueType::all_lane_types()
        .map(|ty| (ty, cdsl_types::ValueType::from(ty).membytes()))
        .filter(|&(_, lane_size)| lane_size != 0 && lane_size < vec_size)
        .map(|(ty, lane_size)| (ty, vec_size / lane_size))
        .map(|(ty, lanes)| cdsl_types::DynamicVectorType::new(ty, lanes))
    {
        emit_type(&cdsl_types::ValueType::from(vec), fmt);
    }
}

/// Emit types using the given formatter object.
fn emit_types(fmt: &mut srcgen::Formatter) {
    // Emit all of the special types, such as types for CPU flags.
    for spec in cdsl_types::ValueType::all_special_types().map(cdsl_types::ValueType::from) {
        emit_type(&spec, fmt);
    }

    // Emit all of the lane types, such integers, floats, and booleans.
    for ty in cdsl_types::ValueType::all_lane_types().map(cdsl_types::ValueType::from) {
        emit_type(&ty, fmt);
    }

    // Emit all reference types.
    for ty in cdsl_types::ValueType::all_reference_types().map(cdsl_types::ValueType::from) {
        emit_type(&ty, fmt);
    }

    // Emit vector definitions for common SIMD sizes.
    // Emit dynamic vector definitions.
    for vec_size in &[64_u64, 128, 256, 512] {
        emit_vectors(*vec_size, fmt);
        emit_dynamic_vectors(*vec_size, fmt);
    }
}

/// Generate the types file.
pub(crate) fn generate(filename: &str, out_dir: &str) -> Result<(), error::Error> {
    let mut fmt = srcgen::Formatter::new();
    emit_types(&mut fmt);
    fmt.update_file(filename, out_dir)?;
    Ok(())
}
