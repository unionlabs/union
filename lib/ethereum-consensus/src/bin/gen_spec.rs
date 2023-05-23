use quote::{format_ident, ToTokens};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs::{self};

use std::mem;
use std::path::{Path, PathBuf};

use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{
    parse_quote, punctuated::Punctuated, token::Comma, ConstParam, FnArg, GenericArgument,
    GenericParam, Ident, Item, ItemConst, ItemFn, ItemMod, ItemUse, PathArguments, PathSegment,
    Signature, Type, UseTree,
};

const ATTESTATION_BOUND_IDENT: &str = "PENDING_ATTESTATIONS_BOUND";
const SYNC_COMMITTEE_SIZE_IDENT: &str = "SYNC_COMMITTEE_SIZE";

const BELLATRIX_BEACON_STATE_BOUNDS: [&str; 4] = [
    "BYTES_PER_LOGS_BLOOM",
    "MAX_EXTRA_DATA_BYTES",
    "MAX_BYTES_PER_TRANSACTION",
    "MAX_TRANSACTIONS_PER_PAYLOAD",
];

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct UseDecl {
    node: ItemUse,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct ConstDecl {
    node: ItemConst,
    item_index: usize,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct FnDecl {
    node: ItemFn,
    item_index: usize,
}

#[derive(Debug)]
struct ImportEditor {
    all_imports: HashMap<Vec<Ident>, HashSet<UseTree>>,
    imports_to_filter: HashSet<String>,
}

impl ImportEditor {
    fn new(imports_to_filter: HashSet<String>) -> Self {
        Self {
            all_imports: Default::default(),
            imports_to_filter,
        }
    }

    fn into_items(self) -> Vec<ItemUse> {
        let mut items = vec![];
        for (path, tree) in self.all_imports {
            let prefix = path
                .iter()
                .map(|node| node.to_string())
                .collect::<Vec<_>>()
                .join("::");
            let names = tree
                .iter()
                .map(|node| node.to_token_stream().to_string())
                .filter(|node| !self.imports_to_filter.contains(node))
                .collect::<Vec<_>>()
                .join(", ");
            let mut use_repr = "use ".to_string();
            use_repr += &prefix;
            use_repr += "::{";
            use_repr += &names;
            use_repr += "};";

            let use_item: ItemUse = syn::parse_str(&use_repr).unwrap();

            items.push(use_item);
        }
        items
    }
}

impl<'ast> Visit<'ast> for ImportEditor {
    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        let mut path = vec![];
        let mut tree = node.tree.clone();
        while let UseTree::Path(use_path) = tree {
            path.push(use_path.ident.clone());
            tree = *use_path.tree;
        }
        if let UseTree::Group(use_group) = tree {
            let imports = self.all_imports.entry(path).or_default();
            imports.extend(use_group.items);
        }

        syn::visit::visit_item_use(self, node);
    }
}

#[derive(Debug, Default)]
struct GenericsEditor {}

impl GenericsEditor {
    fn new() -> Self {
        Self::default()
    }
}

impl VisitMut for GenericsEditor {
    fn visit_type_mut(&mut self, node: &mut Type) {
        if let Type::Path(ty) = node {
            if ty.path.segments.len() == 1 {
                let segment = &mut ty.path.segments[0];
                if segment.ident == "BeaconState"
                    || segment.ident.to_string().contains("BeaconBlock")
                {
                    if let PathArguments::AngleBracketed(arguments) = &mut segment.arguments {
                        for bound in BELLATRIX_BEACON_STATE_BOUNDS {
                            let param: GenericArgument = syn::parse_str(bound).unwrap();
                            arguments.args.push(param);
                        }
                    }
                }
            }
        }
        syn::visit_mut::visit_type_mut(self, node)
    }

    fn visit_signature_mut(&mut self, node: &mut Signature) {
        if node.inputs.iter().any(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                let ty = &*pat_type.ty;
                let ty = match ty {
                    Type::Path(ty) => ty,
                    Type::Reference(ty) => {
                        if let Type::Path(ty) = &*ty.elem {
                            ty
                        } else {
                            return false;
                        }
                    }
                    _ => return false,
                };
                if ty.path.segments.len() == 1 {
                    let segment = &ty.path.segments[0];
                    if segment.ident == "BeaconState"
                        || segment.ident.to_string().contains("BeaconBlock")
                    {
                        return true;
                    }
                }
            }
            false
        }) {
            let generics = &mut node.generics;
            for bound in BELLATRIX_BEACON_STATE_BOUNDS {
                let param: GenericParam = syn::parse_str(&format!("const {bound}: usize")).unwrap();
                generics.params.push(param);
            }
        }
        syn::visit_mut::visit_signature_mut(self, node);
    }
}

