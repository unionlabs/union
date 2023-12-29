//! Types relating to type information provided by validation.

use super::{component::ComponentState, core::Module};
use crate::{
    ComponentExport, ComponentExternalKind, ComponentImport, ComponentTypeRef, Export,
    ExternalKind, FuncType, GlobalType, Import, MemoryType, PrimitiveValType, TableType, TypeRef,
    ValType,
};
use indexmap::{IndexMap, IndexSet};
use std::{
    borrow::Borrow,
    fmt,
    hash::{Hash, Hasher},
    mem,
    ops::Deref,
    sync::Arc,
};
use url::Url;

/// The maximum number of parameters in the canonical ABI that can be passed by value.
///
/// Functions that exceed this limit will instead pass parameters indirectly from
/// linear memory via a single pointer parameter.
const MAX_FLAT_FUNC_PARAMS: usize = 16;
/// The maximum number of results in the canonical ABI that can be returned by a function.
///
/// Functions that exceed this limit have their results written to linear memory via an
/// additional pointer parameter (imports) or return a single pointer value (exports).
const MAX_FLAT_FUNC_RESULTS: usize = 1;

/// The maximum lowered types, including a possible type for a return pointer parameter.
const MAX_LOWERED_TYPES: usize = MAX_FLAT_FUNC_PARAMS + 1;

/// Represents a kebab string slice used in validation.
///
/// This is a wrapper around `str` that ensures the slice is
/// a valid kebab case string according to the component model
/// specification.
///
/// It also provides an equality and hashing implementation
/// that ignores ASCII case.
#[derive(Debug, Eq)]
#[repr(transparent)]
pub struct KebabStr(str);

impl KebabStr {
    /// Creates a new kebab string slice.
    ///
    /// Returns `None` if the given string is not a valid kebab string.
    pub fn new<'a>(s: impl AsRef<str> + 'a) -> Option<&'a Self> {
        let s = Self::new_unchecked(s);
        if s.is_kebab_case() {
            Some(s)
        } else {
            None
        }
    }

    pub(crate) fn new_unchecked<'a>(s: impl AsRef<str> + 'a) -> &'a Self {
        // Safety: `KebabStr` is a transparent wrapper around `str`
        // Therefore transmuting `&str` to `&KebabStr` is safe.
        unsafe { std::mem::transmute::<_, &Self>(s.as_ref()) }
    }

    /// Gets the underlying string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts the slice to an owned string.
    pub fn to_kebab_string(&self) -> KebabString {
        KebabString(self.to_string())
    }

    fn is_kebab_case(&self) -> bool {
        let mut lower = false;
        let mut upper = false;
        for c in self.chars() {
            match c {
                'a'..='z' if !lower && !upper => lower = true,
                'A'..='Z' if !lower && !upper => upper = true,
                'a'..='z' if lower => {}
                'A'..='Z' if upper => {}
                '0'..='9' if lower || upper => {}
                '-' if lower || upper => {
                    lower = false;
                    upper = false;
                }
                _ => return false,
            }
        }

        !self.is_empty() && !self.ends_with('-')
    }
}

impl Deref for KebabStr {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for KebabStr {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.chars()
            .zip(other.chars())
            .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
    }
}

impl PartialEq<KebabString> for KebabStr {
    fn eq(&self, other: &KebabString) -> bool {
        self.eq(other.as_kebab_str())
    }
}

impl Hash for KebabStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);

        for b in self.chars() {
            b.to_ascii_lowercase().hash(state);
        }
    }
}

impl fmt::Display for KebabStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self as &str).fmt(f)
    }
}

impl ToOwned for KebabStr {
    type Owned = KebabString;

    fn to_owned(&self) -> Self::Owned {
        self.to_kebab_string()
    }
}

/// Represents an owned kebab string for validation.
///
/// This is a wrapper around `String` that ensures the string is
/// a valid kebab case string according to the component model
/// specification.
///
/// It also provides an equality and hashing implementation
/// that ignores ASCII case.
#[derive(Debug, Clone, Eq)]
pub struct KebabString(String);

impl KebabString {
    /// Creates a new kebab string.
    ///
    /// Returns `None` if the given string is not a valid kebab string.
    pub fn new(s: impl Into<String>) -> Option<Self> {
        let s = s.into();
        if KebabStr::new(&s).is_some() {
            Some(Self(s))
        } else {
            None
        }
    }

    /// Gets the underlying string.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Converts the kebab string to a kebab string slice.
    pub fn as_kebab_str(&self) -> &KebabStr {
        // Safety: internal string is always valid kebab-case
        KebabStr::new_unchecked(self.as_str())
    }
}

impl Deref for KebabString {
    type Target = KebabStr;

    fn deref(&self) -> &Self::Target {
        self.as_kebab_str()
    }
}

impl Borrow<KebabStr> for KebabString {
    fn borrow(&self) -> &KebabStr {
        self.as_kebab_str()
    }
}

impl PartialEq for KebabString {
    fn eq(&self, other: &Self) -> bool {
        self.as_kebab_str().eq(other.as_kebab_str())
    }
}

impl PartialEq<KebabStr> for KebabString {
    fn eq(&self, other: &KebabStr) -> bool {
        self.as_kebab_str().eq(other)
    }
}

impl Hash for KebabString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_kebab_str().hash(state)
    }
}

impl fmt::Display for KebabString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_kebab_str().fmt(f)
    }
}

/// A simple alloc-free list of types used for calculating lowered function signatures.
pub(crate) struct LoweredTypes {
    types: [ValType; MAX_LOWERED_TYPES],
    len: usize,
    max: usize,
}

