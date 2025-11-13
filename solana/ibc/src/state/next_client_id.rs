use ibc_union_spec::ClientId;

use super::{Serializable, StaticInit};

pub struct NextClientId(pub ClientId);

impl NextClientId {
    pub const fn seed<'a>() -> &'a [&'a [u8]] {
        &[b"next_client_id"]
    }

    pub fn increment(&mut self) {
        self.0 = self.0.checked_add(1).unwrap();
    }
}

impl StaticInit for NextClientId {
    fn static_init() -> Self {
        NextClientId(ClientId!(1))
    }
}

impl<'a> Serializable<'a> for NextClientId {
    fn serialized_size(&self) -> usize {
        4
    }

    fn serialize_into(&self, data: &mut [u8]) {
        data.copy_from_slice(self.0.raw().to_le_bytes().as_slice())
    }

    fn deserialize(data: &[u8]) -> Option<Self> {
        ClientId::deserialize(data).map(Self)
    }
}
