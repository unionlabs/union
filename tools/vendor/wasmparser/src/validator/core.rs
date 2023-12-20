//! State relating to validating a WebAssembly module.
//!
use super::{
    check_max, combine_type_sizes,
    operators::{OperatorValidator, OperatorValidatorAllocations},
    types::{EntityType, Type, TypeId, TypeList},
};
use crate::validator::core::arc::MaybeOwned;
use crate::{
    limits::*, BinaryReaderError, ConstExpr, Data, DataKind, Element, ElementItem, ElementKind,
    ExternalKind, FuncType, Global, GlobalType, MemoryType, Result, TableType, TagType, TypeRef,
    ValType, VisitOperator, WasmFeatures, WasmModuleResources,
};
use indexmap::IndexMap;
use std::mem;
use std::{collections::HashSet, sync::Arc};

fn check_value_type(ty: ValType, features: &WasmFeatures, offset: usize) -> Result<()> {
    match features.check_value_type(ty) {
        Ok(()) => Ok(()),
        Err(e) => Err(BinaryReaderError::new(e, offset)),
    }
}

// Section order for WebAssembly modules.
//
// Component sections are unordered and allow for duplicates,
// so this isn't used for components.
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub enum Order {
    Initial,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Tag,
    Global,
    Export,
    Start,
    Element,
    DataCount,
    Code,
    Data,
}

impl Default for Order {
    fn default() -> Order {
        Order::Initial
    }
}

#[derive(Default)]
pub(crate) struct ModuleState {
    /// Internal state that is incrementally built-up for the module being
    /// validated. This houses type information for all wasm items, like
    /// functions. Note that this starts out as a solely owned `Arc<T>` so we can
    /// get mutable access, but after we get to the code section this is never
    /// mutated to we can clone it cheaply and hand it to sub-validators.
    pub module: arc::MaybeOwned<Module>,

    /// Where we are, order-wise, in the wasm binary.
    order: Order,

    /// The number of data segments in the data section (if present).
    pub data_segment_count: u32,

    /// The number of functions we expect to be defined in the code section, or
    /// basically the length of the function section if it was found. The next
    /// index is where we are, in the code section index space, for the next
    /// entry in the code section (used to figure out what type is next for the
    /// function being validated).
    pub expected_code_bodies: Option<u32>,

    const_expr_allocs: OperatorValidatorAllocations,

    /// When parsing the code section, represents the current index in the section.
    code_section_index: Option<usize>,
}

impl ModuleState {
    pub fn update_order(&mut self, order: Order, offset: usize) -> Result<()> {
        if self.order >= order {
            return Err(BinaryReaderError::new("section out of order", offset));
        }

        self.order = order;

        Ok(())
    }

    pub fn validate_end(&self, offset: usize) -> Result<()> {
        // Ensure that the data count section, if any, was correct.
        if let Some(data_count) = self.module.data_count {
            if data_count != self.data_segment_count {
                return Err(BinaryReaderError::new(
                    "data count and data section have inconsistent lengths",
                    offset,
                ));
            }
        }
        // Ensure that the function section, if nonzero, was paired with a code
        // section with the appropriate length.
        if let Some(n) = self.expected_code_bodies {
            if n > 0 {
                return Err(BinaryReaderError::new(
                    "function and code section have inconsistent lengths",
                    offset,
                ));
            }
        }

        Ok(())
    }

    pub fn next_code_index_and_type(&mut self, offset: usize) -> Result<(u32, u32)> {
        let index = self
            .code_section_index
            .get_or_insert(self.module.num_imported_functions as usize);

        if *index >= self.module.functions.len() {
            return Err(BinaryReaderError::new(
                "code section entry exceeds number of functions",
                offset,
            ));
        }

        let ty = self.module.functions[*index];
        *index += 1;

        Ok(((*index - 1) as u32, ty))
    }

