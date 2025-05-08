use enumorph::Enumorph;
use macros::model;
use subset_of::SubsetOf;

#[model]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleData {}
