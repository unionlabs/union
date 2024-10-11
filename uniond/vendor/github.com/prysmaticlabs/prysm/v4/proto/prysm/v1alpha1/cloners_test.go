package eth_test

import (
	"math/rand"
	"reflect"
	"testing"

	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	v1alpha1 "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestCopyETH1Data(t *testing.T) {
	data := genEth1Data()

	got := v1alpha1.CopyETH1Data(data)
	if !reflect.DeepEqual(got, data) {
		t.Errorf("CopyETH1Data() = %v, want %v", got, data)
	}
	assert.NotEmpty(t, got, "Copied eth1data has empty fields")
}

func TestCopyPendingAttestation(t *testing.T) {
	pa := genPendingAttestation()

	got := v1alpha1.CopyPendingAttestation(pa)
	if !reflect.DeepEqual(got, pa) {
		t.Errorf("CopyPendingAttestation() = %v, want %v", got, pa)
	}
	assert.NotEmpty(t, got, "Copied pending attestation has empty fields")
}

func TestCopyAttestation(t *testing.T) {
	att := genAttestation()

	got := v1alpha1.CopyAttestation(att)
	if !reflect.DeepEqual(got, att) {
		t.Errorf("CopyAttestation() = %v, want %v", got, att)
	}
	assert.NotEmpty(t, got, "Copied attestation has empty fields")
}
func TestCopyAttestationData(t *testing.T) {
	att := genAttData()

	got := v1alpha1.CopyAttestationData(att)
	if !reflect.DeepEqual(got, att) {
		t.Errorf("CopyAttestationData() = %v, want %v", got, att)
	}
	assert.NotEmpty(t, got, "Copied attestation data has empty fields")
}

func TestCopyCheckpoint(t *testing.T) {
	cp := genCheckpoint()

	got := v1alpha1.CopyCheckpoint(cp)
	if !reflect.DeepEqual(got, cp) {
		t.Errorf("CopyCheckpoint() = %v, want %v", got, cp)
	}
	assert.NotEmpty(t, got, "Copied checkpoint has empty fields")
}

func TestCopySignedBeaconBlock(t *testing.T) {
	blk := genSignedBeaconBlock()

	got := v1alpha1.CopySignedBeaconBlock(blk)
	if !reflect.DeepEqual(got, blk) {
		t.Errorf("CopySignedBeaconBlock() = %v, want %v", got, blk)
	}
	assert.NotEmpty(t, got, "Copied signed beacon block has empty fields")
}

func TestCopyBeaconBlock(t *testing.T) {
	blk := genBeaconBlock()

	got := v1alpha1.CopyBeaconBlock(blk)
	if !reflect.DeepEqual(got, blk) {
		t.Errorf("CopyBeaconBlock() = %v, want %v", got, blk)
	}
	assert.NotEmpty(t, got, "Copied beacon block has empty fields")
}

func TestCopyBeaconBlockBody(t *testing.T) {
	body := genBeaconBlockBody()

	got := v1alpha1.CopyBeaconBlockBody(body)
	if !reflect.DeepEqual(got, body) {
		t.Errorf("CopyBeaconBlockBody() = %v, want %v", got, body)
	}
	assert.NotEmpty(t, got, "Copied beacon block body has empty fields")
}

func TestCopySignedBeaconBlockAltair(t *testing.T) {
	sbb := genSignedBeaconBlockAltair()

	got := v1alpha1.CopySignedBeaconBlockAltair(sbb)
	if !reflect.DeepEqual(got, sbb) {
		t.Errorf("CopySignedBeaconBlockAltair() = %v, want %v", got, sbb)
	}
	assert.NotEmpty(t, sbb, "Copied signed beacon block altair has empty fields")
}

func TestCopyBeaconBlockAltair(t *testing.T) {
	b := genBeaconBlockAltair()

	got := v1alpha1.CopyBeaconBlockAltair(b)
	if !reflect.DeepEqual(got, b) {
		t.Errorf("CopyBeaconBlockAltair() = %v, want %v", got, b)
	}
	assert.NotEmpty(t, b, "Copied beacon block altair has empty fields")
}

func TestCopyBeaconBlockBodyAltair(t *testing.T) {
	bb := genBeaconBlockBodyAltair()

	got := v1alpha1.CopyBeaconBlockBodyAltair(bb)
	if !reflect.DeepEqual(got, bb) {
		t.Errorf("CopyBeaconBlockBodyAltair() = %v, want %v", got, bb)
	}
	assert.NotEmpty(t, bb, "Copied beacon block body altair has empty fields")
}

func TestCopyProposerSlashings(t *testing.T) {
	ps := genProposerSlashings(10)

	got := v1alpha1.CopyProposerSlashings(ps)
	if !reflect.DeepEqual(got, ps) {
		t.Errorf("CopyProposerSlashings() = %v, want %v", got, ps)
	}
	assert.NotEmpty(t, got, "Copied proposer slashings have empty fields")
}

func TestCopyProposerSlashing(t *testing.T) {
	ps := genProposerSlashing()

	got := v1alpha1.CopyProposerSlashing(ps)
	if !reflect.DeepEqual(got, ps) {
		t.Errorf("CopyProposerSlashing() = %v, want %v", got, ps)
	}
	assert.NotEmpty(t, got, "Copied proposer slashing has empty fields")
}

func TestCopySignedBeaconBlockHeader(t *testing.T) {
	sbh := genSignedBeaconBlockHeader()

	got := v1alpha1.CopySignedBeaconBlockHeader(sbh)
	if !reflect.DeepEqual(got, sbh) {
		t.Errorf("CopySignedBeaconBlockHeader() = %v, want %v", got, sbh)
	}
	assert.NotEmpty(t, got, "Copied signed beacon block header has empty fields")
}

