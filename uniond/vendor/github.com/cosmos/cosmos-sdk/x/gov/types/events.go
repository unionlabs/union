package types

// Governance module event types
const (
	EventTypeSubmitProposal   = "submit_proposal"
	EventTypeProposalDeposit  = "proposal_deposit"
	EventTypeProposalVote     = "proposal_vote"
	EventTypeInactiveProposal = "inactive_proposal"
	EventTypeActiveProposal   = "active_proposal"
	EventTypeSignalProposal   = "signal_proposal"

	AttributeKeyVoter              = "voter"
	AttributeKeyProposalResult     = "proposal_result"
	AttributeKeyOption             = "option"
	AttributeKeyProposalID         = "proposal_id"
	AttributeKeyProposalMessages   = "proposal_messages" // Msg type_urls in the proposal
	AttributeKeyVotingPeriodStart  = "voting_period_start"
	AttributeValueProposalDropped  = "proposal_dropped"  // didn't meet min deposit
	AttributeValueProposalPassed   = "proposal_passed"   // met vote quorum
	AttributeValueProposalRejected = "proposal_rejected" // didn't meet vote quorum
	AttributeValueProposalFailed   = "proposal_failed"   // error on proposal handler
	AttributeKeyProposalType       = "proposal_type"
	AttributeSignalTitle           = "signal_title"
	AttributeSignalDescription     = "signal_description"
)
