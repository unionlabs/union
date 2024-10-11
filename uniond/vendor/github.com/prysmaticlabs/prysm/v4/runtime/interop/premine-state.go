package interop

import (
	"context"
	"math/big"

	"github.com/ethereum/go-ethereum/core/types"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	b "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stateutil"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

var errUnsupportedVersion = errors.New("schema version not supported by PremineGenesisConfig")

type PremineGenesisConfig struct {
	GenesisTime     uint64
	NVals           uint64
	PregenesisCreds uint64
	Version         int          // as in "github.com/prysmaticlabs/prysm/v4/runtime/version"
	GB              *types.Block // geth genesis block
	depositEntries  *depositEntries
}

type depositEntries struct {
	dds   []*ethpb.Deposit_Data
	roots [][]byte
}

type PremineGenesisOpt func(*PremineGenesisConfig)

func WithDepositData(dds []*ethpb.Deposit_Data, roots [][]byte) PremineGenesisOpt {
	return func(cfg *PremineGenesisConfig) {
		cfg.depositEntries = &depositEntries{
			dds:   dds,
			roots: roots,
		}
	}
}

// NewPreminedGenesis creates a genesis BeaconState at the given fork version, suitable for using as an e2e genesis.
func NewPreminedGenesis(ctx context.Context, t, nvals, pCreds uint64, version int, gb *types.Block, opts ...PremineGenesisOpt) (state.BeaconState, error) {
	cfg := &PremineGenesisConfig{
		GenesisTime:     t,
		NVals:           nvals,
		PregenesisCreds: pCreds,
		Version:         version,
		GB:              gb,
	}
	for _, o := range opts {
		o(cfg)
	}
	return cfg.prepare(ctx)
}

func (s *PremineGenesisConfig) prepare(ctx context.Context) (state.BeaconState, error) {
	switch s.Version {
	case version.Phase0, version.Altair, version.Bellatrix, version.Capella, version.Deneb:
	default:
		return nil, errors.Wrapf(errUnsupportedVersion, "version=%s", version.String(s.Version))
	}

	st, err := s.empty()
	if err != nil {
		return nil, err
	}
	if err = s.processDeposits(ctx, st); err != nil {
		return nil, err
	}
	if err = s.populate(st); err != nil {
		return nil, err
	}

	return st, nil
}

func (s *PremineGenesisConfig) empty() (state.BeaconState, error) {
	var e state.BeaconState
	var err error

	bRoots := make([][]byte, fieldparams.BlockRootsLength)
	for i := range bRoots {
		bRoots[i] = bytesutil.PadTo([]byte{}, 32)
	}
	sRoots := make([][]byte, fieldparams.StateRootsLength)
	for i := range sRoots {
		sRoots[i] = bytesutil.PadTo([]byte{}, 32)
	}
	mixes := make([][]byte, fieldparams.RandaoMixesLength)
	for i := range mixes {
		mixes[i] = bytesutil.PadTo([]byte{}, 32)
	}

	switch s.Version {
	case version.Phase0:
		e, err = state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
			BlockRoots:  bRoots,
			StateRoots:  sRoots,
			RandaoMixes: mixes,
			Balances:    []uint64{},
			Validators:  []*ethpb.Validator{},
		})
		if err != nil {
			return nil, err
		}
	case version.Altair:
		e, err = state_native.InitializeFromProtoAltair(&ethpb.BeaconStateAltair{
			BlockRoots:       bRoots,
			StateRoots:       sRoots,
			RandaoMixes:      mixes,
			Balances:         []uint64{},
			InactivityScores: []uint64{},
			Validators:       []*ethpb.Validator{},
		})
		if err != nil {
			return nil, err
		}
	case version.Bellatrix:
		e, err = state_native.InitializeFromProtoBellatrix(&ethpb.BeaconStateBellatrix{
			BlockRoots:       bRoots,
			StateRoots:       sRoots,
			RandaoMixes:      mixes,
			Balances:         []uint64{},
			InactivityScores: []uint64{},
			Validators:       []*ethpb.Validator{},
		})
		if err != nil {
			return nil, err
		}
	case version.Capella:
		e, err = state_native.InitializeFromProtoCapella(&ethpb.BeaconStateCapella{
			BlockRoots:       bRoots,
			StateRoots:       sRoots,
			RandaoMixes:      mixes,
			Balances:         []uint64{},
			InactivityScores: []uint64{},
			Validators:       []*ethpb.Validator{},
		})
		if err != nil {
			return nil, err
		}
	case version.Deneb:
		e, err = state_native.InitializeFromProtoDeneb(&ethpb.BeaconStateDeneb{})
		if err != nil {
			return nil, err
		}
	default:
		return nil, errUnsupportedVersion
	}
	if err = e.SetSlot(0); err != nil {
		return nil, err
	}
	if err = e.SetJustificationBits([]byte{0}); err != nil {
		return nil, err
	}
	if err = e.SetHistoricalRoots([][]byte{}); err != nil {
		return nil, err
	}
	zcp := &ethpb.Checkpoint{
		Epoch: 0,
		Root:  params.BeaconConfig().ZeroHash[:],
	}
	if err = e.SetPreviousJustifiedCheckpoint(zcp); err != nil {
		return nil, err
	}
	if err = e.SetCurrentJustifiedCheckpoint(zcp); err != nil {
		return nil, err
	}
	if err = e.SetFinalizedCheckpoint(zcp); err != nil {
		return nil, err
	}
	if err = e.SetEth1DataVotes([]*ethpb.Eth1Data{}); err != nil {
		return nil, err
	}
	if s.Version == version.Phase0 {
		if err = e.SetCurrentEpochAttestations([]*ethpb.PendingAttestation{}); err != nil {
			return nil, err
		}
		if err = e.SetPreviousEpochAttestations([]*ethpb.PendingAttestation{}); err != nil {
			return nil, err
		}
	}
	return e.Copy(), nil
}

