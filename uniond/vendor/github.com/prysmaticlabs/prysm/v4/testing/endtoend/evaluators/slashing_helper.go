package evaluators

import (
	"context"
	"crypto/rand"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/protobuf/types/known/emptypb"
)

type doubleAttestationHelper struct {
	valClient    eth.BeaconNodeValidatorClient
	beaconClient eth.BeaconChainClient
	privKeys     []bls.SecretKey
	pubKeys      [][]byte
	domainResp   *eth.DomainResponse
	attData      *eth.AttestationData

	committee []primitives.ValidatorIndex
}

// Initializes helper with details needed to make a double attestation for testint purposes
// Populates the committee of that is responsible for the
func (h *doubleAttestationHelper) setup(ctx context.Context) error {
	chainHead, err := h.beaconClient.GetChainHead(ctx, &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(err, "could not get chain head")
	}
	_, privKeys, err := util.DeterministicDepositsAndKeys(params.BeaconConfig().MinGenesisActiveValidatorCount)
	if err != nil {
		return errors.Wrap(err, "could not get depositsandkeys")
	}

	pubKeys := make([][]byte, len(privKeys))
	for i, priv := range privKeys {
		pubKeys[i] = priv.PublicKey().Marshal()
	}

	duties, err := h.valClient.GetDuties(ctx, &eth.DutiesRequest{
		Epoch:      chainHead.HeadEpoch,
		PublicKeys: pubKeys,
	})

	if err != nil {
		return errors.Wrap(err, "could not get duties")
	}

	var committeeIndex primitives.CommitteeIndex
	var committee []primitives.ValidatorIndex
	for _, duty := range duties.Duties {
		if duty.AttesterSlot == chainHead.HeadSlot {
			committeeIndex = duty.CommitteeIndex
			committee = duty.Committee
			break
		}
	}
	attDataReq := &eth.AttestationDataRequest{
		CommitteeIndex: committeeIndex,
		Slot:           chainHead.HeadSlot,
	}

	attData, err := h.valClient.GetAttestationData(ctx, attDataReq)
	if err != nil {
		return err
	}

	req := &eth.DomainRequest{
		Epoch:  chainHead.HeadEpoch,
		Domain: params.BeaconConfig().DomainBeaconAttester[:],
	}

	domainResp, err := h.valClient.DomainData(ctx, req)
	if err != nil {
		return errors.Wrap(err, "could not get domain data")
	}

	h.privKeys = privKeys
	h.pubKeys = pubKeys
	h.domainResp = domainResp
	h.committee = committee
	h.attData = attData

	return nil
}

// Returns the validatorIndex at index idx of the fetched committee in setup()
func (h *doubleAttestationHelper) validatorIndexAtCommitteeIndex(idx uint64) primitives.ValidatorIndex {
	return h.committee[idx]
}

// Returns a attestation was previously submitted, at the previous slot, modifying it so that it is signed
// by the validator indicated by idx. idx represents the index in the committee of the attestation.
// The block root value is random, which allows this to be seen by P2P networks as
// new, unique blocks.
func (h *doubleAttestationHelper) getSlashableAttestation(idx uint64) (*eth.Attestation, error) {
	// msg must be unique so they are not filtered by P2P
	randVal := make([]byte, 4)
	_, err := rand.Read(randVal)
	if err != nil {
		return nil, errors.Wrap(err, "error reading random val")
	}
	blockRoot := bytesutil.ToBytes32(append(randVal, []byte("muahahahaha evil validator")...))
	h.attData.BeaconBlockRoot = blockRoot[:]

	signingRoot, err := signing.ComputeSigningRoot(h.attData, h.domainResp.SignatureDomain)
	if err != nil {
		return nil, errors.Wrap(err, "could not get chain head")
	}

	valIdx := h.validatorIndexAtCommitteeIndex(idx)

	attBitfield := bitfield.NewBitlist(uint64(len(h.committee)))
	attBitfield.SetBitAt(idx, true)
	att := &eth.Attestation{
		AggregationBits: attBitfield,
		Data:            h.attData,
		Signature:       h.privKeys[valIdx].Sign(signingRoot[:]).Marshal(),
	}
	return att, nil
}
