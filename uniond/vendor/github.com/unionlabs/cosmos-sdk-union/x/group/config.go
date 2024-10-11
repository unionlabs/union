package group

import "time"

// Config used to initialize x/group module avoiding using global variable.
type Config struct {
	// MaxExecutionPeriod defines the max duration after a proposal's voting
	// period ends that members can send a MsgExec to execute the proposal.
	MaxExecutionPeriod time.Duration

	// MaxMetadataLen defines the max chars allowed in all
	// messages that allows creating or updating a group
	// with a metadata field
	MaxMetadataLen uint64

	// title field
	// Defaults to 255 if not explicitly set.
	MaxProposalTitleLen uint64

	// summary field
	// Defaults to 10200 if not explicitly set.
	MaxProposalSummaryLen uint64
}

// DefaultConfig returns the default config for group.
func DefaultConfig() Config {
	return Config{
		MaxExecutionPeriod:    2 * time.Hour * 24 * 7, // Two weeks.
		MaxMetadataLen:        255,
		MaxProposalTitleLen:   255,
		MaxProposalSummaryLen: 10200,
	}
}