impl LoweredTypes {
    fn new(max: usize) -> Self {
        assert!(max <= MAX_LOWERED_TYPES);
        Self {
            types: [ValType::I32; MAX_LOWERED_TYPES],
            len: 0,
            max,
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn maxed(&self) -> bool {
        self.len == self.max
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut ValType> {
        if index < self.len {
            Some(&mut self.types[index])
        } else {
            None
        }
    }

    fn push(&mut self, ty: ValType) -> bool {
        if self.maxed() {
            return false;
        }

        self.types[self.len] = ty;
        self.len += 1;
        true
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    pub fn as_slice(&self) -> &[ValType] {
        &self.types[..self.len]
    }

    pub fn iter(&self) -> impl Iterator<Item = ValType> + '_ {
        self.as_slice().iter().copied()
    }
}

/// Represents information about a component function type lowering.
pub(crate) struct LoweringInfo {
    pub(crate) params: LoweredTypes,
    pub(crate) results: LoweredTypes,
    pub(crate) requires_memory: bool,
    pub(crate) requires_realloc: bool,
}

impl LoweringInfo {
    pub(crate) fn into_func_type(self) -> FuncType {
        FuncType::new(
            self.params.as_slice().iter().copied(),
            self.results.as_slice().iter().copied(),
        )
    }
}

impl Default for LoweringInfo {
    fn default() -> Self {
        Self {
            params: LoweredTypes::new(MAX_FLAT_FUNC_PARAMS),
            results: LoweredTypes::new(MAX_FLAT_FUNC_RESULTS),
            requires_memory: false,
            requires_realloc: false,
        }
    }
}

fn push_primitive_wasm_types(ty: &PrimitiveValType, lowered_types: &mut LoweredTypes) -> bool {
    match ty {
        PrimitiveValType::Bool
        | PrimitiveValType::S8
        | PrimitiveValType::U8
        | PrimitiveValType::S16
        | PrimitiveValType::U16
        | PrimitiveValType::S32
        | PrimitiveValType::U32
        | PrimitiveValType::Char => lowered_types.push(ValType::I32),
        PrimitiveValType::S64 | PrimitiveValType::U64 => lowered_types.push(ValType::I64),
        PrimitiveValType::Float32 => lowered_types.push(ValType::F32),
        PrimitiveValType::Float64 => lowered_types.push(ValType::F64),
        PrimitiveValType::String => {
            lowered_types.push(ValType::I32) && lowered_types.push(ValType::I32)
        }
    }
}

/// Represents a unique identifier for a type known to a [`crate::Validator`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TypeId {
    /// The effective type size for the type.
    ///
    /// This is stored as part of the ID to avoid having to recurse through
    /// the global type list when calculating type sizes.
    pub(crate) type_size: usize,
    /// The index into the global list of types.
    pub(crate) index: usize,
    /// The original type index.
    ///
    /// This will be `None` for implicitly defined types, e.g. types for
    /// modules definitions, component definitions, instantiations, and function
    /// lowerings.
    pub(crate) type_index: Option<usize>,
    /// Whether or not the type is a core type.
    pub(crate) is_core: bool,
}

/// A unified type definition for validating WebAssembly modules and components.
#[derive(Debug)]
pub enum Type {
    /// The definition is for a core function type.
    Func(FuncType),
    /// The definition is for a core module type.
    ///
    /// This variant is only supported when parsing a component.
    Module(ModuleType),
    /// The definition is for a core module instance type.
    ///
    /// This variant is only supported when parsing a component.
    Instance(InstanceType),
    /// The definition is for a component type.
    ///
    /// This variant is only supported when parsing a component.
    Component(ComponentType),
    /// The definition is for a component instance type.
    ///
    /// This variant is only supported when parsing a component.
    ComponentInstance(ComponentInstanceType),
    /// The definition is for a component function type.
    ///
    /// This variant is only supported when parsing a component.
    ComponentFunc(ComponentFuncType),
    /// The definition is for a component defined type.
    ///
    /// This variant is only supported when parsing a component.
    Defined(ComponentDefinedType),
}

impl Type {
    /// Converts the type to a core function type.
    pub fn as_func_type(&self) -> Option<&FuncType> {
        match self {
            Self::Func(ty) => Some(ty),
            _ => None,
        }
    }

    /// Converts the type to a core module type.
    pub fn as_module_type(&self) -> Option<&ModuleType> {
        match self {
            Self::Module(ty) => Some(ty),
            _ => None,
        }
    }

    /// Converts the type to a core module instance type.
    pub fn as_instance_type(&self) -> Option<&InstanceType> {
        match self {
            Self::Instance(ty) => Some(ty),
            _ => None,
        }
    }

    /// Converts the type to a component type.
    pub fn as_component_type(&self) -> Option<&ComponentType> {
        match self {
            Self::Component(ty) => Some(ty),
            _ => None,
        }
    }

    /// Converts the type to a component instance type.
    pub fn as_component_instance_type(&self) -> Option<&ComponentInstanceType> {
        match self {
            Self::ComponentInstance(ty) => Some(ty),
            _ => None,
        }
    }

    /// Converts the type to a component function type.
    pub fn as_component_func_type(&self) -> Option<&ComponentFuncType> {
        match self {
            Self::ComponentFunc(ty) => Some(ty),
            _ => None,
        }
    }

    /// Converts the type to a component defined type.
    pub fn as_defined_type(&self) -> Option<&ComponentDefinedType> {
        match self {
            Self::Defined(ty) => Some(ty),
            _ => None,
        }
    }

    pub(crate) fn type_size(&self) -> usize {
        match self {
            Self::Func(ty) => 1 + ty.params().len() + ty.results().len(),
            Self::Module(ty) => ty.type_size,
            Self::Instance(ty) => ty.type_size,
            Self::Component(ty) => ty.type_size,
            Self::ComponentInstance(ty) => ty.type_size,
            Self::ComponentFunc(ty) => ty.type_size,
            Self::Defined(ty) => ty.type_size(),
        }
    }
}

/// A component value type.
#[derive(Debug, Clone, Copy)]
pub enum ComponentValType {
    /// The value type is one of the primitive types.
    Primitive(PrimitiveValType),
    /// The type is represented with the given type identifier.
    Type(TypeId),
}

impl ComponentValType {
    pub(crate) fn requires_realloc(&self, types: &TypeList) -> bool {
        match self {
            ComponentValType::Primitive(ty) => ty.requires_realloc(),
            ComponentValType::Type(ty) => types[*ty]
                .as_defined_type()
                .unwrap()
                .requires_realloc(types),
        }
    }

    pub(crate) fn is_optional(&self, types: &TypeList) -> bool {
        match self {
            ComponentValType::Primitive(_) => false,
            ComponentValType::Type(ty) => {
                matches!(
                    types[*ty].as_defined_type().unwrap(),
                    ComponentDefinedType::Option(_)
                )
            }
        }
    }

    /// Determines if component value type `a` is a subtype of `b`.
    pub fn is_subtype_of(a: &Self, at: TypesRef, b: &Self, bt: TypesRef) -> bool {
        Self::internal_is_subtype_of(a, at.list, b, bt.list)
    }

    pub(crate) fn internal_is_subtype_of(a: &Self, at: &TypeList, b: &Self, bt: &TypeList) -> bool {
        match (a, b) {
            (ComponentValType::Primitive(a), ComponentValType::Primitive(b)) => {
                PrimitiveValType::is_subtype_of(*a, *b)
            }
            (ComponentValType::Type(a), ComponentValType::Type(b)) => {
                ComponentDefinedType::internal_is_subtype_of(
                    at[*a].as_defined_type().unwrap(),
                    at,
                    bt[*b].as_defined_type().unwrap(),
                    bt,
                )
            }
            (ComponentValType::Primitive(a), ComponentValType::Type(b)) => {
                match bt[*b].as_defined_type().unwrap() {
                    ComponentDefinedType::Primitive(b) => PrimitiveValType::is_subtype_of(*a, *b),
                    _ => false,
                }
            }
            (ComponentValType::Type(a), ComponentValType::Primitive(b)) => {
                match at[*a].as_defined_type().unwrap() {
                    ComponentDefinedType::Primitive(a) => PrimitiveValType::is_subtype_of(*a, *b),
                    _ => false,
                }
            }
        }
    }

    fn push_wasm_types(&self, types: &TypeList, lowered_types: &mut LoweredTypes) -> bool {
        match self {
            Self::Primitive(ty) => push_primitive_wasm_types(ty, lowered_types),
            Self::Type(id) => types[*id]
                .as_defined_type()
                .unwrap()
                .push_wasm_types(types, lowered_types),
        }
    }

    pub(crate) fn type_size(&self) -> usize {
        match self {
            Self::Primitive(_) => 1,
            Self::Type(id) => id.type_size,
        }
    }
}

/// The entity type for imports and exports of a module.
#[derive(Debug, Clone, Copy)]
pub enum EntityType {
    /// The entity is a function.
    Func(TypeId),
    /// The entity is a table.
    Table(TableType),
    /// The entity is a memory.
    Memory(MemoryType),
    /// The entity is a global.
    Global(GlobalType),
    /// The entity is a tag.
    Tag(TypeId),
}

impl EntityType {
    /// Determines if entity type `a` is a subtype of `b`.
    pub fn is_subtype_of(a: &Self, at: TypesRef, b: &Self, bt: TypesRef) -> bool {
        Self::internal_is_subtype_of(a, at.list, b, bt.list)
    }

    pub(crate) fn internal_is_subtype_of(a: &Self, at: &TypeList, b: &Self, bt: &TypeList) -> bool {
        macro_rules! limits_match {
            ($a:expr, $b:expr) => {{
                let a = $a;
                let b = $b;
                a.initial >= b.initial
                    && match b.maximum {
                        Some(b_max) => match a.maximum {
                            Some(a_max) => a_max <= b_max,
                            None => false,
                        },
                        None => true,
                    }
            }};
        }

        match (a, b) {
            (EntityType::Func(a), EntityType::Func(b)) => {
                at[*a].as_func_type().unwrap() == bt[*b].as_func_type().unwrap()
            }
            (EntityType::Table(a), EntityType::Table(b)) => {
                a.element_type == b.element_type && limits_match!(a, b)
            }
            (EntityType::Memory(a), EntityType::Memory(b)) => {
                a.shared == b.shared && a.memory64 == b.memory64 && limits_match!(a, b)
            }
            (EntityType::Global(a), EntityType::Global(b)) => a == b,
            (EntityType::Tag(a), EntityType::Tag(b)) => {
                at[*a].as_func_type().unwrap() == bt[*b].as_func_type().unwrap()
            }
            _ => false,
        }
    }

    pub(crate) fn desc(&self) -> &'static str {
        match self {
            Self::Func(_) => "function",
            Self::Table(_) => "table",
            Self::Memory(_) => "memory",
            Self::Global(_) => "global",
            Self::Tag(_) => "tag",
        }
    }

    pub(crate) fn type_size(&self) -> usize {
        match self {
            Self::Func(id) | Self::Tag(id) => id.type_size,
            Self::Table(_) | Self::Memory(_) | Self::Global(_) => 1,
        }
    }
}

trait ModuleImportKey {
    fn module(&self) -> &str;
    fn name(&self) -> &str;
}

impl<'a> Borrow<dyn ModuleImportKey + 'a> for (String, String) {
    fn borrow(&self) -> &(dyn ModuleImportKey + 'a) {
        self
    }
}

impl Hash for (dyn ModuleImportKey + '_) {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.module().hash(state);
        self.name().hash(state);
    }
}

