//! State relating to validating a WebAssembly component.

use super::{
    check_max, combine_type_sizes,
    core::Module,
    types::{
        ComponentFuncType, ComponentInstanceType, ComponentInstanceTypeKind, ComponentType,
        ComponentValType, EntityType, InstanceType, KebabString, ModuleType, RecordType, Type,
        TypeId, TypeList, VariantCase,
    },
};
use crate::{
    limits::*,
    types::{
        ComponentDefinedType, ComponentEntityType, InstanceTypeKind, KebabStr, LoweringInfo,
        TupleType, UnionType, VariantType,
    },
    BinaryReaderError, CanonicalOption, ComponentExternalKind, ComponentOuterAliasKind,
    ComponentTypeRef, ExternalKind, FuncType, GlobalType, InstantiationArgKind, MemoryType, Result,
    TableType, TypeBounds, ValType, WasmFeatures,
};
use indexmap::{map::Entry, IndexMap, IndexSet};
use std::{collections::HashSet, mem};
use url::Url;

fn to_kebab_str<'a>(s: &'a str, desc: &str, offset: usize) -> Result<&'a KebabStr> {
    match KebabStr::new(s) {
        Some(s) => Ok(s),
        None => {
            if s.is_empty() {
                bail!(offset, "{desc} name cannot be empty");
            }

            bail!(offset, "{desc} name `{s}` is not in kebab case");
        }
    }
}

fn parse_url(url: &str, offset: usize) -> Result<Option<Url>> {
    if url.is_empty() {
        return Ok(None);
    }

    Url::parse(url)
        .map(Some)
        .map_err(|e| BinaryReaderError::new(e.to_string(), offset))
}

pub(crate) struct ComponentState {
    // Core index spaces
    pub core_types: Vec<TypeId>,
    pub core_modules: Vec<TypeId>,
    pub core_instances: Vec<TypeId>,
    pub core_funcs: Vec<TypeId>,
    pub core_memories: Vec<MemoryType>,
    pub core_tables: Vec<TableType>,
    pub core_globals: Vec<GlobalType>,
    pub core_tags: Vec<TypeId>,

    // Component index spaces
    pub types: Vec<TypeId>,
    pub funcs: Vec<TypeId>,
    pub values: Vec<(ComponentValType, bool)>,
    pub instances: Vec<TypeId>,
    pub components: Vec<TypeId>,

    pub imports: IndexMap<KebabString, (Option<Url>, ComponentEntityType)>,
    pub exports: IndexMap<KebabString, (Option<Url>, ComponentEntityType)>,

    // Note: URL validation requires unique URLs by byte comparison, so
    // strings are used here and the URLs are not normalized.
    import_urls: HashSet<String>,
    export_urls: HashSet<String>,

    has_start: bool,
    type_size: usize,
}

impl ComponentState {
    pub fn type_count(&self) -> usize {
        self.core_types.len() + self.types.len()
    }

    pub fn instance_count(&self) -> usize {
        self.core_instances.len() + self.instances.len()
    }

    pub fn function_count(&self) -> usize {
        self.core_funcs.len() + self.funcs.len()
    }

    pub fn add_core_type(
        components: &mut [Self],
        ty: crate::CoreType,
        features: &WasmFeatures,
        types: &mut TypeList,
        offset: usize,
        check_limit: bool,
    ) -> Result<()> {
        let ty = match ty {
            crate::CoreType::Func(ty) => Type::Func(ty),
            crate::CoreType::Module(decls) => Type::Module(Self::create_module_type(
                components,
                decls.into_vec(),
                features,
                types,
                offset,
            )?),
        };

        let current = components.last_mut().unwrap();

        if check_limit {
            check_max(current.type_count(), 1, MAX_WASM_TYPES, "types", offset)?;
        }

        current.core_types.push(TypeId {
            type_size: ty.type_size(),
            index: types.len(),
            type_index: Some(current.core_types.len()),
            is_core: true,
        });
        types.push(ty);

        Ok(())
    }

    pub fn add_core_module(
        &mut self,
        module: &Module,
        types: &mut TypeList,
        offset: usize,
    ) -> Result<()> {
        let imports = module.imports_for_module_type(offset)?;

        // We have to clone the module's imports and exports here
        // because we cannot take the data out of the `MaybeOwned`
        // as it might be shared with a function validator.
        let ty = Type::Module(ModuleType {
            type_size: module.type_size,
            imports,
            exports: module.exports.clone(),
        });

        self.core_modules.push(TypeId {
            type_size: ty.type_size(),
            index: types.len(),
            type_index: None,
            is_core: true,
        });

        types.push(ty);

        Ok(())
    }

    pub fn add_core_instance(
        &mut self,
        instance: crate::Instance,
        types: &mut TypeList,
        offset: usize,
    ) -> Result<()> {
        let instance = match instance {
            crate::Instance::Instantiate { module_index, args } => {
                self.instantiate_module(module_index, args.into_vec(), types, offset)?
            }
            crate::Instance::FromExports(exports) => {
                self.instantiate_core_exports(exports.into_vec(), types, offset)?
            }
        };

        self.core_instances.push(instance);

        Ok(())
    }

    pub fn add_type(
        components: &mut Vec<Self>,
        ty: crate::ComponentType,
        features: &WasmFeatures,
        types: &mut TypeList,
        offset: usize,
        check_limit: bool,
    ) -> Result<()> {
        assert!(!components.is_empty());
        let ty = match ty {
            crate::ComponentType::Defined(ty) => Type::Defined(
                components
                    .last_mut()
                    .unwrap()
                    .create_defined_type(ty, types, offset)?,
            ),
            crate::ComponentType::Func(ty) => Type::ComponentFunc(
                components
                    .last_mut()
                    .unwrap()
                    .create_function_type(ty, types, offset)?,
            ),
            crate::ComponentType::Component(decls) => Type::Component(Self::create_component_type(
                components,
                decls.into_vec(),
                features,
                types,
                offset,
            )?),
            crate::ComponentType::Instance(decls) => Type::ComponentInstance(
                Self::create_instance_type(components, decls.into_vec(), features, types, offset)?,
            ),
        };

        let current = components.last_mut().unwrap();
        if check_limit {
            check_max(current.type_count(), 1, MAX_WASM_TYPES, "types", offset)?;
        }

        current.types.push(TypeId {
            type_size: ty.type_size(),
            index: types.len(),
            type_index: Some(current.types.len()),
            is_core: false,
        });
        types.push(ty);

        Ok(())
    }

