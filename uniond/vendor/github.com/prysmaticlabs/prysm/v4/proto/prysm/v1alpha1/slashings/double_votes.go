package slashings

import (
	"bytes"
)

// SigningRootsDiffer verifies that an incoming vs. existing attestation has a different signing root.
// If the existing signing root is empty, then we consider an attestation as different always.
func SigningRootsDiffer(existingSigningRoot, incomingSigningRoot []byte) bool {
	// If the existing signing root is empty, we always consider the incoming
	// attestation as a double vote to be safe.
	if len(existingSigningRoot) == 0 {
		return true
	}
	// Otherwise, we consider any sort of inequality to be a double vote.
	return !bytes.Equal(existingSigningRoot, incomingSigningRoot)
}