impl PartialEq for (dyn ModuleImportKey + '_) {
    fn eq(&self, other: &Self) -> bool {
        self.module() == other.module() && self.name() == other.name()
    }
}

impl Eq for (dyn ModuleImportKey + '_) {}

impl ModuleImportKey for (String, String) {
    fn module(&self) -> &str {
        &self.0
    }

    fn name(&self) -> &str {
        &self.1
    }
}

impl ModuleImportKey for (&str, &str) {
    fn module(&self) -> &str {
        self.0
    }

    fn name(&self) -> &str {
        self.1
    }
}

/// Represents a core module type.
#[derive(Debug, Clone)]
pub struct ModuleType {
    /// The effective type size for the module type.
    pub(crate) type_size: usize,
    /// The imports of the module type.
    pub imports: IndexMap<(String, String), EntityType>,
    /// The exports of the module type.
    pub exports: IndexMap<String, EntityType>,
}

impl ModuleType {
    /// Looks up an import by its module and name.
    ///
    /// Returns `None` if the import was not found.
    pub fn lookup_import(&self, module: &str, name: &str) -> Option<&EntityType> {
        self.imports.get(&(module, name) as &dyn ModuleImportKey)
    }

    /// Determines if module type `a` is a subtype of `b`.
    pub fn is_subtype_of(a: &Self, at: TypesRef, b: &Self, bt: TypesRef) -> bool {
        Self::internal_is_subtype_of(a, at.list, b, bt.list)
    }

    pub(crate) fn internal_is_subtype_of(a: &Self, at: &TypeList, b: &Self, bt: &TypeList) -> bool {
        // For module type subtyping, all exports in the other module type
        // must be present in this module type's exports (i.e. it can export
        // *more* than what this module type needs).
        // However, for imports, the check is reversed (i.e. it is okay
        // to import *less* than what this module type needs).
        a.imports.iter().all(|(k, a)| match b.imports.get(k) {
            Some(b) => EntityType::internal_is_subtype_of(b, bt, a, at),
            None => false,
        }) && b.exports.iter().all(|(k, b)| match a.exports.get(k) {
            Some(a) => EntityType::internal_is_subtype_of(a, at, b, bt),
            None => false,
        })
    }
}

/// Represents the kind of module instance type.
#[derive(Debug, Clone)]
pub enum InstanceTypeKind {
    /// The instance type is the result of instantiating a module type.
    Instantiated(TypeId),
    /// The instance type is the result of instantiating from exported items.
    Exports(IndexMap<String, EntityType>),
}

/// Represents a module instance type.
#[derive(Debug, Clone)]
pub struct InstanceType {
    /// The effective type size for the module instance type.
    pub(crate) type_size: usize,
    /// The kind of module instance type.
    pub kind: InstanceTypeKind,
}

impl InstanceType {
    /// Gets the exports of the instance type.
    pub fn exports<'a>(&'a self, types: TypesRef<'a>) -> &'a IndexMap<String, EntityType> {
        self.internal_exports(types.list)
    }

    pub(crate) fn internal_exports<'a>(
        &'a self,
        types: &'a TypeList,
    ) -> &'a IndexMap<String, EntityType> {
        match &self.kind {
            InstanceTypeKind::Instantiated(id) => &types[*id].as_module_type().unwrap().exports,
            InstanceTypeKind::Exports(exports) => exports,
        }
    }
}

/// The entity type for imports and exports of a component.
#[derive(Debug, Clone, Copy)]
pub enum ComponentEntityType {
    /// The entity is a core module.
    Module(TypeId),
    /// The entity is a function.
    Func(TypeId),
    /// The entity is a value.
    Value(ComponentValType),
    /// The entity is a type.
    Type(TypeId),
    /// The entity is a component instance.
    Instance(TypeId),
    /// The entity is a component.
    Component(TypeId),
}

impl ComponentEntityType {
    /// Determines if component entity type `a` is a subtype of `b`.
    pub fn is_subtype_of(a: &Self, at: TypesRef, b: &Self, bt: TypesRef) -> bool {
        Self::internal_is_subtype_of(a, at.list, b, bt.list)
    }

    pub(crate) fn internal_is_subtype_of(a: &Self, at: &TypeList, b: &Self, bt: &TypeList) -> bool {
        match (a, b) {
            (Self::Module(a), Self::Module(b)) => ModuleType::internal_is_subtype_of(
                at[*a].as_module_type().unwrap(),
                at,
                bt[*b].as_module_type().unwrap(),
                bt,
            ),
            (Self::Func(a), Self::Func(b)) => ComponentFuncType::internal_is_subtype_of(
                at[*a].as_component_func_type().unwrap(),
                at,
                bt[*b].as_component_func_type().unwrap(),
                bt,
            ),
            (Self::Value(a), Self::Value(b)) => {
                ComponentValType::internal_is_subtype_of(a, at, b, bt)
            }
            (Self::Type(a), Self::Type(b)) => ComponentDefinedType::internal_is_subtype_of(
                at[*a].as_defined_type().unwrap(),
                at,
                bt[*b].as_defined_type().unwrap(),
                bt,
            ),
            (Self::Instance(a), Self::Instance(b)) => {
                ComponentInstanceType::internal_is_subtype_of(
                    at[*a].as_component_instance_type().unwrap(),
                    at,
                    bt[*b].as_component_instance_type().unwrap(),
                    bt,
                )
            }
            (Self::Component(a), Self::Component(b)) => ComponentType::internal_is_subtype_of(
                at[*a].as_component_type().unwrap(),
                at,
                bt[*b].as_component_type().unwrap(),
                bt,
            ),
            _ => false,
        }
    }

    pub(crate) fn desc(&self) -> &'static str {
        match self {
            Self::Module(_) => "module",
            Self::Func(_) => "function",
            Self::Value(_) => "value",
            Self::Type(_) => "type",
            Self::Instance(_) => "instance",
            Self::Component(_) => "component",
        }
    }

    pub(crate) fn type_size(&self) -> usize {
        match self {
            Self::Module(ty)
            | Self::Func(ty)
            | Self::Type(ty)
            | Self::Instance(ty)
            | Self::Component(ty) => ty.type_size,
            Self::Value(ty) => ty.type_size(),
        }
    }
}