func (s *PremineGenesisConfig) processDeposits(ctx context.Context, g state.BeaconState) error {
	deposits, err := s.deposits()
	if err != nil {
		return err
	}
	if err = s.setEth1Data(g); err != nil {
		return err
	}
	if _, err = helpers.UpdateGenesisEth1Data(g, deposits, g.Eth1Data()); err != nil {
		return err
	}
	_, err = b.ProcessPreGenesisDeposits(ctx, g, deposits)
	if err != nil {
		return errors.Wrap(err, "could not process validator deposits")
	}
	return nil
}

func (s *PremineGenesisConfig) deposits() ([]*ethpb.Deposit, error) {
	if s.depositEntries == nil {
		prv, pub, err := s.keys()
		if err != nil {
			return nil, err
		}
		dds, roots, err := DepositDataFromKeysWithExecCreds(prv, pub, s.PregenesisCreds)
		if err != nil {
			return nil, errors.Wrap(err, "could not generate deposit data from keys")
		}
		s.depositEntries = &depositEntries{
			dds:   dds,
			roots: roots,
		}
	}

	t, err := trie.GenerateTrieFromItems(s.depositEntries.roots, params.BeaconConfig().DepositContractTreeDepth)
	if err != nil {
		return nil, errors.Wrap(err, "could not generate Merkle trie for deposit proofs")
	}
	deposits, err := GenerateDepositsFromData(s.depositEntries.dds, t)
	if err != nil {
		return nil, errors.Wrap(err, "could not generate deposits from the deposit data provided")
	}
	return deposits, nil
}

func (s *PremineGenesisConfig) keys() ([]bls.SecretKey, []bls.PublicKey, error) {
	prv, pub, err := DeterministicallyGenerateKeys(0, s.NVals)
	if err != nil {
		return nil, nil, errors.Wrapf(err, "could not deterministically generate keys for %d validators", s.NVals)
	}
	return prv, pub, nil
}

