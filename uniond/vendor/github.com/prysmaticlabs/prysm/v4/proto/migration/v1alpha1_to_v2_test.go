package migration

import (
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpbalpha "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestV1Alpha1SignedContributionAndProofToV2(t *testing.T) {
	alphaContribution := &ethpbalpha.SignedContributionAndProof{
		Message: &ethpbalpha.ContributionAndProof{
			AggregatorIndex: validatorIndex,
			Contribution: &ethpbalpha.SyncCommitteeContribution{
				Slot:              slot,
				BlockRoot:         blockHash,
				SubcommitteeIndex: 1,
				AggregationBits:   bitfield.NewBitvector128(),
				Signature:         signature,
			},
			SelectionProof: signature,
		},
		Signature: signature,
	}
	v2Contribution := V1Alpha1SignedContributionAndProofToV2(alphaContribution)
	require.NotNil(t, v2Contribution)
	require.NotNil(t, v2Contribution.Message)
	require.NotNil(t, v2Contribution.Message.Contribution)
	assert.DeepEqual(t, signature, v2Contribution.Signature)
	msg := v2Contribution.Message
	assert.Equal(t, validatorIndex, msg.AggregatorIndex)
	assert.DeepEqual(t, signature, msg.SelectionProof)
	contrib := msg.Contribution
	assert.Equal(t, slot, contrib.Slot)
	assert.DeepEqual(t, blockHash, contrib.BeaconBlockRoot)
	assert.Equal(t, uint64(1), contrib.SubcommitteeIndex)
	assert.DeepEqual(t, bitfield.NewBitvector128(), contrib.AggregationBits)
	assert.DeepEqual(t, signature, contrib.Signature)
}

func Test_V1Alpha1BeaconBlockAltairToV2(t *testing.T) {
	alphaBlock := util.HydrateBeaconBlockAltair(&ethpbalpha.BeaconBlockAltair{})
	alphaBlock.Slot = slot
	alphaBlock.ProposerIndex = validatorIndex
	alphaBlock.ParentRoot = parentRoot
	alphaBlock.StateRoot = stateRoot
	alphaBlock.Body.RandaoReveal = randaoReveal
	alphaBlock.Body.Eth1Data = &ethpbalpha.Eth1Data{
		DepositRoot:  depositRoot,
		DepositCount: depositCount,
		BlockHash:    blockHash,
	}
	syncCommitteeBits := bitfield.NewBitvector512()
	syncCommitteeBits.SetBitAt(100, true)
	alphaBlock.Body.SyncAggregate = &ethpbalpha.SyncAggregate{
		SyncCommitteeBits:      syncCommitteeBits,
		SyncCommitteeSignature: signature,
	}

	v2Block, err := V1Alpha1BeaconBlockAltairToV2(alphaBlock)
	require.NoError(t, err)
	alphaRoot, err := alphaBlock.HashTreeRoot()
	require.NoError(t, err)
	v2Root, err := v2Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, alphaRoot, v2Root)
}

func Test_V1Alpha1BeaconBlockBellatrixToV2(t *testing.T) {
	alphaBlock := util.HydrateBeaconBlockBellatrix(&ethpbalpha.BeaconBlockBellatrix{})
	alphaBlock.Slot = slot
	alphaBlock.ProposerIndex = validatorIndex
	alphaBlock.ParentRoot = parentRoot
	alphaBlock.StateRoot = stateRoot
	alphaBlock.Body.RandaoReveal = randaoReveal
	alphaBlock.Body.Eth1Data = &ethpbalpha.Eth1Data{
		DepositRoot:  depositRoot,
		DepositCount: depositCount,
		BlockHash:    blockHash,
	}
	syncCommitteeBits := bitfield.NewBitvector512()
	syncCommitteeBits.SetBitAt(100, true)
	alphaBlock.Body.SyncAggregate = &ethpbalpha.SyncAggregate{
		SyncCommitteeBits:      syncCommitteeBits,
		SyncCommitteeSignature: signature,
	}

	v2Block, err := V1Alpha1BeaconBlockBellatrixToV2(alphaBlock)
	require.NoError(t, err)
	alphaRoot, err := alphaBlock.HashTreeRoot()
	require.NoError(t, err)
	v2Root, err := v2Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, alphaRoot, v2Root)
}

func Test_V1Alpha1BeaconBlockBellatrixToV2Blinded(t *testing.T) {
	alphaBlock := util.HydrateBeaconBlockBellatrix(&ethpbalpha.BeaconBlockBellatrix{})
	alphaBlock.Slot = slot
	alphaBlock.ProposerIndex = validatorIndex
	alphaBlock.ParentRoot = parentRoot
	alphaBlock.StateRoot = stateRoot
	alphaBlock.Body.RandaoReveal = randaoReveal
	alphaBlock.Body.Eth1Data = &ethpbalpha.Eth1Data{
		DepositRoot:  depositRoot,
		DepositCount: depositCount,
		BlockHash:    blockHash,
	}
	syncCommitteeBits := bitfield.NewBitvector512()
	syncCommitteeBits.SetBitAt(100, true)
	alphaBlock.Body.SyncAggregate = &ethpbalpha.SyncAggregate{
		SyncCommitteeBits:      syncCommitteeBits,
		SyncCommitteeSignature: signature,
	}
	alphaBlock.Body.ExecutionPayload.Transactions = [][]byte{[]byte("transaction1"), []byte("transaction2")}

	v2Block, err := V1Alpha1BeaconBlockBellatrixToV2Blinded(alphaBlock)
	require.NoError(t, err)
	alphaRoot, err := alphaBlock.HashTreeRoot()
	require.NoError(t, err)
	v2Root, err := v2Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, alphaRoot, v2Root)
}

func Test_V1Alpha1BeaconBlockCapellaToV2Blinded(t *testing.T) {
	alphaBlock := util.HydrateBeaconBlockCapella(&ethpbalpha.BeaconBlockCapella{})
	alphaBlock.Slot = slot
	alphaBlock.ProposerIndex = validatorIndex
	alphaBlock.ParentRoot = parentRoot
	alphaBlock.StateRoot = stateRoot
	alphaBlock.Body.RandaoReveal = randaoReveal
	alphaBlock.Body.Eth1Data = &ethpbalpha.Eth1Data{
		DepositRoot:  depositRoot,
		DepositCount: depositCount,
		BlockHash:    blockHash,
	}
	syncCommitteeBits := bitfield.NewBitvector512()
	syncCommitteeBits.SetBitAt(100, true)
	alphaBlock.Body.SyncAggregate = &ethpbalpha.SyncAggregate{
		SyncCommitteeBits:      syncCommitteeBits,
		SyncCommitteeSignature: signature,
	}
	alphaBlock.Body.ExecutionPayload.Transactions = [][]byte{[]byte("transaction1"), []byte("transaction2")}

	v2Block, err := V1Alpha1BeaconBlockCapellaToV2Blinded(alphaBlock)
	require.NoError(t, err)
	alphaRoot, err := alphaBlock.HashTreeRoot()
	require.NoError(t, err)
	v2Root, err := v2Block.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, alphaRoot, v2Root)
}