func TestCopyBeaconBlockHeader(t *testing.T) {
	bh := genBeaconBlockHeader()

	got := v1alpha1.CopyBeaconBlockHeader(bh)
	if !reflect.DeepEqual(got, bh) {
		t.Errorf("CopyBeaconBlockHeader() = %v, want %v", got, bh)
	}
	assert.NotEmpty(t, got, "Copied beacon block header has empty fields")
}

func TestCopyAttesterSlashings(t *testing.T) {
	as := genAttesterSlashings(10)

	got := v1alpha1.CopyAttesterSlashings(as)
	if !reflect.DeepEqual(got, as) {
		t.Errorf("CopyAttesterSlashings() = %v, want %v", got, as)
	}
	assert.NotEmpty(t, got, "Copied attester slashings have empty fields")
}

func TestCopyIndexedAttestation(t *testing.T) {
	ia := genIndexedAttestation()

	got := v1alpha1.CopyIndexedAttestation(ia)
	if !reflect.DeepEqual(got, ia) {
		t.Errorf("CopyIndexedAttestation() = %v, want %v", got, ia)
	}
	assert.NotEmpty(t, got, "Copied indexed attestation has empty fields")
}

func TestCopyAttestations(t *testing.T) {
	atts := genAttestations(10)

	got := v1alpha1.CopyAttestations(atts)
	if !reflect.DeepEqual(got, atts) {
		t.Errorf("CopyAttestations() = %v, want %v", got, atts)
	}
	assert.NotEmpty(t, got, "Copied attestations have empty fields")
}

func TestCopyDeposits(t *testing.T) {
	d := genDeposits(10)

	got := v1alpha1.CopyDeposits(d)
	if !reflect.DeepEqual(got, d) {
		t.Errorf("CopyDeposits() = %v, want %v", got, d)
	}
	assert.NotEmpty(t, got, "Copied deposits have empty fields")
}

func TestCopyDeposit(t *testing.T) {
	d := genDeposit()

	got := v1alpha1.CopyDeposit(d)
	if !reflect.DeepEqual(got, d) {
		t.Errorf("CopyDeposit() = %v, want %v", got, d)
	}
	assert.NotEmpty(t, got, "Copied deposit has empty fields")
}

func TestCopyDepositData(t *testing.T) {
	dd := genDepositData()

	got := v1alpha1.CopyDepositData(dd)
	if !reflect.DeepEqual(got, dd) {
		t.Errorf("CopyDepositData() = %v, want %v", got, dd)
	}
	assert.NotEmpty(t, got, "Copied deposit data has empty fields")
}

func TestCopySignedVoluntaryExits(t *testing.T) {
	sv := genSignedVoluntaryExits(10)

	got := v1alpha1.CopySignedVoluntaryExits(sv)
	if !reflect.DeepEqual(got, sv) {
		t.Errorf("CopySignedVoluntaryExits() = %v, want %v", got, sv)
	}
	assert.NotEmpty(t, got, "Copied signed voluntary exits have empty fields")
}

func TestCopySignedVoluntaryExit(t *testing.T) {
	sv := genSignedVoluntaryExit()

	got := v1alpha1.CopySignedVoluntaryExit(sv)
	if !reflect.DeepEqual(got, sv) {
		t.Errorf("CopySignedVoluntaryExit() = %v, want %v", got, sv)
	}
	assert.NotEmpty(t, got, "Copied signed voluntary exit has empty fields")
}

func TestCopyValidator(t *testing.T) {
	v := genValidator()

	got := v1alpha1.CopyValidator(v)
	if !reflect.DeepEqual(got, v) {
		t.Errorf("CopyValidator() = %v, want %v", got, v)
	}
	assert.NotEmpty(t, got, "Copied validator has empty fields")
}

func TestCopySyncCommitteeMessage(t *testing.T) {
	scm := genSyncCommitteeMessage()

	got := v1alpha1.CopySyncCommitteeMessage(scm)
	if !reflect.DeepEqual(got, scm) {
		t.Errorf("CopySyncCommitteeMessage() = %v, want %v", got, scm)
	}
	assert.NotEmpty(t, got, "Copied sync committee message has empty fields")
}

func TestCopySyncCommitteeContribution(t *testing.T) {
	scc := genSyncCommitteeContribution()

	got := v1alpha1.CopySyncCommitteeContribution(scc)
	if !reflect.DeepEqual(got, scc) {
		t.Errorf("CopySyncCommitteeContribution() = %v, want %v", got, scc)
	}
	assert.NotEmpty(t, got, "Copied sync committee contribution has empty fields")
}

func TestCopySyncAggregate(t *testing.T) {
	sa := genSyncAggregate()

	got := v1alpha1.CopySyncAggregate(sa)
	if !reflect.DeepEqual(got, sa) {
		t.Errorf("CopySyncAggregate() = %v, want %v", got, sa)
	}
	assert.NotEmpty(t, got, "Copied sync aggregate has empty fields")
}

func TestCopyPendingAttestationSlice(t *testing.T) {
	tests := []struct {
		name  string
		input []*v1alpha1.PendingAttestation
	}{
		{
			name:  "nil",
			input: nil,
		},
		{
			name:  "empty",
			input: []*v1alpha1.PendingAttestation{},
		},
		{
			name: "correct copy",
			input: []*v1alpha1.PendingAttestation{
				genPendingAttestation(),
				genPendingAttestation(),
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := v1alpha1.CopyPendingAttestationSlice(tt.input); !reflect.DeepEqual(got, tt.input) {
				t.Errorf("CopyPendingAttestationSlice() = %v, want %v", got, tt.input)
			}
		})
	}
}