func (s *PremineGenesisConfig) setEth1Data(g state.BeaconState) error {
	if err := g.SetEth1DepositIndex(0); err != nil {
		return err
	}
	dr, err := emptyDepositRoot()
	if err != nil {
		return err
	}
	return g.SetEth1Data(&ethpb.Eth1Data{DepositRoot: dr[:], BlockHash: s.GB.Hash().Bytes()})
}

func emptyDepositRoot() ([32]byte, error) {
	t, err := trie.NewTrie(params.BeaconConfig().DepositContractTreeDepth)
	if err != nil {
		return [32]byte{}, err
	}
	return t.HashTreeRoot()
}

func (s *PremineGenesisConfig) populate(g state.BeaconState) error {
	if err := g.SetGenesisTime(s.GenesisTime); err != nil {
		return err
	}
	if err := s.setGenesisValidatorsRoot(g); err != nil {
		return err
	}
	if err := s.setFork(g); err != nil {
		return err
	}
	rao := nSetRoots(uint64(params.BeaconConfig().EpochsPerHistoricalVector), s.GB.Hash().Bytes())
	if err := g.SetRandaoMixes(rao); err != nil {
		return err
	}
	if err := g.SetBlockRoots(nZeroRoots(uint64(params.BeaconConfig().SlotsPerHistoricalRoot))); err != nil {
		return err
	}
	if err := g.SetStateRoots(nZeroRoots(uint64(params.BeaconConfig().SlotsPerHistoricalRoot))); err != nil {
		return err
	}
	if err := g.SetSlashings(make([]uint64, params.BeaconConfig().EpochsPerSlashingsVector)); err != nil {
		return err
	}
	if err := s.setLatestBlockHeader(g); err != nil {
		return err
	}
	if err := s.setInactivityScores(g); err != nil {
		return err
	}
	if err := s.setCurrentEpochParticipation(g); err != nil {
		return err
	}
	if err := s.setPrevEpochParticipation(g); err != nil {
		return err
	}
	if err := s.setSyncCommittees(g); err != nil {
		return err
	}
	if err := s.setExecutionPayload(g); err != nil {
		return err
	}

	// For pre-mined genesis, we want to keep the deposit root set to the root of an empty trie.
	// This needs to be set again because the methods used by processDeposits mutate the state's eth1data.
	return s.setEth1Data(g)
}

func (s *PremineGenesisConfig) setGenesisValidatorsRoot(g state.BeaconState) error {
	vroot, err := stateutil.ValidatorRegistryRoot(g.Validators())
	if err != nil {
		return err
	}
	return g.SetGenesisValidatorsRoot(vroot[:])
}

func (s *PremineGenesisConfig) setFork(g state.BeaconState) error {
	var pv, cv []byte
	switch s.Version {
	case version.Phase0:
		pv, cv = params.BeaconConfig().GenesisForkVersion, params.BeaconConfig().GenesisForkVersion
	case version.Altair:
		pv, cv = params.BeaconConfig().GenesisForkVersion, params.BeaconConfig().AltairForkVersion
	case version.Bellatrix:
		pv, cv = params.BeaconConfig().AltairForkVersion, params.BeaconConfig().BellatrixForkVersion
	case version.Capella:
		pv, cv = params.BeaconConfig().BellatrixForkVersion, params.BeaconConfig().CapellaForkVersion
	case version.Deneb:
		pv, cv = params.BeaconConfig().CapellaForkVersion, params.BeaconConfig().DenebForkVersion
	default:
		return errUnsupportedVersion
	}
	fork := &ethpb.Fork{
		PreviousVersion: pv,
		CurrentVersion:  cv,
		Epoch:           0,
	}
	return g.SetFork(fork)
}

func (s *PremineGenesisConfig) setInactivityScores(g state.BeaconState) error {
	if s.Version < version.Altair {
		return nil
	}

	scores, err := g.InactivityScores()
	if err != nil {
		return err
	}
	scoresMissing := len(g.Validators()) - len(scores)
	if scoresMissing > 0 {
		for i := 0; i < scoresMissing; i++ {
			scores = append(scores, 0)
		}
	}
	return g.SetInactivityScores(scores)
}