    pub fn add_import(
        &mut self,
        import: crate::ComponentImport,
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        let entity = self.check_type_ref(&import.ty, types, offset)?;

        let (len, max, desc) = match entity {
            ComponentEntityType::Module(id) => {
                self.core_modules.push(id);
                (self.core_modules.len(), MAX_WASM_MODULES, "modules")
            }
            ComponentEntityType::Component(id) => {
                self.components.push(id);
                (self.components.len(), MAX_WASM_COMPONENTS, "components")
            }
            ComponentEntityType::Instance(id) => {
                self.instances.push(id);
                (self.instance_count(), MAX_WASM_INSTANCES, "instances")
            }
            ComponentEntityType::Func(id) => {
                self.funcs.push(id);
                (self.function_count(), MAX_WASM_FUNCTIONS, "functions")
            }
            ComponentEntityType::Value(ty) => {
                self.values.push((ty, false));
                (self.values.len(), MAX_WASM_VALUES, "values")
            }
            ComponentEntityType::Type(id) => {
                self.types.push(id);
                (self.types.len(), MAX_WASM_TYPES, "types")
            }
        };

        check_max(len, 0, max, desc, offset)?;
        let name = to_kebab_str(import.name, "import", offset)?;

        match self.imports.entry(name.to_owned()) {
            Entry::Occupied(e) => {
                bail!(
                    offset,
                    "import name `{name}` conflicts with previous import name `{prev}`",
                    name = import.name,
                    prev = e.key()
                );
            }
            Entry::Vacant(e) => {
                let url = parse_url(import.url, offset)?;
                if let Some(url) = url.as_ref() {
                    if !self.import_urls.insert(url.to_string()) {
                        bail!(offset, "duplicate import URL `{url}`");
                    }
                }

                self.type_size = combine_type_sizes(self.type_size, entity.type_size(), offset)?;
                e.insert((url, entity));
            }
        }

        Ok(())
    }

    pub fn add_export(
        &mut self,
        name: &str,
        url: &str,
        ty: ComponentEntityType,
        offset: usize,
        check_limit: bool,
    ) -> Result<()> {
        if check_limit {
            check_max(self.exports.len(), 1, MAX_WASM_EXPORTS, "exports", offset)?;
        }

        let name = to_kebab_str(name, "export", offset)?;

        match self.exports.entry(name.to_owned()) {
            Entry::Occupied(e) => {
                bail!(
                    offset,
                    "export name `{name}` conflicts with previous export name `{prev}`",
                    prev = e.key()
                );
            }
            Entry::Vacant(e) => {
                let url = parse_url(url, offset)?;
                if let Some(url) = url.as_ref() {
                    if !self.export_urls.insert(url.to_string()) {
                        bail!(offset, "duplicate export URL `{url}`");
                    }
                }

                self.type_size = combine_type_sizes(self.type_size, ty.type_size(), offset)?;
                e.insert((url, ty));
            }
        }

        Ok(())
    }

    pub fn lift_function(
        &mut self,
        core_func_index: u32,
        type_index: u32,
        options: Vec<CanonicalOption>,
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        let ty = self.function_type_at(type_index, types, offset)?;
        let core_ty = types[self.core_function_at(core_func_index, offset)?]
            .as_func_type()
            .unwrap();

        // Lifting a function is for an export, so match the expected canonical ABI
        // export signature
        let info = ty.lower(types, false);
        self.check_options(Some(core_ty), &info, &options, types, offset)?;

        if core_ty.params() != info.params.as_slice() {
            bail!(
                offset,
                "lowered parameter types `{:?}` do not match parameter types \
                 `{:?}` of core function {core_func_index}",
                info.params.as_slice(),
                core_ty.params(),
            );
        }

        if core_ty.results() != info.results.as_slice() {
            bail!(
                offset,
                "lowered result types `{:?}` do not match result types \
                 `{:?}` of core function {core_func_index}",
                info.results.as_slice(),
                core_ty.results()
            );
        }

        self.funcs.push(self.types[type_index as usize]);

        Ok(())
    }

    pub fn lower_function(
        &mut self,
        func_index: u32,
        options: Vec<CanonicalOption>,
        types: &mut TypeList,
        offset: usize,
    ) -> Result<()> {
        let ty = types[self.function_at(func_index, offset)?]
            .as_component_func_type()
            .unwrap();

        // Lowering a function is for an import, so use a function type that matches
        // the expected canonical ABI import signature.
        let info = ty.lower(types, true);

        self.check_options(None, &info, &options, types, offset)?;

        let lowered_ty = Type::Func(info.into_func_type());

        self.core_funcs.push(TypeId {
            type_size: lowered_ty.type_size(),
            index: types.len(),
            type_index: None,
            is_core: true,
        });

        types.push(lowered_ty);

        Ok(())
    }

    pub fn add_component(&mut self, component: &mut Self, types: &mut TypeList) {
        let ty = Type::Component(ComponentType {
            type_size: component.type_size,
            imports: mem::take(&mut component.imports),
            exports: mem::take(&mut component.exports),
        });

        self.components.push(TypeId {
            type_size: ty.type_size(),
            index: types.len(),
            type_index: None,
            is_core: false,
        });

        types.push(ty);
    }

    pub fn add_instance(
        &mut self,
        instance: crate::ComponentInstance,
        types: &mut TypeList,
        offset: usize,
    ) -> Result<()> {
        let instance = match instance {
            crate::ComponentInstance::Instantiate {
                component_index,
                args,
            } => self.instantiate_component(component_index, args.into_vec(), types, offset)?,
            crate::ComponentInstance::FromExports(exports) => {
                self.instantiate_exports(exports.into_vec(), types, offset)?
            }
        };

        self.instances.push(instance);

        Ok(())
    }

    pub fn add_alias(
        components: &mut [Self],
        alias: crate::ComponentAlias,
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        match alias {
            crate::ComponentAlias::InstanceExport {
                instance_index,
                kind,
                name,
            } => components.last_mut().unwrap().alias_instance_export(
                instance_index,
                kind,
                name,
                types,
                offset,
            ),
            crate::ComponentAlias::CoreInstanceExport {
                instance_index,
                kind,
                name,
            } => components.last_mut().unwrap().alias_core_instance_export(
                instance_index,
                kind,
                name,
                types,
                offset,
            ),
            crate::ComponentAlias::Outer { kind, count, index } => match kind {
                ComponentOuterAliasKind::CoreModule => {
                    Self::alias_module(components, count, index, offset)
                }
                ComponentOuterAliasKind::CoreType => {
                    Self::alias_core_type(components, count, index, offset)
                }
                ComponentOuterAliasKind::Type => Self::alias_type(components, count, index, offset),
                ComponentOuterAliasKind::Component => {
                    Self::alias_component(components, count, index, offset)
                }
            },
        }
    }

    pub fn add_start(
        &mut self,
        func_index: u32,
        args: &[u32],
        results: u32,
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        if self.has_start {
            return Err(BinaryReaderError::new(
                "component cannot have more than one start function",
                offset,
            ));
        }

        let ft = types[self.function_at(func_index, offset)?]
            .as_component_func_type()
            .unwrap();

        if ft.params.len() != args.len() {
            bail!(
                offset,
                "component start function requires {} arguments but was given {}",
                ft.params.len(),
                args.len()
            );
        }

        if ft.results.len() as u32 != results {
            bail!(
                offset,
                "component start function has a result count of {results} \
                 but the function type has a result count of {type_results}",
                type_results = ft.results.len(),
            );
        }

        for (i, ((_, ty), arg)) in ft.params.iter().zip(args).enumerate() {
            // Ensure the value's type is a subtype of the parameter type
            if !ComponentValType::internal_is_subtype_of(
                self.value_at(*arg, offset)?,
                types,
                ty,
                types,
            ) {
                bail!(
                    offset,
                    "value type mismatch for component start function argument {i}"
                );
            }
        }

        for (_, ty) in ft.results.iter() {
            self.values.push((*ty, false));
        }

        self.has_start = true;

        Ok(())
    }

