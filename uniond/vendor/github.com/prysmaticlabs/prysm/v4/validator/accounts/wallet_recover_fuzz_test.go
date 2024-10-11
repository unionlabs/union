//go:build go1.18

package accounts_test

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
)

func FuzzValidateMnemonic(f *testing.F) {
	f.Add("bag wagon luxury iron swarm yellow course merge trumpet wide decide cash idea disagree deny teach canvas profit slice beach iron sting warfare slide")
	f.Add("bag wagon luxury iron swarm yellow course merge trumpet wide cash idea disagree deny teach canvas profit iron sting warfare slide")
	f.Add("bag wagon luxury iron swarm yellow course merge trumpet cash wide decide profit cash idea disagree deny teach canvas profit slice beach iron sting warfare slide")
	f.Fuzz(func(t *testing.T, input string) {
		err := accounts.ValidateMnemonic(input)
		_ = err
	})
}