func (s *PremineGenesisConfig) setCurrentEpochParticipation(g state.BeaconState) error {
	if s.Version < version.Altair {
		return nil
	}

	p, err := g.CurrentEpochParticipation()
	if err != nil {
		return err
	}
	missing := len(g.Validators()) - len(p)
	if missing > 0 {
		for i := 0; i < missing; i++ {
			p = append(p, 0)
		}
	}
	return g.SetCurrentParticipationBits(p)
}

func (s *PremineGenesisConfig) setPrevEpochParticipation(g state.BeaconState) error {
	if s.Version < version.Altair {
		return nil
	}

	p, err := g.PreviousEpochParticipation()
	if err != nil {
		return err
	}
	missing := len(g.Validators()) - len(p)
	if missing > 0 {
		for i := 0; i < missing; i++ {
			p = append(p, 0)
		}
	}
	return g.SetPreviousParticipationBits(p)
}

func (s *PremineGenesisConfig) setSyncCommittees(g state.BeaconState) error {
	if s.Version < version.Altair {
		return nil
	}
	sc, err := altair.NextSyncCommittee(context.Background(), g)
	if err != nil {
		return err
	}
	if err = g.SetNextSyncCommittee(sc); err != nil {
		return err
	}
	return g.SetCurrentSyncCommittee(sc)
}

type rooter interface {
	HashTreeRoot() ([32]byte, error)
}

