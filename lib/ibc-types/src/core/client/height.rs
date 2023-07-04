use contracts::shared_types::IbcCoreClientV1HeightData;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Height {
    pub revision_number: u64,
    pub revision_height: u64,
}

impl Height {
    pub fn increment(mut self) -> Self {
        self.revision_height += 1;
        self
    }
}

impl From<protos::ibc::core::client::v1::Height> for Height {
    fn from(proto: protos::ibc::core::client::v1::Height) -> Self {
        Self {
            revision_number: proto.revision_number,
            revision_height: proto.revision_height,
        }
    }
}

impl From<Height> for protos::ibc::core::client::v1::Height {
    fn from(value: Height) -> Self {
        Self {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}

impl From<Height> for u64 {
    fn from(value: Height) -> Self {
        value.revision_height
    }
}

// REVIEW(benluelo): ordering for heights with different revision numbers?
impl PartialOrd for Height {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match self.revision_number.cmp(&other.revision_number) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => self.revision_height.cmp(&other.revision_height),
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        })
    }
}

impl std::fmt::Display for Height {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.revision_number, self.revision_height)
    }
}

impl From<Height> for IbcCoreClientV1HeightData {
    fn from(value: Height) -> Self {
        Self {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}

impl From<IbcCoreClientV1HeightData> for Height {
    fn from(value: IbcCoreClientV1HeightData) -> Self {
        Self {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}
