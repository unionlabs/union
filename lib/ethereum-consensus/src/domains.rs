#[derive(Clone, Copy)]
pub enum DomainType {
    BeaconProposer,
    BeaconAttester,
    Randao,
    Deposit,
    VoluntaryExit,
    SelectionProof,
    AggregateAndProof,
    SyncCommittee,
    SyncCommitteeSelectionProof,
    ContributionAndProof,
    BlsToExecutionChange,
    ApplicationMask,
    ApplicationBuilder,
}

impl DomainType {
    pub fn as_bytes(&self) -> [u8; 4] {
        match self {
            Self::ApplicationMask => [0, 0, 0, 1],
            Self::ApplicationBuilder => [0, 0, 0, 1],
            _ => {
                let data = *self as u32;
                data.to_le_bytes()
            }
        }
    }
}