/// Represents a type of a component.
#[derive(Debug, Clone)]
pub struct ComponentType {
    /// The effective type size for the component type.
    pub(crate) type_size: usize,
    /// The imports of the component type.
    pub imports: IndexMap<KebabString, (Option<Url>, ComponentEntityType)>,
    /// The exports of the component type.
    pub exports: IndexMap<KebabString, (Option<Url>, ComponentEntityType)>,
}

impl ComponentType {
    /// Determines if component type `a` is a subtype of `b`.
    pub fn is_subtype_of(a: &Self, at: TypesRef, b: &Self, bt: TypesRef) -> bool {
        Self::internal_is_subtype_of(a, at.list, b, bt.list)
    }

    pub(crate) fn internal_is_subtype_of(a: &Self, at: &TypeList, b: &Self, bt: &TypeList) -> bool {
        // For component type subtyping, all exports in the other component type
        // must be present in this component type's exports (i.e. it can export
        // *more* than what this component type needs).
        // However, for imports, the check is reversed (i.e. it is okay
        // to import *less* than what this component type needs).
        a.imports.iter().all(|(k, (_, a))| match b.imports.get(k) {
            Some((_, b)) => ComponentEntityType::internal_is_subtype_of(b, bt, a, at),
            None => false,
        }) && b.exports.iter().all(|(k, (_, b))| match a.exports.get(k) {
            Some((_, a)) => ComponentEntityType::internal_is_subtype_of(a, at, b, bt),
            None => false,
        })
    }
}

/// Represents the kind of a component instance.
#[derive(Debug, Clone)]
pub enum ComponentInstanceTypeKind {
    /// The instance type is from a definition.
    Defined(IndexMap<KebabString, (Option<Url>, ComponentEntityType)>),
    /// The instance type is the result of instantiating a component type.
    Instantiated(TypeId),
    /// The instance type is the result of instantiating from exported items.
    Exports(IndexMap<KebabString, (Option<Url>, ComponentEntityType)>),
}

/// Represents a type of a component instance.
#[derive(Debug, Clone)]
pub struct ComponentInstanceType {
    /// The effective type size for the instance type.
    pub(crate) type_size: usize,
    /// The kind of instance type.
    pub kind: ComponentInstanceTypeKind,
}

impl ComponentInstanceType {
    /// Gets the exports of the instance type.
    pub fn exports<'a>(
        &'a self,
        types: TypesRef<'a>,
    ) -> impl ExactSizeIterator<Item = (&'a KebabStr, &'a Option<Url>, ComponentEntityType)> + Clone
    {
        self.internal_exports(types.list)
            .iter()
            .map(|(n, (u, t))| (n.as_kebab_str(), u, *t))
    }

    pub(crate) fn internal_exports<'a>(
        &'a self,
        types: &'a TypeList,
    ) -> &'a IndexMap<KebabString, (Option<Url>, ComponentEntityType)> {
        match &self.kind {
            ComponentInstanceTypeKind::Defined(exports)
            | ComponentInstanceTypeKind::Exports(exports) => exports,
            ComponentInstanceTypeKind::Instantiated(id) => {
                &types[*id].as_component_type().unwrap().exports
            }
        }
    }

    /// Determines if component instance type `a` is a subtype of `b`.
    pub fn is_subtype_of(a: &Self, at: TypesRef, b: &Self, bt: TypesRef) -> bool {
        Self::internal_is_subtype_of(a, at.list, b, bt.list)
    }

    pub(crate) fn internal_is_subtype_of(a: &Self, at: &TypeList, b: &Self, bt: &TypeList) -> bool {
        let exports = a.internal_exports(at);

        // For instance type subtyping, all exports in the other instance type
        // must be present in this instance type's exports (i.e. it can export
        // *more* than what this instance type needs).
        b.internal_exports(bt)
            .iter()
            .all(|(k, (_, b))| match exports.get(k) {
                Some((_, a)) => ComponentEntityType::internal_is_subtype_of(a, at, b, bt),
                None => false,
            })
    }
}

/// Represents a type of a component function.
#[derive(Debug, Clone)]
pub struct ComponentFuncType {
    /// The effective type size for the component function type.
    pub(crate) type_size: usize,
    /// The function parameters.
    pub params: Box<[(KebabString, ComponentValType)]>,
    /// The function's results.
    pub results: Box<[(Option<KebabString>, ComponentValType)]>,
}

impl ComponentFuncType {
    /// Determines if component function type `a` is a subtype of `b`.
    pub fn is_subtype_of(a: &Self, at: TypesRef, b: &Self, bt: TypesRef) -> bool {
        Self::internal_is_subtype_of(a, at.list, b, bt.list)
    }

    pub(crate) fn internal_is_subtype_of(a: &Self, at: &TypeList, b: &Self, bt: &TypeList) -> bool {
        // Subtyping rules:
        // https://github.com/WebAssembly/component-model/blob/main/design/mvp/Subtyping.md

        // The subtype cannot have more parameters than the supertype
        if b.params.len() < a.params.len() {
            return false;
        }

        // The supertype cannot have more results than the subtype
        if a.results.len() < b.results.len() {
            return false;
        }

        for ((an, a), (bn, b)) in a.params.iter().zip(b.params.iter()) {
            // All overlapping parameters must have the same name
            if an != bn {
                return false;
            }

            // Contravariant on parameters
            if !ComponentValType::internal_is_subtype_of(b, bt, a, at) {
                return false;
            }
        }

        // All remaining parameters in the supertype must be optional
        if !b
            .params
            .iter()
            .skip(a.params.len())
            .all(|(_, b)| b.is_optional(bt))
        {
            return false;
        }

        for ((an, a), (bn, b)) in a.results.iter().zip(b.results.iter()) {
            // All overlapping results must have the same name
            if an != bn {
                return false;
            }

            // Covariant on results
            if !ComponentValType::internal_is_subtype_of(a, at, b, bt) {
                return false;
            }
        }

        true
    }

    /// Lowers the component function type to core parameter and result types for the
    /// canonical ABI.
    pub(crate) fn lower(&self, types: &TypeList, import: bool) -> LoweringInfo {
        let mut info = LoweringInfo::default();

        for (_, ty) in self.params.iter() {
            // When `import` is false, it means we're lifting a core function,
            // check if the parameters needs realloc
            if !import && !info.requires_realloc {
                info.requires_realloc = ty.requires_realloc(types);
            }

            if !ty.push_wasm_types(types, &mut info.params) {
                // Too many parameters to pass directly
                // Function will have a single pointer parameter to pass the arguments
                // via linear memory
                info.params.clear();
                assert!(info.params.push(ValType::I32));
                info.requires_memory = true;

                // We need realloc as well when lifting a function
                if !import {
                    info.requires_realloc = true;
                }
                break;
            }
        }

        for (_, ty) in self.results.iter() {
            // When `import` is true, it means we're lowering a component function,
            // check if the result needs realloc
            if import && !info.requires_realloc {
                info.requires_realloc = ty.requires_realloc(types);
            }

            if !ty.push_wasm_types(types, &mut info.results) {
                // Too many results to return directly, either a retptr parameter will be used (import)
                // or a single pointer will be returned (export)
                info.results.clear();
                if import {
                    info.params.max = MAX_LOWERED_TYPES;
                    assert!(info.params.push(ValType::I32));
                } else {
                    assert!(info.results.push(ValType::I32));
                }
                info.requires_memory = true;
                break;
            }
        }

        // Memory is always required when realloc is required
        info.requires_memory |= info.requires_realloc;

        info
    }
}

/// Represents a variant case.
#[derive(Debug, Clone)]
pub struct VariantCase {
    /// The variant case type.
    pub ty: Option<ComponentValType>,
    /// The name of the variant case refined by this one.
    pub refines: Option<KebabString>,
}

