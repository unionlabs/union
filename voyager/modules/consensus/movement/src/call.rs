use enumorph::Enumorph;
use queue_msg::queue_msg;

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleCall {
    FetchUpdate(FetchUpdate),
}

#[queue_msg]
pub struct FetchUpdate {
    pub from: u64,
    pub to: u64,
}