func TestCopyPayloadHeader(t *testing.T) {
	p := genPayloadHeader()

	got := v1alpha1.CopyExecutionPayloadHeader(p)
	if !reflect.DeepEqual(got, p) {
		t.Errorf("CopyExecutionPayloadHeader() = %v, want %v", got, p)
	}
	assert.NotEmpty(t, got, "Copied execution payload header has empty fields")
}

func TestCopyPayloadHeaderCapella(t *testing.T) {
	p := genPayloadHeaderCapella()

	got := v1alpha1.CopyExecutionPayloadHeaderCapella(p)
	if !reflect.DeepEqual(got, p) {
		t.Errorf("TestCopyPayloadHeaderCapella() = %v, want %v", got, p)
	}
	assert.NotEmpty(t, got, "Copied execution payload header has empty fields")
}

func TestCopyPayloadHeaderDeneb(t *testing.T) {
	p := genPayloadHeaderDeneb()

	got := v1alpha1.CopyExecutionPayloadHeaderDeneb(p)
	if !reflect.DeepEqual(got, p) {
		t.Errorf("TestCopyPayloadHeaderDeneb() = %v, want %v", got, p)
	}
	assert.NotEmpty(t, got, "Copied execution payload header has empty fields")
}

func TestCopySignedBeaconBlockBellatrix(t *testing.T) {
	sbb := genSignedBeaconBlockBellatrix()

	got := v1alpha1.CopySignedBeaconBlockBellatrix(sbb)
	if !reflect.DeepEqual(got, sbb) {
		t.Errorf("CopySignedBeaconBlockBellatrix() = %v, want %v", got, sbb)
	}
	assert.NotEmpty(t, sbb, "Copied signed beacon block Bellatrix has empty fields")
}

func TestCopyBeaconBlockBellatrix(t *testing.T) {
	b := genBeaconBlockBellatrix()

	got := v1alpha1.CopyBeaconBlockBellatrix(b)
	if !reflect.DeepEqual(got, b) {
		t.Errorf("CopyBeaconBlockBellatrix() = %v, want %v", got, b)
	}
	assert.NotEmpty(t, b, "Copied beacon block Bellatrix has empty fields")
}

func TestCopyBeaconBlockBodyBellatrix(t *testing.T) {
	bb := genBeaconBlockBodyBellatrix()

	got := v1alpha1.CopyBeaconBlockBodyBellatrix(bb)
	if !reflect.DeepEqual(got, bb) {
		t.Errorf("CopyBeaconBlockBodyBellatrix() = %v, want %v", got, bb)
	}
	assert.NotEmpty(t, bb, "Copied beacon block body Bellatrix has empty fields")
}

func TestCopySignedBlindedBeaconBlockBellatrix(t *testing.T) {
	sbb := genSignedBeaconBlockBellatrix()

	got := v1alpha1.CopySignedBeaconBlockBellatrix(sbb)
	if !reflect.DeepEqual(got, sbb) {
		t.Errorf("CopySignedBeaconBlockBellatrix() = %v, want %v", got, sbb)
	}
	assert.NotEmpty(t, sbb, "Copied signed blinded beacon block Bellatrix has empty fields")
}

func TestCopyBlindedBeaconBlockBellatrix(t *testing.T) {
	b := genBeaconBlockBellatrix()

	got := v1alpha1.CopyBeaconBlockBellatrix(b)
	if !reflect.DeepEqual(got, b) {
		t.Errorf("CopyBeaconBlockBellatrix() = %v, want %v", got, b)
	}
	assert.NotEmpty(t, b, "Copied blinded beacon block Bellatrix has empty fields")
}

func TestCopyBlindedBeaconBlockBodyBellatrix(t *testing.T) {
	bb := genBeaconBlockBodyBellatrix()

	got := v1alpha1.CopyBeaconBlockBodyBellatrix(bb)
	if !reflect.DeepEqual(got, bb) {
		t.Errorf("CopyBeaconBlockBodyBellatrix() = %v, want %v", got, bb)
	}
	assert.NotEmpty(t, bb, "Copied blinded beacon block body Bellatrix has empty fields")
}

func TestCopySignedBeaconBlockCapella(t *testing.T) {
	sbb := genSignedBeaconBlockCapella()

	got := v1alpha1.CopySignedBeaconBlockCapella(sbb)
	if !reflect.DeepEqual(got, sbb) {
		t.Errorf("CopySignedBeaconBlockCapella() = %v, want %v", got, sbb)
	}
	assert.NotEmpty(t, sbb, "Copied signed beacon block Capella has empty fields")
}

func TestCopyBeaconBlockCapella(t *testing.T) {
	b := genBeaconBlockCapella()

	got := v1alpha1.CopyBeaconBlockCapella(b)
	if !reflect.DeepEqual(got, b) {
		t.Errorf("CopyBeaconBlockCapella() = %v, want %v", got, b)
	}
	assert.NotEmpty(t, b, "Copied beacon block Capella has empty fields")
}

func TestCopyBeaconBlockBodyCapella(t *testing.T) {
	bb := genBeaconBlockBodyCapella()

	got := v1alpha1.CopyBeaconBlockBodyCapella(bb)
	if !reflect.DeepEqual(got, bb) {
		t.Errorf("CopyBeaconBlockBodyCapella() = %v, want %v", got, bb)
	}
	assert.NotEmpty(t, bb, "Copied beacon block body Capella has empty fields")
}