func TestBeaconStateAltairToProto(t *testing.T) {
	source, err := util.NewBeaconStateAltair(util.FillRootsNaturalOptAltair, func(state *ethpbalpha.BeaconStateAltair) error {
		state.GenesisTime = 1
		state.GenesisValidatorsRoot = bytesutil.PadTo([]byte("genesisvalidatorsroot"), 32)
		state.Slot = 2
		state.Fork = &ethpbalpha.Fork{
			PreviousVersion: bytesutil.PadTo([]byte("123"), 4),
			CurrentVersion:  bytesutil.PadTo([]byte("456"), 4),
			Epoch:           3,
		}
		state.LatestBlockHeader = &ethpbalpha.BeaconBlockHeader{
			Slot:          4,
			ProposerIndex: 5,
			ParentRoot:    bytesutil.PadTo([]byte("lbhparentroot"), 32),
			StateRoot:     bytesutil.PadTo([]byte("lbhstateroot"), 32),
			BodyRoot:      bytesutil.PadTo([]byte("lbhbodyroot"), 32),
		}
		state.BlockRoots = [][]byte{bytesutil.PadTo([]byte("blockroots"), 32)}
		state.StateRoots = [][]byte{bytesutil.PadTo([]byte("stateroots"), 32)}
		state.HistoricalRoots = [][]byte{bytesutil.PadTo([]byte("historicalroots"), 32)}
		state.Eth1Data = &ethpbalpha.Eth1Data{
			DepositRoot:  bytesutil.PadTo([]byte("e1ddepositroot"), 32),
			DepositCount: 6,
			BlockHash:    bytesutil.PadTo([]byte("e1dblockhash"), 32),
		}
		state.Eth1DataVotes = []*ethpbalpha.Eth1Data{{
			DepositRoot:  bytesutil.PadTo([]byte("e1dvdepositroot"), 32),
			DepositCount: 7,
			BlockHash:    bytesutil.PadTo([]byte("e1dvblockhash"), 32),
		}}
		state.Eth1DepositIndex = 8
		state.Validators = []*ethpbalpha.Validator{{
			PublicKey:                  bytesutil.PadTo([]byte("publickey"), 48),
			WithdrawalCredentials:      bytesutil.PadTo([]byte("withdrawalcredentials"), 32),
			EffectiveBalance:           9,
			Slashed:                    true,
			ActivationEligibilityEpoch: 10,
			ActivationEpoch:            11,
			ExitEpoch:                  12,
			WithdrawableEpoch:          13,
		}}
		state.Balances = []uint64{14}
		state.RandaoMixes = [][]byte{bytesutil.PadTo([]byte("randaomixes"), 32)}
		state.Slashings = []uint64{15}
		state.JustificationBits = bitfield.Bitvector4{1}
		state.PreviousJustifiedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 30,
			Root:  bytesutil.PadTo([]byte("pjcroot"), 32),
		}
		state.CurrentJustifiedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 31,
			Root:  bytesutil.PadTo([]byte("cjcroot"), 32),
		}
		state.FinalizedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 32,
			Root:  bytesutil.PadTo([]byte("fcroot"), 32),
		}
		state.PreviousEpochParticipation = []byte("previousepochparticipation")
		state.CurrentEpochParticipation = []byte("currentepochparticipation")
		state.InactivityScores = []uint64{1, 2, 3}
		state.CurrentSyncCommittee = &ethpbalpha.SyncCommittee{
			Pubkeys:         [][]byte{bytesutil.PadTo([]byte("cscpubkeys"), 48)},
			AggregatePubkey: bytesutil.PadTo([]byte("cscaggregatepubkey"), 48),
		}
		state.NextSyncCommittee = &ethpbalpha.SyncCommittee{
			Pubkeys:         [][]byte{bytesutil.PadTo([]byte("nscpubkeys"), 48)},
			AggregatePubkey: bytesutil.PadTo([]byte("nscaggregatepubkey"), 48),
		}
		return nil
	})
	require.NoError(t, err)

	result, err := BeaconStateAltairToProto(source)
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, uint64(1), result.GenesisTime)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("genesisvalidatorsroot"), 32), result.GenesisValidatorsRoot)
	assert.Equal(t, primitives.Slot(2), result.Slot)
	resultFork := result.Fork
	require.NotNil(t, resultFork)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("123"), 4), resultFork.PreviousVersion)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("456"), 4), resultFork.CurrentVersion)
	assert.Equal(t, primitives.Epoch(3), resultFork.Epoch)
	resultLatestBlockHeader := result.LatestBlockHeader
	require.NotNil(t, resultLatestBlockHeader)
	assert.Equal(t, primitives.Slot(4), resultLatestBlockHeader.Slot)
	assert.Equal(t, primitives.ValidatorIndex(5), resultLatestBlockHeader.ProposerIndex)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhparentroot"), 32), resultLatestBlockHeader.ParentRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhstateroot"), 32), resultLatestBlockHeader.StateRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhbodyroot"), 32), resultLatestBlockHeader.BodyRoot)
	assert.Equal(t, 8192, len(result.BlockRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("blockroots"), 32), result.BlockRoots[0])
	assert.Equal(t, 8192, len(result.StateRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("stateroots"), 32), result.StateRoots[0])
	assert.Equal(t, 1, len(result.HistoricalRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("historicalroots"), 32), result.HistoricalRoots[0])
	resultEth1Data := result.Eth1Data
	require.NotNil(t, resultEth1Data)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1ddepositroot"), 32), resultEth1Data.DepositRoot)
	assert.Equal(t, uint64(6), resultEth1Data.DepositCount)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dblockhash"), 32), resultEth1Data.BlockHash)
	require.Equal(t, 1, len(result.Eth1DataVotes))
	resultEth1DataVote := result.Eth1DataVotes[0]
	require.NotNil(t, resultEth1DataVote)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dvdepositroot"), 32), resultEth1DataVote.DepositRoot)
	assert.Equal(t, uint64(7), resultEth1DataVote.DepositCount)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dvblockhash"), 32), resultEth1DataVote.BlockHash)
	assert.Equal(t, uint64(8), result.Eth1DepositIndex)
	require.Equal(t, 1, len(result.Validators))
	resultValidator := result.Validators[0]
	require.NotNil(t, resultValidator)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("publickey"), 48), resultValidator.Pubkey)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("withdrawalcredentials"), 32), resultValidator.WithdrawalCredentials)
	assert.Equal(t, uint64(9), resultValidator.EffectiveBalance)
	assert.Equal(t, true, resultValidator.Slashed)
	assert.Equal(t, primitives.Epoch(10), resultValidator.ActivationEligibilityEpoch)
	assert.Equal(t, primitives.Epoch(11), resultValidator.ActivationEpoch)
	assert.Equal(t, primitives.Epoch(12), resultValidator.ExitEpoch)
	assert.Equal(t, primitives.Epoch(13), resultValidator.WithdrawableEpoch)
	assert.DeepEqual(t, []uint64{14}, result.Balances)
	assert.Equal(t, 65536, len(result.RandaoMixes))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("randaomixes"), 32), result.RandaoMixes[0])
	assert.DeepEqual(t, []uint64{15}, result.Slashings)
	assert.DeepEqual(t, bitfield.Bitvector4{1}, result.JustificationBits)
	resultPrevJustifiedCheckpoint := result.PreviousJustifiedCheckpoint
	require.NotNil(t, resultPrevJustifiedCheckpoint)
	assert.Equal(t, primitives.Epoch(30), resultPrevJustifiedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("pjcroot"), 32), resultPrevJustifiedCheckpoint.Root)
	resultCurrJustifiedCheckpoint := result.CurrentJustifiedCheckpoint
	require.NotNil(t, resultCurrJustifiedCheckpoint)
	assert.Equal(t, primitives.Epoch(31), resultCurrJustifiedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("cjcroot"), 32), resultCurrJustifiedCheckpoint.Root)
	resultFinalizedCheckpoint := result.FinalizedCheckpoint
	require.NotNil(t, resultFinalizedCheckpoint)
	assert.Equal(t, primitives.Epoch(32), resultFinalizedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("fcroot"), 32), resultFinalizedCheckpoint.Root)
	assert.DeepEqual(t, []byte("previousepochparticipation"), result.PreviousEpochParticipation)
	assert.DeepEqual(t, []byte("currentepochparticipation"), result.CurrentEpochParticipation)
	assert.DeepEqual(t, []uint64{1, 2, 3}, result.InactivityScores)
	require.NotNil(t, result.CurrentSyncCommittee)
	assert.DeepEqual(t, [][]byte{bytesutil.PadTo([]byte("cscpubkeys"), 48)}, result.CurrentSyncCommittee.Pubkeys)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("cscaggregatepubkey"), 48), result.CurrentSyncCommittee.AggregatePubkey)
	require.NotNil(t, result.NextSyncCommittee)
	assert.DeepEqual(t, [][]byte{bytesutil.PadTo([]byte("nscpubkeys"), 48)}, result.NextSyncCommittee.Pubkeys)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("nscaggregatepubkey"), 48), result.NextSyncCommittee.AggregatePubkey)
}

