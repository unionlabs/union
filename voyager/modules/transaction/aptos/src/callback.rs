use enumorph::Enumorph;
use queue_msg::{queue_msg, SubsetOf};

#[queue_msg]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleCallback {}
