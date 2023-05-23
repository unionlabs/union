use crate::test_utils::{load_snappy_ssz_bytes, load_yaml};
use ethereum_consensus::primitives::{Bytes32, Root};
use serde::Deserialize;
use std::path::Path;

fn load_ssz_static_test_case(test_case_path: &str) -> (RootData, Vec<u8>) {
    let path = test_case_path.to_string() + "/roots.yaml";
    let data: RootData = load_yaml(&path);

    let path = test_case_path.to_string() + "/serialized.ssz_snappy";
    let path = Path::new(&path);
    let encoding = load_snappy_ssz_bytes(path);
    (data, encoding)
}

#[derive(Deserialize)]
struct RootData {
    root: Bytes32,
}

pub struct AggregateAndProofTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl AggregateAndProofTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct AttestationTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl AttestationTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct AttestationDataTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl AttestationDataTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct AttesterSlashingTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl AttesterSlashingTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct BeaconBlockTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl BeaconBlockTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct BeaconBlockBodyTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl BeaconBlockBodyTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct BeaconBlockHeaderTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl BeaconBlockHeaderTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct BeaconStateTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl BeaconStateTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct CheckpointTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl CheckpointTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct ContributionAndProofTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl ContributionAndProofTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct DepositTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl DepositTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct DepositDataTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl DepositDataTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct DepositMessageTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl DepositMessageTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct Eth1BlockTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl Eth1BlockTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct Eth1DataTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl Eth1DataTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct ExecutionPayloadTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl ExecutionPayloadTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct ExecutionPayloadHeaderTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl ExecutionPayloadHeaderTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct ForkTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl ForkTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct ForkDataTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl ForkDataTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct HistoricalBatchTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl HistoricalBatchTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct IndexedAttestationTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl IndexedAttestationTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct LightClientUpdateTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl LightClientUpdateTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct PendingAttestationTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl PendingAttestationTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct PowBlockTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl PowBlockTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct ProposerSlashingTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl ProposerSlashingTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct SignedAggregateAndProofTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl SignedAggregateAndProofTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct SignedBeaconBlockTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl SignedBeaconBlockTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct SignedBeaconBlockHeaderTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl SignedBeaconBlockHeaderTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct SignedContributionAndProofTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl SignedContributionAndProofTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct SignedVoluntaryExitTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl SignedVoluntaryExitTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct SigningDataTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl SigningDataTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct SyncAggregateTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl SyncAggregateTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct SyncAggregatorSelectionDataTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl SyncAggregatorSelectionDataTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct SyncCommitteeTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl SyncCommitteeTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct SyncCommitteeContributionTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl SyncCommitteeContributionTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct SyncCommitteeMessageTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl SyncCommitteeMessageTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct ValidatorTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl ValidatorTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}

pub struct VoluntaryExitTestCase {
    data: RootData,
    encoding: Vec<u8>,
}

impl VoluntaryExitTestCase {
    pub fn from(test_case_path: &str) -> Self {
        let (data, encoding) = load_ssz_static_test_case(test_case_path);
        Self { data, encoding }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(&[u8]) -> (Vec<u8>, Root),
    {
        let (encoding, root) = f(&self.encoding);
        assert_eq!(encoding, self.encoding);
        assert_eq!(root.as_ref(), self.data.root.as_ref());
    }
}
