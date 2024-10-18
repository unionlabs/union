use enumorph::Enumorph;
use voyager_message::macros::model;
use voyager_message::data::IbcMessage;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    SubmitTransaction(Vec<IbcMessage>),
}
