use enumorph::Enumorph;
use queue_msg::queue_msg;
use voyager_message::data::IbcMessage;

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleCall {
    SubmitTransaction(Vec<IbcMessage>),
}