    pub fn add_global(
        &mut self,
        global: Global,
        features: &WasmFeatures,
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        self.module
            .check_global_type(&global.ty, features, offset)?;
        self.check_const_expr(&global.init_expr, global.ty.content_type, features, types)?;
        self.module.assert_mut().globals.push(global.ty);
        Ok(())
    }

    pub fn add_data_segment(
        &mut self,
        data: Data,
        features: &WasmFeatures,
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        match data.kind {
            DataKind::Passive => Ok(()),
            DataKind::Active {
                memory_index,
                offset_expr,
            } => {
                let ty = self.module.memory_at(memory_index, offset)?.index_type();
                self.check_const_expr(&offset_expr, ty, features, types)
            }
        }
    }

    pub fn add_element_segment(
        &mut self,
        e: Element,
        features: &WasmFeatures,
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        // the `funcref` value type is allowed all the way back to the MVP, so
        // don't check it here
        if e.ty != ValType::FuncRef {
            check_value_type(e.ty, features, offset)?;
        }
        if !e.ty.is_reference_type() {
            return Err(BinaryReaderError::new("malformed reference type", offset));
        }
        match e.kind {
            ElementKind::Active {
                table_index,
                offset_expr,
            } => {
                let table = self.module.table_at(table_index, offset)?;
                if e.ty != table.element_type {
                    return Err(BinaryReaderError::new(
                        "invalid element type for table type",
                        offset,
                    ));
                }

                self.check_const_expr(&offset_expr, ValType::I32, features, types)?;
            }
            ElementKind::Passive | ElementKind::Declared => {
                if !features.bulk_memory {
                    return Err(BinaryReaderError::new(
                        "bulk memory must be enabled",
                        offset,
                    ));
                }
            }
        }
        let mut items = e.items.get_items_reader()?;
        if items.get_count() > MAX_WASM_TABLE_ENTRIES as u32 {
            return Err(BinaryReaderError::new(
                "number of elements is out of bounds",
                offset,
            ));
        }
        for _ in 0..items.get_count() {
            let offset = items.original_position();
            match items.read()? {
                ElementItem::Expr(expr) => {
                    self.check_const_expr(&expr, e.ty, features, types)?;
                }
                ElementItem::Func(f) => {
                    if e.ty != ValType::FuncRef {
                        return Err(BinaryReaderError::new(
                            "type mismatch: segment does not have funcref type",
                            offset,
                        ));
                    }
                    self.module.get_func_type(f, types, offset)?;
                    self.module.assert_mut().function_references.insert(f);
                }
            }
        }

        self.module.assert_mut().element_types.push(e.ty);
        Ok(())
    }

    fn check_const_expr(
        &mut self,
        expr: &ConstExpr<'_>,
        expected_ty: ValType,
        features: &WasmFeatures,
        types: &TypeList,
    ) -> Result<()> {
        let mut validator = VisitConstOperator {
            offset: 0,
            order: self.order,
            uninserted_funcref: false,
            ops: OperatorValidator::new_const_expr(
                features,
                expected_ty,
                mem::take(&mut self.const_expr_allocs),
            ),
            resources: OperatorValidatorResources {
                types,
                module: &mut self.module,
            },
        };

        let mut ops = expr.get_operators_reader();
        while !ops.eof() {
            validator.offset = ops.original_position();
            ops.visit_operator(&mut validator)??;
        }
        validator.ops.finish(ops.original_position())?;

        // See comment in `RefFunc` below for why this is an assert.
        assert!(!validator.uninserted_funcref);

        self.const_expr_allocs = validator.ops.into_allocations();

        return Ok(());

        struct VisitConstOperator<'a> {
            offset: usize,
            uninserted_funcref: bool,
            ops: OperatorValidator,
            resources: OperatorValidatorResources<'a>,
            order: Order,
        }