func TestCopySignedBlindedBeaconBlockCapella(t *testing.T) {
	sbb := genSignedBlindedBeaconBlockCapella()

	got := v1alpha1.CopySignedBlindedBeaconBlockCapella(sbb)
	if !reflect.DeepEqual(got, sbb) {
		t.Errorf("CopySignedBlindedBeaconBlockCapella() = %v, want %v", got, sbb)
	}
	assert.NotEmpty(t, sbb, "Copied signed blinded beacon block Capella has empty fields")
}

func TestCopyBlindedBeaconBlockCapella(t *testing.T) {
	b := genBlindedBeaconBlockCapella()

	got := v1alpha1.CopyBlindedBeaconBlockCapella(b)
	if !reflect.DeepEqual(got, b) {
		t.Errorf("CopyBlindedBeaconBlockCapella() = %v, want %v", got, b)
	}
	assert.NotEmpty(t, b, "Copied blinded beacon block Capella has empty fields")
}

func TestCopyBlindedBeaconBlockBodyCapella(t *testing.T) {
	bb := genBlindedBeaconBlockBodyCapella()

	got := v1alpha1.CopyBlindedBeaconBlockBodyCapella(bb)
	if !reflect.DeepEqual(got, bb) {
		t.Errorf("CopyBlindedBeaconBlockBodyCapella() = %v, want %v", got, bb)
	}
	assert.NotEmpty(t, bb, "Copied blinded beacon block body Capella has empty fields")
}

func TestCopySignedBeaconBlockDeneb(t *testing.T) {
	sbb := genSignedBeaconBlockDeneb()

	got := v1alpha1.CopySignedBeaconBlockDeneb(sbb)
	if !reflect.DeepEqual(got, sbb) {
		t.Errorf("CopySignedBeaconBlockDeneb() = %v, want %v", got, sbb)
	}
	assert.NotEmpty(t, sbb, "Copied signed beacon block Deneb has empty fields")
}

func TestCopyBeaconBlockDeneb(t *testing.T) {
	b := genBeaconBlockDeneb()

	got := v1alpha1.CopyBeaconBlockDeneb(b)
	if !reflect.DeepEqual(got, b) {
		t.Errorf("CopyBeaconBlockDeneb() = %v, want %v", got, b)
	}
	assert.NotEmpty(t, b, "Copied beacon block Deneb has empty fields")
}

func TestCopyBeaconBlockBodyDeneb(t *testing.T) {
	bb := genBeaconBlockBodyDeneb()

	got := v1alpha1.CopyBeaconBlockBodyDeneb(bb)
	if !reflect.DeepEqual(got, bb) {
		t.Errorf("CopyBeaconBlockBodyDeneb() = %v, want %v", got, bb)
	}
	assert.NotEmpty(t, bb, "Copied beacon block body Deneb has empty fields")
}

func TestCopySignedBlindedBeaconBlockDeneb(t *testing.T) {
	sbb := genSignedBlindedBeaconBlockDeneb()

	got := v1alpha1.CopySignedBlindedBeaconBlockDeneb(sbb)
	if !reflect.DeepEqual(got, sbb) {
		t.Errorf("CopySignedBlindedBeaconBlockDeneb() = %v, want %v", got, sbb)
	}
	assert.NotEmpty(t, sbb, "Copied signed blinded beacon block Deneb has empty fields")
}

func TestCopyBlindedBeaconBlockDeneb(t *testing.T) {
	b := genBlindedBeaconBlockDeneb()

	got := v1alpha1.CopyBlindedBeaconBlockDeneb(b)
	if !reflect.DeepEqual(got, b) {
		t.Errorf("CopyBlindedBeaconBlockDeneb() = %v, want %v", got, b)
	}
	assert.NotEmpty(t, b, "Copied blinded beacon block Deneb has empty fields")
}

func TestCopyBlindedBeaconBlockBodyDeneb(t *testing.T) {
	bb := genBlindedBeaconBlockBodyDeneb()

	got := v1alpha1.CopyBlindedBeaconBlockBodyDeneb(bb)
	if !reflect.DeepEqual(got, bb) {
		t.Errorf("CopyBlindedBeaconBlockBodyDeneb() = %v, want %v", got, bb)
	}
	assert.NotEmpty(t, bb, "Copied blinded beacon block body Deneb has empty fields")
}

func bytes(length int) []byte {
	b := make([]byte, length)
	for i := 0; i < length; i++ {
		b[i] = uint8(rand.Int31n(255) + 1)
	}
	return b
}

func TestCopyWithdrawals(t *testing.T) {
	ws := genWithdrawals(10)

	got := v1alpha1.CopyWithdrawalSlice(ws)
	if !reflect.DeepEqual(got, ws) {
		t.Errorf("TestCopyWithdrawals() = %v, want %v", got, ws)
	}
	assert.NotEmpty(t, got, "Copied withdrawals have empty fields")
}

func TestCopyWithdrawal(t *testing.T) {
	w := genWithdrawal()

	got := v1alpha1.CopyWithdrawal(w)
	if !reflect.DeepEqual(got, w) {
		t.Errorf("TestCopyWithdrawal() = %v, want %v", got, w)
	}
	assert.NotEmpty(t, got, "Copied withdrawal has empty fields")
}

func TestCopyBLSToExecutionChanges(t *testing.T) {
	changes := genBLSToExecutionChanges(10)

	got := v1alpha1.CopyBLSToExecutionChanges(changes)
	if !reflect.DeepEqual(got, changes) {
		t.Errorf("TestCopyBLSToExecutionChanges() = %v, want %v", got, changes)
	}
}

