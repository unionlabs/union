use enumorph::Enumorph;
use macros::model;
use queue_msg::queue_msg;

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleCall {
    FetchBlock(FetchBlock),
    FetchBlocks(FetchBlocks),
}

#[model]
pub struct FetchBlocks {
    pub from_height: u64,
    pub to_height: u64,
}

#[model]
pub struct FetchBlock {
    pub height: u64,
}