func TestBeaconStateBellatrixToProto(t *testing.T) {
	source, err := util.NewBeaconStateBellatrix(util.FillRootsNaturalOptBellatrix, func(state *ethpbalpha.BeaconStateBellatrix) error {
		state.GenesisTime = 1
		state.GenesisValidatorsRoot = bytesutil.PadTo([]byte("genesisvalidatorsroot"), 32)
		state.Slot = 2
		state.Fork = &ethpbalpha.Fork{
			PreviousVersion: bytesutil.PadTo([]byte("123"), 4),
			CurrentVersion:  bytesutil.PadTo([]byte("456"), 4),
			Epoch:           3,
		}
		state.LatestBlockHeader = &ethpbalpha.BeaconBlockHeader{
			Slot:          4,
			ProposerIndex: 5,
			ParentRoot:    bytesutil.PadTo([]byte("lbhparentroot"), 32),
			StateRoot:     bytesutil.PadTo([]byte("lbhstateroot"), 32),
			BodyRoot:      bytesutil.PadTo([]byte("lbhbodyroot"), 32),
		}
		state.BlockRoots = [][]byte{bytesutil.PadTo([]byte("blockroots"), 32)}
		state.StateRoots = [][]byte{bytesutil.PadTo([]byte("stateroots"), 32)}
		state.HistoricalRoots = [][]byte{bytesutil.PadTo([]byte("historicalroots"), 32)}
		state.Eth1Data = &ethpbalpha.Eth1Data{
			DepositRoot:  bytesutil.PadTo([]byte("e1ddepositroot"), 32),
			DepositCount: 6,
			BlockHash:    bytesutil.PadTo([]byte("e1dblockhash"), 32),
		}
		state.Eth1DataVotes = []*ethpbalpha.Eth1Data{{
			DepositRoot:  bytesutil.PadTo([]byte("e1dvdepositroot"), 32),
			DepositCount: 7,
			BlockHash:    bytesutil.PadTo([]byte("e1dvblockhash"), 32),
		}}
		state.Eth1DepositIndex = 8
		state.Validators = []*ethpbalpha.Validator{{
			PublicKey:                  bytesutil.PadTo([]byte("publickey"), 48),
			WithdrawalCredentials:      bytesutil.PadTo([]byte("withdrawalcredentials"), 32),
			EffectiveBalance:           9,
			Slashed:                    true,
			ActivationEligibilityEpoch: 10,
			ActivationEpoch:            11,
			ExitEpoch:                  12,
			WithdrawableEpoch:          13,
		}}
		state.Balances = []uint64{14}
		state.RandaoMixes = [][]byte{bytesutil.PadTo([]byte("randaomixes"), 32)}
		state.Slashings = []uint64{15}
		state.JustificationBits = bitfield.Bitvector4{1}
		state.PreviousJustifiedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 30,
			Root:  bytesutil.PadTo([]byte("pjcroot"), 32),
		}
		state.CurrentJustifiedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 31,
			Root:  bytesutil.PadTo([]byte("cjcroot"), 32),
		}
		state.FinalizedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 32,
			Root:  bytesutil.PadTo([]byte("fcroot"), 32),
		}
		state.PreviousEpochParticipation = []byte("previousepochparticipation")
		state.CurrentEpochParticipation = []byte("currentepochparticipation")
		state.InactivityScores = []uint64{1, 2, 3}
		state.CurrentSyncCommittee = &ethpbalpha.SyncCommittee{
			Pubkeys:         [][]byte{bytesutil.PadTo([]byte("cscpubkeys"), 48)},
			AggregatePubkey: bytesutil.PadTo([]byte("cscaggregatepubkey"), 48),
		}
		state.NextSyncCommittee = &ethpbalpha.SyncCommittee{
			Pubkeys:         [][]byte{bytesutil.PadTo([]byte("nscpubkeys"), 48)},
			AggregatePubkey: bytesutil.PadTo([]byte("nscaggregatepubkey"), 48),
		}
		state.LatestExecutionPayloadHeader = &enginev1.ExecutionPayloadHeader{
			ParentHash:       bytesutil.PadTo([]byte("parenthash"), 32),
			FeeRecipient:     bytesutil.PadTo([]byte("feerecipient"), 20),
			StateRoot:        bytesutil.PadTo([]byte("stateroot"), 32),
			ReceiptsRoot:     bytesutil.PadTo([]byte("receiptroot"), 32),
			LogsBloom:        bytesutil.PadTo([]byte("logsbloom"), 256),
			PrevRandao:       bytesutil.PadTo([]byte("prevrandao"), 32),
			BlockNumber:      123,
			GasLimit:         456,
			GasUsed:          789,
			Timestamp:        012,
			ExtraData:        []byte("extradata"),
			BaseFeePerGas:    bytesutil.PadTo([]byte("basefeepergas"), 32),
			BlockHash:        bytesutil.PadTo([]byte("blockhash"), 32),
			TransactionsRoot: bytesutil.PadTo([]byte("transactionsroot"), 32),
		}
		return nil
	})
	require.NoError(t, err)

	result, err := BeaconStateBellatrixToProto(source)
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, uint64(1), result.GenesisTime)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("genesisvalidatorsroot"), 32), result.GenesisValidatorsRoot)
	assert.Equal(t, primitives.Slot(2), result.Slot)
	resultFork := result.Fork
	require.NotNil(t, resultFork)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("123"), 4), resultFork.PreviousVersion)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("456"), 4), resultFork.CurrentVersion)
	assert.Equal(t, primitives.Epoch(3), resultFork.Epoch)
	resultLatestBlockHeader := result.LatestBlockHeader
	require.NotNil(t, resultLatestBlockHeader)
	assert.Equal(t, primitives.Slot(4), resultLatestBlockHeader.Slot)
	assert.Equal(t, primitives.ValidatorIndex(5), resultLatestBlockHeader.ProposerIndex)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhparentroot"), 32), resultLatestBlockHeader.ParentRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhstateroot"), 32), resultLatestBlockHeader.StateRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhbodyroot"), 32), resultLatestBlockHeader.BodyRoot)
	assert.Equal(t, 8192, len(result.BlockRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("blockroots"), 32), result.BlockRoots[0])
	assert.Equal(t, 8192, len(result.StateRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("stateroots"), 32), result.StateRoots[0])
	assert.Equal(t, 1, len(result.HistoricalRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("historicalroots"), 32), result.HistoricalRoots[0])
	resultEth1Data := result.Eth1Data
	require.NotNil(t, resultEth1Data)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1ddepositroot"), 32), resultEth1Data.DepositRoot)
	assert.Equal(t, uint64(6), resultEth1Data.DepositCount)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dblockhash"), 32), resultEth1Data.BlockHash)
	require.Equal(t, 1, len(result.Eth1DataVotes))
	resultEth1DataVote := result.Eth1DataVotes[0]
	require.NotNil(t, resultEth1DataVote)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dvdepositroot"), 32), resultEth1DataVote.DepositRoot)
	assert.Equal(t, uint64(7), resultEth1DataVote.DepositCount)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dvblockhash"), 32), resultEth1DataVote.BlockHash)
	assert.Equal(t, uint64(8), result.Eth1DepositIndex)
	require.Equal(t, 1, len(result.Validators))
	resultValidator := result.Validators[0]
	require.NotNil(t, resultValidator)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("publickey"), 48), resultValidator.Pubkey)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("withdrawalcredentials"), 32), resultValidator.WithdrawalCredentials)
	assert.Equal(t, uint64(9), resultValidator.EffectiveBalance)
	assert.Equal(t, true, resultValidator.Slashed)
	assert.Equal(t, primitives.Epoch(10), resultValidator.ActivationEligibilityEpoch)
	assert.Equal(t, primitives.Epoch(11), resultValidator.ActivationEpoch)
	assert.Equal(t, primitives.Epoch(12), resultValidator.ExitEpoch)
	assert.Equal(t, primitives.Epoch(13), resultValidator.WithdrawableEpoch)
	assert.DeepEqual(t, []uint64{14}, result.Balances)
	assert.Equal(t, 65536, len(result.RandaoMixes))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("randaomixes"), 32), result.RandaoMixes[0])
	assert.DeepEqual(t, []uint64{15}, result.Slashings)
	assert.DeepEqual(t, bitfield.Bitvector4{1}, result.JustificationBits)
	resultPrevJustifiedCheckpoint := result.PreviousJustifiedCheckpoint
	require.NotNil(t, resultPrevJustifiedCheckpoint)
	assert.Equal(t, primitives.Epoch(30), resultPrevJustifiedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("pjcroot"), 32), resultPrevJustifiedCheckpoint.Root)
	resultCurrJustifiedCheckpoint := result.CurrentJustifiedCheckpoint
	require.NotNil(t, resultCurrJustifiedCheckpoint)
	assert.Equal(t, primitives.Epoch(31), resultCurrJustifiedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("cjcroot"), 32), resultCurrJustifiedCheckpoint.Root)
	resultFinalizedCheckpoint := result.FinalizedCheckpoint
	require.NotNil(t, resultFinalizedCheckpoint)
	assert.Equal(t, primitives.Epoch(32), resultFinalizedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("fcroot"), 32), resultFinalizedCheckpoint.Root)
	assert.DeepEqual(t, []byte("previousepochparticipation"), result.PreviousEpochParticipation)
	assert.DeepEqual(t, []byte("currentepochparticipation"), result.CurrentEpochParticipation)
	assert.DeepEqual(t, []uint64{1, 2, 3}, result.InactivityScores)
	require.NotNil(t, result.CurrentSyncCommittee)
	assert.DeepEqual(t, [][]byte{bytesutil.PadTo([]byte("cscpubkeys"), 48)}, result.CurrentSyncCommittee.Pubkeys)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("cscaggregatepubkey"), 48), result.CurrentSyncCommittee.AggregatePubkey)
	require.NotNil(t, result.NextSyncCommittee)
	assert.DeepEqual(t, [][]byte{bytesutil.PadTo([]byte("nscpubkeys"), 48)}, result.NextSyncCommittee.Pubkeys)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("nscaggregatepubkey"), 48), result.NextSyncCommittee.AggregatePubkey)
	resultLatestExecutionPayloadHeader := result.LatestExecutionPayloadHeader
	require.NotNil(t, resultLatestExecutionPayloadHeader)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("parenthash"), 32), resultLatestExecutionPayloadHeader.ParentHash)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("feerecipient"), 20), resultLatestExecutionPayloadHeader.FeeRecipient)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("stateroot"), 32), resultLatestExecutionPayloadHeader.StateRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("receiptroot"), 32), resultLatestExecutionPayloadHeader.ReceiptsRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("logsbloom"), 256), resultLatestExecutionPayloadHeader.LogsBloom)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("prevrandao"), 32), resultLatestExecutionPayloadHeader.PrevRandao)
	assert.Equal(t, uint64(123), resultLatestExecutionPayloadHeader.BlockNumber)
	assert.Equal(t, uint64(456), resultLatestExecutionPayloadHeader.GasLimit)
	assert.Equal(t, uint64(789), resultLatestExecutionPayloadHeader.GasUsed)
	assert.Equal(t, uint64(012), resultLatestExecutionPayloadHeader.Timestamp)
	assert.DeepEqual(t, []byte("extradata"), resultLatestExecutionPayloadHeader.ExtraData)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("basefeepergas"), 32), resultLatestExecutionPayloadHeader.BaseFeePerGas)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("blockhash"), 32), resultLatestExecutionPayloadHeader.BlockHash)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("transactionsroot"), 32), resultLatestExecutionPayloadHeader.TransactionsRoot)
}

