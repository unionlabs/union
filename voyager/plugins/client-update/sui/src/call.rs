use enumorph::Enumorph;
use macros::model;

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
