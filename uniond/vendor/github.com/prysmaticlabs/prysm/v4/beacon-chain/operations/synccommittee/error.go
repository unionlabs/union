package synccommittee

import "github.com/pkg/errors"

var (
	errNilMessage      = errors.New("sync committee message is nil")
	errNilContribution = errors.New("sync committee contribution is nil")
)