func TestBeaconStateCapellaToProto(t *testing.T) {
	source, err := util.NewBeaconStateCapella(util.FillRootsNaturalOptCapella, func(state *ethpbalpha.BeaconStateCapella) error {
		state.GenesisTime = 1
		state.GenesisValidatorsRoot = bytesutil.PadTo([]byte("genesisvalidatorsroot"), 32)
		state.Slot = 2
		state.Fork = &ethpbalpha.Fork{
			PreviousVersion: bytesutil.PadTo([]byte("123"), 4),
			CurrentVersion:  bytesutil.PadTo([]byte("456"), 4),
			Epoch:           3,
		}
		state.LatestBlockHeader = &ethpbalpha.BeaconBlockHeader{
			Slot:          4,
			ProposerIndex: 5,
			ParentRoot:    bytesutil.PadTo([]byte("lbhparentroot"), 32),
			StateRoot:     bytesutil.PadTo([]byte("lbhstateroot"), 32),
			BodyRoot:      bytesutil.PadTo([]byte("lbhbodyroot"), 32),
		}
		state.BlockRoots = [][]byte{bytesutil.PadTo([]byte("blockroots"), 32)}
		state.StateRoots = [][]byte{bytesutil.PadTo([]byte("stateroots"), 32)}
		state.HistoricalRoots = [][]byte{bytesutil.PadTo([]byte("historicalroots"), 32)}
		state.Eth1Data = &ethpbalpha.Eth1Data{
			DepositRoot:  bytesutil.PadTo([]byte("e1ddepositroot"), 32),
			DepositCount: 6,
			BlockHash:    bytesutil.PadTo([]byte("e1dblockhash"), 32),
		}
		state.Eth1DataVotes = []*ethpbalpha.Eth1Data{{
			DepositRoot:  bytesutil.PadTo([]byte("e1dvdepositroot"), 32),
			DepositCount: 7,
			BlockHash:    bytesutil.PadTo([]byte("e1dvblockhash"), 32),
		}}
		state.Eth1DepositIndex = 8
		state.Validators = []*ethpbalpha.Validator{{
			PublicKey:                  bytesutil.PadTo([]byte("publickey"), 48),
			WithdrawalCredentials:      bytesutil.PadTo([]byte("withdrawalcredentials"), 32),
			EffectiveBalance:           9,
			Slashed:                    true,
			ActivationEligibilityEpoch: 10,
			ActivationEpoch:            11,
			ExitEpoch:                  12,
			WithdrawableEpoch:          13,
		}}
		state.Balances = []uint64{14}
		state.RandaoMixes = [][]byte{bytesutil.PadTo([]byte("randaomixes"), 32)}
		state.Slashings = []uint64{15}
		state.JustificationBits = bitfield.Bitvector4{1}
		state.PreviousJustifiedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 30,
			Root:  bytesutil.PadTo([]byte("pjcroot"), 32),
		}
		state.CurrentJustifiedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 31,
			Root:  bytesutil.PadTo([]byte("cjcroot"), 32),
		}
		state.FinalizedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 32,
			Root:  bytesutil.PadTo([]byte("fcroot"), 32),
		}
		state.PreviousEpochParticipation = []byte("previousepochparticipation")
		state.CurrentEpochParticipation = []byte("currentepochparticipation")
		state.InactivityScores = []uint64{1, 2, 3}
		state.CurrentSyncCommittee = &ethpbalpha.SyncCommittee{
			Pubkeys:         [][]byte{bytesutil.PadTo([]byte("cscpubkeys"), 48)},
			AggregatePubkey: bytesutil.PadTo([]byte("cscaggregatepubkey"), 48),
		}
		state.NextSyncCommittee = &ethpbalpha.SyncCommittee{
			Pubkeys:         [][]byte{bytesutil.PadTo([]byte("nscpubkeys"), 48)},
			AggregatePubkey: bytesutil.PadTo([]byte("nscaggregatepubkey"), 48),
		}
		state.LatestExecutionPayloadHeader = &enginev1.ExecutionPayloadHeaderCapella{
			ParentHash:       bytesutil.PadTo([]byte("parenthash"), 32),
			FeeRecipient:     bytesutil.PadTo([]byte("feerecipient"), 20),
			StateRoot:        bytesutil.PadTo([]byte("stateroot"), 32),
			ReceiptsRoot:     bytesutil.PadTo([]byte("receiptroot"), 32),
			LogsBloom:        bytesutil.PadTo([]byte("logsbloom"), 256),
			PrevRandao:       bytesutil.PadTo([]byte("prevrandao"), 32),
			BlockNumber:      123,
			GasLimit:         456,
			GasUsed:          789,
			Timestamp:        012,
			ExtraData:        []byte("extradata"),
			BaseFeePerGas:    bytesutil.PadTo([]byte("basefeepergas"), 32),
			BlockHash:        bytesutil.PadTo([]byte("blockhash"), 32),
			TransactionsRoot: bytesutil.PadTo([]byte("transactionsroot"), 32),
			WithdrawalsRoot:  bytesutil.PadTo([]byte("withdrawalsroot"), 32),
		}
		state.NextWithdrawalIndex = 123
		state.NextWithdrawalValidatorIndex = 123
		state.HistoricalSummaries = []*ethpbalpha.HistoricalSummary{
			{
				BlockSummaryRoot: bytesutil.PadTo([]byte("blocksummaryroot"), 32),
				StateSummaryRoot: bytesutil.PadTo([]byte("statesummaryroot"), 32),
			},
			{
				BlockSummaryRoot: bytesutil.PadTo([]byte("blocksummaryroot2"), 32),
				StateSummaryRoot: bytesutil.PadTo([]byte("statesummaryroot2"), 32),
			}}
		return nil
	})
	require.NoError(t, err)

	result, err := BeaconStateCapellaToProto(source)
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, uint64(1), result.GenesisTime)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("genesisvalidatorsroot"), 32), result.GenesisValidatorsRoot)
	assert.Equal(t, primitives.Slot(2), result.Slot)
	resultFork := result.Fork
	require.NotNil(t, resultFork)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("123"), 4), resultFork.PreviousVersion)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("456"), 4), resultFork.CurrentVersion)
	assert.Equal(t, primitives.Epoch(3), resultFork.Epoch)
	resultLatestBlockHeader := result.LatestBlockHeader
	require.NotNil(t, resultLatestBlockHeader)
	assert.Equal(t, primitives.Slot(4), resultLatestBlockHeader.Slot)
	assert.Equal(t, primitives.ValidatorIndex(5), resultLatestBlockHeader.ProposerIndex)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhparentroot"), 32), resultLatestBlockHeader.ParentRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhstateroot"), 32), resultLatestBlockHeader.StateRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhbodyroot"), 32), resultLatestBlockHeader.BodyRoot)
	assert.Equal(t, 8192, len(result.BlockRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("blockroots"), 32), result.BlockRoots[0])
	assert.Equal(t, 8192, len(result.StateRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("stateroots"), 32), result.StateRoots[0])
	assert.Equal(t, 1, len(result.HistoricalRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("historicalroots"), 32), result.HistoricalRoots[0])
	resultEth1Data := result.Eth1Data
	require.NotNil(t, resultEth1Data)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1ddepositroot"), 32), resultEth1Data.DepositRoot)
	assert.Equal(t, uint64(6), resultEth1Data.DepositCount)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dblockhash"), 32), resultEth1Data.BlockHash)
	require.Equal(t, 1, len(result.Eth1DataVotes))
	resultEth1DataVote := result.Eth1DataVotes[0]
	require.NotNil(t, resultEth1DataVote)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dvdepositroot"), 32), resultEth1DataVote.DepositRoot)
	assert.Equal(t, uint64(7), resultEth1DataVote.DepositCount)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dvblockhash"), 32), resultEth1DataVote.BlockHash)
	assert.Equal(t, uint64(8), result.Eth1DepositIndex)
	require.Equal(t, 1, len(result.Validators))
	resultValidator := result.Validators[0]
	require.NotNil(t, resultValidator)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("publickey"), 48), resultValidator.Pubkey)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("withdrawalcredentials"), 32), resultValidator.WithdrawalCredentials)
	assert.Equal(t, uint64(9), resultValidator.EffectiveBalance)
	assert.Equal(t, true, resultValidator.Slashed)
	assert.Equal(t, primitives.Epoch(10), resultValidator.ActivationEligibilityEpoch)
	assert.Equal(t, primitives.Epoch(11), resultValidator.ActivationEpoch)
	assert.Equal(t, primitives.Epoch(12), resultValidator.ExitEpoch)
	assert.Equal(t, primitives.Epoch(13), resultValidator.WithdrawableEpoch)
	assert.DeepEqual(t, []uint64{14}, result.Balances)
	assert.Equal(t, 65536, len(result.RandaoMixes))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("randaomixes"), 32), result.RandaoMixes[0])
	assert.DeepEqual(t, []uint64{15}, result.Slashings)
	assert.DeepEqual(t, bitfield.Bitvector4{1}, result.JustificationBits)
	resultPrevJustifiedCheckpoint := result.PreviousJustifiedCheckpoint
	require.NotNil(t, resultPrevJustifiedCheckpoint)
	assert.Equal(t, primitives.Epoch(30), resultPrevJustifiedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("pjcroot"), 32), resultPrevJustifiedCheckpoint.Root)
	resultCurrJustifiedCheckpoint := result.CurrentJustifiedCheckpoint
	require.NotNil(t, resultCurrJustifiedCheckpoint)
	assert.Equal(t, primitives.Epoch(31), resultCurrJustifiedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("cjcroot"), 32), resultCurrJustifiedCheckpoint.Root)
	resultFinalizedCheckpoint := result.FinalizedCheckpoint
	require.NotNil(t, resultFinalizedCheckpoint)
	assert.Equal(t, primitives.Epoch(32), resultFinalizedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("fcroot"), 32), resultFinalizedCheckpoint.Root)
	assert.DeepEqual(t, []byte("previousepochparticipation"), result.PreviousEpochParticipation)
	assert.DeepEqual(t, []byte("currentepochparticipation"), result.CurrentEpochParticipation)
	assert.DeepEqual(t, []uint64{1, 2, 3}, result.InactivityScores)
	require.NotNil(t, result.CurrentSyncCommittee)
	assert.DeepEqual(t, [][]byte{bytesutil.PadTo([]byte("cscpubkeys"), 48)}, result.CurrentSyncCommittee.Pubkeys)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("cscaggregatepubkey"), 48), result.CurrentSyncCommittee.AggregatePubkey)
	require.NotNil(t, result.NextSyncCommittee)
	assert.DeepEqual(t, [][]byte{bytesutil.PadTo([]byte("nscpubkeys"), 48)}, result.NextSyncCommittee.Pubkeys)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("nscaggregatepubkey"), 48), result.NextSyncCommittee.AggregatePubkey)
	resultLatestExecutionPayloadHeader := result.LatestExecutionPayloadHeader
	require.NotNil(t, resultLatestExecutionPayloadHeader)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("parenthash"), 32), resultLatestExecutionPayloadHeader.ParentHash)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("feerecipient"), 20), resultLatestExecutionPayloadHeader.FeeRecipient)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("stateroot"), 32), resultLatestExecutionPayloadHeader.StateRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("receiptroot"), 32), resultLatestExecutionPayloadHeader.ReceiptsRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("logsbloom"), 256), resultLatestExecutionPayloadHeader.LogsBloom)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("prevrandao"), 32), resultLatestExecutionPayloadHeader.PrevRandao)
	assert.Equal(t, uint64(123), resultLatestExecutionPayloadHeader.BlockNumber)
	assert.Equal(t, uint64(456), resultLatestExecutionPayloadHeader.GasLimit)
	assert.Equal(t, uint64(789), resultLatestExecutionPayloadHeader.GasUsed)
	assert.Equal(t, uint64(012), resultLatestExecutionPayloadHeader.Timestamp)
	assert.DeepEqual(t, []byte("extradata"), resultLatestExecutionPayloadHeader.ExtraData)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("basefeepergas"), 32), resultLatestExecutionPayloadHeader.BaseFeePerGas)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("blockhash"), 32), resultLatestExecutionPayloadHeader.BlockHash)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("transactionsroot"), 32), resultLatestExecutionPayloadHeader.TransactionsRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("withdrawalsroot"), 32), resultLatestExecutionPayloadHeader.WithdrawalsRoot)
	assert.Equal(t, uint64(123), result.NextWithdrawalIndex)
	assert.Equal(t, primitives.ValidatorIndex(123), result.NextWithdrawalValidatorIndex)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("blocksummaryroot"), 32), result.HistoricalSummaries[0].BlockSummaryRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("statesummaryroot"), 32), result.HistoricalSummaries[0].StateSummaryRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("blocksummaryroot2"), 32), result.HistoricalSummaries[1].BlockSummaryRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("statesummaryroot2"), 32), result.HistoricalSummaries[1].StateSummaryRoot)
}

