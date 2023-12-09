package types

import (
	"bytes"
	"fmt"
	"strings"

	"github.com/cometbft/cometbft/libs/bits"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
)

const (
	// MaxVotesCount is the maximum number of votes in a set. Used in ValidateBasic funcs for
	// protection against DOS attacks. Note this implies a corresponding equal limit to
	// the number of validators.
	MaxVotesCount = 10000
)

// UNSTABLE
// XXX: duplicate of p2p.ID to avoid dependence between packages.
// Perhaps we can have a minimal types package containing this (and other things?)
// that both `types` and `p2p` import ?
type P2PID string

/*
VoteSet helps collect signatures from validators at each height+round for a
predefined vote type.

We need VoteSet to be able to keep track of conflicting votes when validators
double-sign.  Yet, we can't keep track of *all* the votes seen, as that could
be a DoS attack vector.

There are two storage areas for votes.
1. voteSet.votes
2. voteSet.votesByBlock

`.votes` is the "canonical" list of votes.  It always has at least one vote,
if a vote from a validator had been seen at all.  Usually it keeps track of
the first vote seen, but when a 2/3 majority is found, votes for that get
priority and are copied over from `.votesByBlock`.

`.votesByBlock` keeps track of a list of votes for a particular block.  There
are two ways a &blockVotes{} gets created in `.votesByBlock`.
1. the first vote seen by a validator was for the particular block.
2. a peer claims to have seen 2/3 majority for the particular block.

Since the first vote from a validator will always get added in `.votesByBlock`
, all votes in `.votes` will have a corresponding entry in `.votesByBlock`.

When a &blockVotes{} in `.votesByBlock` reaches a 2/3 majority quorum, its
votes are copied into `.votes`.

All this is memory bounded because conflicting votes only get added if a peer
told us to track that block, each peer only gets to tell us 1 such block, and,
there's only a limited number of peers.

NOTE: Assumes that the sum total of voting power does not exceed MaxUInt64.
*/
type VoteSet struct {
	chainID           string
	height            int64
	round             int32
	signedMsgType     cmtproto.SignedMsgType
	valSet            *ValidatorSet
	extensionsEnabled bool

	mtx           cmtsync.Mutex
	votesBitArray *bits.BitArray
	votes         []*Vote                // Primary votes to share
	sum           int64                  // Sum of voting power for seen votes, discounting conflicts
	maj23         *BlockID               // First 2/3 majority seen
	votesByBlock  map[string]*blockVotes // string(blockHash|blockParts) -> blockVotes
	peerMaj23s    map[P2PID]BlockID      // Maj23 for each peer
}

// NewVoteSet instantiates all fields of a new vote set. This constructor requires
// that no vote extension data be present on the votes that are added to the set.
func NewVoteSet(chainID string, height int64, round int32,
	signedMsgType cmtproto.SignedMsgType, valSet *ValidatorSet) *VoteSet {
	if height == 0 {
		panic("Cannot make VoteSet for height == 0, doesn't make sense.")
	}
	return &VoteSet{
		chainID:       chainID,
		height:        height,
		round:         round,
		signedMsgType: signedMsgType,
		valSet:        valSet,
		votesBitArray: bits.NewBitArray(valSet.Size()),
		votes:         make([]*Vote, valSet.Size()),
		sum:           0,
		maj23:         nil,
		votesByBlock:  make(map[string]*blockVotes, valSet.Size()),
		peerMaj23s:    make(map[P2PID]BlockID),
	}
}

// NewExtendedVoteSet constructs a vote set with additional vote verification logic.
// The VoteSet constructed with NewExtendedVoteSet verifies the vote extension
// data for every vote added to the set.
func NewExtendedVoteSet(chainID string, height int64, round int32,
	signedMsgType cmtproto.SignedMsgType, valSet *ValidatorSet) *VoteSet {
	vs := NewVoteSet(chainID, height, round, signedMsgType, valSet)
	vs.extensionsEnabled = true
	return vs
}