        impl VisitConstOperator<'_> {
            fn validator(&mut self) -> impl VisitOperator<'_, Output = Result<()>> {
                self.ops.with_resources(&self.resources, self.offset)
            }

            fn validate_extended_const(&mut self) -> Result<()> {
                if self.ops.features.extended_const {
                    Ok(())
                } else {
                    Err(BinaryReaderError::new(
                        "constant expression required: non-constant operator",
                        self.offset,
                    ))
                }
            }

            fn validate_global(&mut self, index: u32) -> Result<()> {
                let module = &self.resources.module;
                let global = module.global_at(index, self.offset)?;
                if index >= module.num_imported_globals {
                    return Err(BinaryReaderError::new(
                        "constant expression required: global.get of locally defined global",
                        self.offset,
                    ));
                }
                if global.mutable {
                    return Err(BinaryReaderError::new(
                        "constant expression required: global.get of mutable global",
                        self.offset,
                    ));
                }
                Ok(())
            }

            // Functions in initialization expressions are only valid in
            // element segment initialization expressions and globals. In
            // these contexts we want to record all function references.
            //
            // Initialization expressions can also be found in the data
            // section, however. A `RefFunc` instruction in those situations
            // is always invalid and needs to produce a validation error. In
            // this situation, though, we can no longer modify
            // the state since it's been "snapshot" already for
            // parallel validation of functions.
            //
            // If we cannot modify the function references then this function
            // *should* result in a validation error, but we defer that
            // validation error to happen later. The `uninserted_funcref`
            // boolean here is used to track this and will cause a panic
            // (aka a fuzz bug) if we somehow forget to emit an error somewhere
            // else.
            fn insert_ref_func(&mut self, index: u32) {
                if self.order == Order::Data {
                    self.uninserted_funcref = true;
                } else {
                    self.resources
                        .module
                        .assert_mut()
                        .function_references
                        .insert(index);
                }
            }
        }

        macro_rules! define_visit_operator {
            ($(@$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident)*) => {
                $(
                    #[allow(unused_variables)]
                    fn $visit(&mut self $($(,$arg: $argty)*)?) -> Self::Output {
                        define_visit_operator!(@visit self $visit $($($arg)*)?)
                    }
                )*
            };

            // These are always valid in const expressions
            (@visit $self:ident visit_i32_const $val:ident) => {{
                $self.validator().visit_i32_const($val)
            }};
            (@visit $self:ident visit_i64_const $val:ident) => {{
                $self.validator().visit_i64_const($val)
            }};
            (@visit $self:ident visit_f32_const $val:ident) => {{
                $self.validator().visit_f32_const($val)
            }};
            (@visit $self:ident visit_f64_const $val:ident) => {{
                $self.validator().visit_f64_const($val)
            }};
            (@visit $self:ident visit_v128_const $val:ident) => {{
                $self.validator().visit_v128_const($val)
            }};
            (@visit $self:ident visit_ref_null $val:ident) => {{
                $self.validator().visit_ref_null($val)
            }};
            (@visit $self:ident visit_end) => {{
                $self.validator().visit_end()
            }};


            // These are valid const expressions when the extended-const proposal is enabled.
            (@visit $self:ident visit_i32_add) => {{
                $self.validate_extended_const()?;
                $self.validator().visit_i32_add()
            }};
            (@visit $self:ident visit_i32_sub) => {{
                $self.validate_extended_const()?;
                $self.validator().visit_i32_sub()
            }};
            (@visit $self:ident visit_i32_mul) => {{
                $self.validate_extended_const()?;
                $self.validator().visit_i32_mul()
            }};
            (@visit $self:ident visit_i64_add) => {{
                $self.validate_extended_const()?;
                $self.validator().visit_i64_add()
            }};
            (@visit $self:ident visit_i64_sub) => {{
                $self.validate_extended_const()?;
                $self.validator().visit_i64_sub()
            }};
            (@visit $self:ident visit_i64_mul) => {{
                $self.validate_extended_const()?;
                $self.validator().visit_i64_mul()
            }};

            // `global.get` is a valid const expression for imported, immutable
            // globals.
            (@visit $self:ident visit_global_get $idx:ident) => {{
                $self.validate_global($idx)?;
                $self.validator().visit_global_get($idx)
            }};
            // `ref.func`, if it's in a `global` initializer, will insert into
            // the set of referenced functions so it's processed here.
            (@visit $self:ident visit_ref_func $idx:ident) => {{
                $self.insert_ref_func($idx);
                $self.validator().visit_ref_func($idx)
            }};

            (@visit $self:ident $op:ident $($args:tt)*) => {{
                Err(BinaryReaderError::new(
                    "constant expression required: non-constant operator",
                    $self.offset,
                ))
            }}
        }

        impl<'a> VisitOperator<'a> for VisitConstOperator<'a> {
            type Output = Result<()>;

            for_each_operator!(define_visit_operator);
        }
    }
}