func TestBeaconStateDenebToProto(t *testing.T) {
	source, err := util.NewBeaconStateDeneb(util.FillRootsNaturalOptDeneb, func(state *ethpbalpha.BeaconStateDeneb) error {
		state.GenesisTime = 1
		state.GenesisValidatorsRoot = bytesutil.PadTo([]byte("genesisvalidatorsroot"), 32)
		state.Slot = 2
		state.Fork = &ethpbalpha.Fork{
			PreviousVersion: bytesutil.PadTo([]byte("123"), 4),
			CurrentVersion:  bytesutil.PadTo([]byte("456"), 4),
			Epoch:           3,
		}
		state.LatestBlockHeader = &ethpbalpha.BeaconBlockHeader{
			Slot:          4,
			ProposerIndex: 5,
			ParentRoot:    bytesutil.PadTo([]byte("lbhparentroot"), 32),
			StateRoot:     bytesutil.PadTo([]byte("lbhstateroot"), 32),
			BodyRoot:      bytesutil.PadTo([]byte("lbhbodyroot"), 32),
		}
		state.BlockRoots = [][]byte{bytesutil.PadTo([]byte("blockroots"), 32)}
		state.StateRoots = [][]byte{bytesutil.PadTo([]byte("stateroots"), 32)}
		state.HistoricalRoots = [][]byte{bytesutil.PadTo([]byte("historicalroots"), 32)}
		state.Eth1Data = &ethpbalpha.Eth1Data{
			DepositRoot:  bytesutil.PadTo([]byte("e1ddepositroot"), 32),
			DepositCount: 6,
			BlockHash:    bytesutil.PadTo([]byte("e1dblockhash"), 32),
		}
		state.Eth1DataVotes = []*ethpbalpha.Eth1Data{{
			DepositRoot:  bytesutil.PadTo([]byte("e1dvdepositroot"), 32),
			DepositCount: 7,
			BlockHash:    bytesutil.PadTo([]byte("e1dvblockhash"), 32),
		}}
		state.Eth1DepositIndex = 8
		state.Validators = []*ethpbalpha.Validator{{
			PublicKey:                  bytesutil.PadTo([]byte("publickey"), 48),
			WithdrawalCredentials:      bytesutil.PadTo([]byte("withdrawalcredentials"), 32),
			EffectiveBalance:           9,
			Slashed:                    true,
			ActivationEligibilityEpoch: 10,
			ActivationEpoch:            11,
			ExitEpoch:                  12,
			WithdrawableEpoch:          13,
		}}
		state.Balances = []uint64{14}
		state.RandaoMixes = [][]byte{bytesutil.PadTo([]byte("randaomixes"), 32)}
		state.Slashings = []uint64{15}
		state.JustificationBits = bitfield.Bitvector4{1}
		state.PreviousJustifiedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 30,
			Root:  bytesutil.PadTo([]byte("pjcroot"), 32),
		}
		state.CurrentJustifiedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 31,
			Root:  bytesutil.PadTo([]byte("cjcroot"), 32),
		}
		state.FinalizedCheckpoint = &ethpbalpha.Checkpoint{
			Epoch: 32,
			Root:  bytesutil.PadTo([]byte("fcroot"), 32),
		}
		state.PreviousEpochParticipation = []byte("previousepochparticipation")
		state.CurrentEpochParticipation = []byte("currentepochparticipation")
		state.InactivityScores = []uint64{1, 2, 3}
		state.CurrentSyncCommittee = &ethpbalpha.SyncCommittee{
			Pubkeys:         [][]byte{bytesutil.PadTo([]byte("cscpubkeys"), 48)},
			AggregatePubkey: bytesutil.PadTo([]byte("cscaggregatepubkey"), 48),
		}
		state.NextSyncCommittee = &ethpbalpha.SyncCommittee{
			Pubkeys:         [][]byte{bytesutil.PadTo([]byte("nscpubkeys"), 48)},
			AggregatePubkey: bytesutil.PadTo([]byte("nscaggregatepubkey"), 48),
		}
		state.LatestExecutionPayloadHeader = &enginev1.ExecutionPayloadHeaderDeneb{
			ParentHash:       bytesutil.PadTo([]byte("parenthash"), 32),
			FeeRecipient:     bytesutil.PadTo([]byte("feerecipient"), 20),
			StateRoot:        bytesutil.PadTo([]byte("stateroot"), 32),
			ReceiptsRoot:     bytesutil.PadTo([]byte("receiptroot"), 32),
			LogsBloom:        bytesutil.PadTo([]byte("logsbloom"), 256),
			PrevRandao:       bytesutil.PadTo([]byte("prevrandao"), 32),
			BlockNumber:      123,
			GasLimit:         456,
			GasUsed:          789,
			Timestamp:        012,
			ExtraData:        []byte("extradata"),
			BaseFeePerGas:    bytesutil.PadTo([]byte("basefeepergas"), 32),
			BlockHash:        bytesutil.PadTo([]byte("blockhash"), 32),
			TransactionsRoot: bytesutil.PadTo([]byte("transactionsroot"), 32),
			WithdrawalsRoot:  bytesutil.PadTo([]byte("withdrawalsroot"), 32),
			BlobGasUsed:      343,
			ExcessBlobGas:    676,
		}
		state.NextWithdrawalIndex = 123
		state.NextWithdrawalValidatorIndex = 123
		state.HistoricalSummaries = []*ethpbalpha.HistoricalSummary{
			{
				BlockSummaryRoot: bytesutil.PadTo([]byte("blocksummaryroot"), 32),
				StateSummaryRoot: bytesutil.PadTo([]byte("statesummaryroot"), 32),
			},
			{
				BlockSummaryRoot: bytesutil.PadTo([]byte("blocksummaryroot2"), 32),
				StateSummaryRoot: bytesutil.PadTo([]byte("statesummaryroot2"), 32),
			}}
		return nil
	})
	require.NoError(t, err)

	result, err := BeaconStateDenebToProto(source)
	require.NoError(t, err)
	require.NotNil(t, result)
	assert.Equal(t, uint64(1), result.GenesisTime)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("genesisvalidatorsroot"), 32), result.GenesisValidatorsRoot)
	assert.Equal(t, primitives.Slot(2), result.Slot)
	resultFork := result.Fork
	require.NotNil(t, resultFork)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("123"), 4), resultFork.PreviousVersion)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("456"), 4), resultFork.CurrentVersion)
	assert.Equal(t, primitives.Epoch(3), resultFork.Epoch)
	resultLatestBlockHeader := result.LatestBlockHeader
	require.NotNil(t, resultLatestBlockHeader)
	assert.Equal(t, primitives.Slot(4), resultLatestBlockHeader.Slot)
	assert.Equal(t, primitives.ValidatorIndex(5), resultLatestBlockHeader.ProposerIndex)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhparentroot"), 32), resultLatestBlockHeader.ParentRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhstateroot"), 32), resultLatestBlockHeader.StateRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("lbhbodyroot"), 32), resultLatestBlockHeader.BodyRoot)
	assert.Equal(t, 8192, len(result.BlockRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("blockroots"), 32), result.BlockRoots[0])
	assert.Equal(t, 8192, len(result.StateRoots))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("stateroots"), 32), result.StateRoots[0])
	resultEth1Data := result.Eth1Data
	require.NotNil(t, resultEth1Data)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1ddepositroot"), 32), resultEth1Data.DepositRoot)
	assert.Equal(t, uint64(6), resultEth1Data.DepositCount)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dblockhash"), 32), resultEth1Data.BlockHash)
	require.Equal(t, 1, len(result.Eth1DataVotes))
	resultEth1DataVote := result.Eth1DataVotes[0]
	require.NotNil(t, resultEth1DataVote)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dvdepositroot"), 32), resultEth1DataVote.DepositRoot)
	assert.Equal(t, uint64(7), resultEth1DataVote.DepositCount)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("e1dvblockhash"), 32), resultEth1DataVote.BlockHash)
	assert.Equal(t, uint64(8), result.Eth1DepositIndex)
	require.Equal(t, 1, len(result.Validators))
	resultValidator := result.Validators[0]
	require.NotNil(t, resultValidator)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("publickey"), 48), resultValidator.Pubkey)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("withdrawalcredentials"), 32), resultValidator.WithdrawalCredentials)
	assert.Equal(t, uint64(9), resultValidator.EffectiveBalance)
	assert.Equal(t, true, resultValidator.Slashed)
	assert.Equal(t, primitives.Epoch(10), resultValidator.ActivationEligibilityEpoch)
	assert.Equal(t, primitives.Epoch(11), resultValidator.ActivationEpoch)
	assert.Equal(t, primitives.Epoch(12), resultValidator.ExitEpoch)
	assert.Equal(t, primitives.Epoch(13), resultValidator.WithdrawableEpoch)
	assert.DeepEqual(t, []uint64{14}, result.Balances)
	assert.Equal(t, 65536, len(result.RandaoMixes))
	assert.DeepEqual(t, bytesutil.PadTo([]byte("randaomixes"), 32), result.RandaoMixes[0])
	assert.DeepEqual(t, []uint64{15}, result.Slashings)
	assert.DeepEqual(t, bitfield.Bitvector4{1}, result.JustificationBits)
	resultPrevJustifiedCheckpoint := result.PreviousJustifiedCheckpoint
	require.NotNil(t, resultPrevJustifiedCheckpoint)
	assert.Equal(t, primitives.Epoch(30), resultPrevJustifiedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("pjcroot"), 32), resultPrevJustifiedCheckpoint.Root)
	resultCurrJustifiedCheckpoint := result.CurrentJustifiedCheckpoint
	require.NotNil(t, resultCurrJustifiedCheckpoint)
	assert.Equal(t, primitives.Epoch(31), resultCurrJustifiedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("cjcroot"), 32), resultCurrJustifiedCheckpoint.Root)
	resultFinalizedCheckpoint := result.FinalizedCheckpoint
	require.NotNil(t, resultFinalizedCheckpoint)
	assert.Equal(t, primitives.Epoch(32), resultFinalizedCheckpoint.Epoch)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("fcroot"), 32), resultFinalizedCheckpoint.Root)
	assert.DeepEqual(t, []byte("previousepochparticipation"), result.PreviousEpochParticipation)
	assert.DeepEqual(t, []byte("currentepochparticipation"), result.CurrentEpochParticipation)
	assert.DeepEqual(t, []uint64{1, 2, 3}, result.InactivityScores)
	require.NotNil(t, result.CurrentSyncCommittee)
	assert.DeepEqual(t, [][]byte{bytesutil.PadTo([]byte("cscpubkeys"), 48)}, result.CurrentSyncCommittee.Pubkeys)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("cscaggregatepubkey"), 48), result.CurrentSyncCommittee.AggregatePubkey)
	require.NotNil(t, result.NextSyncCommittee)
	assert.DeepEqual(t, [][]byte{bytesutil.PadTo([]byte("nscpubkeys"), 48)}, result.NextSyncCommittee.Pubkeys)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("nscaggregatepubkey"), 48), result.NextSyncCommittee.AggregatePubkey)
	resultLatestExecutionPayloadHeader := result.LatestExecutionPayloadHeader
	require.NotNil(t, resultLatestExecutionPayloadHeader)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("parenthash"), 32), resultLatestExecutionPayloadHeader.ParentHash)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("feerecipient"), 20), resultLatestExecutionPayloadHeader.FeeRecipient)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("stateroot"), 32), resultLatestExecutionPayloadHeader.StateRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("receiptroot"), 32), resultLatestExecutionPayloadHeader.ReceiptsRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("logsbloom"), 256), resultLatestExecutionPayloadHeader.LogsBloom)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("prevrandao"), 32), resultLatestExecutionPayloadHeader.PrevRandao)
	assert.Equal(t, uint64(123), resultLatestExecutionPayloadHeader.BlockNumber)
	assert.Equal(t, uint64(456), resultLatestExecutionPayloadHeader.GasLimit)
	assert.Equal(t, uint64(789), resultLatestExecutionPayloadHeader.GasUsed)
	assert.Equal(t, uint64(012), resultLatestExecutionPayloadHeader.Timestamp)
	assert.DeepEqual(t, []byte("extradata"), resultLatestExecutionPayloadHeader.ExtraData)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("basefeepergas"), 32), resultLatestExecutionPayloadHeader.BaseFeePerGas)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("blockhash"), 32), resultLatestExecutionPayloadHeader.BlockHash)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("transactionsroot"), 32), resultLatestExecutionPayloadHeader.TransactionsRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("withdrawalsroot"), 32), resultLatestExecutionPayloadHeader.WithdrawalsRoot)
	assert.DeepEqual(t, uint64(343), resultLatestExecutionPayloadHeader.BlobGasUsed)
	assert.DeepEqual(t, uint64(676), resultLatestExecutionPayloadHeader.ExcessBlobGas)
	assert.Equal(t, uint64(123), result.NextWithdrawalIndex)
	assert.Equal(t, primitives.ValidatorIndex(123), result.NextWithdrawalValidatorIndex)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("blocksummaryroot"), 32), result.HistoricalSummaries[0].BlockSummaryRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("statesummaryroot"), 32), result.HistoricalSummaries[0].StateSummaryRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("blocksummaryroot2"), 32), result.HistoricalSummaries[1].BlockSummaryRoot)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("statesummaryroot2"), 32), result.HistoricalSummaries[1].StateSummaryRoot)
}

func TestV1Alpha1SignedBLSToExecChangeToV2(t *testing.T) {
	alphaChange := &ethpbalpha.SignedBLSToExecutionChange{
		Message: &ethpbalpha.BLSToExecutionChange{
			ValidatorIndex:     validatorIndex,
			FromBlsPubkey:      bytesutil.PadTo([]byte("fromblspubkey"), 48),
			ToExecutionAddress: bytesutil.PadTo([]byte("toexecutionaddress"), 20),
		},
		Signature: signature,
	}
	change := V1Alpha1SignedBLSToExecChangeToV2(alphaChange)
	require.NotNil(t, change)
	require.NotNil(t, change.Message)
	assert.DeepEqual(t, signature, change.Signature)
	assert.Equal(t, validatorIndex, change.Message.ValidatorIndex)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("fromblspubkey"), 48), change.Message.FromBlsPubkey)
	assert.DeepEqual(t, bytesutil.PadTo([]byte("toexecutionaddress"), 20), change.Message.ToExecutionAddress)
}
