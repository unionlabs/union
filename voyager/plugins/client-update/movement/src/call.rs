use enumorph::Enumorph;
use voyager_message::macros::model;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    FetchUpdate(FetchUpdate),
}

#[model]
pub struct FetchUpdate {
    pub from: u64,
    pub to: u64,
}