func (s *PremineGenesisConfig) setLatestBlockHeader(g state.BeaconState) error {
	var body rooter
	switch s.Version {
	case version.Phase0:
		body = &ethpb.BeaconBlockBody{
			RandaoReveal: make([]byte, 96),
			Eth1Data: &ethpb.Eth1Data{
				DepositRoot: make([]byte, 32),
				BlockHash:   make([]byte, 32),
			},
			Graffiti: make([]byte, 32),
		}
	case version.Altair:
		body = &ethpb.BeaconBlockBodyAltair{
			RandaoReveal: make([]byte, 96),
			Eth1Data: &ethpb.Eth1Data{
				DepositRoot: make([]byte, 32),
				BlockHash:   make([]byte, 32),
			},
			Graffiti: make([]byte, 32),
			SyncAggregate: &ethpb.SyncAggregate{
				SyncCommitteeBits:      make([]byte, fieldparams.SyncCommitteeLength/8),
				SyncCommitteeSignature: make([]byte, fieldparams.BLSSignatureLength),
			},
		}
	case version.Bellatrix:
		body = &ethpb.BeaconBlockBodyBellatrix{
			RandaoReveal: make([]byte, 96),
			Eth1Data: &ethpb.Eth1Data{
				DepositRoot: make([]byte, 32),
				BlockHash:   make([]byte, 32),
			},
			Graffiti: make([]byte, 32),
			SyncAggregate: &ethpb.SyncAggregate{
				SyncCommitteeBits:      make([]byte, fieldparams.SyncCommitteeLength/8),
				SyncCommitteeSignature: make([]byte, fieldparams.BLSSignatureLength),
			},
			ExecutionPayload: &enginev1.ExecutionPayload{
				ParentHash:    make([]byte, 32),
				FeeRecipient:  make([]byte, 20),
				StateRoot:     make([]byte, 32),
				ReceiptsRoot:  make([]byte, 32),
				LogsBloom:     make([]byte, 256),
				PrevRandao:    make([]byte, 32),
				ExtraData:     make([]byte, 0),
				BaseFeePerGas: make([]byte, 32),
				BlockHash:     make([]byte, 32),
				Transactions:  make([][]byte, 0),
			},
		}
	case version.Capella:
		body = &ethpb.BeaconBlockBodyCapella{
			RandaoReveal: make([]byte, 96),
			Eth1Data: &ethpb.Eth1Data{
				DepositRoot: make([]byte, 32),
				BlockHash:   make([]byte, 32),
			},
			Graffiti: make([]byte, 32),
			SyncAggregate: &ethpb.SyncAggregate{
				SyncCommitteeBits:      make([]byte, fieldparams.SyncCommitteeLength/8),
				SyncCommitteeSignature: make([]byte, fieldparams.BLSSignatureLength),
			},
			ExecutionPayload: &enginev1.ExecutionPayloadCapella{
				ParentHash:    make([]byte, 32),
				FeeRecipient:  make([]byte, 20),
				StateRoot:     make([]byte, 32),
				ReceiptsRoot:  make([]byte, 32),
				LogsBloom:     make([]byte, 256),
				PrevRandao:    make([]byte, 32),
				ExtraData:     make([]byte, 0),
				BaseFeePerGas: make([]byte, 32),
				BlockHash:     make([]byte, 32),
				Transactions:  make([][]byte, 0),
				Withdrawals:   make([]*enginev1.Withdrawal, 0),
			},
			BlsToExecutionChanges: make([]*ethpb.SignedBLSToExecutionChange, 0),
		}
	case version.Deneb:
		body = &ethpb.BeaconBlockBodyDeneb{
			RandaoReveal: make([]byte, 96),
			Eth1Data: &ethpb.Eth1Data{
				DepositRoot: make([]byte, 32),
				BlockHash:   make([]byte, 32),
			},
			Graffiti: make([]byte, 32),
			SyncAggregate: &ethpb.SyncAggregate{
				SyncCommitteeBits:      make([]byte, fieldparams.SyncCommitteeLength/8),
				SyncCommitteeSignature: make([]byte, fieldparams.BLSSignatureLength),
			},
			ExecutionPayload: &enginev1.ExecutionPayloadDeneb{
				ParentHash:    make([]byte, 32),
				FeeRecipient:  make([]byte, 20),
				StateRoot:     make([]byte, 32),
				ReceiptsRoot:  make([]byte, 32),
				LogsBloom:     make([]byte, 256),
				PrevRandao:    make([]byte, 32),
				ExtraData:     make([]byte, 0),
				BaseFeePerGas: make([]byte, 32),
				BlockHash:     make([]byte, 32),
				Transactions:  make([][]byte, 0),
				Withdrawals:   make([]*enginev1.Withdrawal, 0),
			},
			BlsToExecutionChanges: make([]*ethpb.SignedBLSToExecutionChange, 0),
			BlobKzgCommitments:    make([][]byte, 0),
		}
	default:
		return errUnsupportedVersion
	}

	root, err := body.HashTreeRoot()
	if err != nil {
		return errors.Wrap(err, "could not hash tree root empty block body")
	}
	lbh := &ethpb.BeaconBlockHeader{
		ParentRoot: params.BeaconConfig().ZeroHash[:],
		StateRoot:  params.BeaconConfig().ZeroHash[:],
		BodyRoot:   root[:],
	}
	return g.SetLatestBlockHeader(lbh)
}