func TestCopyHistoricalSummaries(t *testing.T) {
	summaries := []*v1alpha1.HistoricalSummary{
		{BlockSummaryRoot: []byte("block summary root 0"), StateSummaryRoot: []byte("state summary root 0")},
		{BlockSummaryRoot: []byte("block summary root 1"), StateSummaryRoot: []byte("state summary root 1")},
	}

	got := v1alpha1.CopyHistoricalSummaries(summaries)
	if !reflect.DeepEqual(got, summaries) {
		t.Errorf("TestCopyHistoricalSummariesing() = %v, want %v", got, summaries)
	}
}

func genAttestation() *v1alpha1.Attestation {
	return &v1alpha1.Attestation{
		AggregationBits: bytes(32),
		Data:            genAttData(),
		Signature:       bytes(32),
	}
}

func genAttestations(num int) []*v1alpha1.Attestation {
	atts := make([]*v1alpha1.Attestation, num)
	for i := 0; i < num; i++ {
		atts[i] = genAttestation()
	}
	return atts
}

func genAttData() *v1alpha1.AttestationData {
	return &v1alpha1.AttestationData{
		Slot:            1,
		CommitteeIndex:  2,
		BeaconBlockRoot: bytes(32),
		Source:          genCheckpoint(),
		Target:          genCheckpoint(),
	}
}

func genCheckpoint() *v1alpha1.Checkpoint {
	return &v1alpha1.Checkpoint{
		Epoch: 1,
		Root:  bytes(32),
	}
}

func genEth1Data() *v1alpha1.Eth1Data {
	return &v1alpha1.Eth1Data{
		DepositRoot:  bytes(32),
		DepositCount: 4,
		BlockHash:    bytes(32),
	}
}

func genPendingAttestation() *v1alpha1.PendingAttestation {
	return &v1alpha1.PendingAttestation{
		AggregationBits: bytes(32),
		Data:            genAttData(),
		InclusionDelay:  3,
		ProposerIndex:   5,
	}
}

func genSignedBeaconBlock() *v1alpha1.SignedBeaconBlock {
	return &v1alpha1.SignedBeaconBlock{
		Block:     genBeaconBlock(),
		Signature: bytes(32),
	}
}

func genBeaconBlock() *v1alpha1.BeaconBlock {
	return &v1alpha1.BeaconBlock{
		Slot:          4,
		ProposerIndex: 5,
		ParentRoot:    bytes(32),
		StateRoot:     bytes(32),
		Body:          genBeaconBlockBody(),
	}
}

func genBeaconBlockBody() *v1alpha1.BeaconBlockBody {
	return &v1alpha1.BeaconBlockBody{
		RandaoReveal:      bytes(32),
		Eth1Data:          genEth1Data(),
		Graffiti:          bytes(32),
		ProposerSlashings: genProposerSlashings(5),
		AttesterSlashings: genAttesterSlashings(5),
		Attestations:      genAttestations(5),
		Deposits:          genDeposits(5),
		VoluntaryExits:    genSignedVoluntaryExits(5),
	}
}

func genProposerSlashing() *v1alpha1.ProposerSlashing {
	return &v1alpha1.ProposerSlashing{
		Header_1: genSignedBeaconBlockHeader(),
		Header_2: genSignedBeaconBlockHeader(),
	}
}

func genProposerSlashings(num int) []*v1alpha1.ProposerSlashing {
	ps := make([]*v1alpha1.ProposerSlashing, num)
	for i := 0; i < num; i++ {
		ps[i] = genProposerSlashing()
	}
	return ps
}

func genAttesterSlashing() *v1alpha1.AttesterSlashing {
	return &v1alpha1.AttesterSlashing{
		Attestation_1: genIndexedAttestation(),
		Attestation_2: genIndexedAttestation(),
	}
}

func genIndexedAttestation() *v1alpha1.IndexedAttestation {
	return &v1alpha1.IndexedAttestation{
		AttestingIndices: []uint64{1, 2, 3},
		Data:             genAttData(),
		Signature:        bytes(32),
	}
}

func genAttesterSlashings(num int) []*v1alpha1.AttesterSlashing {
	as := make([]*v1alpha1.AttesterSlashing, num)
	for i := 0; i < num; i++ {
		as[i] = genAttesterSlashing()
	}
	return as
}

func genBeaconBlockHeader() *v1alpha1.BeaconBlockHeader {
	return &v1alpha1.BeaconBlockHeader{
		Slot:          10,
		ProposerIndex: 15,
		ParentRoot:    bytes(32),
		StateRoot:     bytes(32),
		BodyRoot:      bytes(32),
	}
}

func genSignedBeaconBlockHeader() *v1alpha1.SignedBeaconBlockHeader {
	return &v1alpha1.SignedBeaconBlockHeader{
		Header:    genBeaconBlockHeader(),
		Signature: bytes(32),
	}
}

func genDepositData() *v1alpha1.Deposit_Data {
	return &v1alpha1.Deposit_Data{
		PublicKey:             bytes(32),
		WithdrawalCredentials: bytes(32),
		Amount:                20000,
		Signature:             bytes(32),
	}
}

func genDeposit() *v1alpha1.Deposit {
	return &v1alpha1.Deposit{
		Data:  genDepositData(),
		Proof: [][]byte{bytes(32), bytes(32), bytes(32), bytes(32)},
	}
}

func genDeposits(num int) []*v1alpha1.Deposit {
	d := make([]*v1alpha1.Deposit, num)
	for i := 0; i < num; i++ {
		d[i] = genDeposit()
	}
	return d
}

