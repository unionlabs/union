use enumorph::Enumorph;
use macros::model;
use queue_msg::SubsetOf;

#[model]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleData {}
