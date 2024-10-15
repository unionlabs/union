use enumorph::Enumorph;
use voyager_message::macros::model;
use subset_of::SubsetOf;

#[model]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleData {}
