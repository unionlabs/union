package jose

import (
	"errors"
)

// Plaintext (no signing) signing algorithm implementation
type Plaintext struct{}

func init() {
	RegisterJws(new(Plaintext))
}

func (alg *Plaintext) Name() string {
	return NONE
}

func (alg *Plaintext) Verify(securedInput []byte, signature []byte, key interface{}) error {

	if key != nil {
		return errors.New("Plaintext.Verify() expects key to be nil")
	}

	if len(signature) != 0 {
		return errors.New("Plaintext.Verify() expects signature to be empty.")
	}

	return nil
}

func (alg *Plaintext) Sign(securedInput []byte, key interface{}) (signature []byte, err error) {

	if key != nil {
		return nil, errors.New("Plaintext.Verify() expects key to be nil")
	}

	return []byte{}, nil
}