func genVoluntaryExit() *v1alpha1.VoluntaryExit {
	return &v1alpha1.VoluntaryExit{
		Epoch:          5432,
		ValidatorIndex: 888888,
	}
}

func genSignedVoluntaryExit() *v1alpha1.SignedVoluntaryExit {
	return &v1alpha1.SignedVoluntaryExit{
		Exit:      genVoluntaryExit(),
		Signature: bytes(32),
	}
}

func genSignedVoluntaryExits(num int) []*v1alpha1.SignedVoluntaryExit {
	sv := make([]*v1alpha1.SignedVoluntaryExit, num)
	for i := 0; i < num; i++ {
		sv[i] = genSignedVoluntaryExit()
	}
	return sv
}

func genValidator() *v1alpha1.Validator {
	return &v1alpha1.Validator{
		PublicKey:                  bytes(32),
		WithdrawalCredentials:      bytes(32),
		EffectiveBalance:           12345,
		Slashed:                    true,
		ActivationEligibilityEpoch: 14322,
		ActivationEpoch:            14325,
		ExitEpoch:                  23425,
		WithdrawableEpoch:          30000,
	}
}

func genSyncCommitteeContribution() *v1alpha1.SyncCommitteeContribution {
	return &v1alpha1.SyncCommitteeContribution{
		Slot:              12333,
		BlockRoot:         bytes(32),
		SubcommitteeIndex: 4444,
		AggregationBits:   bytes(32),
		Signature:         bytes(32),
	}
}

func genSyncAggregate() *v1alpha1.SyncAggregate {
	return &v1alpha1.SyncAggregate{
		SyncCommitteeBits:      bytes(32),
		SyncCommitteeSignature: bytes(32),
	}
}

func genBeaconBlockBodyAltair() *v1alpha1.BeaconBlockBodyAltair {
	return &v1alpha1.BeaconBlockBodyAltair{
		RandaoReveal:      bytes(32),
		Eth1Data:          genEth1Data(),
		Graffiti:          bytes(32),
		ProposerSlashings: genProposerSlashings(5),
		AttesterSlashings: genAttesterSlashings(5),
		Attestations:      genAttestations(10),
		Deposits:          genDeposits(5),
		VoluntaryExits:    genSignedVoluntaryExits(12),
		SyncAggregate:     genSyncAggregate(),
	}
}

func genBeaconBlockAltair() *v1alpha1.BeaconBlockAltair {
	return &v1alpha1.BeaconBlockAltair{
		Slot:          123455,
		ProposerIndex: 55433,
		ParentRoot:    bytes(32),
		StateRoot:     bytes(32),
		Body:          genBeaconBlockBodyAltair(),
	}
}

func genSignedBeaconBlockAltair() *v1alpha1.SignedBeaconBlockAltair {
	return &v1alpha1.SignedBeaconBlockAltair{
		Block:     genBeaconBlockAltair(),
		Signature: bytes(32),
	}
}

func genBeaconBlockBodyBellatrix() *v1alpha1.BeaconBlockBodyBellatrix {
	return &v1alpha1.BeaconBlockBodyBellatrix{
		RandaoReveal:      bytes(32),
		Eth1Data:          genEth1Data(),
		Graffiti:          bytes(32),
		ProposerSlashings: genProposerSlashings(5),
		AttesterSlashings: genAttesterSlashings(5),
		Attestations:      genAttestations(10),
		Deposits:          genDeposits(5),
		VoluntaryExits:    genSignedVoluntaryExits(12),
		SyncAggregate:     genSyncAggregate(),
		ExecutionPayload:  genPayload(),
	}
}

func genBeaconBlockBellatrix() *v1alpha1.BeaconBlockBellatrix {
	return &v1alpha1.BeaconBlockBellatrix{
		Slot:          123455,
		ProposerIndex: 55433,
		ParentRoot:    bytes(32),
		StateRoot:     bytes(32),
		Body:          genBeaconBlockBodyBellatrix(),
	}
}

func genSignedBeaconBlockBellatrix() *v1alpha1.SignedBeaconBlockBellatrix {
	return &v1alpha1.SignedBeaconBlockBellatrix{
		Block:     genBeaconBlockBellatrix(),
		Signature: bytes(32),
	}
}

func genBeaconBlockBodyCapella() *v1alpha1.BeaconBlockBodyCapella {
	return &v1alpha1.BeaconBlockBodyCapella{
		RandaoReveal:          bytes(96),
		Eth1Data:              genEth1Data(),
		Graffiti:              bytes(32),
		ProposerSlashings:     genProposerSlashings(5),
		AttesterSlashings:     genAttesterSlashings(5),
		Attestations:          genAttestations(10),
		Deposits:              genDeposits(5),
		VoluntaryExits:        genSignedVoluntaryExits(12),
		SyncAggregate:         genSyncAggregate(),
		ExecutionPayload:      genPayloadCapella(),
		BlsToExecutionChanges: genBLSToExecutionChanges(10),
	}
}

func genBeaconBlockCapella() *v1alpha1.BeaconBlockCapella {
	return &v1alpha1.BeaconBlockCapella{
		Slot:          123455,
		ProposerIndex: 55433,
		ParentRoot:    bytes(32),
		StateRoot:     bytes(32),
		Body:          genBeaconBlockBodyCapella(),
	}
}

func genSignedBeaconBlockCapella() *v1alpha1.SignedBeaconBlockCapella {
	return &v1alpha1.SignedBeaconBlockCapella{
		Block:     genBeaconBlockCapella(),
		Signature: bytes(96),
	}
}

