use enumorph::Enumorph;
use macros::model;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    SubmitMulticall(Vec<ibc_union_spec::datagram::Datagram>),
}