#[derive(Debug, Clone, Copy)]
enum Specs {
    Phase0,
    Altair,
    Bellatrix,
}

impl fmt::Display for Specs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Phase0 => write!(f, "phase0"),
            Self::Altair => write!(f, "altair"),
            Self::Bellatrix => write!(f, "bellatrix"),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Modules {
    BlockProcessing,
    EpochProcessing,
    Helpers,
    StateTransition,
    SlotProcessing,
}

impl fmt::Display for Modules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockProcessing => write!(f, "block_processing"),
            Self::EpochProcessing => write!(f, "epoch_processing"),
            Self::Helpers => write!(f, "helpers"),
            Self::StateTransition => write!(f, "state_transition"),
            Self::SlotProcessing => write!(f, "slot_processing"),
        }
    }
}

const ALL_MODULES: &[Modules; 5] = &[
    Modules::BlockProcessing,
    Modules::EpochProcessing,
    Modules::Helpers,
    Modules::StateTransition,
    Modules::SlotProcessing,
];

#[derive(Debug)]
struct Module {
    spec: Specs,
    module: Modules,

    item_index: usize,
    item_count: usize,

    uses: HashSet<UseDecl>,
    consts: HashSet<ConstDecl>,
    fns: HashMap<String, FnDecl>,
    fns_to_import: HashSet<String>,
}

impl Module {
    fn new(module: Modules, spec: Specs) -> Self {
        Self {
            spec,
            module,
            item_index: 0,
            item_count: 0,
            uses: Default::default(),
            consts: Default::default(),
            fns: Default::default(),
            fns_to_import: Default::default(),
        }
    }

    fn parse_if_exists(mut self, module_dir: &Path) -> Option<Self> {
        let path = if matches!(self.spec, Specs::Bellatrix)
            && matches!(self.module, Modules::StateTransition)
        {
            module_dir
                .join("state_transition_bellatrix")
                .with_extension("rs")
        } else {
            module_dir
                .join(self.module.to_string())
                .with_extension("rs")
        };
        if !path.exists() {
            return None;
        }

        let source = fs::read_to_string(path).unwrap();
        let file = syn::parse_str::<syn::File>(&source).unwrap();

        self.visit_file(&file);
        assert_eq!(self.item_count, self.item_index);

        self.fns_to_import = self.fns.keys().cloned().collect();

        Some(self)
    }

    fn validate_item_types(&self, items: &[syn::Item]) {
        for item in items {
            if !matches!(item, Item::Use(..) | Item::Fn(..) | Item::Const(..)) {
                println!("{item:?}, {self:?}");
                panic!("found top-level items this tool is not prepared to handle");
            }
        }
    }

    fn get_imports_to_filter(&self) -> HashSet<String> {
        let mut filter = self.fns.keys().cloned().collect::<HashSet<_>>();
        match self.spec {
            Specs::Phase0 => {}
            Specs::Altair => {
                filter.insert("PendingAttestation".to_string());
            }
            Specs::Bellatrix => {}
        }
        filter
    }

    fn unify_uses(&mut self) {
        let uses = mem::take(&mut self.uses);
        let items = uses
            .into_iter()
            .map(|decl| Item::Use(decl.node))
            .collect::<Vec<_>>();
        let file = syn::File {
            shebang: None,
            attrs: vec![],
            items,
        };
        let imports_to_filter = self.get_imports_to_filter();
        let mut editor = ImportEditor::new(imports_to_filter);
        editor.visit_file(&file);
        self.uses = editor
            .into_items()
            .into_iter()
            .map(|node| UseDecl { node })
            .collect();
    }

    fn merge(&mut self, previous: &Self) {
        self.uses = self
            .uses
            .union(&previous.uses)
            .filter(|use_decl| {
                // NOTE: remove any spec aliases
                let repr = &use_decl.node.tree.to_token_stream().to_string();
                !repr.contains("as spec")
            })
            .cloned()
            .collect();
        self.consts = self.consts.union(&previous.consts).cloned().collect();
        for (fn_name, fn_decl) in &previous.fns {
            if self.fns.contains_key(fn_name) {
                continue;
            }
            self.fns.insert(fn_name.clone(), fn_decl.clone());
        }
        self.unify_uses()
    }