/// Represents a record type.
#[derive(Debug, Clone)]
pub struct RecordType {
    /// The effective type size for the record type.
    pub(crate) type_size: usize,
    /// The map of record fields.
    pub fields: IndexMap<KebabString, ComponentValType>,
}

/// Represents a variant type.
#[derive(Debug, Clone)]
pub struct VariantType {
    /// The effective type size for the variant type.
    pub(crate) type_size: usize,
    /// The map of variant cases.
    pub cases: IndexMap<KebabString, VariantCase>,
}

/// Represents a tuple type.
#[derive(Debug, Clone)]
pub struct TupleType {
    /// The effective type size for the tuple type.
    pub(crate) type_size: usize,
    /// The types of the tuple.
    pub types: Box<[ComponentValType]>,
}

/// Represents a union type.
#[derive(Debug, Clone)]
pub struct UnionType {
    /// The inclusive type count for the union type.
    pub(crate) type_size: usize,
    /// The types of the union.
    pub types: Box<[ComponentValType]>,
}

/// Represents a component defined type.
#[derive(Debug, Clone)]
pub enum ComponentDefinedType {
    /// The type is a primitive value type.
    Primitive(PrimitiveValType),
    /// The type is a record.
    Record(RecordType),
    /// The type is a variant.
    Variant(VariantType),
    /// The type is a list.
    List(ComponentValType),
    /// The type is a tuple.
    Tuple(TupleType),
    /// The type is a set of flags.
    Flags(IndexSet<KebabString>),
    /// The type is an enumeration.
    Enum(IndexSet<KebabString>),
    /// The type is a union.
    Union(UnionType),
    /// The type is an `option`.
    Option(ComponentValType),
    /// The type is a `result`.
    Result {
        /// The `ok` type.
        ok: Option<ComponentValType>,
        /// The `error` type.
        err: Option<ComponentValType>,
    },
}

impl ComponentDefinedType {
    pub(crate) fn requires_realloc(&self, types: &TypeList) -> bool {
        match self {
            Self::Primitive(ty) => ty.requires_realloc(),
            Self::Record(r) => r.fields.values().any(|ty| ty.requires_realloc(types)),
            Self::Variant(v) => v.cases.values().any(|case| {
                case.ty
                    .map(|ty| ty.requires_realloc(types))
                    .unwrap_or(false)
            }),
            Self::List(_) => true,
            Self::Tuple(t) => t.types.iter().any(|ty| ty.requires_realloc(types)),
            Self::Union(u) => u.types.iter().any(|ty| ty.requires_realloc(types)),
            Self::Flags(_) | Self::Enum(_) => false,
            Self::Option(ty) => ty.requires_realloc(types),
            Self::Result { ok, err } => {
                ok.map(|ty| ty.requires_realloc(types)).unwrap_or(false)
                    || err.map(|ty| ty.requires_realloc(types)).unwrap_or(false)
            }
        }
    }

    /// Determines if component defined type `a` is a subtype of `b`.
    pub fn is_subtype_of(a: &Self, at: TypesRef, b: &Self, bt: TypesRef) -> bool {
        Self::internal_is_subtype_of(a, at.list, b, bt.list)
    }

    pub(crate) fn internal_is_subtype_of(a: &Self, at: &TypeList, b: &Self, bt: &TypeList) -> bool {
        // Subtyping rules according to
        // https://github.com/WebAssembly/component-model/blob/main/design/mvp/Subtyping.md
        match (a, b) {
            (Self::Primitive(a), Self::Primitive(b)) => PrimitiveValType::is_subtype_of(*a, *b),
            (Self::Record(a), Self::Record(b)) => {
                for (name, a) in a.fields.iter() {
                    if let Some(b) = b.fields.get(name) {
                        if !ComponentValType::internal_is_subtype_of(a, at, b, bt) {
                            return false;
                        }
                    } else {
                        // Superfluous fields can be ignored in the subtype
                    }
                }
                // Check for missing required fields in the supertype
                for (name, b) in b.fields.iter() {
                    if !b.is_optional(bt) && !a.fields.contains_key(name) {
                        return false;
                    }
                }
                true
            }
            (Self::Variant(a), Self::Variant(b)) => {
                for (name, a) in a.cases.iter() {
                    if let Some(b) = b.cases.get(name) {
                        // Covariant subtype on the case type
                        if !Self::is_optional_subtype_of(a.ty, at, b.ty, bt) {
                            return false;
                        }
                    } else if let Some(refines) = &a.refines {
                        if !b.cases.contains_key(refines) {
                            // The refinement case is not in the supertype
                            return false;
                        }
                    } else {
                        // This case is not in the supertype and there is no
                        // default
                        return false;
                    }
                }
                true
            }
            (Self::List(a), Self::List(b)) | (Self::Option(a), Self::Option(b)) => {
                ComponentValType::internal_is_subtype_of(a, at, b, bt)
            }
            (Self::Tuple(a), Self::Tuple(b)) => {
                if a.types.len() != b.types.len() {
                    return false;
                }
                a.types
                    .iter()
                    .zip(b.types.iter())
                    .all(|(a, b)| ComponentValType::internal_is_subtype_of(a, at, b, bt))
            }
            (Self::Union(a), Self::Union(b)) => {
                if a.types.len() != b.types.len() {
                    return false;
                }
                a.types
                    .iter()
                    .zip(b.types.iter())
                    .all(|(a, b)| ComponentValType::internal_is_subtype_of(a, at, b, bt))
            }
            (Self::Flags(a), Self::Flags(b)) | (Self::Enum(a), Self::Enum(b)) => a.is_subset(b),
            (Self::Result { ok: ao, err: ae }, Self::Result { ok: bo, err: be }) => {
                Self::is_optional_subtype_of(*ao, at, *bo, bt)
                    && Self::is_optional_subtype_of(*ae, at, *be, bt)
            }
            _ => false,
        }
    }

    pub(crate) fn type_size(&self) -> usize {
        match self {
            Self::Primitive(_) => 1,
            Self::Flags(_) | Self::Enum(_) => 1,
            Self::Record(r) => r.type_size,
            Self::Variant(v) => v.type_size,
            Self::Tuple(t) => t.type_size,
            Self::Union(u) => u.type_size,
            Self::List(ty) | Self::Option(ty) => ty.type_size(),
            Self::Result { ok, err } => {
                ok.map(|ty| ty.type_size()).unwrap_or(1) + err.map(|ty| ty.type_size()).unwrap_or(1)
            }
        }
    }

    fn is_optional_subtype_of(
        a: Option<ComponentValType>,
        at: &TypeList,
        b: Option<ComponentValType>,
        bt: &TypeList,
    ) -> bool {
        match (a, b) {
            (None, None) | (Some(_), None) => true,
            (None, Some(_)) => false,
            (Some(a), Some(b)) => ComponentValType::internal_is_subtype_of(&a, at, &b, bt),
        }
    }
    fn push_wasm_types(&self, types: &TypeList, lowered_types: &mut LoweredTypes) -> bool {
        match self {
            Self::Primitive(ty) => push_primitive_wasm_types(ty, lowered_types),
            Self::Record(r) => r
                .fields
                .iter()
                .all(|(_, ty)| ty.push_wasm_types(types, lowered_types)),
            Self::Variant(v) => Self::push_variant_wasm_types(
                v.cases.iter().filter_map(|(_, case)| case.ty.as_ref()),
                types,
                lowered_types,
            ),
            Self::List(_) => lowered_types.push(ValType::I32) && lowered_types.push(ValType::I32),
            Self::Tuple(t) => t
                .types
                .iter()
                .all(|ty| ty.push_wasm_types(types, lowered_types)),
            Self::Flags(names) => {
                (0..(names.len() + 31) / 32).all(|_| lowered_types.push(ValType::I32))
            }
            Self::Enum(_) => lowered_types.push(ValType::I32),
            Self::Union(u) => Self::push_variant_wasm_types(u.types.iter(), types, lowered_types),
            Self::Option(ty) => {
                Self::push_variant_wasm_types([ty].into_iter(), types, lowered_types)
            }
            Self::Result { ok, err } => {
                Self::push_variant_wasm_types(ok.iter().chain(err.iter()), types, lowered_types)
            }
        }
    }