    fn check_options(
        &self,
        core_ty: Option<&FuncType>,
        info: &LoweringInfo,
        options: &[CanonicalOption],
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        fn display(option: CanonicalOption) -> &'static str {
            match option {
                CanonicalOption::UTF8 => "utf8",
                CanonicalOption::UTF16 => "utf16",
                CanonicalOption::CompactUTF16 => "latin1-utf16",
                CanonicalOption::Memory(_) => "memory",
                CanonicalOption::Realloc(_) => "realloc",
                CanonicalOption::PostReturn(_) => "post-return",
            }
        }

        let mut encoding = None;
        let mut memory = None;
        let mut realloc = None;
        let mut post_return = None;

        for option in options {
            match option {
                CanonicalOption::UTF8 | CanonicalOption::UTF16 | CanonicalOption::CompactUTF16 => {
                    match encoding {
                        Some(existing) => {
                            bail!(
                                offset,
                                "canonical encoding option `{}` conflicts with option `{}`",
                                display(existing),
                                display(*option),
                            )
                        }
                        None => encoding = Some(*option),
                    }
                }
                CanonicalOption::Memory(idx) => {
                    memory = match memory {
                        None => {
                            self.memory_at(*idx, offset)?;
                            Some(*idx)
                        }
                        Some(_) => {
                            return Err(BinaryReaderError::new(
                                "canonical option `memory` is specified more than once",
                                offset,
                            ))
                        }
                    }
                }
                CanonicalOption::Realloc(idx) => {
                    realloc = match realloc {
                        None => {
                            let ty = types[self.core_function_at(*idx, offset)?]
                                .as_func_type()
                                .unwrap();
                            if ty.params()
                                != [ValType::I32, ValType::I32, ValType::I32, ValType::I32]
                                || ty.results() != [ValType::I32]
                            {
                                return Err(BinaryReaderError::new(
                                    "canonical option `realloc` uses a core function with an incorrect signature",
                                    offset,
                                ));
                            }
                            Some(*idx)
                        }
                        Some(_) => {
                            return Err(BinaryReaderError::new(
                                "canonical option `realloc` is specified more than once",
                                offset,
                            ))
                        }
                    }
                }
                CanonicalOption::PostReturn(idx) => {
                    post_return = match post_return {
                        None => {
                            let core_ty = core_ty.ok_or_else(|| {
                                BinaryReaderError::new(
                                    "canonical option `post-return` cannot be specified for lowerings",
                                    offset,
                                )
                            })?;

                            let ty = types[self.core_function_at(*idx, offset)?]
                                .as_func_type()
                                .unwrap();

                            if ty.params() != core_ty.results() || !ty.results().is_empty() {
                                return Err(BinaryReaderError::new(
                                    "canonical option `post-return` uses a core function with an incorrect signature",
                                    offset,
                                ));
                            }
                            Some(*idx)
                        }
                        Some(_) => {
                            return Err(BinaryReaderError::new(
                                "canonical option `post-return` is specified more than once",
                                offset,
                            ))
                        }
                    }
                }
            }
        }

        if info.requires_memory && memory.is_none() {
            return Err(BinaryReaderError::new(
                "canonical option `memory` is required",
                offset,
            ));
        }

        if info.requires_realloc && realloc.is_none() {
            return Err(BinaryReaderError::new(
                "canonical option `realloc` is required",
                offset,
            ));
        }

