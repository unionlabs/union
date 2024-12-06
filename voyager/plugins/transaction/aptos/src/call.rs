use enumorph::Enumorph;
use macros::model;
use voyager_message::ibc_union::IbcMsg;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    SubmitTransaction(Vec<IbcMsg>),
}