    fn push_variant_wasm_types<'a>(
        cases: impl Iterator<Item = &'a ComponentValType>,
        types: &TypeList,
        lowered_types: &mut LoweredTypes,
    ) -> bool {
        // Push the discriminant
        if !lowered_types.push(ValType::I32) {
            return false;
        }

        let start = lowered_types.len();

        for ty in cases {
            let mut temp = LoweredTypes::new(lowered_types.max);

            if !ty.push_wasm_types(types, &mut temp) {
                return false;
            }

            for (i, ty) in temp.iter().enumerate() {
                match lowered_types.get_mut(start + i) {
                    Some(prev) => *prev = Self::join_types(*prev, ty),
                    None => {
                        if !lowered_types.push(ty) {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }

    fn join_types(a: ValType, b: ValType) -> ValType {
        use ValType::*;

        match (a, b) {
            (I32, I32) | (I64, I64) | (F32, F32) | (F64, F64) => a,
            (I32, F32) | (F32, I32) => I32,
            (_, I64 | F64) | (I64 | F64, _) => I64,
            _ => panic!("unexpected wasm type for canonical ABI"),
        }
    }
}

#[allow(clippy::large_enum_variant)]
enum TypesKind {
    Module(Arc<Module>),
    Component(ComponentState),
}

/// Represents the types known to a [`crate::Validator`] once validation has completed.
///
/// The type information is returned via the [`crate::Validator::end`] method.
pub struct Types {
    list: TypeList,
    kind: TypesKind,
}

#[derive(Clone, Copy)]
enum TypesRefKind<'a> {
    Module(&'a Module),
    Component(&'a ComponentState),
}

/// Represents the types known to a [`crate::Validator`] during validation.
///
/// Retrieved via the [`crate::Validator::types`] method.
#[derive(Clone, Copy)]
pub struct TypesRef<'a> {
    list: &'a TypeList,
    kind: TypesRefKind<'a>,
}

impl<'a> TypesRef<'a> {
    pub(crate) fn from_module(types: &'a TypeList, module: &'a Module) -> Self {
        Self {
            list: types,
            kind: TypesRefKind::Module(module),
        }
    }

    pub(crate) fn from_component(types: &'a TypeList, component: &'a ComponentState) -> Self {
        Self {
            list: types,
            kind: TypesRefKind::Component(component),
        }
    }

    fn types(&self, core: bool) -> Option<&'a [TypeId]> {
        Some(match &self.kind {
            TypesRefKind::Module(module) => {
                if core {
                    &module.types
                } else {
                    return None;
                }
            }
            TypesRefKind::Component(component) => {
                if core {
                    &component.core_types
                } else {
                    &component.types
                }
            }
        })
    }

    /// Gets a type based on its type id.
    ///
    /// Returns `None` if the type id is unknown.
    pub fn type_from_id(&self, id: TypeId) -> Option<&'a Type> {
        self.list.get(id.index)
    }

    /// Gets a type id from a type index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn id_from_type_index(&self, index: u32, core: bool) -> Option<TypeId> {
        self.types(core)?.get(index as usize).copied()
    }

    /// Gets a type at the given type index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn type_at(&self, index: u32, core: bool) -> Option<&'a Type> {
        self.type_from_id(*self.types(core)?.get(index as usize)?)
    }

    /// Gets a defined core function type at the given type index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn func_type_at(&self, index: u32) -> Option<&'a FuncType> {
        match self.type_at(index, true)? {
            Type::Func(ty) => Some(ty),
            _ => None,
        }
    }

    /// Gets the type of a table at the given table index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn table_at(&self, index: u32) -> Option<TableType> {
        let tables = match &self.kind {
            TypesRefKind::Module(module) => &module.tables,
            TypesRefKind::Component(component) => &component.core_tables,
        };

        tables.get(index as usize).copied()
    }

    /// Gets the type of a memory at the given memory index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn memory_at(&self, index: u32) -> Option<MemoryType> {
        let memories = match &self.kind {
            TypesRefKind::Module(module) => &module.memories,
            TypesRefKind::Component(component) => &component.core_memories,
        };

        memories.get(index as usize).copied()
    }

    /// Gets the type of a global at the given global index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn global_at(&self, index: u32) -> Option<GlobalType> {
        let globals = match &self.kind {
            TypesRefKind::Module(module) => &module.globals,
            TypesRefKind::Component(component) => &component.core_globals,
        };

        globals.get(index as usize).copied()
    }

    /// Gets the type of a tag at the given tag index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn tag_at(&self, index: u32) -> Option<&'a FuncType> {
        let tags = match &self.kind {
            TypesRefKind::Module(module) => &module.tags,
            TypesRefKind::Component(component) => &component.core_tags,
        };

        Some(
            self.list[*tags.get(index as usize)?]
                .as_func_type()
                .unwrap(),
        )
    }

    /// Gets the type of a core function at the given function index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn function_at(&self, index: u32) -> Option<&'a FuncType> {
        let id = match &self.kind {
            TypesRefKind::Module(module) => {
                &module.types[*module.functions.get(index as usize)? as usize]
            }
            TypesRefKind::Component(component) => component.core_funcs.get(index as usize)?,
        };

        match &self.list[*id] {
            Type::Func(ty) => Some(ty),
            _ => None,
        }
    }

    /// Gets the type of an element segment at the given element segment index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn element_at(&self, index: u32) -> Option<ValType> {
        match &self.kind {
            TypesRefKind::Module(module) => module.element_types.get(index as usize).copied(),
            TypesRefKind::Component(_) => None,
        }
    }

    /// Gets the type of a component function at the given function index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn component_function_at(&self, index: u32) -> Option<&'a ComponentFuncType> {
        match &self.kind {
            TypesRefKind::Module(_) => None,
            TypesRefKind::Component(component) => Some(
                self.list[*component.funcs.get(index as usize)?]
                    .as_component_func_type()
                    .unwrap(),
            ),
        }
    }

    /// Gets the type of a module at the given module index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn module_at(&self, index: u32) -> Option<&'a ModuleType> {
        match &self.kind {
            TypesRefKind::Module(_) => None,
            TypesRefKind::Component(component) => Some(
                self.list[*component.core_modules.get(index as usize)?]
                    .as_module_type()
                    .unwrap(),
            ),
        }
    }

    /// Gets the type of a module instance at the given module instance index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn instance_at(&self, index: u32) -> Option<&'a InstanceType> {
        match &self.kind {
            TypesRefKind::Module(_) => None,
            TypesRefKind::Component(component) => {
                let id = component.core_instances.get(index as usize)?;
                match &self.list[*id] {
                    Type::Instance(ty) => Some(ty),
                    _ => None,
                }
            }
        }
    }

    /// Gets the type of a component at the given component index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn component_at(&self, index: u32) -> Option<&'a ComponentType> {
        match &self.kind {
            TypesRefKind::Module(_) => None,
            TypesRefKind::Component(component) => Some(
                self.list[*component.components.get(index as usize)?]
                    .as_component_type()
                    .unwrap(),
            ),
        }
    }

    /// Gets the type of an component instance at the given component instance index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn component_instance_at(&self, index: u32) -> Option<&'a ComponentInstanceType> {
        match &self.kind {
            TypesRefKind::Module(_) => None,
            TypesRefKind::Component(component) => {
                let id = component.instances.get(index as usize)?;
                match &self.list[*id] {
                    Type::ComponentInstance(ty) => Some(ty),
                    _ => None,
                }
            }
        }
    }

    /// Gets the type of a value at the given value index.
    ///
    /// Returns `None` if the type index is out of bounds or the type has not
    /// been parsed yet.
    pub fn value_at(&self, index: u32) -> Option<ComponentValType> {
        match &self.kind {
            TypesRefKind::Module(_) => None,
            TypesRefKind::Component(component) => {
                component.values.get(index as usize).map(|(r, _)| *r)
            }
        }
    }

    /// Gets the entity type for the given import.
    pub fn entity_type_from_import(&self, import: &Import) -> Option<EntityType> {
        match &self.kind {
            TypesRefKind::Module(module) => Some(match import.ty {
                TypeRef::Func(idx) => EntityType::Func(*module.types.get(idx as usize)?),
                TypeRef::Table(ty) => EntityType::Table(ty),
                TypeRef::Memory(ty) => EntityType::Memory(ty),
                TypeRef::Global(ty) => EntityType::Global(ty),
                TypeRef::Tag(ty) => EntityType::Tag(*module.types.get(ty.func_type_idx as usize)?),
            }),
            TypesRefKind::Component(_) => None,
        }
    }

    /// Gets the entity type from the given export.
    pub fn entity_type_from_export(&self, export: &Export) -> Option<EntityType> {
        match &self.kind {
            TypesRefKind::Module(module) => Some(match export.kind {
                ExternalKind::Func => EntityType::Func(
                    module.types[*module.functions.get(export.index as usize)? as usize],
                ),
                ExternalKind::Table => {
                    EntityType::Table(*module.tables.get(export.index as usize)?)
                }
                ExternalKind::Memory => {
                    EntityType::Memory(*module.memories.get(export.index as usize)?)
                }
                ExternalKind::Global => {
                    EntityType::Global(*module.globals.get(export.index as usize)?)
                }
                ExternalKind::Tag => EntityType::Tag(
                    module.types[*module.functions.get(export.index as usize)? as usize],
                ),
            }),
            TypesRefKind::Component(_) => None,
        }
    }

    /// Gets the component entity type for the given component import.
    pub fn component_entity_type_from_import(
        &self,
        import: &ComponentImport,
    ) -> Option<ComponentEntityType> {
        match &self.kind {
            TypesRefKind::Module(_) => None,
            TypesRefKind::Component(component) => Some(match import.ty {
                ComponentTypeRef::Module(idx) => {
                    ComponentEntityType::Module(*component.core_types.get(idx as usize)?)
                }
                ComponentTypeRef::Func(idx) => {
                    ComponentEntityType::Func(*component.types.get(idx as usize)?)
                }
                ComponentTypeRef::Value(ty) => ComponentEntityType::Value(match ty {
                    crate::ComponentValType::Primitive(ty) => ComponentValType::Primitive(ty),
                    crate::ComponentValType::Type(idx) => {
                        ComponentValType::Type(*component.types.get(idx as usize)?)
                    }
                }),
                ComponentTypeRef::Type(_, idx) => {
                    ComponentEntityType::Type(*component.types.get(idx as usize)?)
                }
                ComponentTypeRef::Instance(idx) => {
                    ComponentEntityType::Instance(*component.types.get(idx as usize)?)
                }
                ComponentTypeRef::Component(idx) => {
                    ComponentEntityType::Component(*component.types.get(idx as usize)?)
                }
            }),
        }
    }

    /// Gets the component entity type from the given component export.
    pub fn component_entity_type_from_export(
        &self,
        export: &ComponentExport,
    ) -> Option<ComponentEntityType> {
        match &self.kind {
            TypesRefKind::Module(_) => None,
            TypesRefKind::Component(component) => Some(match export.kind {
                ComponentExternalKind::Module => {
                    ComponentEntityType::Module(*component.core_modules.get(export.index as usize)?)
                }
                ComponentExternalKind::Func => {
                    ComponentEntityType::Func(*component.funcs.get(export.index as usize)?)
                }
                ComponentExternalKind::Value => ComponentEntityType::Value(
                    component
                        .values
                        .get(export.index as usize)
                        .map(|(r, _)| *r)?,
                ),
                ComponentExternalKind::Type => {
                    ComponentEntityType::Type(*component.types.get(export.index as usize)?)
                }
                ComponentExternalKind::Instance => {
                    ComponentEntityType::Instance(*component.instances.get(export.index as usize)?)
                }
                ComponentExternalKind::Component => ComponentEntityType::Component(
                    *component.components.get(export.index as usize)?,
                ),
            }),
        }
    }
}