pub(crate) struct Module {
    // This is set once the code section starts.
    // `WasmModuleResources` implementations use the snapshot to
    // enable parallel validation of functions.
    pub snapshot: Option<Arc<TypeList>>,
    // Stores indexes into the validator's types list.
    pub types: Vec<TypeId>,
    pub tables: Vec<TableType>,
    pub memories: Vec<MemoryType>,
    pub globals: Vec<GlobalType>,
    pub element_types: Vec<ValType>,
    pub data_count: Option<u32>,
    // Stores indexes into `types`.
    pub functions: Vec<u32>,
    pub tags: Vec<TypeId>,
    pub function_references: HashSet<u32>,
    pub imports: IndexMap<(String, String), Vec<EntityType>>,
    pub exports: IndexMap<String, EntityType>,
    pub type_size: usize,
    num_imported_globals: u32,
    num_imported_functions: u32,
}

impl Module {
    pub fn add_type(
        &mut self,
        ty: crate::Type,
        features: &WasmFeatures,
        types: &mut TypeList,
        offset: usize,
        check_limit: bool,
    ) -> Result<()> {
        let ty = match ty {
            crate::Type::Func(t) => {
                for ty in t.params().iter().chain(t.results()) {
                    check_value_type(*ty, features, offset)?;
                }
                if t.results().len() > 1 && !features.multi_value {
                    return Err(BinaryReaderError::new(
                        "func type returns multiple values but the multi-value feature is not enabled",
                        offset,
                    ));
                }
                Type::Func(t)
            }
        };

        if check_limit {
            check_max(self.types.len(), 1, MAX_WASM_TYPES, "types", offset)?;
        }

        self.types.push(TypeId {
            type_size: ty.type_size(),
            index: types.len(),
            type_index: Some(self.types.len()),
            is_core: true,
        });
        types.push(ty);
        Ok(())
    }

    pub fn add_import(
        &mut self,
        import: crate::Import,
        features: &WasmFeatures,
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        let entity = self.check_type_ref(&import.ty, features, types, offset)?;

        let (len, max, desc) = match import.ty {
            TypeRef::Func(type_index) => {
                self.functions.push(type_index);
                self.num_imported_functions += 1;
                (self.functions.len(), MAX_WASM_FUNCTIONS, "functions")
            }
            TypeRef::Table(ty) => {
                self.tables.push(ty);
                (self.tables.len(), self.max_tables(features), "tables")
            }
            TypeRef::Memory(ty) => {
                self.memories.push(ty);
                (self.memories.len(), self.max_memories(features), "memories")
            }
            TypeRef::Tag(ty) => {
                self.tags.push(self.types[ty.func_type_idx as usize]);
                (self.tags.len(), MAX_WASM_TAGS, "tags")
            }
            TypeRef::Global(ty) => {
                if !features.mutable_global && ty.mutable {
                    return Err(BinaryReaderError::new(
                        "mutable global support is not enabled",
                        offset,
                    ));
                }
                self.globals.push(ty);
                self.num_imported_globals += 1;
                (self.globals.len(), MAX_WASM_GLOBALS, "globals")
            }
        };

        check_max(len, 0, max, desc, offset)?;

        self.type_size = combine_type_sizes(self.type_size, entity.type_size(), offset)?;

        self.imports
            .entry((import.module.to_string(), import.name.to_string()))
            .or_default()
            .push(entity);

        Ok(())
    }

