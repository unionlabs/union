/// Only named projected types can be imported.
/// See import_unnamed.rs for unnamed projected types.

mod pub_ {
    use pin_project::pin_project;

    #[pin_project(project = DProj, project_ref = DProjRef)]
    pub struct Default(());

    #[pin_project(project = RProj, project_ref = RProjRef, project_replace = RProjOwn)]
    pub struct Replace(());
}
pub mod pub_use {
    #[rustfmt::skip]
    pub use crate::pub_::DProj; //~ ERROR E0365
    #[rustfmt::skip]
    pub use crate::pub_::DProjRef; //~ ERROR E0365
    #[rustfmt::skip]
    pub use crate::pub_::RProj; //~ ERROR E0365
    #[rustfmt::skip]
    pub use crate::pub_::RProjOwn; //~ ERROR E0365
    #[rustfmt::skip]
    pub use crate::pub_::RProjRef; //~ ERROR E0365

    // Confirm that the visibility of the original type is not changed.
    pub use crate::pub_::{Default, Replace};
}
pub mod pub_use2 {
    // Ok
    #[allow(unused_imports)]
    pub(crate) use crate::pub_::{DProj, DProjRef, RProj, RProjOwn, RProjRef};
}

mod pub_crate {
    use pin_project::pin_project;

    #[pin_project(project = DProj, project_ref = DProjRef)]
    pub(crate) struct Default(());

    #[pin_project(project = RProj, project_ref = RProjRef, project_replace = RProjOwn)]
    pub(crate) struct Replace(());
}
pub mod pub_crate_use {
    // Ok
    #[allow(unused_imports)]
    pub(crate) use crate::pub_crate::{DProj, DProjRef, RProj, RProjOwn, RProjRef};
}

fn main() {}
