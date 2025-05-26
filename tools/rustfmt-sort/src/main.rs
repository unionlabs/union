use std::fs;

use clap::Parser;
use quote::ToTokens;
use syn::{visit_mut::VisitMut, Item};

#[derive(Parser, Debug)]
#[command(name = "rustfmt-sort", about = "Sort Rust items in a file")]
struct Args {
    #[arg(value_name = "FILE")]
    file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let content = fs::read_to_string(&args.file)?;
    let mut file = syn::parse_file(&content)?;

    SortItemsVisitor.visit_file_mut(&mut file);

    let out = file.into_token_stream().to_string();

    Ok(())
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