    pub fn add_export(
        &mut self,
        name: &str,
        ty: EntityType,
        features: &WasmFeatures,
        offset: usize,
        check_limit: bool,
    ) -> Result<()> {
        if !features.mutable_global {
            if let EntityType::Global(global_type) = ty {
                if global_type.mutable {
                    return Err(BinaryReaderError::new(
                        "mutable global support is not enabled",
                        offset,
                    ));
                }
            }
        }

        if check_limit {
            check_max(self.exports.len(), 1, MAX_WASM_EXPORTS, "exports", offset)?;
        }

        self.type_size = combine_type_sizes(self.type_size, ty.type_size(), offset)?;

        match self.exports.insert(name.to_string(), ty) {
            Some(_) => Err(format_err!(
                offset,
                "duplicate export name `{name}` already defined"
            )),
            None => Ok(()),
        }
    }

    pub fn add_function(&mut self, type_index: u32, types: &TypeList, offset: usize) -> Result<()> {
        self.func_type_at(type_index, types, offset)?;
        self.functions.push(type_index);
        Ok(())
    }

    pub fn add_table(
        &mut self,
        ty: TableType,
        features: &WasmFeatures,
        offset: usize,
    ) -> Result<()> {
        self.check_table_type(&ty, features, offset)?;
        self.tables.push(ty);
        Ok(())
    }

    pub fn add_memory(
        &mut self,
        ty: MemoryType,
        features: &WasmFeatures,
        offset: usize,
    ) -> Result<()> {
        self.check_memory_type(&ty, features, offset)?;
        self.memories.push(ty);
        Ok(())
    }

    pub fn add_tag(
        &mut self,
        ty: TagType,
        features: &WasmFeatures,
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        self.check_tag_type(&ty, features, types, offset)?;
        self.tags.push(self.types[ty.func_type_idx as usize]);
        Ok(())
    }

    pub fn type_at(&self, idx: u32, offset: usize) -> Result<TypeId> {
        self.types
            .get(idx as usize)
            .copied()
            .ok_or_else(|| format_err!(offset, "unknown type {idx}: type index out of bounds"))
    }

