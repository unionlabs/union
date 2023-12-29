// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! A `Compilation` contains the compiled function bodies for a WebAssembly
//! module (`CompiledFunction`).

use crate::entity::PrimaryMap;
use crate::lib::std::vec::Vec;
use crate::{ArchivedCompiledFunctionUnwindInfo, TrapInformation};
use crate::{CompiledFunctionUnwindInfo, FunctionAddressMap};
use crate::{
    CustomSection, FunctionIndex, LocalFunctionIndex, Relocation, SectionIndex, SignatureIndex,
};
use rkyv::option::ArchivedOption;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

use super::unwind::CompiledFunctionUnwindInfoLike;

/// The frame info for a Compiled function.
///
/// This structure is only used for reconstructing
/// the frame information after a `Trap`.
#[cfg_attr(feature = "enable-serde", derive(Deserialize, Serialize))]
#[derive(RkyvSerialize, RkyvDeserialize, Archive, Debug, Clone, PartialEq, Eq, Default)]
#[archive_attr(derive(rkyv::CheckBytes))]
pub struct CompiledFunctionFrameInfo {
    /// The traps (in the function body).
    ///
    /// Code offsets of the traps MUST be in ascending order.
    pub traps: Vec<TrapInformation>,

    /// The address map.
    pub address_map: FunctionAddressMap,
}

/// The function body.
#[cfg_attr(feature = "enable-serde", derive(Deserialize, Serialize))]
#[derive(RkyvSerialize, RkyvDeserialize, Archive, Debug, Clone, PartialEq, Eq)]
#[archive_attr(derive(rkyv::CheckBytes))]
pub struct FunctionBody {
    /// The function body bytes.
    #[cfg_attr(feature = "enable-serde", serde(with = "serde_bytes"))]
    pub body: Vec<u8>,

    /// The function unwind info
    pub unwind_info: Option<CompiledFunctionUnwindInfo>,
}

/// Any struct that acts like a `FunctionBody`.
#[allow(missing_docs)]
pub trait FunctionBodyLike<'a> {
    type UnwindInfo: CompiledFunctionUnwindInfoLike<'a>;

    fn body(&'a self) -> &'a [u8];
    fn unwind_info(&'a self) -> Option<&Self::UnwindInfo>;
}

impl<'a> FunctionBodyLike<'a> for FunctionBody {
    type UnwindInfo = CompiledFunctionUnwindInfo;

    fn body(&'a self) -> &'a [u8] {
        self.body.as_ref()
    }

    fn unwind_info(&'a self) -> Option<&Self::UnwindInfo> {
        self.unwind_info.as_ref()
    }
}

impl<'a> FunctionBodyLike<'a> for ArchivedFunctionBody {
    type UnwindInfo = ArchivedCompiledFunctionUnwindInfo;

    fn body(&'a self) -> &'a [u8] {
        self.body.as_ref()
    }

    fn unwind_info(&'a self) -> Option<&Self::UnwindInfo> {
        match self.unwind_info {
            ArchivedOption::Some(ref x) => Some(x),
            ArchivedOption::None => None,
        }
    }
}

/// The result of compiling a WebAssembly function.
///
/// This structure only have the compiled information data
/// (function bytecode body, relocations, traps, jump tables
/// and unwind information).
#[cfg_attr(feature = "enable-serde", derive(Deserialize, Serialize))]
#[derive(RkyvSerialize, RkyvDeserialize, Archive, Debug, Clone, PartialEq, Eq)]
#[archive_attr(derive(rkyv::CheckBytes))]
pub struct CompiledFunction {
    /// The function body.
    pub body: FunctionBody,

    /// The relocations (in the body)
    pub relocations: Vec<Relocation>,

    /// The frame information.
    pub frame_info: CompiledFunctionFrameInfo,
}

/// The compiled functions map (index in the Wasm -> function)
pub type Functions = PrimaryMap<LocalFunctionIndex, CompiledFunction>;

/// The custom sections for a Compilation.
pub type CustomSections = PrimaryMap<SectionIndex, CustomSection>;

/// The DWARF information for this Compilation.
///
/// It is used for retrieving the unwind information once an exception
/// happens.
/// In the future this structure may also hold other information useful
/// for debugging.
#[cfg_attr(feature = "enable-serde", derive(Deserialize, Serialize))]
#[derive(
    RkyvSerialize, RkyvDeserialize, Archive, rkyv::CheckBytes, Debug, PartialEq, Eq, Clone,
)]
#[archive(as = "Self")]
pub struct Dwarf {
    /// The section index in the [`Compilation`] that corresponds to the exception frames.
    /// [Learn
    /// more](https://refspecs.linuxfoundation.org/LSB_3.0.0/LSB-PDA/LSB-PDA/ehframechpt.html).
    pub eh_frame: SectionIndex,
}

impl Dwarf {
    /// Creates a `Dwarf` struct with the corresponding indices for its sections
    pub fn new(eh_frame: SectionIndex) -> Self {
        Self { eh_frame }
    }
}

/// The result of compiling a WebAssembly module's functions.
#[cfg_attr(feature = "enable-serde", derive(Deserialize, Serialize))]
#[derive(Debug, PartialEq, Eq)]
pub struct Compilation {
    /// Compiled code for the function bodies.
    pub functions: Functions,

    /// Custom sections for the module.
    /// It will hold the data, for example, for constants used in a
    /// function, global variables, rodata_64, hot/cold function partitioning, ...
    pub custom_sections: CustomSections,

    /// Trampolines to call a function defined locally in the wasm via a
    /// provided `Vec` of values.
    ///
    /// This allows us to call easily Wasm functions, such as:
    ///
    /// ```ignore
    /// let func = instance.exports.get_function("my_func");
    /// func.call(&[Value::I32(1)]);
    /// ```
    pub function_call_trampolines: PrimaryMap<SignatureIndex, FunctionBody>,

    /// Trampolines to call a dynamic function defined in
    /// a host, from a Wasm module.
    ///
    /// This allows us to create dynamic Wasm functions, such as:
    ///
    /// ```ignore
    /// fn my_func(values: &[Val]) -> Result<Vec<Val>, RuntimeError> {
    ///     // do something
    /// }
    ///
    /// let my_func_type = FunctionType::new(vec![Type::I32], vec![Type::I32]);
    /// let imports = imports!{
    ///     "namespace" => {
    ///         "my_func" => Function::new(&store, my_func_type, my_func),
    ///     }
    /// }
    /// ```
    ///
    /// Note: Dynamic function trampolines are only compiled for imported function types.
    pub dynamic_function_trampolines: PrimaryMap<FunctionIndex, FunctionBody>,

    /// Section ids corresponding to the Dwarf debug info
    pub debug: Option<Dwarf>,
}
