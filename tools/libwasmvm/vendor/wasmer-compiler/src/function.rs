// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! A `Compilation` contains the compiled function bodies for a WebAssembly
//! module (`CompiledFunction`).

use crate::lib::std::vec::Vec;
use crate::section::{CustomSection, SectionIndex};
use crate::trap::TrapInformation;
use crate::{CompiledFunctionUnwindInfo, FunctionAddressMap, Relocation};
use loupe::MemoryUsage;
#[cfg(feature = "enable-rkyv")]
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};
use wasmer_types::entity::PrimaryMap;
use wasmer_types::{FunctionIndex, LocalFunctionIndex, SignatureIndex};

/// The frame info for a Compiled function.
///
/// This structure is only used for reconstructing
/// the frame information after a `Trap`.
#[cfg_attr(feature = "enable-serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[derive(Debug, Clone, PartialEq, Eq, Default, MemoryUsage)]
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
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[derive(Debug, Clone, PartialEq, Eq, MemoryUsage)]
pub struct FunctionBody {
    /// The function body bytes.
    #[cfg_attr(feature = "enable-serde", serde(with = "serde_bytes"))]
    pub body: Vec<u8>,

    /// The function unwind info
    pub unwind_info: Option<CompiledFunctionUnwindInfo>,
}

/// The result of compiling a WebAssembly function.
///
/// This structure only have the compiled information data
/// (function bytecode body, relocations, traps, jump tables
/// and unwind information).
#[cfg_attr(feature = "enable-serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[derive(Debug, PartialEq, Eq, Clone, MemoryUsage)]
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
    functions: Functions,

    /// Custom sections for the module.
    /// It will hold the data, for example, for constants used in a
    /// function, global variables, rodata_64, hot/cold function partitioning, ...
    custom_sections: CustomSections,

    /// Trampolines to call a function defined locally in the wasm via a
    /// provided `Vec` of values.
    ///
    /// This allows us to call easily Wasm functions, such as:
    ///
    /// ```ignore
    /// let func = instance.exports.get_function("my_func");
    /// func.call(&[Value::I32(1)]);
    /// ```
    function_call_trampolines: PrimaryMap<SignatureIndex, FunctionBody>,

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
    dynamic_function_trampolines: PrimaryMap<FunctionIndex, FunctionBody>,

    /// Section ids corresponding to the Dwarf debug info
    debug: Option<Dwarf>,
}

impl Compilation {
    /// Creates a compilation artifact from a contiguous function buffer and a set of ranges
    pub fn new(
        functions: Functions,
        custom_sections: CustomSections,
        function_call_trampolines: PrimaryMap<SignatureIndex, FunctionBody>,
        dynamic_function_trampolines: PrimaryMap<FunctionIndex, FunctionBody>,
        debug: Option<Dwarf>,
    ) -> Self {
        Self {
            functions,
            custom_sections,
            function_call_trampolines,
            dynamic_function_trampolines,
            debug,
        }
    }

    /// Gets the bytes of a single function
    pub fn get(&self, func: LocalFunctionIndex) -> &CompiledFunction {
        &self.functions[func]
    }

    /// Gets the number of functions defined.
    pub fn len(&self) -> usize {
        self.functions.len()
    }

    /// Returns whether there are no functions defined.
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }

    /// Gets functions relocations.
    pub fn get_relocations(&self) -> PrimaryMap<LocalFunctionIndex, Vec<Relocation>> {
        self.functions
            .iter()
            .map(|(_, func)| func.relocations.clone())
            .collect::<PrimaryMap<LocalFunctionIndex, _>>()
    }

    /// Gets functions bodies.
    pub fn get_function_bodies(&self) -> PrimaryMap<LocalFunctionIndex, FunctionBody> {
        self.functions
            .iter()
            .map(|(_, func)| func.body.clone())
            .collect::<PrimaryMap<LocalFunctionIndex, _>>()
    }

    /// Gets functions frame info.
    pub fn get_frame_info(&self) -> PrimaryMap<LocalFunctionIndex, CompiledFunctionFrameInfo> {
        self.functions
            .iter()
            .map(|(_, func)| func.frame_info.clone())
            .collect::<PrimaryMap<LocalFunctionIndex, _>>()
    }

    /// Gets function call trampolines.
    pub fn get_function_call_trampolines(&self) -> PrimaryMap<SignatureIndex, FunctionBody> {
        self.function_call_trampolines.clone()
    }

    /// Gets function call trampolines.
    pub fn get_dynamic_function_trampolines(&self) -> PrimaryMap<FunctionIndex, FunctionBody> {
        self.dynamic_function_trampolines.clone()
    }

    /// Gets custom section data.
    pub fn get_custom_sections(&self) -> PrimaryMap<SectionIndex, CustomSection> {
        self.custom_sections.clone()
    }

    /// Gets relocations that apply to custom sections.
    pub fn get_custom_section_relocations(&self) -> PrimaryMap<SectionIndex, Vec<Relocation>> {
        self.custom_sections
            .iter()
            .map(|(_, section)| section.relocations.clone())
            .collect::<PrimaryMap<SectionIndex, _>>()
    }

    /// Returns the Dwarf info.
    pub fn get_debug(&self) -> Option<Dwarf> {
        self.debug.clone()
    }
}

impl<'a> IntoIterator for &'a Compilation {
    type IntoIter = Iter<'a>;
    type Item = <Self::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iterator: self.functions.iter(),
        }
    }
}

pub struct Iter<'a> {
    iterator: <&'a Functions as IntoIterator>::IntoIter,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a CompiledFunction;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|(_, b)| b)
    }
}