func genBlindedBeaconBlockBodyCapella() *v1alpha1.BlindedBeaconBlockBodyCapella {
	return &v1alpha1.BlindedBeaconBlockBodyCapella{
		RandaoReveal:           bytes(96),
		Eth1Data:               genEth1Data(),
		Graffiti:               bytes(32),
		ProposerSlashings:      genProposerSlashings(5),
		AttesterSlashings:      genAttesterSlashings(5),
		Attestations:           genAttestations(10),
		Deposits:               genDeposits(5),
		VoluntaryExits:         genSignedVoluntaryExits(12),
		SyncAggregate:          genSyncAggregate(),
		ExecutionPayloadHeader: genPayloadHeaderCapella(),
		BlsToExecutionChanges:  genBLSToExecutionChanges(10),
	}
}

func genBlindedBeaconBlockCapella() *v1alpha1.BlindedBeaconBlockCapella {
	return &v1alpha1.BlindedBeaconBlockCapella{
		Slot:          123455,
		ProposerIndex: 55433,
		ParentRoot:    bytes(32),
		StateRoot:     bytes(32),
		Body:          genBlindedBeaconBlockBodyCapella(),
	}
}

func genSignedBlindedBeaconBlockCapella() *v1alpha1.SignedBlindedBeaconBlockCapella {
	return &v1alpha1.SignedBlindedBeaconBlockCapella{
		Block:     genBlindedBeaconBlockCapella(),
		Signature: bytes(32),
	}
}

func genBeaconBlockBodyDeneb() *v1alpha1.BeaconBlockBodyDeneb {
	return &v1alpha1.BeaconBlockBodyDeneb{
		RandaoReveal:          bytes(96),
		Eth1Data:              genEth1Data(),
		Graffiti:              bytes(32),
		ProposerSlashings:     genProposerSlashings(5),
		AttesterSlashings:     genAttesterSlashings(5),
		Attestations:          genAttestations(10),
		Deposits:              genDeposits(5),
		VoluntaryExits:        genSignedVoluntaryExits(12),
		SyncAggregate:         genSyncAggregate(),
		ExecutionPayload:      genPayloadDeneb(),
		BlsToExecutionChanges: genBLSToExecutionChanges(10),
		BlobKzgCommitments:    getKZGCommitments(4),
	}
}

func genBeaconBlockDeneb() *v1alpha1.BeaconBlockDeneb {
	return &v1alpha1.BeaconBlockDeneb{
		Slot:          123455,
		ProposerIndex: 55433,
		ParentRoot:    bytes(32),
		StateRoot:     bytes(32),
		Body:          genBeaconBlockBodyDeneb(),
	}
}

func genSignedBeaconBlockDeneb() *v1alpha1.SignedBeaconBlockDeneb {
	return &v1alpha1.SignedBeaconBlockDeneb{
		Block:     genBeaconBlockDeneb(),
		Signature: bytes(96),
	}
}

func genBlindedBeaconBlockBodyDeneb() *v1alpha1.BlindedBeaconBlockBodyDeneb {
	return &v1alpha1.BlindedBeaconBlockBodyDeneb{
		RandaoReveal:           bytes(96),
		Eth1Data:               genEth1Data(),
		Graffiti:               bytes(32),
		ProposerSlashings:      genProposerSlashings(5),
		AttesterSlashings:      genAttesterSlashings(5),
		Attestations:           genAttestations(10),
		Deposits:               genDeposits(5),
		VoluntaryExits:         genSignedVoluntaryExits(12),
		SyncAggregate:          genSyncAggregate(),
		ExecutionPayloadHeader: genPayloadHeaderDeneb(),
		BlsToExecutionChanges:  genBLSToExecutionChanges(10),
		BlobKzgCommitments:     getKZGCommitments(4),
	}
}

func getKZGCommitments(n int) [][]byte {
	kzgs := make([][]byte, n)
	for i := 0; i < n; i++ {
		kzgs[i] = bytes(48)
	}
	return kzgs
}

func genBlindedBeaconBlockDeneb() *v1alpha1.BlindedBeaconBlockDeneb {
	return &v1alpha1.BlindedBeaconBlockDeneb{
		Slot:          123455,
		ProposerIndex: 55433,
		ParentRoot:    bytes(32),
		StateRoot:     bytes(32),
		Body:          genBlindedBeaconBlockBodyDeneb(),
	}
}

func genSignedBlindedBeaconBlockDeneb() *v1alpha1.SignedBlindedBeaconBlockDeneb {
	return &v1alpha1.SignedBlindedBeaconBlockDeneb{
		Message:   genBlindedBeaconBlockDeneb(),
		Signature: bytes(32),
	}
}

func genSyncCommitteeMessage() *v1alpha1.SyncCommitteeMessage {
	return &v1alpha1.SyncCommitteeMessage{
		Slot:           424555,
		BlockRoot:      bytes(32),
		ValidatorIndex: 5443,
		Signature:      bytes(32),
	}
}

func genPayload() *enginev1.ExecutionPayload {
	return &enginev1.ExecutionPayload{
		ParentHash:    bytes(32),
		FeeRecipient:  bytes(32),
		StateRoot:     bytes(32),
		ReceiptsRoot:  bytes(32),
		LogsBloom:     bytes(32),
		PrevRandao:    bytes(32),
		BlockNumber:   1,
		GasLimit:      2,
		GasUsed:       3,
		Timestamp:     4,
		ExtraData:     bytes(32),
		BaseFeePerGas: bytes(32),
		BlockHash:     bytes(32),
		Transactions:  [][]byte{{'a'}, {'b'}, {'c'}},
	}
}