impl Types {
    pub(crate) fn from_module(types: TypeList, module: Arc<Module>) -> Self {
        Self {
            list: types,
            kind: TypesKind::Module(module),
        }
    }

    pub(crate) fn from_component(types: TypeList, component: ComponentState) -> Self {
        Self {
            list: types,
            kind: TypesKind::Component(component),
        }
    }

    /// Gets a reference to this validation type information.
    pub fn as_ref(&self) -> TypesRef {
        TypesRef {
            list: &self.list,
            kind: match &self.kind {
                TypesKind::Module(module) => TypesRefKind::Module(module),
                TypesKind::Component(component) => TypesRefKind::Component(component),
            },
        }
    }

    /// Gets a type based on its type id.
    ///
    /// Returns `None` if the type id is unknown.
    pub fn type_from_id(&self, id: TypeId) -> Option<&Type> {
        self.as_ref().type_from_id(id)
    }

    /// Gets a type id from a type index.
    ///
    /// Returns `None` if the type index is out of bounds.
    pub fn id_from_type_index(&self, index: u32, core: bool) -> Option<TypeId> {
        self.as_ref().id_from_type_index(index, core)
    }

    /// Gets a type at the given type index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn type_at(&self, index: u32, core: bool) -> Option<&Type> {
        self.as_ref().type_at(index, core)
    }

    /// Gets a defined core function type at the given type index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn func_type_at(&self, index: u32) -> Option<&FuncType> {
        self.as_ref().func_type_at(index)
    }

    /// Gets the count of core types.
    pub fn type_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(module) => module.types.len(),
            TypesKind::Component(component) => component.core_types.len(),
        }
    }

    /// Gets the type of a table at the given table index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn table_at(&self, index: u32) -> Option<TableType> {
        self.as_ref().table_at(index)
    }

    /// Gets the count of imported and defined tables.
    pub fn table_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(module) => module.tables.len(),
            TypesKind::Component(component) => component.core_tables.len(),
        }
    }

    /// Gets the type of a memory at the given memory index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn memory_at(&self, index: u32) -> Option<MemoryType> {
        self.as_ref().memory_at(index)
    }

    /// Gets the count of imported and defined memories.
    pub fn memory_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(module) => module.memories.len(),
            TypesKind::Component(component) => component.core_memories.len(),
        }
    }

    /// Gets the type of a global at the given global index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn global_at(&self, index: u32) -> Option<GlobalType> {
        self.as_ref().global_at(index)
    }

    /// Gets the count of imported and defined globals.
    pub fn global_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(module) => module.globals.len(),
            TypesKind::Component(component) => component.core_globals.len(),
        }
    }

    /// Gets the type of a tag at the given tag index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn tag_at(&self, index: u32) -> Option<&FuncType> {
        self.as_ref().tag_at(index)
    }

    /// Gets the count of imported and defined tags.
    pub fn tag_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(module) => module.tags.len(),
            TypesKind::Component(component) => component.core_tags.len(),
        }
    }

    /// Gets the type of a core function at the given function index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn function_at(&self, index: u32) -> Option<&FuncType> {
        self.as_ref().function_at(index)
    }

    /// Gets the count of imported and defined core functions.
    ///
    /// The count also includes aliased core functions in components.
    pub fn function_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(module) => module.functions.len(),
            TypesKind::Component(component) => component.core_funcs.len(),
        }
    }

    /// Gets the type of an element segment at the given element segment index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn element_at(&self, index: u32) -> Option<ValType> {
        self.as_ref().element_at(index)
    }

    /// Gets the count of element segments.
    pub fn element_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(module) => module.element_types.len(),
            TypesKind::Component(_) => 0,
        }
    }

    /// Gets the type of a component function at the given function index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn component_function_at(&self, index: u32) -> Option<&ComponentFuncType> {
        self.as_ref().component_function_at(index)
    }

    /// Gets the count of imported, exported, or aliased component functions.
    pub fn component_function_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(_) => 0,
            TypesKind::Component(component) => component.funcs.len(),
        }
    }

    /// Gets the type of a module at the given module index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn module_at(&self, index: u32) -> Option<&ModuleType> {
        self.as_ref().module_at(index)
    }

    /// Gets the count of imported, exported, or aliased modules.
    pub fn module_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(_) => 0,
            TypesKind::Component(component) => component.core_modules.len(),
        }
    }

    /// Gets the type of a module instance at the given module instance index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn instance_at(&self, index: u32) -> Option<&InstanceType> {
        self.as_ref().instance_at(index)
    }

    /// Gets the count of imported, exported, or aliased core module instances.
    pub fn instance_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(_) => 0,
            TypesKind::Component(component) => component.core_instances.len(),
        }
    }

    /// Gets the type of a component at the given component index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn component_at(&self, index: u32) -> Option<&ComponentType> {
        self.as_ref().component_at(index)
    }

    /// Gets the count of imported, exported, or aliased components.
    pub fn component_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(_) => 0,
            TypesKind::Component(component) => component.components.len(),
        }
    }

    /// Gets the type of an component instance at the given component instance index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn component_instance_at(&self, index: u32) -> Option<&ComponentInstanceType> {
        self.as_ref().component_instance_at(index)
    }

    /// Gets the count of imported, exported, or aliased component instances.
    pub fn component_instance_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(_) => 0,
            TypesKind::Component(component) => component.instances.len(),
        }
    }

    /// Gets the type of a value at the given value index.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn value_at(&self, index: u32) -> Option<ComponentValType> {
        self.as_ref().value_at(index)
    }

    /// Gets the count of imported, exported, or aliased values.
    pub fn value_count(&self) -> usize {
        match &self.kind {
            TypesKind::Module(_) => 0,
            TypesKind::Component(component) => component.values.len(),
        }
    }

    /// Gets the entity type from the given import.
    pub fn entity_type_from_import(&self, import: &Import) -> Option<EntityType> {
        self.as_ref().entity_type_from_import(import)
    }

    /// Gets the entity type from the given export.
    pub fn entity_type_from_export(&self, export: &Export) -> Option<EntityType> {
        self.as_ref().entity_type_from_export(export)
    }

    /// Gets the component entity type for the given component import.
    pub fn component_entity_type_from_import(
        &self,
        import: &ComponentImport,
    ) -> Option<ComponentEntityType> {
        self.as_ref().component_entity_type_from_import(import)
    }

    /// Gets the component entity type from the given component export.
    pub fn component_entity_type_from_export(
        &self,
        export: &ComponentExport,
    ) -> Option<ComponentEntityType> {
        self.as_ref().component_entity_type_from_export(export)
    }
}

