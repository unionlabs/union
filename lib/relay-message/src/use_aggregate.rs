use crate::{data::AnyData, AnyLightClientIdentified};

pub trait IsAggregateData = TryFrom<AnyLightClientIdentified<AnyData>, Error = AnyLightClientIdentified<AnyData>>
    + Into<AnyLightClientIdentified<AnyData>>;