    fn to_items(&self) -> Vec<Item> {
        let mut items = vec![];

        if matches!(self.module, Modules::StateTransition) {
            let mod_decl: ItemMod = parse_quote! {
                pub mod block_processing;
            };
            items.push(Item::Mod(mod_decl));
            let mod_decl: ItemMod = parse_quote! {
                pub mod epoch_processing;
            };
            items.push(Item::Mod(mod_decl));
            let mod_decl: ItemMod = parse_quote! {
                pub mod helpers;
            };
            items.push(Item::Mod(mod_decl));
            let mod_decl: ItemMod = parse_quote! {
                pub mod slot_processing;
            };
            items.push(Item::Mod(mod_decl));
        }

        let spec_item = format!("use crate::{} as spec;", self.spec);
        let spec_import: ItemUse = syn::parse_str(&spec_item).unwrap();
        items.push(Item::Use(spec_import));
        for fn_to_import in &self.fns_to_import {
            let module = if matches!(self.spec, Specs::Bellatrix)
                && matches!(self.module, Modules::StateTransition)
            {
                "state_transition_bellatrix".to_string()
            } else {
                self.module.to_string()
            };
            let use_repr = format!(
                "pub use crate::{}::{}::{};",
                self.spec, module, fn_to_import
            );
            let use_item: ItemUse = syn::parse_str(&use_repr).unwrap();
            items.push(Item::Use(use_item));
        }
        for use_item in &self.uses {
            items.push(Item::Use(use_item.node.clone()));
        }

        for const_item in &self.consts {
            items.push(Item::Const(const_item.node.clone()));
        }

        let mut fn_names = self.fns.keys().collect::<Vec<_>>();
        fn_names.sort();
        for fn_name in fn_names {
            if self.fns_to_import.contains(fn_name) {
                continue;
            }
            let fn_item = self.fns.get(fn_name).unwrap();
            items.push(Item::Fn(fn_item.node.clone()));
        }

        items
    }
}

impl<'ast> Visit<'ast> for Module {
    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        let use_decl = UseDecl { node: node.clone() };
        self.item_index += 1;
        self.uses.insert(use_decl);
        syn::visit::visit_item_use(self, node)
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let fn_name = node.sig.ident.to_string();
        let fn_decl = FnDecl {
            node: node.clone(),
            item_index: self.item_index,
        };
        self.item_index += 1;
        self.fns.insert(fn_name, fn_decl);
        syn::visit::visit_item_fn(self, node)
    }

    fn visit_item_const(&mut self, node: &'ast ItemConst) {
        let const_decl = ConstDecl {
            node: node.clone(),
            item_index: self.item_index,
        };
        self.item_index += 1;
        self.consts.insert(const_decl);
        syn::visit::visit_item_const(self, node)
    }

    fn visit_file(&mut self, node: &'ast syn::File) {
        self.validate_item_types(&node.items);
        self.item_count = node.items.len();
        syn::visit::visit_file(self, node);
    }
}

#[derive(Debug)]
struct Spec {
    spec: Specs,
    modules: HashMap<Modules, Module>,
}

fn load_modules(spec: Specs, modules: &[Modules]) -> HashMap<Modules, Module> {
    let module_dir = PathBuf::from(format!("src/{spec}"));
    let mut loaded_modules = HashMap::new();
    for &m in modules {
        let module = Module::new(m, spec);
        if let Some(module) = module.parse_if_exists(&module_dir) {
            loaded_modules.insert(m, module);
        }
    }
    loaded_modules
}

fn process_generic_type_altair<'a>(segments: impl Iterator<Item = &'a mut PathSegment>) -> bool {
    let mut did_mutate = false;
    for segment in segments {
        let ident = segment.ident.to_string();
        if ident.contains("BeaconState") {
            if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
                for arg in &mut args.args {
                    if let GenericArgument::Type(Type::Path(path_type)) = arg {
                        let segments = &mut path_type.path.segments;
                        if segments.len() != 1 {
                            panic!("invariant broken");
                        }
                        let segment = &mut segments[0].ident;
                        let repr = segment.to_string();
                        if repr == ATTESTATION_BOUND_IDENT {
                            *segment = format_ident!("{}", SYNC_COMMITTEE_SIZE_IDENT);
                            did_mutate = true;
                        }
                    }
                }
            }
        } else if ident.contains("BeaconBlock") {
            if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
                let arg: Type = syn::parse_str("SYNC_COMMITTEE_SIZE").unwrap();
                args.args.push(GenericArgument::Type(arg));
            }
            did_mutate = true;
        }
    }
    did_mutate
}