    fn func_type_at<'a>(
        &self,
        type_index: u32,
        types: &'a TypeList,
        offset: usize,
    ) -> Result<&'a FuncType> {
        types[self.type_at(type_index, offset)?]
            .as_func_type()
            .ok_or_else(|| format_err!(offset, "type index {type_index} is not a function type"))
    }

    pub fn check_type_ref(
        &self,
        type_ref: &TypeRef,
        features: &WasmFeatures,
        types: &TypeList,
        offset: usize,
    ) -> Result<EntityType> {
        Ok(match type_ref {
            TypeRef::Func(type_index) => {
                self.func_type_at(*type_index, types, offset)?;
                EntityType::Func(self.types[*type_index as usize])
            }
            TypeRef::Table(t) => {
                self.check_table_type(t, features, offset)?;
                EntityType::Table(*t)
            }
            TypeRef::Memory(t) => {
                self.check_memory_type(t, features, offset)?;
                EntityType::Memory(*t)
            }
            TypeRef::Tag(t) => {
                self.check_tag_type(t, features, types, offset)?;
                EntityType::Tag(self.types[t.func_type_idx as usize])
            }
            TypeRef::Global(t) => {
                self.check_global_type(t, features, offset)?;
                EntityType::Global(*t)
            }
        })
    }

    fn check_table_type(
        &self,
        ty: &TableType,
        features: &WasmFeatures,
        offset: usize,
    ) -> Result<()> {
        // the `funcref` value type is allowed all the way back to the MVP, so
        // don't check it here
        if ty.element_type != ValType::FuncRef {
            check_value_type(ty.element_type, features, offset)?;
        }

        if !ty.element_type.is_reference_type() {
            return Err(BinaryReaderError::new(
                "element is not reference type",
                offset,
            ));
        }
        self.check_limits(ty.initial, ty.maximum, offset)?;
        if ty.initial > MAX_WASM_TABLE_ENTRIES as u32 {
            return Err(BinaryReaderError::new(
                "minimum table size is out of bounds",
                offset,
            ));
        }
        Ok(())
    }

    fn check_memory_type(
        &self,
        ty: &MemoryType,
        features: &WasmFeatures,
        offset: usize,
    ) -> Result<()> {
        self.check_limits(ty.initial, ty.maximum, offset)?;
        let (true_maximum, err) = if ty.memory64 {
            if !features.memory64 {
                return Err(BinaryReaderError::new(
                    "memory64 must be enabled for 64-bit memories",
                    offset,
                ));
            }
            (
                MAX_WASM_MEMORY64_PAGES,
                "memory size must be at most 2**48 pages",
            )
        } else {
            (
                MAX_WASM_MEMORY32_PAGES,
                "memory size must be at most 65536 pages (4GiB)",
            )
        };
        if ty.initial > true_maximum {
            return Err(BinaryReaderError::new(err, offset));
        }
        if let Some(maximum) = ty.maximum {
            if maximum > true_maximum {
                return Err(BinaryReaderError::new(err, offset));
            }
        }
        if ty.shared {
            if !features.threads {
                return Err(BinaryReaderError::new(
                    "threads must be enabled for shared memories",
                    offset,
                ));
            }
            if ty.maximum.is_none() {
                return Err(BinaryReaderError::new(
                    "shared memory must have maximum size",
                    offset,
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn imports_for_module_type(
        &self,
        offset: usize,
    ) -> Result<IndexMap<(String, String), EntityType>> {
        // Ensure imports are unique, which is a requirement of the component model
        self.imports
            .iter()
            .map(|((module, name), types)| {
                if types.len() != 1 {
                    bail!(
                        offset,
                        "module has a duplicate import name `{module}:{name}` \
                         that is not allowed in components",
                    );
                }
                Ok(((module.clone(), name.clone()), types[0]))
            })
            .collect::<Result<_>>()
    }

    fn check_tag_type(
        &self,
        ty: &TagType,
        features: &WasmFeatures,
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        if !features.exceptions {
            return Err(BinaryReaderError::new(
                "exceptions proposal not enabled",
                offset,
            ));
        }
        let ty = self.func_type_at(ty.func_type_idx, types, offset)?;
        if !ty.results().is_empty() {
            return Err(BinaryReaderError::new(
                "invalid exception type: non-empty tag result type",
                offset,
            ));
        }
        Ok(())
    }

    fn check_global_type(
        &self,
        ty: &GlobalType,
        features: &WasmFeatures,
        offset: usize,
    ) -> Result<()> {
        check_value_type(ty.content_type, features, offset)
    }

    fn check_limits<T>(&self, initial: T, maximum: Option<T>, offset: usize) -> Result<()>
    where
        T: Into<u64>,
    {
        if let Some(max) = maximum {
            if initial.into() > max.into() {
                return Err(BinaryReaderError::new(
                    "size minimum must not be greater than maximum",
                    offset,
                ));
            }
        }
        Ok(())
    }

    pub fn max_tables(&self, features: &WasmFeatures) -> usize {
        if features.reference_types {
            MAX_WASM_TABLES
        } else {
            1
        }
    }

    pub fn max_memories(&self, features: &WasmFeatures) -> usize {
        if features.multi_memory {
            MAX_WASM_MEMORIES
        } else {
            1
        }
    }

    pub fn export_to_entity_type(
        &mut self,
        export: &crate::Export,
        offset: usize,
    ) -> Result<EntityType> {
        let check = |ty: &str, index: u32, total: usize| {
            if index as usize >= total {
                Err(format_err!(
                    offset,
                    "unknown {ty} {index}: exported {ty} index out of bounds",
                ))
            } else {
                Ok(())
            }
        };

        Ok(match export.kind {
            ExternalKind::Func => {
                check("function", export.index, self.functions.len())?;
                self.function_references.insert(export.index);
                EntityType::Func(self.types[self.functions[export.index as usize] as usize])
            }
            ExternalKind::Table => {
                check("table", export.index, self.tables.len())?;
                EntityType::Table(self.tables[export.index as usize])
            }
            ExternalKind::Memory => {
                check("memory", export.index, self.memories.len())?;
                EntityType::Memory(self.memories[export.index as usize])
            }
            ExternalKind::Global => {
                check("global", export.index, self.globals.len())?;
                EntityType::Global(self.globals[export.index as usize])
            }
            ExternalKind::Tag => {
                check("tag", export.index, self.tags.len())?;
                EntityType::Tag(self.tags[export.index as usize])
            }
        })
    }

    pub fn get_func_type<'a>(
        &self,
        func_idx: u32,
        types: &'a TypeList,
        offset: usize,
    ) -> Result<&'a FuncType> {
        match self.functions.get(func_idx as usize) {
            Some(idx) => self.func_type_at(*idx, types, offset),
            None => Err(format_err!(
                offset,
                "unknown function {func_idx}: func index out of bounds",
            )),
        }
    }

    fn global_at(&self, idx: u32, offset: usize) -> Result<&GlobalType> {
        match self.globals.get(idx as usize) {
            Some(t) => Ok(t),
            None => Err(format_err!(
                offset,
                "unknown global {idx}: global index out of bounds"
            )),
        }
    }

    fn table_at(&self, idx: u32, offset: usize) -> Result<&TableType> {
        match self.tables.get(idx as usize) {
            Some(t) => Ok(t),
            None => Err(format_err!(
                offset,
                "unknown table {idx}: table index out of bounds"
            )),
        }
    }

    fn memory_at(&self, idx: u32, offset: usize) -> Result<&MemoryType> {
        match self.memories.get(idx as usize) {
            Some(t) => Ok(t),
            None => Err(format_err!(
                offset,
                "unknown memory {idx}: memory index out of bounds"
            )),
        }
    }
}

impl Default for Module {
    fn default() -> Self {
        Self {
            snapshot: Default::default(),
            types: Default::default(),
            tables: Default::default(),
            memories: Default::default(),
            globals: Default::default(),
            element_types: Default::default(),
            data_count: Default::default(),
            functions: Default::default(),
            tags: Default::default(),
            function_references: Default::default(),
            imports: Default::default(),
            exports: Default::default(),
            type_size: 1,
            num_imported_globals: Default::default(),
            num_imported_functions: Default::default(),
        }
    }
}

struct OperatorValidatorResources<'a> {
    module: &'a mut MaybeOwned<Module>,
    types: &'a TypeList,
}

impl WasmModuleResources for OperatorValidatorResources<'_> {
    type FuncType = crate::FuncType;

    fn table_at(&self, at: u32) -> Option<TableType> {
        self.module.tables.get(at as usize).cloned()
    }

    fn memory_at(&self, at: u32) -> Option<MemoryType> {
        self.module.memories.get(at as usize).cloned()
    }

    fn tag_at(&self, at: u32) -> Option<&Self::FuncType> {
        Some(
            self.types[*self.module.tags.get(at as usize)?]
                .as_func_type()
                .unwrap(),
        )
    }

    fn global_at(&self, at: u32) -> Option<GlobalType> {
        self.module.globals.get(at as usize).cloned()
    }

    fn func_type_at(&self, at: u32) -> Option<&Self::FuncType> {
        Some(
            self.types[*self.module.types.get(at as usize)?]
                .as_func_type()
                .unwrap(),
        )
    }

    fn type_of_function(&self, at: u32) -> Option<&Self::FuncType> {
        self.func_type_at(*self.module.functions.get(at as usize)?)
    }

    fn element_type_at(&self, at: u32) -> Option<ValType> {
        self.module.element_types.get(at as usize).cloned()
    }

    fn element_count(&self) -> u32 {
        self.module.element_types.len() as u32
    }

    fn data_count(&self) -> Option<u32> {
        self.module.data_count
    }

    fn is_function_referenced(&self, idx: u32) -> bool {
        self.module.function_references.contains(&idx)
    }
}

