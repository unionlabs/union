package types_test

import (
	errorsmod "cosmossdk.io/errors"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

func (suite *TypesTestSuite) TestIsValid() {
	var timeout types.Timeout

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success: valid timeout with height and timestamp",
			func() {
				timeout = types.NewTimeout(clienttypes.NewHeight(1, 100), 100)
			},
			true,
		},
		{
			"success: valid timeout with height and zero timestamp",
			func() {
				timeout = types.NewTimeout(clienttypes.NewHeight(1, 100), 0)
			},
			true,
		},
		{
			"success: valid timeout with timestamp and zero height",
			func() {
				timeout = types.NewTimeout(clienttypes.ZeroHeight(), 100)
			},
			true,
		},
		{
			"invalid timeout with zero height and zero timestamp",
			func() {
				timeout = types.NewTimeout(clienttypes.ZeroHeight(), 0)
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			tc.malleate()

			isValid := timeout.IsValid()
			if tc.expPass {
				suite.Require().True(isValid)
			} else {
				suite.Require().False(isValid)
			}
		})
	}
}

func (suite *TypesTestSuite) TestElapsed() {
	// elapsed is expected to be true when either timeout height or timestamp
	// is greater than or equal to 2
	var (
		height    = clienttypes.NewHeight(0, 2)
		timestamp = uint64(2)
	)

	testCases := []struct {
		name       string
		timeout    types.Timeout
		expElapsed bool
	}{
		{
			"elapsed: both timeout with height and timestamp",
			types.NewTimeout(height, timestamp),
			true,
		},
		{
			"elapsed: timeout with height and zero timestamp",
			types.NewTimeout(height, 0),
			true,
		},
		{
			"elapsed: timeout with timestamp and zero height",
			types.NewTimeout(clienttypes.ZeroHeight(), timestamp),
			true,
		},
		{
			"elapsed: height elapsed, timestamp did not",
			types.NewTimeout(height, timestamp+1),
			true,
		},
		{
			"elapsed: timestamp elapsed, height did not",
			types.NewTimeout(height.Increment().(clienttypes.Height), timestamp),
			true,
		},
		{
			"elapsed: timestamp elapsed when less than current timestamp",
			types.NewTimeout(clienttypes.ZeroHeight(), timestamp-1),
			true,
		},
		{
			"elapsed: height elapsed when less than current height",
			types.NewTimeout(clienttypes.NewHeight(0, 1), 0),
			true,
		},
		{
			"not elapsed: invalid timeout",
			types.NewTimeout(clienttypes.ZeroHeight(), 0),
			false,
		},
		{
			"not elapsed: neither height nor timeout elapsed",
			types.NewTimeout(height.Increment().(clienttypes.Height), timestamp+1),
			false,
		},
		{
			"not elapsed: timeout not reached with height and zero timestamp",
			types.NewTimeout(height.Increment().(clienttypes.Height), 0),
			false,
		},
		{
			"elapsed: timeout not reached with timestamp and zero height",
			types.NewTimeout(clienttypes.ZeroHeight(), timestamp+1),
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			elapsed := tc.timeout.Elapsed(height, timestamp)
			suite.Require().Equal(tc.expElapsed, elapsed)
		})
	}
}

func (suite *TypesTestSuite) TestErrTimeoutElapsed() {
	// elapsed is expected to be true when either timeout height or timestamp
	// is greater than or equal to 2
	var (
		height    = clienttypes.NewHeight(0, 2)
		timestamp = uint64(2)
	)

	testCases := []struct {
		name     string
		timeout  types.Timeout
		expError error
	}{
		{
			"both timeout with height and timestamp",
			types.NewTimeout(height, timestamp),
			errorsmod.Wrapf(types.ErrTimeoutElapsed, "current height: %s, timeout height %s", height, height),
		},
		{
			"timeout with height and zero timestamp",
			types.NewTimeout(height, 0),
			errorsmod.Wrapf(types.ErrTimeoutElapsed, "current height: %s, timeout height %s", height, height),
		},
		{
			"timeout with timestamp and zero height",
			types.NewTimeout(clienttypes.ZeroHeight(), timestamp),
			errorsmod.Wrapf(types.ErrTimeoutElapsed, "current timestamp: %d, timeout timestamp %d", timestamp, timestamp),
		},
		{
			"height elapsed, timestamp did not",
			types.NewTimeout(height, timestamp+1),
			errorsmod.Wrapf(types.ErrTimeoutElapsed, "current height: %s, timeout height %s", height, height),
		},
		{
			"timestamp elapsed, height did not",
			types.NewTimeout(height.Increment().(clienttypes.Height), timestamp),
			errorsmod.Wrapf(types.ErrTimeoutElapsed, "current timestamp: %d, timeout timestamp %d", timestamp, timestamp),
		},
		{
			"height elapsed when less than current height",
			types.NewTimeout(clienttypes.NewHeight(0, 1), 0),
			errorsmod.Wrapf(types.ErrTimeoutElapsed, "current height: %s, timeout height %s", height, clienttypes.NewHeight(0, 1)),
		},
		{
			"timestamp elapsed when less than current timestamp",
			types.NewTimeout(clienttypes.ZeroHeight(), timestamp-1),
			errorsmod.Wrapf(types.ErrTimeoutElapsed, "current timestamp: %d, timeout timestamp %d", timestamp, timestamp-1),
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			err := tc.timeout.ErrTimeoutElapsed(height, timestamp)
			suite.Require().Equal(tc.expError.Error(), err.Error())
		})
	}
}

func (suite *TypesTestSuite) TestErrTimeoutNotReached() {
	// elapsed is expected to be true when either timeout height or timestamp
	// is greater than or equal to 2
	var (
		height    = clienttypes.NewHeight(0, 2)
		timestamp = uint64(2)
	)

	testCases := []struct {
		name     string
		timeout  types.Timeout
		expError error
	}{
		{
			"neither timeout reached with height and timestamp",
			types.NewTimeout(height.Increment().(clienttypes.Height), timestamp+1),
			errorsmod.Wrapf(types.ErrTimeoutNotReached, "current height: %s, timeout height %s", height, height.Increment().(clienttypes.Height)),
		},
		{
			"timeout not reached with height and zero timestamp",
			types.NewTimeout(height.Increment().(clienttypes.Height), 0),
			errorsmod.Wrapf(types.ErrTimeoutNotReached, "current height: %s, timeout height %s", height, height.Increment().(clienttypes.Height)),
		},
		{
			"timeout not reached with timestamp and zero height",
			types.NewTimeout(clienttypes.ZeroHeight(), timestamp+1),
			errorsmod.Wrapf(types.ErrTimeoutNotReached, "current timestamp: %d, timeout timestamp %d", timestamp, timestamp+1),
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			err := tc.timeout.ErrTimeoutNotReached(height, timestamp)
			suite.Require().Equal(tc.expError.Error(), err.Error())
		})
	}
}