impl Spec {
    fn new(spec: Specs) -> Self {
        let modules = load_modules(spec, ALL_MODULES);
        Self { spec, modules }
    }

    fn merge(&mut self, previous_spec: Self) {
        for module_name in ALL_MODULES {
            let previous_module = previous_spec.modules.get(module_name).unwrap();
            let module = self
                .modules
                .entry(*module_name)
                .or_insert_with_key(|m| Module::new(*m, self.spec));
            module.merge(previous_module);
        }
    }

    fn patch_altair_types(&mut self) {
        for (_, module) in self.modules.iter_mut() {
            for (_, fn_decl) in module.fns.iter_mut() {
                let mut did_mutate = false;
                let signature = &mut fn_decl.node.sig;
                for input in &mut signature.inputs {
                    if let FnArg::Typed(arg) = input {
                        let ty = &mut arg.ty;
                        match &mut **ty {
                            Type::Reference(ref_type) => {
                                if let Type::Path(path_type) = &mut *ref_type.elem {
                                    let did_mutate_type = process_generic_type_altair(
                                        path_type.path.segments.iter_mut(),
                                    );
                                    if !did_mutate && did_mutate_type {
                                        did_mutate = true;
                                    }
                                }
                            }
                            Type::Path(path_type) => {
                                let did_mutate_type =
                                    process_generic_type_altair(path_type.path.segments.iter_mut());
                                if !did_mutate && did_mutate_type {
                                    did_mutate = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                let generic_params = &mut signature.generics.params;
                *generic_params = Punctuated::<GenericParam, Comma>::from_iter(
                    generic_params.iter().cloned().filter(|param| {
                        if let GenericParam::Const(param) = param {
                            let ident = param.ident.to_string();
                            if ident == ATTESTATION_BOUND_IDENT {
                                return false;
                            }
                        }
                        true
                    }),
                );
                if did_mutate {
                    let ident: Ident = syn::parse_str(SYNC_COMMITTEE_SIZE_IDENT).unwrap();
                    let bound: ConstParam = parse_quote! {
                        const #ident: usize
                    };
                    generic_params.push(GenericParam::Const(bound));
                }
            }
        }
    }

    fn patch_altair_imports(&mut self) {
        for (module_to_patch, imports_to_add) in HashMap::<Modules, Vec<&str>>::from_iter([
            (
                Modules::EpochProcessing,
                vec![
                    "use std::mem;",
                    "use std::collections::HashSet;",
                    "use ssz_rs::prelude::*;",
                    "use integer_sqrt::IntegerSquareRoot;",
                ],
            ),
            (
                Modules::Helpers,
                vec![
                    "use std::cmp;",
                    "use std::collections::HashSet;",
                    "use crate::signing::compute_signing_root;",
                    "use ssz_rs::prelude::*;",
                ],
            ),
            (
                Modules::BlockProcessing,
                vec![
                    "use std::cmp;",
                    "use crate::signing::compute_signing_root;",
                    "use ssz_rs::prelude::*;",
                    "use crate::ssz::ByteVector;",
                ],
            ),
            (
                Modules::SlotProcessing,
                vec!["use crate::primitives::Slot;", "use ssz_rs::prelude::*;"],
            ),
            (Modules::StateTransition, vec!["use ssz_rs::prelude::*;"]),
        ]) {
            let module = self.modules.get_mut(&module_to_patch).unwrap();
            for use_repr in imports_to_add {
                let node: ItemUse = syn::parse_str(use_repr).unwrap();
                module.uses.insert(UseDecl { node });
            }
        }
    }

    fn patch_altair_fns(&mut self) {
        for (module_to_patch, fns_to_remove) in HashMap::<Modules, Vec<&str>>::from_iter([
            (
                Modules::EpochProcessing,
                vec![
                    "get_unslashed_attesting_indices",
                    "get_matching_source_attestations",
                    "get_matching_target_attestations",
                    "get_matching_head_attestations",
                    "get_source_deltas",
                    "get_target_deltas",
                    "get_head_deltas",
                    "get_inclusion_delay_deltas",
                    "get_inactivity_penalty_deltas",
                    "get_attestation_deltas",
                    "get_attestation_component_deltas",
                    "get_attesting_balance",
                    "process_participation_record_updates",
                ],
            ),
            (Modules::Helpers, vec![]),
            (Modules::BlockProcessing, vec![]),
        ]) {
            let module = self.modules.get_mut(&module_to_patch).unwrap();
            for fn_to_remove in fns_to_remove {
                assert!(module.fns.remove(fn_to_remove).is_some());
            }
        }
    }

    fn patch_altair(&mut self) {
        self.patch_altair_fns();
        self.patch_altair_types();
        self.patch_altair_imports();
    }

    fn patch_bellatrix_imports(&mut self) {
        for (module_to_patch, imports_to_add) in HashMap::<Modules, Vec<&str>>::from_iter([
            (
                Modules::EpochProcessing,
                vec![
                    "use ssz_rs::prelude::*;",
                    "use spec::get_inactivity_penalty_deltas;",
                    "use std::mem;",
                ],
            ),
            (
                Modules::Helpers,
                vec![
                    "use crate::signing::compute_signing_root;",
                    "use integer_sqrt::IntegerSquareRoot;",
                    "use std::collections::HashSet;",
                    "use ssz_rs::prelude::*;",
                    "use std::cmp;",
                ],
            ),
            (
                Modules::BlockProcessing,
                vec![
                    "use ssz_rs::prelude::*;",
                    "use crate::ssz::ByteVector;",
                    "use crate::signing::compute_signing_root;",
                    "use std::iter::zip;",
                ],
            ),
            (
                Modules::SlotProcessing,
                vec!["use ssz_rs::prelude::*;", "use crate::primitives::Slot;"],
            ),
            (Modules::StateTransition, vec!["use ssz_rs::prelude::*;"]),
        ]) {
            let module = self.modules.get_mut(&module_to_patch).unwrap();
            for use_repr in imports_to_add {
                let node: ItemUse = syn::parse_str(use_repr).unwrap();
                module.uses.insert(UseDecl { node });
            }
        }
    }

    fn patch_bellatrix_types(&mut self) {
        for (_, module) in self.modules.iter_mut() {
            let fns = mem::take(&mut module.fns);
            let items = fns
                .into_values()
                .map(|decl| Item::Fn(decl.node))
                .collect::<Vec<_>>();
            let mut file = syn::File {
                shebang: None,
                attrs: vec![],
                items,
            };
            let mut generics_editor = GenericsEditor::new();
            generics_editor.visit_file_mut(&mut file);
            module.fns = file
                .items
                .into_iter()
                .map(|item| match item {
                    Item::Fn(node) => (
                        node.sig.ident.to_string(),
                        FnDecl {
                            node,
                            item_index: 0,
                        },
                    ),
                    _ => unreachable!(),
                })
                .collect();
        }
    }

    fn patch_bellatrix(&mut self) {
        self.patch_bellatrix_types();
        self.patch_bellatrix_imports();
    }

    fn patch(&mut self) {
        match self.spec {
            Specs::Phase0 => {}
            Specs::Altair => self.patch_altair(),
            Specs::Bellatrix => self.patch_bellatrix(),
        }
    }

    fn render(&self) {
        let target_dir = PathBuf::from(format!("src/{}/state_transition", self.spec));
        for module_name in ALL_MODULES {
            let module = self.modules.get(module_name).unwrap();
            let items = module.to_items();
            let warning = parse_quote! {
                //! WARNING: This file was derived by the `gen-spec` utility. DO NOT EDIT MANUALLY.
            };
            let target_file = syn::File {
                shebang: None,
                attrs: vec![warning],
                items,
            };
            let output = prettyplease::unparse(&target_file);
            let mut module_name = module_name.to_string();
            if module_name == "state_transition" {
                module_name = "mod".to_string()
            }
            let path = target_dir.join(module_name).with_extension("rs");
            fs::create_dir_all(&target_dir).unwrap();
            fs::write(path, output).unwrap();
        }
    }
}

#[derive(Default, Debug)]
struct Generator {
    specs_to_generate: Vec<Specs>,
}

impl Generator {
    fn run(self) {
        let mut previous_spec = None;
        for spec_to_generate in self.specs_to_generate {
            let mut current_spec = Spec::new(spec_to_generate);

            if let Some(previous_spec) = previous_spec {
                current_spec.merge(previous_spec);
                current_spec.patch();
                current_spec.render();
            }

            previous_spec = Some(current_spec);
        }
    }
}

fn main() {
    // NOTE: must run specs from beginning, in chronological order
    let specs = vec![Specs::Phase0, Specs::Altair, Specs::Bellatrix];
    let generator = Generator {
        specs_to_generate: specs,
    };
    generator.run();
}
