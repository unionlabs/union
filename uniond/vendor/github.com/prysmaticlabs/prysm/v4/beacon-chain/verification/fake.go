package verification

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
)

// BlobSidecarNoop is a FAKE verification function that simply launders a ROBlob->VerifiedROBlob.
// TODO: find all code that uses this method and replace it with full verification.
func BlobSidecarNoop(b blocks.ROBlob) (blocks.VerifiedROBlob, error) {
	return blocks.NewVerifiedROBlob(b), nil
}

// BlobSidecarSliceNoop is a FAKE verification function that simply launders a ROBlob->VerifiedROBlob.
// TODO: find all code that uses this method and replace it with full verification.
func BlobSidecarSliceNoop(b []blocks.ROBlob) ([]blocks.VerifiedROBlob, error) {
	vbs := make([]blocks.VerifiedROBlob, len(b))
	for i := range b {
		vbs[i] = blocks.NewVerifiedROBlob(b[i])
	}
	return vbs, nil
}

// FakeVerifyForTest can be used by tests that need a VerifiedROBlob but don't want to do all the
// expensive set up to perform full validation.
func FakeVerifyForTest(t *testing.T, b blocks.ROBlob) blocks.VerifiedROBlob {
	// log so that t is truly required
	t.Log("producing fake VerifiedROBlob for a test")
	return blocks.NewVerifiedROBlob(b)
}

// FakeVerifySliceForTest can be used by tests that need a []VerifiedROBlob but don't want to do all the
// expensive set up to perform full validation.
func FakeVerifySliceForTest(t *testing.T, b []blocks.ROBlob) []blocks.VerifiedROBlob {
	// log so that t is truly required
	t.Log("producing fake []VerifiedROBlob for a test")
	// tautological assertion that ensures this function can only be used in tests.
	vbs := make([]blocks.VerifiedROBlob, len(b))
	for i := range b {
		vbs[i] = blocks.NewVerifiedROBlob(b[i])
	}
	return vbs
}
