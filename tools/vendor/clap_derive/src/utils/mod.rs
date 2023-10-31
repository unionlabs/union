pub mod error;

mod doc_comments;
mod spanned;
mod ty;

pub use doc_comments::extract_doc_comment;
pub use doc_comments::format_doc_comment;

pub use self::{
    spanned::Sp,
    ty::{inner_type, is_simple_ty, sub_type, subty_if_name, Ty},
};