/// The implementation of [`WasmModuleResources`] used by
/// [`Validator`](crate::Validator).
pub struct ValidatorResources(pub(crate) Arc<Module>);

impl WasmModuleResources for ValidatorResources {
    type FuncType = crate::FuncType;

    fn table_at(&self, at: u32) -> Option<TableType> {
        self.0.tables.get(at as usize).cloned()
    }

    fn memory_at(&self, at: u32) -> Option<MemoryType> {
        self.0.memories.get(at as usize).cloned()
    }

    fn tag_at(&self, at: u32) -> Option<&Self::FuncType> {
        Some(
            self.0.snapshot.as_ref().unwrap()[*self.0.tags.get(at as usize)?]
                .as_func_type()
                .unwrap(),
        )
    }

    fn global_at(&self, at: u32) -> Option<GlobalType> {
        self.0.globals.get(at as usize).cloned()
    }

    fn func_type_at(&self, at: u32) -> Option<&Self::FuncType> {
        Some(
            self.0.snapshot.as_ref().unwrap()[*self.0.types.get(at as usize)?]
                .as_func_type()
                .unwrap(),
        )
    }

    fn type_of_function(&self, at: u32) -> Option<&Self::FuncType> {
        self.func_type_at(*self.0.functions.get(at as usize)?)
    }

