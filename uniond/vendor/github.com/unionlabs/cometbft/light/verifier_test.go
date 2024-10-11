package light_test

import (
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	cmtmath "github.com/cometbft/cometbft/libs/math"
	"github.com/cometbft/cometbft/light"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
)

const (
	maxClockDrift = 10 * time.Second
)

func TestVerifyAdjacentHeaders(t *testing.T) {
	const (
		chainID    = "TestVerifyAdjacentHeaders"
		lastHeight = 1
		nextHeight = 2
	)

	var (
		keys = genPrivKeys(4)
		// 20, 30, 40, 50 - the first 3 don't have 2/3, the last 3 do!
		vals     = keys.ToValidators(20, 10)
		bTime, _ = time.Parse(time.RFC3339, "2006-01-02T15:04:05Z")
		header   = keys.GenSignedHeader(chainID, lastHeight, bTime, nil, vals, vals,
			hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys))
	)

	testCases := []struct {
		newHeader      *types.SignedHeader
		newVals        *types.ValidatorSet
		trustingPeriod time.Duration
		now            time.Time
		expErr         error
		expErrText     string
	}{
		// same header -> no error
		0: {
			header,
			vals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"headers must be adjacent in height",
		},
		// different chainID -> error
		1: {
			keys.GenSignedHeader("different-chainID", nextHeight, bTime.Add(1*time.Hour), nil, vals, vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
			vals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"header belongs to another chain",
		},
		// new header's time is before old header's time -> error
		2: {
			keys.GenSignedHeader(chainID, nextHeight, bTime.Add(-1*time.Hour), nil, vals, vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
			vals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"to be after old header time",
		},
		// new header's time is from the future -> error
		3: {
			keys.GenSignedHeader(chainID, nextHeight, bTime.Add(3*time.Hour), nil, vals, vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
			vals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"new header has a time from the future",
		},
		// new header's time is from the future, but it's acceptable (< maxClockDrift) -> no error
		4: {
			keys.GenSignedHeader(chainID, nextHeight,
				bTime.Add(2*time.Hour).Add(maxClockDrift).Add(-1*time.Millisecond), nil, vals, vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
			vals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"",
		},
		// 3/3 signed -> no error
		5: {
			keys.GenSignedHeader(chainID, nextHeight, bTime.Add(1*time.Hour), nil, vals, vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
			vals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"",
		},
		// 2/3 signed -> no error
		6: {
			keys.GenSignedHeader(chainID, nextHeight, bTime.Add(1*time.Hour), nil, vals, vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 1, len(keys)),
			vals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"",
		},
		// 1/3 signed -> error
		7: {
			keys.GenSignedHeader(chainID, nextHeight, bTime.Add(1*time.Hour), nil, vals, vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), len(keys)-1, len(keys)),
			vals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			light.ErrInvalidHeader{Reason: types.ErrNotEnoughVotingPowerSigned{Got: 50, Needed: 93}},
			"",
		},
		// vals does not match with what we have -> error
		8: {
			keys.GenSignedHeader(chainID, nextHeight, bTime.Add(1*time.Hour), nil, keys.ToValidators(10, 1), vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
			keys.ToValidators(10, 1),
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"to match those from new header",
		},
		// vals are inconsistent with newHeader -> error
		9: {
			keys.GenSignedHeader(chainID, nextHeight, bTime.Add(1*time.Hour), nil, vals, vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
			keys.ToValidators(10, 1),
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"to match those that were supplied",
		},
		// old header has expired -> error
		10: {
			keys.GenSignedHeader(chainID, nextHeight, bTime.Add(1*time.Hour), nil, vals, vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
			keys.ToValidators(10, 1),
			1 * time.Hour,
			bTime.Add(1 * time.Hour),
			nil,
			"old header has expired",
		},
	}

	for i, tc := range testCases {
		t.Run(fmt.Sprintf("#%d", i), func(t *testing.T) {
			err := light.VerifyAdjacent(header, tc.newHeader, tc.newVals, tc.trustingPeriod, tc.now, maxClockDrift)
			switch {
			case tc.expErr != nil && assert.Error(t, err): //nolint:testifylint // require.Error doesn't work with the logic here
				assert.Equal(t, tc.expErr, err)
			case tc.expErrText != "":
				assert.Contains(t, err.Error(), tc.expErrText)
			default:
				require.NoError(t, err)
			}
		})
	}
}

func TestVerifyNonAdjacentHeaders(t *testing.T) {
	const (
		chainID    = "TestVerifyNonAdjacentHeaders"
		lastHeight = 1
	)

	var (
		keys = genPrivKeys(4)
		// 20, 30, 40, 50 - the first 3 don't have 2/3, the last 3 do!
		vals     = keys.ToValidators(20, 10)
		bTime, _ = time.Parse(time.RFC3339, "2006-01-02T15:04:05Z")
		header   = keys.GenSignedHeader(chainID, lastHeight, bTime, nil, vals, vals,
			hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys))

		// 30, 40, 50
		twoThirds     = keys[1:]
		twoThirdsVals = twoThirds.ToValidators(30, 10)

		// 50
		oneThird     = keys[len(keys)-1:]
		oneThirdVals = oneThird.ToValidators(50, 10)

		// 20
		lessThanOneThird     = keys[0:1]
		lessThanOneThirdVals = lessThanOneThird.ToValidators(20, 10)
	)

	testCases := []struct {
		newHeader      *types.SignedHeader
		newVals        *types.ValidatorSet
		trustingPeriod time.Duration
		now            time.Time
		expErr         error
		expErrText     string
	}{
		// 3/3 new vals signed, 3/3 old vals present -> no error
		0: {
			keys.GenSignedHeader(chainID, 3, bTime.Add(1*time.Hour), nil, vals, vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys)),
			vals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"",
		},
		// 2/3 new vals signed, 3/3 old vals present -> no error
		1: {
			keys.GenSignedHeader(chainID, 4, bTime.Add(1*time.Hour), nil, vals, vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 1, len(keys)),
			vals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"",
		},
		// 1/3 new vals signed, 3/3 old vals present -> error
		2: {
			keys.GenSignedHeader(chainID, 5, bTime.Add(1*time.Hour), nil, vals, vals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), len(keys)-1, len(keys)),
			vals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			light.ErrInvalidHeader{types.ErrNotEnoughVotingPowerSigned{Got: 50, Needed: 93}},
			"",
		},
		// 3/3 new vals signed, 2/3 old vals present -> no error
		3: {
			twoThirds.GenSignedHeader(chainID, 5, bTime.Add(1*time.Hour), nil, twoThirdsVals, twoThirdsVals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(twoThirds)),
			twoThirdsVals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"",
		},
		// 3/3 new vals signed, 1/3 old vals present -> no error
		4: {
			oneThird.GenSignedHeader(chainID, 5, bTime.Add(1*time.Hour), nil, oneThirdVals, oneThirdVals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(oneThird)),
			oneThirdVals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			nil,
			"",
		},
		// 3/3 new vals signed, less than 1/3 old vals present -> error
		5: {
			lessThanOneThird.GenSignedHeader(chainID, 5, bTime.Add(1*time.Hour), nil, lessThanOneThirdVals, lessThanOneThirdVals,
				hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(lessThanOneThird)),
			lessThanOneThirdVals,
			3 * time.Hour,
			bTime.Add(2 * time.Hour),
			light.ErrNewValSetCantBeTrusted{types.ErrNotEnoughVotingPowerSigned{Got: 20, Needed: 46}},
			"",
		},
	}

	for i, tc := range testCases {
		t.Run(fmt.Sprintf("#%d", i), func(t *testing.T) {
			err := light.VerifyNonAdjacent(header, vals, tc.newHeader, tc.newVals, tc.trustingPeriod,
				tc.now, maxClockDrift,
				light.DefaultTrustLevel)

			switch {
			case tc.expErr != nil && assert.Error(t, err): //nolint:testifylint // require.Error doesn't work with the logic here
				assert.Equal(t, tc.expErr, err)
			case tc.expErrText != "":
				assert.Contains(t, err.Error(), tc.expErrText)
			default:
				require.NoError(t, err)
			}
		})
	}
}

func TestVerifyReturnsErrorIfTrustLevelIsInvalid(t *testing.T) {
	const (
		chainID    = "TestVerifyReturnsErrorIfTrustLevelIsInvalid"
		lastHeight = 1
	)

	var (
		keys = genPrivKeys(4)
		// 20, 30, 40, 50 - the first 3 don't have 2/3, the last 3 do!
		vals     = keys.ToValidators(20, 10)
		bTime, _ = time.Parse(time.RFC3339, "2006-01-02T15:04:05Z")
		header   = keys.GenSignedHeader(chainID, lastHeight, bTime, nil, vals, vals,
			hash("app_hash"), hash("cons_hash"), hash("results_hash"), 0, len(keys))
	)

	err := light.Verify(header, vals, header, vals, 2*time.Hour, cmttime.Now(), maxClockDrift,
		cmtmath.Fraction{Numerator: 2, Denominator: 1})
	require.Error(t, err)
}

func TestValidateTrustLevel(t *testing.T) {
	testCases := []struct {
		lvl   cmtmath.Fraction
		valid bool
	}{
		// valid
		0: {cmtmath.Fraction{Numerator: 1, Denominator: 1}, true},
		1: {cmtmath.Fraction{Numerator: 1, Denominator: 3}, true},
		2: {cmtmath.Fraction{Numerator: 2, Denominator: 3}, true},
		3: {cmtmath.Fraction{Numerator: 3, Denominator: 3}, true},
		4: {cmtmath.Fraction{Numerator: 4, Denominator: 5}, true},

		// invalid
		5: {cmtmath.Fraction{Numerator: 6, Denominator: 5}, false},
		6: {cmtmath.Fraction{Numerator: 0, Denominator: 1}, false},
		7: {cmtmath.Fraction{Numerator: 0, Denominator: 0}, false},
		8: {cmtmath.Fraction{Numerator: 1, Denominator: 0}, false},
	}

	for _, tc := range testCases {
		err := light.ValidateTrustLevel(tc.lvl)
		if !tc.valid {
			require.Error(t, err)
		} else {
			require.NoError(t, err)
		}
	}
}