func genPayloadCapella() *enginev1.ExecutionPayloadCapella {
	return &enginev1.ExecutionPayloadCapella{
		ParentHash:    bytes(32),
		FeeRecipient:  bytes(20),
		StateRoot:     bytes(32),
		ReceiptsRoot:  bytes(32),
		LogsBloom:     bytes(256),
		PrevRandao:    bytes(32),
		BlockNumber:   1,
		GasLimit:      2,
		GasUsed:       3,
		Timestamp:     4,
		ExtraData:     bytes(32),
		BaseFeePerGas: bytes(32),
		BlockHash:     bytes(32),
		Transactions:  [][]byte{{'a'}, {'b'}, {'c'}},
		Withdrawals: []*enginev1.Withdrawal{
			{
				Index:          123,
				ValidatorIndex: 123,
				Address:        bytes(20),
				Amount:         123,
			},
			{
				Index:          124,
				ValidatorIndex: 456,
				Address:        bytes(20),
				Amount:         456,
			},
		},
	}
}

func genPayloadDeneb() *enginev1.ExecutionPayloadDeneb {
	return &enginev1.ExecutionPayloadDeneb{
		ParentHash:    bytes(32),
		FeeRecipient:  bytes(20),
		StateRoot:     bytes(32),
		ReceiptsRoot:  bytes(32),
		LogsBloom:     bytes(256),
		PrevRandao:    bytes(32),
		BlockNumber:   1,
		GasLimit:      2,
		GasUsed:       3,
		Timestamp:     4,
		ExtraData:     bytes(32),
		BaseFeePerGas: bytes(32),
		BlockHash:     bytes(32),
		Transactions:  [][]byte{{'a'}, {'b'}, {'c'}},
		Withdrawals: []*enginev1.Withdrawal{
			{
				Index:          123,
				ValidatorIndex: 123,
				Address:        bytes(20),
				Amount:         123,
			},
			{
				Index:          124,
				ValidatorIndex: 456,
				Address:        bytes(20),
				Amount:         456,
			},
		},
		BlobGasUsed:   5,
		ExcessBlobGas: 6,
	}
}

func genPayloadHeader() *enginev1.ExecutionPayloadHeader {
	return &enginev1.ExecutionPayloadHeader{
		ParentHash:       bytes(32),
		FeeRecipient:     bytes(32),
		StateRoot:        bytes(32),
		ReceiptsRoot:     bytes(32),
		LogsBloom:        bytes(32),
		PrevRandao:       bytes(32),
		BlockNumber:      1,
		GasLimit:         2,
		GasUsed:          3,
		Timestamp:        4,
		ExtraData:        bytes(32),
		BaseFeePerGas:    bytes(32),
		BlockHash:        bytes(32),
		TransactionsRoot: bytes(32),
	}
}

func genPayloadHeaderCapella() *enginev1.ExecutionPayloadHeaderCapella {
	return &enginev1.ExecutionPayloadHeaderCapella{
		ParentHash:       bytes(32),
		FeeRecipient:     bytes(20),
		StateRoot:        bytes(32),
		ReceiptsRoot:     bytes(32),
		LogsBloom:        bytes(256),
		PrevRandao:       bytes(32),
		BlockNumber:      1,
		GasLimit:         2,
		GasUsed:          3,
		Timestamp:        4,
		ExtraData:        bytes(32),
		BaseFeePerGas:    bytes(32),
		BlockHash:        bytes(32),
		TransactionsRoot: bytes(32),
		WithdrawalsRoot:  bytes(32),
	}
}

func genPayloadHeaderDeneb() *enginev1.ExecutionPayloadHeaderDeneb {
	return &enginev1.ExecutionPayloadHeaderDeneb{
		ParentHash:       bytes(32),
		FeeRecipient:     bytes(20),
		StateRoot:        bytes(32),
		ReceiptsRoot:     bytes(32),
		LogsBloom:        bytes(256),
		PrevRandao:       bytes(32),
		BlockNumber:      1,
		GasLimit:         2,
		GasUsed:          3,
		Timestamp:        4,
		ExtraData:        bytes(32),
		BaseFeePerGas:    bytes(32),
		BlockHash:        bytes(32),
		TransactionsRoot: bytes(32),
		WithdrawalsRoot:  bytes(32),
		BlobGasUsed:      5,
		ExcessBlobGas:    6,
	}
}

func genWithdrawals(num int) []*enginev1.Withdrawal {
	ws := make([]*enginev1.Withdrawal, num)
	for i := 0; i < num; i++ {
		ws[i] = genWithdrawal()
	}
	return ws
}

func genWithdrawal() *enginev1.Withdrawal {
	return &enginev1.Withdrawal{
		Index:          123456,
		ValidatorIndex: 654321,
		Address:        bytes(20),
		Amount:         55555,
	}
}

func genBLSToExecutionChanges(num int) []*v1alpha1.SignedBLSToExecutionChange {
	changes := make([]*v1alpha1.SignedBLSToExecutionChange, num)
	for i := 0; i < num; i++ {
		changes[i] = genBLSToExecutionChange()
	}
	return changes
}

func genBLSToExecutionChange() *v1alpha1.SignedBLSToExecutionChange {
	return &v1alpha1.SignedBLSToExecutionChange{
		Message: &v1alpha1.BLSToExecutionChange{
			ValidatorIndex:     123456,
			FromBlsPubkey:      bytes(48),
			ToExecutionAddress: bytes(20),
		},
		Signature: bytes(96),
	}
}