    fn element_type_at(&self, at: u32) -> Option<ValType> {
        self.0.element_types.get(at as usize).cloned()
    }

    fn element_count(&self) -> u32 {
        self.0.element_types.len() as u32
    }

    fn data_count(&self) -> Option<u32> {
        self.0.data_count
    }

    fn is_function_referenced(&self, idx: u32) -> bool {
        self.0.function_references.contains(&idx)
    }
}

const _: () = {
    fn assert_send<T: Send>() {}

    // Assert that `ValidatorResources` is Send so function validation
    // can be parallelizable
    fn assert() {
        assert_send::<ValidatorResources>();
    }
};

mod arc {
    use std::ops::Deref;
    use std::sync::Arc;

    enum Inner<T> {
        Owned(T),
        Shared(Arc<T>),

        Empty, // Only used for swapping from owned to shared.
    }

    pub struct MaybeOwned<T> {
        inner: Inner<T>,
    }

    impl<T> MaybeOwned<T> {
        #[inline]
        fn as_mut(&mut self) -> Option<&mut T> {
            match &mut self.inner {
                Inner::Owned(x) => Some(x),
                Inner::Shared(_) => None,
                Inner::Empty => Self::unreachable(),
            }
        }

        #[inline]
        pub fn assert_mut(&mut self) -> &mut T {
            self.as_mut().unwrap()
        }

        pub fn arc(&mut self) -> &Arc<T> {
            self.make_shared();
            match &self.inner {
                Inner::Shared(x) => x,
                _ => Self::unreachable(),
            }
        }

        #[inline]
        fn make_shared(&mut self) {
            if let Inner::Shared(_) = self.inner {
                return;
            }

            let inner = std::mem::replace(&mut self.inner, Inner::Empty);
            let x = match inner {
                Inner::Owned(x) => x,
                _ => Self::unreachable(),
            };
            let x = Arc::new(x);
            self.inner = Inner::Shared(x);
        }

        #[cold]
        #[inline(never)]
        fn unreachable() -> ! {
            unreachable!()
        }
    }

    impl<T: Default> Default for MaybeOwned<T> {
        fn default() -> MaybeOwned<T> {
            MaybeOwned {
                inner: Inner::Owned(T::default()),
            }
        }
    }

    impl<T> Deref for MaybeOwned<T> {
        type Target = T;

        fn deref(&self) -> &T {
            match &self.inner {
                Inner::Owned(x) => x,
                Inner::Shared(x) => x,
                Inner::Empty => Self::unreachable(),
            }
        }
    }
}
