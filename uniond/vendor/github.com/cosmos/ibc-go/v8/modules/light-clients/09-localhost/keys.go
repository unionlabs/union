package localhost

const (
	// ModuleName defines the 09-localhost light client module name
	ModuleName = "09-localhost"
)

// SentinelProof defines the 09-localhost sentinel proof.
// Submission of nil or empty proofs is disallowed in core IBC messaging.
// This serves as a placeholder value for relayers to leverage as the proof field in various message types.
// Localhost client state verification will fail if the sentintel proof value is not provided.
var SentinelProof = []byte{0x01}
