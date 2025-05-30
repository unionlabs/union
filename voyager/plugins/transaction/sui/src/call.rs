use enumorph::Enumorph;
use macros::model;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    SubmitTransaction(Vec<ibc_union_spec::datagram::Datagram>),
}
