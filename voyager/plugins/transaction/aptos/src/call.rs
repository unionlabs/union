use enumorph::Enumorph;
use macros::model;
use voyager_message::data::IbcMessage;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    SubmitTransaction(Vec<IbcMessage>),
}
