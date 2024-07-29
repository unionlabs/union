use enumorph::Enumorph;
use queue_msg::queue_msg;

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleAggregate {}