func (voteSet *VoteSet) ChainID() string {
	return voteSet.chainID
}

// Implements VoteSetReader.
func (voteSet *VoteSet) GetHeight() int64 {
	if voteSet == nil {
		return 0
	}
	return voteSet.height
}

// Implements VoteSetReader.
func (voteSet *VoteSet) GetRound() int32 {
	if voteSet == nil {
		return -1
	}
	return voteSet.round
}

// Implements VoteSetReader.
func (voteSet *VoteSet) Type() byte {
	if voteSet == nil {
		return 0x00
	}
	return byte(voteSet.signedMsgType)
}

// Implements VoteSetReader.
func (voteSet *VoteSet) Size() int {
	if voteSet == nil {
		return 0
	}
	return voteSet.valSet.Size()
}

// Returns added=true if vote is valid and new.
// Otherwise returns err=ErrVote[
//
//	UnexpectedStep | InvalidIndex | InvalidAddress |
//	InvalidSignature | InvalidBlockHash | ConflictingVotes ]
//
// Duplicate votes return added=false, err=nil.
// Conflicting votes return added=*, err=ErrVoteConflictingVotes.
// NOTE: vote should not be mutated after adding.
// NOTE: VoteSet must not be nil
// NOTE: Vote must not be nil
func (voteSet *VoteSet) AddVote(vote *Vote) (added bool, err error) {
	if voteSet == nil {
		panic("AddVote() on nil VoteSet")
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()

	return voteSet.addVote(vote)
}

// NOTE: Validates as much as possible before attempting to verify the signature.
func (voteSet *VoteSet) addVote(vote *Vote) (added bool, err error) {
	if vote == nil {
		return false, ErrVoteNil
	}
	valIndex := vote.ValidatorIndex
	valAddr := vote.ValidatorAddress
	blockKey := vote.BlockID.Key()

	// Ensure that validator index was set
	if valIndex < 0 {
		return false, fmt.Errorf("index < 0: %w", ErrVoteInvalidValidatorIndex)
	} else if len(valAddr) == 0 {
		return false, fmt.Errorf("empty address: %w", ErrVoteInvalidValidatorAddress)
	}

	// Make sure the step matches.
	if (vote.Height != voteSet.height) ||
		(vote.Round != voteSet.round) ||
		(vote.Type != voteSet.signedMsgType) {
		return false, fmt.Errorf("expected %d/%d/%d, but got %d/%d/%d: %w",
			voteSet.height, voteSet.round, voteSet.signedMsgType,
			vote.Height, vote.Round, vote.Type, ErrVoteUnexpectedStep)
	}

	// Ensure that signer is a validator.
	lookupAddr, val := voteSet.valSet.GetByIndex(valIndex)
	if val == nil {
		return false, fmt.Errorf(
			"cannot find validator %d in valSet of size %d: %w",
			valIndex, voteSet.valSet.Size(), ErrVoteInvalidValidatorIndex)
	}

	// Ensure that the signer has the right address.
	if !bytes.Equal(valAddr, lookupAddr) {
		return false, fmt.Errorf(
			"vote.ValidatorAddress (%X) does not match address (%X) for vote.ValidatorIndex (%d)\n"+
				"Ensure the genesis file is correct across all validators: %w",
			valAddr, lookupAddr, valIndex, ErrVoteInvalidValidatorAddress)
	}

	// If we already know of this vote, return false.
	if existing, ok := voteSet.getVote(valIndex, blockKey); ok {
		if bytes.Equal(existing.Signature, vote.Signature) {
			return false, nil // duplicate
		}
		return false, fmt.Errorf("existing vote: %v; new vote: %v: %w", existing, vote, ErrVoteNonDeterministicSignature)
	}

	// Check signature.
	if voteSet.extensionsEnabled {
		if err := vote.VerifyVoteAndExtension(voteSet.chainID, val.PubKey); err != nil {
			return false, fmt.Errorf("failed to verify extended vote with ChainID %s and PubKey %s: %w", voteSet.chainID, val.PubKey, err)
		}
	} else {
		if err := vote.Verify(voteSet.chainID, val.PubKey); err != nil {
			return false, fmt.Errorf("failed to verify vote with ChainID %s and PubKey %s: %w", voteSet.chainID, val.PubKey, err)
		}
		if len(vote.ExtensionSignature) > 0 || len(vote.Extension) > 0 {
			return false, fmt.Errorf("unexpected vote extension data present in vote; ext_len %d, sig_len %d",
				len(vote.Extension),
				len(vote.ExtensionSignature),
			)
		}
	}

	// Add vote and get conflicting vote if any.
	added, conflicting := voteSet.addVerifiedVote(vote, blockKey, val.VotingPower)
	if conflicting != nil {
		return added, NewConflictingVoteError(conflicting, vote)
	}
	if !added {
		panic("Expected to add non-conflicting vote")
	}
	return added, nil
}

// Returns (vote, true) if vote exists for valIndex and blockKey.
func (voteSet *VoteSet) getVote(valIndex int32, blockKey string) (vote *Vote, ok bool) {
	if existing := voteSet.votes[valIndex]; existing != nil && existing.BlockID.Key() == blockKey {
		return existing, true
	}
	if existing := voteSet.votesByBlock[blockKey].getByIndex(valIndex); existing != nil {
		return existing, true
	}
	return nil, false
}

// Assumes signature is valid.
// If conflicting vote exists, returns it.
func (voteSet *VoteSet) addVerifiedVote(
	vote *Vote,
	blockKey string,
	votingPower int64,
) (added bool, conflicting *Vote) {
	valIndex := vote.ValidatorIndex

	// Already exists in voteSet.votes?
	if existing := voteSet.votes[valIndex]; existing != nil {
		if existing.BlockID.Equals(vote.BlockID) {
			panic("addVerifiedVote does not expect duplicate votes")
		}
		conflicting = existing
		// Replace vote if blockKey matches voteSet.maj23.
		if voteSet.maj23 != nil && voteSet.maj23.Key() == blockKey {
			voteSet.votes[valIndex] = vote
			voteSet.votesBitArray.SetIndex(int(valIndex), true)
		}
		// Otherwise don't add it to voteSet.votes
	} else {
		// Add to voteSet.votes and incr .sum
		voteSet.votes[valIndex] = vote
		voteSet.votesBitArray.SetIndex(int(valIndex), true)
		voteSet.sum += votingPower
	}

	votesByBlock, ok := voteSet.votesByBlock[blockKey]
	if ok {
		if conflicting != nil && !votesByBlock.peerMaj23 {
			// There's a conflict and no peer claims that this block is special.
			return false, conflicting
		}
		// We'll add the vote in a bit.
	} else {
		// .votesByBlock doesn't exist...
		if conflicting != nil {
			// ... and there's a conflicting vote.
			// We're not even tracking this blockKey, so just forget it.
			return false, conflicting
		}
		// ... and there's no conflicting vote.
		// Start tracking this blockKey
		votesByBlock = newBlockVotes(false, voteSet.valSet.Size())
		voteSet.votesByBlock[blockKey] = votesByBlock
		// We'll add the vote in a bit.
	}

	// Before adding to votesByBlock, see if we'll exceed quorum
	origSum := votesByBlock.sum
	quorum := voteSet.valSet.TotalVotingPower()*2/3 + 1

	// Add vote to votesByBlock
	votesByBlock.addVerifiedVote(vote, votingPower)

	// If we just crossed the quorum threshold and have 2/3 majority...
	if origSum < quorum && quorum <= votesByBlock.sum {
		// Only consider the first quorum reached
		if voteSet.maj23 == nil {
			maj23BlockID := vote.BlockID
			voteSet.maj23 = &maj23BlockID
			// And also copy votes over to voteSet.votes
			for i, vote := range votesByBlock.votes {
				if vote != nil {
					voteSet.votes[i] = vote
				}
			}
		}
	}

	return true, conflicting
}

// If a peer claims that it has 2/3 majority for given blockKey, call this.
// NOTE: if there are too many peers, or too much peer churn,
// this can cause memory issues.
// TODO: implement ability to remove peers too
// NOTE: VoteSet must not be nil
func (voteSet *VoteSet) SetPeerMaj23(peerID P2PID, blockID BlockID) error {
	if voteSet == nil {
		panic("SetPeerMaj23() on nil VoteSet")
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()

	blockKey := blockID.Key()

	// Make sure peer hasn't already told us something.
	if existing, ok := voteSet.peerMaj23s[peerID]; ok {
		if existing.Equals(blockID) {
			return nil // Nothing to do
		}
		return fmt.Errorf("setPeerMaj23: Received conflicting blockID from peer %v. Got %v, expected %v",
			peerID, blockID, existing)
	}
	voteSet.peerMaj23s[peerID] = blockID

	// Create .votesByBlock entry if needed.
	votesByBlock, ok := voteSet.votesByBlock[blockKey]
	if ok {
		if votesByBlock.peerMaj23 {
			return nil // Nothing to do
		}
		votesByBlock.peerMaj23 = true
		// No need to copy votes, already there.
	} else {
		votesByBlock = newBlockVotes(true, voteSet.valSet.Size())
		voteSet.votesByBlock[blockKey] = votesByBlock
		// No need to copy votes, no votes to copy over.
	}
	return nil
}

// Implements VoteSetReader.
func (voteSet *VoteSet) BitArray() *bits.BitArray {
	if voteSet == nil {
		return nil
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	return voteSet.votesBitArray.Copy()
}

func (voteSet *VoteSet) BitArrayByBlockID(blockID BlockID) *bits.BitArray {
	if voteSet == nil {
		return nil
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	votesByBlock, ok := voteSet.votesByBlock[blockID.Key()]
	if ok {
		return votesByBlock.bitArray.Copy()
	}
	return nil
}

// NOTE: if validator has conflicting votes, returns "canonical" vote
// Implements VoteSetReader.
func (voteSet *VoteSet) GetByIndex(valIndex int32) *Vote {
	if voteSet == nil {
		return nil
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	return voteSet.votes[valIndex]
}

// List returns a copy of the list of votes stored by the VoteSet.
func (voteSet *VoteSet) List() []Vote {
	if voteSet == nil || voteSet.votes == nil {
		return nil
	}
	votes := make([]Vote, 0, len(voteSet.votes))
	for i := range voteSet.votes {
		if voteSet.votes[i] != nil {
			votes = append(votes, *voteSet.votes[i])
		}
	}
	return votes
}

func (voteSet *VoteSet) GetByAddress(address []byte) *Vote {
	if voteSet == nil {
		return nil
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	valIndex, val := voteSet.valSet.GetByAddress(address)
	if val == nil {
		panic("GetByAddress(address) returned nil")
	}
	return voteSet.votes[valIndex]
}

func (voteSet *VoteSet) HasTwoThirdsMajority() bool {
	if voteSet == nil {
		return false
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	return voteSet.maj23 != nil
}

// Implements VoteSetReader.
func (voteSet *VoteSet) IsCommit() bool {
	if voteSet == nil {
		return false
	}
	if voteSet.signedMsgType != cmtproto.PrecommitType {
		return false
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	return voteSet.maj23 != nil
}

func (voteSet *VoteSet) HasTwoThirdsAny() bool {
	if voteSet == nil {
		return false
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	return voteSet.sum > voteSet.valSet.TotalVotingPower()*2/3
}

func (voteSet *VoteSet) HasAll() bool {
	if voteSet == nil {
		return false
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	return voteSet.sum == voteSet.valSet.TotalVotingPower()
}

// If there was a +2/3 majority for blockID, return blockID and true.
// Else, return the empty BlockID{} and false.
func (voteSet *VoteSet) TwoThirdsMajority() (blockID BlockID, ok bool) {
	if voteSet == nil {
		return BlockID{}, false
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	if voteSet.maj23 != nil {
		return *voteSet.maj23, true
	}
	return BlockID{}, false
}

//--------------------------------------------------------------------------------
// Strings and JSON

const nilVoteSetString = "nil-VoteSet"

// String returns a string representation of VoteSet.
//
// See StringIndented.
func (voteSet *VoteSet) String() string {
	if voteSet == nil {
		return nilVoteSetString
	}
	return voteSet.StringIndented("")
}

// StringIndented returns an indented String.
//
// Height Round Type
// Votes
// Votes bit array
// 2/3+ majority
//
// See Vote#String.
func (voteSet *VoteSet) StringIndented(indent string) string {
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()

	voteStrings := make([]string, len(voteSet.votes))
	for i, vote := range voteSet.votes {
		if vote == nil {
			voteStrings[i] = nilVoteStr
		} else {
			voteStrings[i] = vote.String()
		}
	}
	return fmt.Sprintf(`VoteSet{
%s  H:%v R:%v T:%v
%s  %v
%s  %v
%s  %v
%s}`,
		indent, voteSet.height, voteSet.round, voteSet.signedMsgType,
		indent, strings.Join(voteStrings, "\n"+indent+"  "),
		indent, voteSet.votesBitArray,
		indent, voteSet.peerMaj23s,
		indent)
}

// Marshal the VoteSet to JSON. Same as String(), just in JSON,
// and without the height/round/signedMsgType (since its already included in the votes).
func (voteSet *VoteSet) MarshalJSON() ([]byte, error) {
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	return cmtjson.Marshal(VoteSetJSON{
		voteSet.voteStrings(),
		voteSet.bitArrayString(),
		voteSet.peerMaj23s,
	})
}

// More human readable JSON of the vote set
// NOTE: insufficient for unmarshalling from (compressed votes)
// TODO: make the peerMaj23s nicer to read (eg just the block hash)
type VoteSetJSON struct {
	Votes         []string          `json:"votes"`
	VotesBitArray string            `json:"votes_bit_array"`
	PeerMaj23s    map[P2PID]BlockID `json:"peer_maj_23s"`
}

// Return the bit-array of votes including
// the fraction of power that has voted like:
// "BA{29:xx__x__x_x___x__x_______xxx__} 856/1304 = 0.66"
func (voteSet *VoteSet) BitArrayString() string {
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	return voteSet.bitArrayString()
}

func (voteSet *VoteSet) bitArrayString() string {
	bAString := voteSet.votesBitArray.String()
	voted, total, fracVoted := voteSet.sumTotalFrac()
	return fmt.Sprintf("%s %d/%d = %.2f", bAString, voted, total, fracVoted)
}

// Returns a list of votes compressed to more readable strings.
func (voteSet *VoteSet) VoteStrings() []string {
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	return voteSet.voteStrings()
}

func (voteSet *VoteSet) voteStrings() []string {
	voteStrings := make([]string, len(voteSet.votes))
	for i, vote := range voteSet.votes {
		if vote == nil {
			voteStrings[i] = nilVoteStr
		} else {
			voteStrings[i] = vote.String()
		}
	}
	return voteStrings
}

// StringShort returns a short representation of VoteSet.
//
// 1. height
// 2. round
// 3. signed msg type
// 4. first 2/3+ majority
// 5. fraction of voted power
// 6. votes bit array
// 7. 2/3+ majority for each peer
func (voteSet *VoteSet) StringShort() string {
	if voteSet == nil {
		return nilVoteSetString
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	_, _, frac := voteSet.sumTotalFrac()
	return fmt.Sprintf(`VoteSet{H:%v R:%v T:%v +2/3:%v(%v) %v %v}`,
		voteSet.height, voteSet.round, voteSet.signedMsgType, voteSet.maj23, frac, voteSet.votesBitArray, voteSet.peerMaj23s)
}

// LogString produces a logging suitable string representation of the
// vote set.
func (voteSet *VoteSet) LogString() string {
	if voteSet == nil {
		return nilVoteSetString
	}
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()
	voted, total, frac := voteSet.sumTotalFrac()

	return fmt.Sprintf("Votes:%d/%d(%.3f)", voted, total, frac)
}

// return the power voted, the total, and the fraction
func (voteSet *VoteSet) sumTotalFrac() (int64, int64, float64) {
	voted, total := voteSet.sum, voteSet.valSet.TotalVotingPower()
	fracVoted := float64(voted) / float64(total)
	return voted, total, fracVoted
}

//--------------------------------------------------------------------------------
// Commit

// MakeExtendedCommit constructs a Commit from the VoteSet. It only includes
// precommits for the block, which has 2/3+ majority, and nil.
//
// Panics if the vote type is not PrecommitType or if there's no +2/3 votes for
// a single block.
func (voteSet *VoteSet) MakeExtendedCommit(ap ABCIParams) *ExtendedCommit {
	voteSet.mtx.Lock()
	defer voteSet.mtx.Unlock()

	if voteSet.signedMsgType != cmtproto.PrecommitType {
		panic("Cannot MakeExtendCommit() unless VoteSet.Type is PrecommitType")
	}

	// Make sure we have a 2/3 majority
	if voteSet.maj23 == nil {
		panic("Cannot MakeExtendCommit() unless a blockhash has +2/3")
	}

	// For every validator, get the precommit with extensions
	sigs := make([]ExtendedCommitSig, len(voteSet.votes))
	for i, v := range voteSet.votes {
		sig := v.ExtendedCommitSig()
		// if block ID exists but doesn't match, exclude sig
		if sig.BlockIDFlag == BlockIDFlagCommit && !v.BlockID.Equals(*voteSet.maj23) {
			sig = NewExtendedCommitSigAbsent()
		}

		sigs[i] = sig
	}

	ec := &ExtendedCommit{
		Height:             voteSet.GetHeight(),
		Round:              voteSet.GetRound(),
		BlockID:            *voteSet.maj23,
		ExtendedSignatures: sigs,
	}
	if err := ec.EnsureExtensions(ap.VoteExtensionsEnabled(ec.Height)); err != nil {
		panic(fmt.Errorf("problem with vote extension data when making extended commit of height %d; %w",
			ec.Height, err))
	}
	return ec
}

//--------------------------------------------------------------------------------

/*
Votes for a particular block
There are two ways a *blockVotes gets created for a blockKey.
1. first (non-conflicting) vote of a validator w/ blockKey (peerMaj23=false)
2. A peer claims to have a 2/3 majority w/ blockKey (peerMaj23=true)
*/
type blockVotes struct {
	peerMaj23 bool           // peer claims to have maj23
	bitArray  *bits.BitArray // valIndex -> hasVote?
	votes     []*Vote        // valIndex -> *Vote
	sum       int64          // vote sum
}

func newBlockVotes(peerMaj23 bool, numValidators int) *blockVotes {
	return &blockVotes{
		peerMaj23: peerMaj23,
		bitArray:  bits.NewBitArray(numValidators),
		votes:     make([]*Vote, numValidators),
		sum:       0,
	}
}

func (vs *blockVotes) addVerifiedVote(vote *Vote, votingPower int64) {
	valIndex := vote.ValidatorIndex
	if existing := vs.votes[valIndex]; existing == nil {
		vs.bitArray.SetIndex(int(valIndex), true)
		vs.votes[valIndex] = vote
		vs.sum += votingPower
	}
}

func (vs *blockVotes) getByIndex(index int32) *Vote {
	if vs == nil {
		return nil
	}
	return vs.votes[index]
}

//--------------------------------------------------------------------------------

// Common interface between *consensus.VoteSet and types.Commit
type VoteSetReader interface {
	GetHeight() int64
	GetRound() int32
	Type() byte
	Size() int
	BitArray() *bits.BitArray
	GetByIndex(int32) *Vote
	IsCommit() bool
}
