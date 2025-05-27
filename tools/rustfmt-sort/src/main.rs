use std::{env, fs};

use quote::ToTokens;
use syn::{visit_mut::VisitMut, Item};

fn main() {
    let mut file =
        syn::parse_file(&fs::read_to_string(env::args().nth(1).unwrap()).unwrap()).unwrap();

    SortItemsVisitor.visit_file_mut(&mut file);

    let out = file.into_token_stream().to_string();

    println!("{out}");
}

struct SortItemsVisitor;

impl VisitMut for SortItemsVisitor {
    fn visit_file_mut(&mut self, i: &mut syn::File) {
        syn::visit_mut::visit_file_mut(self, i);

        sort_items(&mut i.items);
    }

    fn visit_item_mod_mut(&mut self, i: &mut syn::ItemMod) {
        syn::visit_mut::visit_item_mod_mut(self, i);

        if let Some((_, ref mut content)) = i.content {
            sort_items(content);
        }
    }
}

fn sort_items(content: &mut Vec<Item>) {
    let (uses, mut items) = content
        .drain(..)
        .partition::<Vec<_>, _>(|i| matches!(i, Item::Use(_)));

    // the token display repr should be stable since we pin the rustc version in the repo
    items.sort_by_cached_key(|i| i.to_token_stream().to_string().into_bytes());

    *content = uses.into_iter().chain(items).collect();
}