/// This is a type which mirrors a subset of the `Vec<T>` API, but is intended
/// to be able to be cheaply snapshotted and cloned.
///
/// When each module's code sections start we "commit" the current list of types
/// in the global list of types. This means that the temporary `cur` vec here is
/// pushed onto `snapshots` and wrapped up in an `Arc`. At that point we clone
/// this entire list (which is then O(modules), not O(types in all modules)) and
/// pass out as a context to each function validator.
///
/// Otherwise, though, this type behaves as if it were a large `Vec<T>`, but
/// it's represented by lists of contiguous chunks.
pub(crate) struct SnapshotList<T> {
    // All previous snapshots, the "head" of the list that this type represents.
    // The first entry in this pair is the starting index for all elements
    // contained in the list, and the second element is the list itself. Note
    // the `Arc` wrapper around sub-lists, which makes cloning time for this
    // `SnapshotList` O(snapshots) rather than O(snapshots_total), which for
    // us in this context means the number of modules, not types.
    //
    // Note that this list is sorted least-to-greatest in order of the index for
    // binary searching.
    snapshots: Vec<(usize, Arc<Vec<T>>)>,

    // This is the total length of all lists in the `snapshots` array.
    snapshots_total: usize,

    // The current list of types for the current snapshot that are being built.
    cur: Vec<T>,
}

impl<T> SnapshotList<T> {
    /// Same as `<&[T]>::get`
    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        // Check to see if this index falls on our local list
        if index >= self.snapshots_total {
            return self.cur.get(index - self.snapshots_total);
        }
        // ... and failing that we do a binary search to figure out which bucket
        // it's in. Note the `i-1` in the `Err` case because if we don't find an
        // exact match the type is located in the previous bucket.
        let i = match self.snapshots.binary_search_by_key(&index, |(i, _)| *i) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let (len, list) = &self.snapshots[i];
        Some(&list[index - len])
    }

    /// Same as `<&mut [T]>::get_mut`, except only works for indexes into the
    /// current snapshot being built.
    ///
    /// # Panics
    ///
    /// Panics if an index is passed in which falls within the
    /// previously-snapshotted list of types. This should never happen in our
    /// context and the panic is intended to weed out possible bugs in
    /// wasmparser.
    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.snapshots_total {
            return self.cur.get_mut(index - self.snapshots_total);
        }
        panic!("cannot get a mutable reference in snapshotted part of list")
    }

    /// Same as `Vec::push`
    pub(crate) fn push(&mut self, val: T) {
        self.cur.push(val);
    }

    /// Same as `<[T]>::len`
    pub(crate) fn len(&self) -> usize {
        self.cur.len() + self.snapshots_total
    }

    /// Reserve space for an additional count of items.
    pub(crate) fn reserve(&mut self, additional: usize) {
        self.cur.reserve(additional);
    }

    /// Commits previously pushed types into this snapshot vector, and returns a
    /// clone of this list.
    ///
    /// The returned `SnapshotList` can be used to access all the same types as
    /// this list itself. This list also is not changed (from an external
    /// perspective) and can continue to access all the same types.
    pub(crate) fn commit(&mut self) -> SnapshotList<T> {
        // If the current chunk has new elements, commit them in to an
        // `Arc`-wrapped vector in the snapshots list. Note the `shrink_to_fit`
        // ahead of time to hopefully keep memory usage lower than it would
        // otherwise be.
        let len = self.cur.len();
        if len > 0 {
            self.cur.shrink_to_fit();
            self.snapshots
                .push((self.snapshots_total, Arc::new(mem::take(&mut self.cur))));
            self.snapshots_total += len;
        }
        SnapshotList {
            snapshots: self.snapshots.clone(),
            snapshots_total: self.snapshots_total,
            cur: Vec::new(),
        }
    }
}

impl<T> std::ops::Index<usize> for SnapshotList<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        self.get(index).unwrap()
    }
}

impl<T> std::ops::IndexMut<usize> for SnapshotList<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.get_mut(index).unwrap()
    }
}

impl<T> std::ops::Index<TypeId> for SnapshotList<T> {
    type Output = T;

    #[inline]
    fn index(&self, id: TypeId) -> &T {
        self.get(id.index).unwrap()
    }
}

impl<T> std::ops::IndexMut<TypeId> for SnapshotList<T> {
    #[inline]
    fn index_mut(&mut self, id: TypeId) -> &mut T {
        self.get_mut(id.index).unwrap()
    }
}

impl<T> Default for SnapshotList<T> {
    fn default() -> SnapshotList<T> {
        SnapshotList {
            snapshots: Vec::new(),
            snapshots_total: 0,
            cur: Vec::new(),
        }
    }
}

/// A snapshot list of types.
pub(crate) type TypeList = SnapshotList<Type>;