func (s *PremineGenesisConfig) setExecutionPayload(g state.BeaconState) error {
	if s.Version < version.Bellatrix {
		return nil
	}

	gb := s.GB

	var ed interfaces.ExecutionData
	switch s.Version {
	case version.Bellatrix:
		payload := &enginev1.ExecutionPayload{
			ParentHash:    gb.ParentHash().Bytes(),
			FeeRecipient:  gb.Coinbase().Bytes(),
			StateRoot:     gb.Root().Bytes(),
			ReceiptsRoot:  gb.ReceiptHash().Bytes(),
			LogsBloom:     gb.Bloom().Bytes(),
			PrevRandao:    params.BeaconConfig().ZeroHash[:],
			BlockNumber:   gb.NumberU64(),
			GasLimit:      gb.GasLimit(),
			GasUsed:       gb.GasUsed(),
			Timestamp:     gb.Time(),
			ExtraData:     gb.Extra()[:32],
			BaseFeePerGas: bytesutil.PadTo(bytesutil.ReverseByteOrder(gb.BaseFee().Bytes()), fieldparams.RootLength),
			BlockHash:     gb.Hash().Bytes(),
			Transactions:  make([][]byte, 0),
		}
		wep, err := blocks.WrappedExecutionPayload(payload)
		if err != nil {
			return err
		}
		eph, err := blocks.PayloadToHeader(wep)
		if err != nil {
			return err
		}
		ed, err = blocks.WrappedExecutionPayloadHeader(eph)
		if err != nil {
			return err
		}
	case version.Capella:
		payload := &enginev1.ExecutionPayloadCapella{
			ParentHash:    gb.ParentHash().Bytes(),
			FeeRecipient:  gb.Coinbase().Bytes(),
			StateRoot:     gb.Root().Bytes(),
			ReceiptsRoot:  gb.ReceiptHash().Bytes(),
			LogsBloom:     gb.Bloom().Bytes(),
			PrevRandao:    params.BeaconConfig().ZeroHash[:],
			BlockNumber:   gb.NumberU64(),
			GasLimit:      gb.GasLimit(),
			GasUsed:       gb.GasUsed(),
			Timestamp:     gb.Time(),
			ExtraData:     gb.Extra()[:32],
			BaseFeePerGas: bytesutil.PadTo(bytesutil.ReverseByteOrder(gb.BaseFee().Bytes()), fieldparams.RootLength),
			BlockHash:     gb.Hash().Bytes(),
			Transactions:  make([][]byte, 0),
			Withdrawals:   make([]*enginev1.Withdrawal, 0),
		}
		wep, err := blocks.WrappedExecutionPayloadCapella(payload, big.NewInt(0))
		if err != nil {
			return err
		}
		eph, err := blocks.PayloadToHeaderCapella(wep)
		if err != nil {
			return err
		}
		ed, err = blocks.WrappedExecutionPayloadHeaderCapella(eph, big.NewInt(0))
		if err != nil {
			return err
		}
	case version.Deneb:
		payload := &enginev1.ExecutionPayloadDeneb{
			ParentHash:    gb.ParentHash().Bytes(),
			FeeRecipient:  gb.Coinbase().Bytes(),
			StateRoot:     gb.Root().Bytes(),
			ReceiptsRoot:  gb.ReceiptHash().Bytes(),
			LogsBloom:     gb.Bloom().Bytes(),
			PrevRandao:    params.BeaconConfig().ZeroHash[:],
			BlockNumber:   gb.NumberU64(),
			GasLimit:      gb.GasLimit(),
			GasUsed:       gb.GasUsed(),
			Timestamp:     gb.Time(),
			ExtraData:     gb.Extra()[:32],
			BaseFeePerGas: bytesutil.PadTo(bytesutil.ReverseByteOrder(gb.BaseFee().Bytes()), fieldparams.RootLength),
			BlockHash:     gb.Hash().Bytes(),
			Transactions:  make([][]byte, 0),
			Withdrawals:   make([]*enginev1.Withdrawal, 0),
			ExcessBlobGas: *gb.ExcessBlobGas(),
			BlobGasUsed:   *gb.BlobGasUsed(),
		}
		wep, err := blocks.WrappedExecutionPayloadDeneb(payload, big.NewInt(0))
		if err != nil {
			return err
		}
		eph, err := blocks.PayloadToHeaderDeneb(wep)
		if err != nil {
			return err
		}
		ed, err = blocks.WrappedExecutionPayloadHeaderDeneb(eph, big.NewInt(0))
		if err != nil {
			return err
		}
	default:
		return errUnsupportedVersion
	}
	return g.SetLatestExecutionPayloadHeader(ed)
}

func nZeroRoots(n uint64) [][]byte {
	roots := make([][]byte, n)
	zh := params.BeaconConfig().ZeroHash[:]
	for i := uint64(0); i < n; i++ {
		roots[i] = zh
	}
	return roots
}

func nSetRoots(n uint64, r []byte) [][]byte {
	roots := make([][]byte, n)
	for i := uint64(0); i < n; i++ {
		h := make([]byte, 32)
		copy(h, r)
		roots[i] = h
	}
	return roots
}
