package types

import (
	"fmt"
	"math/big"
	"regexp"
	"strconv"
	"strings"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ exported.Height = (*Height)(nil)

// IsRevisionFormat checks if a chainID is in the format required for parsing revisions
// The chainID must be in the form: `{chainID}-{revision}`.
// 24-host may enforce stricter checks on chainID
var IsRevisionFormat = regexp.MustCompile(`^.*[^\n-]-{1}[1-9][0-9]*$`).MatchString

// ZeroHeight is a helper function which returns an uninitialized height.
func ZeroHeight() Height {
	return Height{}
}

// NewHeight is a constructor for the IBC height type
func NewHeight(revisionNumber, revisionHeight uint64) Height {
	return Height{
		RevisionNumber: revisionNumber,
		RevisionHeight: revisionHeight,
	}
}

// GetRevisionNumber returns the revision-number of the height
func (h Height) GetRevisionNumber() uint64 {
	return h.RevisionNumber
}

// GetRevisionHeight returns the revision-height of the height
func (h Height) GetRevisionHeight() uint64 {
	return h.RevisionHeight
}

// Compare implements a method to compare two heights. When comparing two heights a, b
// we can call a.Compare(b) which will return
// -1 if a < b
// 0  if a = b
// 1  if a > b
//
// It first compares based on revision numbers, whichever has the higher revision number is the higher height
// If revision number is the same, then the revision height is compared
func (h Height) Compare(other exported.Height) int64 {
	height, ok := other.(Height)
	if !ok {
		panic(fmt.Errorf("cannot compare against invalid height type: %T. expected height type: %T", other, h))
	}
	var a, b big.Int
	if h.RevisionNumber != height.RevisionNumber {
		a.SetUint64(h.RevisionNumber)
		b.SetUint64(height.RevisionNumber)
	} else {
		a.SetUint64(h.RevisionHeight)
		b.SetUint64(height.RevisionHeight)
	}
	return int64(a.Cmp(&b))
}

// LT Helper comparison function returns true if h < other
func (h Height) LT(other exported.Height) bool {
	return h.Compare(other) == -1
}

// LTE Helper comparison function returns true if h <= other
func (h Height) LTE(other exported.Height) bool {
	cmp := h.Compare(other)
	return cmp <= 0
}

// GT Helper comparison function returns true if h > other
func (h Height) GT(other exported.Height) bool {
	return h.Compare(other) == 1
}

// GTE Helper comparison function returns true if h >= other
func (h Height) GTE(other exported.Height) bool {
	cmp := h.Compare(other)
	return cmp >= 0
}

// EQ Helper comparison function returns true if h == other
func (h Height) EQ(other exported.Height) bool {
	return h.Compare(other) == 0
}

// String returns a string representation of Height
func (h Height) String() string {
	return fmt.Sprintf("%d-%d", h.RevisionNumber, h.RevisionHeight)
}

// Decrement will return a new height with the RevisionHeight decremented
// If the RevisionHeight is already at lowest value (1), then false success flag is returend
func (h Height) Decrement() (decremented exported.Height, success bool) {
	if h.RevisionHeight == 0 {
		return Height{}, false
	}
	return NewHeight(h.RevisionNumber, h.RevisionHeight-1), true
}

// Increment will return a height with the same revision number but an
// incremented revision height
func (h Height) Increment() exported.Height {
	return NewHeight(h.RevisionNumber, h.RevisionHeight+1)
}

// IsZero returns true if height revision and revision-height are both 0
func (h Height) IsZero() bool {
	return h.RevisionNumber == 0 && h.RevisionHeight == 0
}

// MustParseHeight will attempt to parse a string representation of a height and panic if
// parsing fails.
func MustParseHeight(heightStr string) Height {
	height, err := ParseHeight(heightStr)
	if err != nil {
		panic(err)
	}

	return height
}

// ParseHeight is a utility function that takes a string representation of the height
// and returns a Height struct
func ParseHeight(heightStr string) (Height, error) {
	splitStr := strings.Split(heightStr, "-")
	if len(splitStr) != 2 {
		return Height{}, errorsmod.Wrapf(ibcerrors.ErrInvalidHeight, "expected height string format: {revision}-{height}. Got: %s", heightStr)
	}
	revisionNumber, err := strconv.ParseUint(splitStr[0], 10, 64)
	if err != nil {
		return Height{}, errorsmod.Wrapf(ibcerrors.ErrInvalidHeight, "invalid revision number. parse err: %s", err)
	}
	revisionHeight, err := strconv.ParseUint(splitStr[1], 10, 64)
	if err != nil {
		return Height{}, errorsmod.Wrapf(ibcerrors.ErrInvalidHeight, "invalid revision height. parse err: %s", err)
	}
	return NewHeight(revisionNumber, revisionHeight), nil
}

// SetRevisionNumber takes a chainID in valid revision format and swaps the revision number
// in the chainID with the given revision number.
func SetRevisionNumber(chainID string, revision uint64) (string, error) {
	if !IsRevisionFormat(chainID) {
		return "", errorsmod.Wrapf(
			ibcerrors.ErrInvalidChainID, "chainID is not in revision format: %s", chainID,
		)
	}

	splitStr := strings.Split(chainID, "-")
	// swap out revision number with given revision
	splitStr[len(splitStr)-1] = strconv.Itoa(int(revision))
	return strings.Join(splitStr, "-"), nil
}

// ParseChainID is a utility function that returns an revision number from the given ChainID.
// ParseChainID attempts to parse a chain id in the format: `{chainID}-{revision}`
// and return the revisionnumber as a uint64.
// If the chainID is not in the expected format, a default revision value of 0 is returned.
func ParseChainID(chainID string) uint64 {
	if !IsRevisionFormat(chainID) {
		// chainID is not in revision format, return 0 as default
		return 0
	}
	splitStr := strings.Split(chainID, "-")
	revision, err := strconv.ParseUint(splitStr[len(splitStr)-1], 10, 64)
	// sanity check: error should always be nil since regex only allows numbers in last element
	if err != nil {
		panic(fmt.Errorf("regex allowed non-number value as last split element for chainID: %s", chainID))
	}
	return revision
}

// GetSelfHeight is a utility function that returns self height given context
// Revision number is retrieved from ctx.ChainID()
func GetSelfHeight(ctx sdk.Context) Height {
	revision := ParseChainID(ctx.ChainID())
	return NewHeight(revision, uint64(ctx.BlockHeight()))
}