        Ok(())
    }

    pub fn check_type_ref(
        &self,
        ty: &ComponentTypeRef,
        types: &TypeList,
        offset: usize,
    ) -> Result<ComponentEntityType> {
        Ok(match ty {
            ComponentTypeRef::Module(index) => {
                let id = self.type_at(*index, true, offset)?;
                types[id].as_module_type().ok_or_else(|| {
                    format_err!(offset, "core type index {index} is not a module type")
                })?;
                ComponentEntityType::Module(id)
            }
            ComponentTypeRef::Func(index) => {
                let id = self.type_at(*index, false, offset)?;
                types[id].as_component_func_type().ok_or_else(|| {
                    format_err!(offset, "type index {index} is not a function type")
                })?;
                ComponentEntityType::Func(id)
            }
            ComponentTypeRef::Value(ty) => {
                let ty = match ty {
                    crate::ComponentValType::Primitive(ty) => ComponentValType::Primitive(*ty),
                    crate::ComponentValType::Type(index) => {
                        ComponentValType::Type(self.defined_type_at(*index, types, offset)?)
                    }
                };
                ComponentEntityType::Value(ty)
            }
            ComponentTypeRef::Type(TypeBounds::Eq, index) => {
                ComponentEntityType::Type(self.type_at(*index, false, offset)?)
            }
            ComponentTypeRef::Instance(index) => {
                let id = self.type_at(*index, false, offset)?;
                types[id].as_component_instance_type().ok_or_else(|| {
                    format_err!(offset, "type index {index} is not an instance type")
                })?;
                ComponentEntityType::Instance(id)
            }
            ComponentTypeRef::Component(index) => {
                let id = self.type_at(*index, false, offset)?;
                types[id].as_component_type().ok_or_else(|| {
                    format_err!(offset, "type index {index} is not a component type")
                })?;
                ComponentEntityType::Component(id)
            }
        })
    }

    pub fn export_to_entity_type(
        &mut self,
        export: &crate::ComponentExport,
        offset: usize,
    ) -> Result<ComponentEntityType> {
        Ok(match export.kind {
            ComponentExternalKind::Module => {
                ComponentEntityType::Module(self.module_at(export.index, offset)?)
            }
            ComponentExternalKind::Func => {
                ComponentEntityType::Func(self.function_at(export.index, offset)?)
            }
            ComponentExternalKind::Value => {
                ComponentEntityType::Value(*self.value_at(export.index, offset)?)
            }
            ComponentExternalKind::Type => {
                ComponentEntityType::Type(self.type_at(export.index, false, offset)?)
            }
            ComponentExternalKind::Instance => {
                ComponentEntityType::Instance(self.instance_at(export.index, offset)?)
            }
            ComponentExternalKind::Component => {
                ComponentEntityType::Component(self.component_at(export.index, offset)?)
            }
        })
    }

    fn create_module_type(
        components: &[Self],
        decls: Vec<crate::ModuleTypeDeclaration>,
        features: &WasmFeatures,
        types: &mut TypeList,
        offset: usize,
    ) -> Result<ModuleType> {
        let mut state = Module::default();

        for decl in decls {
            match decl {
                crate::ModuleTypeDeclaration::Type(ty) => {
                    state.add_type(ty, features, types, offset, true)?;
                }
                crate::ModuleTypeDeclaration::Export { name, ty } => {
                    let ty = state.check_type_ref(&ty, features, types, offset)?;
                    state.add_export(name, ty, features, offset, true)?;
                }
                crate::ModuleTypeDeclaration::OuterAlias { kind, count, index } => {
                    if count > 1 {
                        return Err(BinaryReaderError::new(
                                    "outer type aliases in module type declarations are limited to a maximum count of 1",
                                    offset,
                                ));
                    }
                    match kind {
                        crate::OuterAliasKind::Type => {
                            let ty = if count == 0 {
                                // Local alias, check the local module state
                                state.type_at(index, offset)?
                            } else {
                                // Otherwise, check the enclosing component state
                                let component =
                                    Self::check_alias_count(components, count - 1, offset)?;
                                component.type_at(index, true, offset)?
                            };

                            check_max(state.types.len(), 1, MAX_WASM_TYPES, "types", offset)?;

                            state.types.push(ty);
                        }
                    }
                }
                crate::ModuleTypeDeclaration::Import(import) => {
                    state.add_import(import, features, types, offset)?;
                }
            }
        }

        let imports = state.imports_for_module_type(offset)?;

        Ok(ModuleType {
            type_size: state.type_size,
            imports,
            exports: state.exports,
        })
    }

    fn create_component_type(
        components: &mut Vec<Self>,
        decls: Vec<crate::ComponentTypeDeclaration>,
        features: &WasmFeatures,
        types: &mut TypeList,
        offset: usize,
    ) -> Result<ComponentType> {
        components.push(ComponentState::default());

        for decl in decls {
            match decl {
                crate::ComponentTypeDeclaration::CoreType(ty) => {
                    Self::add_core_type(components, ty, features, types, offset, true)?;
                }
                crate::ComponentTypeDeclaration::Type(ty) => {
                    Self::add_type(components, ty, features, types, offset, true)?;
                }
                crate::ComponentTypeDeclaration::Export { name, url, ty } => {
                    let current = components.last_mut().unwrap();
                    let ty = current.check_type_ref(&ty, types, offset)?;
                    current.add_export(name, url, ty, offset, true)?;
                }
                crate::ComponentTypeDeclaration::Import(import) => {
                    components
                        .last_mut()
                        .unwrap()
                        .add_import(import, types, offset)?;
                }
                crate::ComponentTypeDeclaration::Alias(alias) => {
                    match alias {
                        crate::ComponentAlias::Outer { kind, count, index }
                            if kind == ComponentOuterAliasKind::CoreType
                                || kind == ComponentOuterAliasKind::Type =>
                        {
                            match kind {
                                ComponentOuterAliasKind::CoreType => {
                                    Self::alias_core_type(components, count, index, offset)?
                                }
                                ComponentOuterAliasKind::Type => {
                                    Self::alias_type(components, count, index, offset)?
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => return Err(BinaryReaderError::new(
                            "only outer type aliases are allowed in component type declarations",
                            offset,
                        )),
                    }
                }
            };
        }

        let state = components.pop().unwrap();

        Ok(ComponentType {
            type_size: state.type_size,
            imports: state.imports,
            exports: state.exports,
        })
    }

    fn create_instance_type(
        components: &mut Vec<Self>,
        decls: Vec<crate::InstanceTypeDeclaration>,
        features: &WasmFeatures,
        types: &mut TypeList,
        offset: usize,
    ) -> Result<ComponentInstanceType> {
        components.push(ComponentState::default());

        for decl in decls {
            match decl {
                crate::InstanceTypeDeclaration::CoreType(ty) => {
                    Self::add_core_type(components, ty, features, types, offset, true)?;
                }
                crate::InstanceTypeDeclaration::Type(ty) => {
                    Self::add_type(components, ty, features, types, offset, true)?;
                }
                crate::InstanceTypeDeclaration::Export { name, url, ty } => {
                    let current = components.last_mut().unwrap();
                    let ty = current.check_type_ref(&ty, types, offset)?;
                    current.add_export(name, url, ty, offset, true)?;
                }
                crate::InstanceTypeDeclaration::Alias(alias) => match alias {
                    crate::ComponentAlias::Outer { kind, count, index }
                        if kind == ComponentOuterAliasKind::CoreType
                            || kind == ComponentOuterAliasKind::Type =>
                    {
                        match kind {
                            ComponentOuterAliasKind::CoreType => {
                                Self::alias_core_type(components, count, index, offset)?
                            }
                            ComponentOuterAliasKind::Type => {
                                Self::alias_type(components, count, index, offset)?
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => {
                        return Err(BinaryReaderError::new(
                            "only outer type aliases are allowed in instance type declarations",
                            offset,
                        ))
                    }
                },
            };
        }

        let state = components.pop().unwrap();

        Ok(ComponentInstanceType {
            type_size: state.type_size,
            kind: ComponentInstanceTypeKind::Defined(state.exports),
        })
    }

    fn create_function_type(
        &self,
        ty: crate::ComponentFuncType,
        types: &TypeList,
        offset: usize,
    ) -> Result<ComponentFuncType> {
        let mut type_size = 1;

        let mut set =
            HashSet::with_capacity(std::cmp::max(ty.params.len(), ty.results.type_count()));

        let params = ty
            .params
            .iter()
            .map(|(name, ty)| {
                let name = to_kebab_str(name, "function parameter", offset)?;
                if !set.insert(name) {
                    bail!(
                        offset,
                        "function parameter name `{name}` conflicts with previous parameter name `{prev}`",
                        prev = set.get(&name).unwrap(),
                    );
                }

                let ty = self.create_component_val_type(*ty, types, offset)?;
                type_size = combine_type_sizes(type_size, ty.type_size(), offset)?;
                Ok((name.to_owned(), ty))
            })
            .collect::<Result<_>>()?;

        set.clear();

        let results = ty
            .results
            .iter()
            .map(|(name, ty)| {
                let name = name
                    .map(|name| {
                        let name = to_kebab_str(name, "function result", offset)?;
                        if !set.insert(name) {
                            bail!(
                                offset,
                                "function result name `{name}` conflicts with previous result name `{prev}`",
                                prev = set.get(name).unwrap(),
                            );
                        }

                        Ok(name.to_owned())
                    })
                    .transpose()?;

                let ty = self.create_component_val_type(*ty, types, offset)?;
                type_size = combine_type_sizes(type_size, ty.type_size(), offset)?;
                Ok((name, ty))
            })
            .collect::<Result<_>>()?;

        Ok(ComponentFuncType {
            type_size,
            params,
            results,
        })
    }

    fn instantiate_module(
        &self,
        module_index: u32,
        module_args: Vec<crate::InstantiationArg>,
        types: &mut TypeList,
        offset: usize,
    ) -> Result<TypeId> {
        fn insert_arg<'a>(
            name: &'a str,
            arg: &'a InstanceType,
            args: &mut IndexMap<&'a str, &'a InstanceType>,
            offset: usize,
        ) -> Result<()> {
            if args.insert(name, arg).is_some() {
                bail!(
                    offset,
                    "duplicate module instantiation argument named `{name}`"
                );
            }

            Ok(())
        }

        let module_type_id = self.module_at(module_index, offset)?;
        let mut args = IndexMap::new();

        // Populate the arguments
        for module_arg in module_args {
            match module_arg.kind {
                InstantiationArgKind::Instance => {
                    let instance_type = types[self.core_instance_at(module_arg.index, offset)?]
                        .as_instance_type()
                        .unwrap();
                    insert_arg(module_arg.name, instance_type, &mut args, offset)?;
                }
            }
        }

        // Validate the arguments
        let module_type = types[module_type_id].as_module_type().unwrap();
        for ((module, name), expected) in module_type.imports.iter() {
            let instance = args.get(module.as_str()).ok_or_else(|| {
                format_err!(
                    offset,
                    "missing module instantiation argument named `{module}`"
                )
            })?;

            let arg = instance
                .internal_exports(types)
                .get(name.as_str())
                .ok_or_else(|| {
                    format_err!(
                        offset,
                        "module instantiation argument `{module}` does not \
                         export an item named `{name}`",
                    )
                })?;

            match (arg, expected) {
                (EntityType::Func(_), EntityType::Func(_))
                | (EntityType::Table(_), EntityType::Table(_))
                | (EntityType::Memory(_), EntityType::Memory(_))
                | (EntityType::Global(_), EntityType::Global(_))
                | (EntityType::Tag(_), EntityType::Tag(_)) => {}
                _ => {
                    bail!(
                        offset,
                        "module instantiation argument `{module}` exports \
                         an item named `{name}` but it is not a {}",
                        expected.desc()
                    )
                }
            }

            if !EntityType::internal_is_subtype_of(arg, types, expected, types) {
                bail!(
                    offset,
                    "{} type mismatch for export `{name}` of module \
                     instantiation argument `{module}`",
                    expected.desc(),
                );
            }
        }

        let ty = Type::Instance(InstanceType {
            type_size: module_type
                .exports
                .iter()
                .fold(1, |acc, (_, ty)| acc + ty.type_size()),
            kind: InstanceTypeKind::Instantiated(module_type_id),
        });

        let id = TypeId {
            type_size: ty.type_size(),
            index: types.len(),
            type_index: None,
            is_core: true,
        };

        types.push(ty);

        Ok(id)
    }

    fn instantiate_component(
        &mut self,
        component_index: u32,
        component_args: Vec<crate::ComponentInstantiationArg>,
        types: &mut TypeList,
        offset: usize,
    ) -> Result<TypeId> {
        fn insert_arg<'a>(
            name: &'a str,
            arg: ComponentEntityType,
            args: &mut IndexMap<&'a KebabStr, ComponentEntityType>,
            offset: usize,
        ) -> Result<()> {
            let name = to_kebab_str(name, "instantiation argument", offset)?;
            match args.entry(name) {
                Entry::Occupied(e) => {
                    bail!(
                        offset,
                        "instantiation argument `{name}` conflicts with previous argument `{prev}`",
                        prev = e.key()
                    );
                }
                Entry::Vacant(e) => {
                    e.insert(arg);
                }
            }

            Ok(())
        }

        let component_type_id = self.component_at(component_index, offset)?;
        let mut args = IndexMap::new();

        // Populate the arguments
        for component_arg in component_args {
            match component_arg.kind {
                ComponentExternalKind::Module => {
                    insert_arg(
                        component_arg.name,
                        ComponentEntityType::Module(self.module_at(component_arg.index, offset)?),
                        &mut args,
                        offset,
                    )?;
                }
                ComponentExternalKind::Component => {
                    insert_arg(
                        component_arg.name,
                        ComponentEntityType::Component(
                            self.component_at(component_arg.index, offset)?,
                        ),
                        &mut args,
                        offset,
                    )?;
                }
                ComponentExternalKind::Instance => {
                    insert_arg(
                        component_arg.name,
                        ComponentEntityType::Instance(
                            self.instance_at(component_arg.index, offset)?,
                        ),
                        &mut args,
                        offset,
                    )?;
                }
                ComponentExternalKind::Func => {
                    insert_arg(
                        component_arg.name,
                        ComponentEntityType::Func(self.function_at(component_arg.index, offset)?),
                        &mut args,
                        offset,
                    )?;
                }
                ComponentExternalKind::Value => {
                    insert_arg(
                        component_arg.name,
                        ComponentEntityType::Value(*self.value_at(component_arg.index, offset)?),
                        &mut args,
                        offset,
                    )?;
                }
                ComponentExternalKind::Type => {
                    // Type arguments are ignored
                }
            }
        }

        // Validate the arguments
        let component_type = types[component_type_id].as_component_type().unwrap();
        for (name, (_, expected)) in component_type.imports.iter() {
            match args.get(&name.as_kebab_str()) {
                Some(arg) => {
                    match (arg, expected) {
                        (ComponentEntityType::Module(_), ComponentEntityType::Module(_))
                        | (ComponentEntityType::Component(_), ComponentEntityType::Component(_))
                        | (ComponentEntityType::Instance(_), ComponentEntityType::Instance(_))
                        | (ComponentEntityType::Func(_), ComponentEntityType::Func(_))
                        | (ComponentEntityType::Value(_), ComponentEntityType::Value(_)) => {}
                        _ => {
                            bail!(
                                offset,
                                "expected component instantiation argument \
                                 `{name}` to be of type `{}`",
                                expected.desc()
                            )
                        }
                    };

                    if !ComponentEntityType::internal_is_subtype_of(arg, types, expected, types) {
                        bail!(
                            offset,
                            "{} type mismatch for component instantiation argument `{name}`",
                            expected.desc(),
                        );
                    }
                }
                None => {
                    bail!(
                        offset,
                        "missing component instantiation argument named `{name}`"
                    );
                }
            }
        }

        let ty = Type::ComponentInstance(ComponentInstanceType {
            type_size: component_type
                .exports
                .iter()
                .fold(1, |acc, (_, (_, ty))| acc + ty.type_size()),
            kind: ComponentInstanceTypeKind::Instantiated(component_type_id),
        });

        let id = TypeId {
            type_size: ty.type_size(),
            index: types.len(),
            type_index: None,
            is_core: false,
        };

        types.push(ty);

        Ok(id)
    }

    fn instantiate_exports(
        &mut self,
        exports: Vec<crate::ComponentExport>,
        types: &mut TypeList,
        offset: usize,
    ) -> Result<TypeId> {
        fn insert_export(
            name: &str,
            export: ComponentEntityType,
            exports: &mut IndexMap<KebabString, (Option<Url>, ComponentEntityType)>,
            type_size: &mut usize,
            offset: usize,
        ) -> Result<()> {
            let name = to_kebab_str(name, "instance export", offset)?;
            match exports.entry(name.to_owned()) {
                Entry::Occupied(e) => bail!(
                    offset,
                    "instance export name `{name}` conflicts with previous export name `{prev}`",
                    prev = e.key()
                ),
                Entry::Vacant(e) => {
                    *type_size = combine_type_sizes(*type_size, export.type_size(), offset)?;
                    e.insert((None, export));
                }
            }

            Ok(())
        }

        let mut type_size = 1;
        let mut inst_exports = IndexMap::new();
        for export in exports {
            match export.kind {
                ComponentExternalKind::Module => {
                    insert_export(
                        export.name,
                        ComponentEntityType::Module(self.module_at(export.index, offset)?),
                        &mut inst_exports,
                        &mut type_size,
                        offset,
                    )?;
                }
                ComponentExternalKind::Component => {
                    insert_export(
                        export.name,
                        ComponentEntityType::Component(self.component_at(export.index, offset)?),
                        &mut inst_exports,
                        &mut type_size,
                        offset,
                    )?;
                }
                ComponentExternalKind::Instance => {
                    insert_export(
                        export.name,
                        ComponentEntityType::Instance(self.instance_at(export.index, offset)?),
                        &mut inst_exports,
                        &mut type_size,
                        offset,
                    )?;
                }
                ComponentExternalKind::Func => {
                    insert_export(
                        export.name,
                        ComponentEntityType::Func(self.function_at(export.index, offset)?),
                        &mut inst_exports,
                        &mut type_size,
                        offset,
                    )?;
                }
                ComponentExternalKind::Value => {
                    insert_export(
                        export.name,
                        ComponentEntityType::Value(*self.value_at(export.index, offset)?),
                        &mut inst_exports,
                        &mut type_size,
                        offset,
                    )?;
                }
                ComponentExternalKind::Type => {
                    insert_export(
                        export.name,
                        ComponentEntityType::Type(self.type_at(export.index, false, offset)?),
                        &mut inst_exports,
                        &mut type_size,
                        offset,
                    )?;
                }
            }
        }

        let ty = Type::ComponentInstance(ComponentInstanceType {
            type_size,
            kind: ComponentInstanceTypeKind::Exports(inst_exports),
        });

        let id = TypeId {
            type_size: ty.type_size(),
            index: types.len(),
            type_index: None,
            is_core: false,
        };

        types.push(ty);

        Ok(id)
    }

    fn instantiate_core_exports(
        &mut self,
        exports: Vec<crate::Export>,
        types: &mut TypeList,
        offset: usize,
    ) -> Result<TypeId> {
        fn insert_export(
            name: &str,
            export: EntityType,
            exports: &mut IndexMap<String, EntityType>,
            type_size: &mut usize,
            offset: usize,
        ) -> Result<()> {
            *type_size = combine_type_sizes(*type_size, export.type_size(), offset)?;

            if exports.insert(name.to_string(), export).is_some() {
                bail!(
                    offset,
                    "duplicate instantiation export name `{name}` already defined",
                )
            }

            Ok(())
        }

        let mut type_size = 1;
        let mut inst_exports = IndexMap::new();
        for export in exports {
            match export.kind {
                ExternalKind::Func => {
                    insert_export(
                        export.name,
                        EntityType::Func(self.core_function_at(export.index, offset)?),
                        &mut inst_exports,
                        &mut type_size,
                        offset,
                    )?;
                }
                ExternalKind::Table => insert_export(
                    export.name,
                    EntityType::Table(*self.table_at(export.index, offset)?),
                    &mut inst_exports,
                    &mut type_size,
                    offset,
                )?,
                ExternalKind::Memory => insert_export(
                    export.name,
                    EntityType::Memory(*self.memory_at(export.index, offset)?),
                    &mut inst_exports,
                    &mut type_size,
                    offset,
                )?,
                ExternalKind::Global => {
                    insert_export(
                        export.name,
                        EntityType::Global(*self.global_at(export.index, offset)?),
                        &mut inst_exports,
                        &mut type_size,
                        offset,
                    )?;
                }
                ExternalKind::Tag => insert_export(
                    export.name,
                    EntityType::Tag(self.core_function_at(export.index, offset)?),
                    &mut inst_exports,
                    &mut type_size,
                    offset,
                )?,
            }
        }

        let ty = Type::Instance(InstanceType {
            type_size,
            kind: InstanceTypeKind::Exports(inst_exports),
        });

        let id = TypeId {
            type_size: ty.type_size(),
            index: types.len(),
            type_index: None,
            is_core: true,
        };

        types.push(ty);

        Ok(id)
    }

    fn alias_core_instance_export(
        &mut self,
        instance_index: u32,
        kind: ExternalKind,
        name: &str,
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        macro_rules! push_module_export {
            ($expected:path, $collection:ident, $ty:literal) => {{
                match self.core_instance_export(instance_index, name, types, offset)? {
                    $expected(ty) => {
                        self.$collection.push(*ty);
                        Ok(())
                    }
                    _ => {
                        bail!(
                            offset,
                            "export `{name}` for core instance {instance_index} is not a {}",
                            $ty
                        )
                    }
                }
            }};
        }

        match kind {
            ExternalKind::Func => {
                check_max(
                    self.function_count(),
                    1,
                    MAX_WASM_FUNCTIONS,
                    "functions",
                    offset,
                )?;
                push_module_export!(EntityType::Func, core_funcs, "function")
            }
            ExternalKind::Table => {
                check_max(self.core_tables.len(), 1, MAX_WASM_TABLES, "tables", offset)?;
                push_module_export!(EntityType::Table, core_tables, "table")
            }
            ExternalKind::Memory => {
                check_max(
                    self.core_memories.len(),
                    1,
                    MAX_WASM_MEMORIES,
                    "memories",
                    offset,
                )?;
                push_module_export!(EntityType::Memory, core_memories, "memory")
            }
            ExternalKind::Global => {
                check_max(
                    self.core_globals.len(),
                    1,
                    MAX_WASM_GLOBALS,
                    "globals",
                    offset,
                )?;
                push_module_export!(EntityType::Global, core_globals, "global")
            }
            ExternalKind::Tag => {
                check_max(self.core_tags.len(), 1, MAX_WASM_TAGS, "tags", offset)?;
                push_module_export!(EntityType::Tag, core_tags, "tag")
            }
        }
    }

    fn alias_instance_export(
        &mut self,
        instance_index: u32,
        kind: ComponentExternalKind,
        name: &str,
        types: &TypeList,
        offset: usize,
    ) -> Result<()> {
        let name = to_kebab_str(name, "alias export", offset)?;

        macro_rules! push_component_export {
            ($expected:path, $collection:ident, $ty:literal) => {{
                match self.instance_export(instance_index, name, types, offset)? {
                    $expected(ty) => {
                        self.$collection.push(*ty);
                        Ok(())
                    }
                    _ => {
                        bail!(
                            offset,
                            "export `{name}` for instance {instance_index} is not a {}",
                            $ty
                        )
                    }
                }
            }};
        }

        match kind {
            ComponentExternalKind::Module => {
                check_max(
                    self.core_modules.len(),
                    1,
                    MAX_WASM_MODULES,
                    "modules",
                    offset,
                )?;
                push_component_export!(ComponentEntityType::Module, core_modules, "module")
            }
            ComponentExternalKind::Component => {
                check_max(
                    self.components.len(),
                    1,
                    MAX_WASM_COMPONENTS,
                    "components",
                    offset,
                )?;
                push_component_export!(ComponentEntityType::Component, components, "component")
            }
            ComponentExternalKind::Instance => {
                check_max(
                    self.instance_count(),
                    1,
                    MAX_WASM_INSTANCES,
                    "instances",
                    offset,
                )?;
                push_component_export!(ComponentEntityType::Instance, instances, "instance")
            }
            ComponentExternalKind::Func => {
                check_max(
                    self.function_count(),
                    1,
                    MAX_WASM_FUNCTIONS,
                    "functions",
                    offset,
                )?;
                push_component_export!(ComponentEntityType::Func, funcs, "function")
            }
            ComponentExternalKind::Value => {
                check_max(self.values.len(), 1, MAX_WASM_VALUES, "values", offset)?;
                match self.instance_export(instance_index, name, types, offset)? {
                    ComponentEntityType::Value(ty) => {
                        self.values.push((*ty, false));
                        Ok(())
                    }
                    _ => bail!(
                        offset,
                        "export `{name}` for instance {instance_index} is not a value",
                    ),
                }
            }
            ComponentExternalKind::Type => {
                check_max(self.type_count(), 1, MAX_WASM_TYPES, "types", offset)?;
                push_component_export!(ComponentEntityType::Type, types, "type")
            }
        }
    }

    fn alias_module(components: &mut [Self], count: u32, index: u32, offset: usize) -> Result<()> {
        let component = Self::check_alias_count(components, count, offset)?;
        let ty = component.module_at(index, offset)?;

        let current = components.last_mut().unwrap();
        check_max(
            current.core_modules.len(),
            1,
            MAX_WASM_MODULES,
            "modules",
            offset,
        )?;

        current.core_modules.push(ty);
        Ok(())
    }

    fn alias_component(
        components: &mut [Self],
        count: u32,
        index: u32,
        offset: usize,
    ) -> Result<()> {
        let component = Self::check_alias_count(components, count, offset)?;
        let ty = component.component_at(index, offset)?;

        let current = components.last_mut().unwrap();
        check_max(
            current.components.len(),
            1,
            MAX_WASM_COMPONENTS,
            "components",
            offset,
        )?;

        current.components.push(ty);
        Ok(())
    }

    fn alias_core_type(
        components: &mut [Self],
        count: u32,
        index: u32,
        offset: usize,
    ) -> Result<()> {
        let component = Self::check_alias_count(components, count, offset)?;
        let ty = component.type_at(index, true, offset)?;

        let current = components.last_mut().unwrap();
        check_max(current.type_count(), 1, MAX_WASM_TYPES, "types", offset)?;

        current.core_types.push(TypeId {
            type_size: ty.type_size,
            index: ty.index,
            type_index: Some(current.core_types.len()),
            is_core: true,
        });

        Ok(())
    }

    fn alias_type(components: &mut [Self], count: u32, index: u32, offset: usize) -> Result<()> {
        let component = Self::check_alias_count(components, count, offset)?;
        let ty = component.type_at(index, false, offset)?;

        let current = components.last_mut().unwrap();
        check_max(current.type_count(), 1, MAX_WASM_TYPES, "types", offset)?;

        current.types.push(TypeId {
            type_size: ty.type_size,
            index: ty.index,
            type_index: Some(current.types.len()),
            is_core: false,
        });

        Ok(())
    }

    fn check_alias_count(components: &[Self], count: u32, offset: usize) -> Result<&Self> {
        let count = count as usize;
        if count >= components.len() {
            bail!(offset, "invalid outer alias count of {count}");
        }

        Ok(&components[components.len() - count - 1])
    }

    fn create_defined_type(
        &self,
        ty: crate::ComponentDefinedType,
        types: &TypeList,
        offset: usize,
    ) -> Result<ComponentDefinedType> {
        match ty {
            crate::ComponentDefinedType::Primitive(ty) => Ok(ComponentDefinedType::Primitive(ty)),
            crate::ComponentDefinedType::Record(fields) => {
                self.create_record_type(fields.as_ref(), types, offset)
            }
            crate::ComponentDefinedType::Variant(cases) => {
                self.create_variant_type(cases.as_ref(), types, offset)
            }
            crate::ComponentDefinedType::List(ty) => Ok(ComponentDefinedType::List(
                self.create_component_val_type(ty, types, offset)?,
            )),
            crate::ComponentDefinedType::Tuple(tys) => {
                self.create_tuple_type(tys.as_ref(), types, offset)
            }
            crate::ComponentDefinedType::Flags(names) => {
                self.create_flags_type(names.as_ref(), offset)
            }
            crate::ComponentDefinedType::Enum(cases) => {
                self.create_enum_type(cases.as_ref(), offset)
            }
            crate::ComponentDefinedType::Union(tys) => {
                self.create_union_type(tys.as_ref(), types, offset)
            }
            crate::ComponentDefinedType::Option(ty) => Ok(ComponentDefinedType::Option(
                self.create_component_val_type(ty, types, offset)?,
            )),
            crate::ComponentDefinedType::Result { ok, err } => Ok(ComponentDefinedType::Result {
                ok: ok
                    .map(|ty| self.create_component_val_type(ty, types, offset))
                    .transpose()?,
                err: err
                    .map(|ty| self.create_component_val_type(ty, types, offset))
                    .transpose()?,
            }),
        }
    }

    fn create_record_type(
        &self,
        fields: &[(&str, crate::ComponentValType)],
        types: &TypeList,
        offset: usize,
    ) -> Result<ComponentDefinedType> {
        let mut type_size = 1;
        let mut field_map = IndexMap::with_capacity(fields.len());

        for (name, ty) in fields {
            let name = to_kebab_str(name, "record field", offset)?;
            let ty = self.create_component_val_type(*ty, types, offset)?;

            match field_map.entry(name.to_owned()) {
                Entry::Occupied(e) => bail!(
                    offset,
                    "record field name `{name}` conflicts with previous field name `{prev}`",
                    prev = e.key()
                ),
                Entry::Vacant(e) => {
                    type_size = combine_type_sizes(type_size, ty.type_size(), offset)?;
                    e.insert(ty);
                }
            }
        }

        Ok(ComponentDefinedType::Record(RecordType {
            type_size,
            fields: field_map,
        }))
    }

    fn create_variant_type(
        &self,
        cases: &[crate::VariantCase],
        types: &TypeList,
        offset: usize,
    ) -> Result<ComponentDefinedType> {
        let mut type_size = 1;
        let mut case_map: IndexMap<KebabString, VariantCase> = IndexMap::with_capacity(cases.len());

        if cases.is_empty() {
            return Err(BinaryReaderError::new(
                "variant type must have at least one case",
                offset,
            ));
        }

        if cases.len() > u32::MAX as usize {
            return Err(BinaryReaderError::new(
                "variant type cannot be represented with a 32-bit discriminant value",
                offset,
            ));
        }

        for (i, case) in cases.iter().enumerate() {
            if let Some(refines) = case.refines {
                if refines >= i as u32 {
                    return Err(BinaryReaderError::new(
                        "variant case can only refine a previously defined case",
                        offset,
                    ));
                }
            }

            let name = to_kebab_str(case.name, "variant case", offset)?;

            let ty = case
                .ty
                .map(|ty| self.create_component_val_type(ty, types, offset))
                .transpose()?;

            match case_map.entry(name.to_owned()) {
                Entry::Occupied(e) => bail!(
                    offset,
                    "variant case name `{name}` conflicts with previous case name `{prev}`",
                    name = case.name,
                    prev = e.key()
                ),
                Entry::Vacant(e) => {
                    type_size = combine_type_sizes(
                        type_size,
                        ty.map(|ty| ty.type_size()).unwrap_or(1),
                        offset,
                    )?;

                    // Safety: the use of `KebabStr::new_unchecked` here is safe because the string
                    // was already verified to be kebab case.
                    e.insert(VariantCase {
                        ty,
                        refines: case
                            .refines
                            .map(|i| KebabStr::new_unchecked(cases[i as usize].name).to_owned()),
                    });
                }
            }
        }

        Ok(ComponentDefinedType::Variant(VariantType {
            type_size,
            cases: case_map,
        }))
    }

    fn create_tuple_type(
        &self,
        tys: &[crate::ComponentValType],
        types: &TypeList,
        offset: usize,
    ) -> Result<ComponentDefinedType> {
        let mut type_size = 1;
        let types = tys
            .iter()
            .map(|ty| {
                let ty = self.create_component_val_type(*ty, types, offset)?;
                type_size = combine_type_sizes(type_size, ty.type_size(), offset)?;
                Ok(ty)
            })
            .collect::<Result<_>>()?;

        Ok(ComponentDefinedType::Tuple(TupleType { type_size, types }))
    }

    fn create_flags_type(&self, names: &[&str], offset: usize) -> Result<ComponentDefinedType> {
        let mut names_set = IndexSet::with_capacity(names.len());

        for name in names {
            let name = to_kebab_str(name, "flag", offset)?;
            if !names_set.insert(name.to_owned()) {
                bail!(
                    offset,
                    "flag name `{name}` conflicts with previous flag name `{prev}`",
                    prev = names_set.get(name).unwrap()
                );
            }
        }

        Ok(ComponentDefinedType::Flags(names_set))
    }

    fn create_enum_type(&self, cases: &[&str], offset: usize) -> Result<ComponentDefinedType> {
        if cases.len() > u32::MAX as usize {
            return Err(BinaryReaderError::new(
                "enumeration type cannot be represented with a 32-bit discriminant value",
                offset,
            ));
        }

        let mut tags = IndexSet::with_capacity(cases.len());

        for tag in cases {
            let tag = to_kebab_str(tag, "enum tag", offset)?;
            if !tags.insert(tag.to_owned()) {
                bail!(
                    offset,
                    "enum tag name `{tag}` conflicts with previous tag name `{prev}`",
                    prev = tags.get(tag).unwrap()
                );
            }
        }

        Ok(ComponentDefinedType::Enum(tags))
    }

    fn create_union_type(
        &self,
        tys: &[crate::ComponentValType],
        types: &TypeList,
        offset: usize,
    ) -> Result<ComponentDefinedType> {
        let mut type_size = 1;
        let types = tys
            .iter()
            .map(|ty| {
                let ty = self.create_component_val_type(*ty, types, offset)?;
                type_size = combine_type_sizes(type_size, ty.type_size(), offset)?;
                Ok(ty)
            })
            .collect::<Result<_>>()?;

        Ok(ComponentDefinedType::Union(UnionType { type_size, types }))
    }

    fn create_component_val_type(
        &self,
        ty: crate::ComponentValType,
        types: &TypeList,
        offset: usize,
    ) -> Result<ComponentValType> {
        Ok(match ty {
            crate::ComponentValType::Primitive(pt) => ComponentValType::Primitive(pt),
            crate::ComponentValType::Type(idx) => {
                ComponentValType::Type(self.defined_type_at(idx, types, offset)?)
            }
        })
    }

    pub fn type_at(&self, idx: u32, core: bool, offset: usize) -> Result<TypeId> {
        let types = if core { &self.core_types } else { &self.types };
        types
            .get(idx as usize)
            .copied()
            .ok_or_else(|| format_err!(offset, "unknown type {idx}: type index out of bounds"))
    }

    fn function_type_at<'a>(
        &self,
        idx: u32,
        types: &'a TypeList,
        offset: usize,
    ) -> Result<&'a ComponentFuncType> {
        types[self.type_at(idx, false, offset)?]
            .as_component_func_type()
            .ok_or_else(|| format_err!(offset, "type index {idx} is not a function type"))
    }

    fn function_at(&self, idx: u32, offset: usize) -> Result<TypeId> {
        self.funcs.get(idx as usize).copied().ok_or_else(|| {
            format_err!(
                offset,
                "unknown function {idx}: function index out of bounds"
            )
        })
    }

    fn component_at(&self, idx: u32, offset: usize) -> Result<TypeId> {
        self.components.get(idx as usize).copied().ok_or_else(|| {
            format_err!(
                offset,
                "unknown component {idx}: component index out of bounds"
            )
        })
    }

    fn instance_at(&self, idx: u32, offset: usize) -> Result<TypeId> {
        self.instances.get(idx as usize).copied().ok_or_else(|| {
            format_err!(
                offset,
                "unknown instance {idx}: instance index out of bounds"
            )
        })
    }

    fn instance_export<'a>(
        &self,
        instance_index: u32,
        name: &KebabStr,
        types: &'a TypeList,
        offset: usize,
    ) -> Result<&'a ComponentEntityType> {
        match types[self.instance_at(instance_index, offset)?]
            .as_component_instance_type()
            .unwrap()
            .internal_exports(types)
            .get(name)
        {
            Some((_, ty)) => Ok(ty),
            None => bail!(
                offset,
                "instance {instance_index} has no export named `{name}`"
            ),
        }
    }

    fn value_at(&mut self, idx: u32, offset: usize) -> Result<&ComponentValType> {
        match self.values.get_mut(idx as usize) {
            Some((ty, used)) if !*used => {
                *used = true;
                Ok(ty)
            }
            Some(_) => bail!(offset, "value {idx} cannot be used more than once"),
            None => bail!(offset, "unknown value {idx}: value index out of bounds"),
        }
    }

    fn defined_type_at(&self, idx: u32, types: &TypeList, offset: usize) -> Result<TypeId> {
        let id = self.type_at(idx, false, offset)?;
        match &types[id] {
            Type::Defined(_) => Ok(id),
            _ => bail!(offset, "type index {} is not a defined type", idx),
        }
    }

    fn core_function_at(&self, idx: u32, offset: usize) -> Result<TypeId> {
        match self.core_funcs.get(idx as usize) {
            Some(id) => Ok(*id),
            None => bail!(
                offset,
                "unknown core function {idx}: function index out of bounds"
            ),
        }
    }

    fn module_at(&self, idx: u32, offset: usize) -> Result<TypeId> {
        match self.core_modules.get(idx as usize) {
            Some(id) => Ok(*id),
            None => bail!(offset, "unknown module {idx}: module index out of bounds"),
        }
    }

    fn core_instance_at(&self, idx: u32, offset: usize) -> Result<TypeId> {
        match self.core_instances.get(idx as usize) {
            Some(id) => Ok(*id),
            None => bail!(
                offset,
                "unknown core instance {idx}: instance index out of bounds"
            ),
        }
    }

    fn core_instance_export<'a>(
        &self,
        instance_index: u32,
        name: &str,
        types: &'a TypeList,
        offset: usize,
    ) -> Result<&'a EntityType> {
        match types[self.core_instance_at(instance_index, offset)?]
            .as_instance_type()
            .unwrap()
            .internal_exports(types)
            .get(name)
        {
            Some(export) => Ok(export),
            None => bail!(
                offset,
                "core instance {instance_index} has no export named `{name}`"
            ),
        }
    }

    fn global_at(&self, idx: u32, offset: usize) -> Result<&GlobalType> {
        match self.core_globals.get(idx as usize) {
            Some(t) => Ok(t),
            None => bail!(offset, "unknown global {idx}: global index out of bounds"),
        }
    }

    fn table_at(&self, idx: u32, offset: usize) -> Result<&TableType> {
        match self.core_tables.get(idx as usize) {
            Some(t) => Ok(t),
            None => bail!(offset, "unknown table {idx}: table index out of bounds"),
        }
    }

    fn memory_at(&self, idx: u32, offset: usize) -> Result<&MemoryType> {
        match self.core_memories.get(idx as usize) {
            Some(t) => Ok(t),
            None => bail!(offset, "unknown memory {idx}: memory index out of bounds"),
        }
    }
}

impl Default for ComponentState {
    fn default() -> Self {
        Self {
            core_types: Default::default(),
            core_modules: Default::default(),
            core_instances: Default::default(),
            core_funcs: Default::default(),
            core_memories: Default::default(),
            core_tables: Default::default(),
            core_globals: Default::default(),
            core_tags: Default::default(),
            types: Default::default(),
            funcs: Default::default(),
            values: Default::default(),
            instances: Default::default(),
            components: Default::default(),
            imports: Default::default(),
            exports: Default::default(),
            import_urls: Default::default(),
            export_urls: Default::default(),
            has_start: Default::default(),
            type_size: 1,
        }
    }
}
