package types

// ProposalMetadata is the metadata of a proposal
// This metadata is supposed to live off-chain when submitted in a proposal
type ProposalMetadata struct {
	Title             string   `json:"title"`
	Authors           []string `json:"authors"`
	Summary           string   `json:"summary"`
	Details           string   `json:"details"`
	ProposalForumUrl  string   `json:"proposal_forum_url"`
	VoteOptionContext string   `json:"vote_option_context"`
}
