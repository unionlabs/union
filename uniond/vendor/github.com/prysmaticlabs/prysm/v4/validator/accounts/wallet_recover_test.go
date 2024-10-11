package accounts

import (
	"testing"

	"github.com/pkg/errors"
)

func TestValidateMnemonic(t *testing.T) {
	tests := []struct {
		name     string
		mnemonic string
		wantErr  bool
		err      error
	}{
		{
			name:     "Valid Mnemonic",
			mnemonic: "bag wagon luxury iron swarm yellow course merge trumpet wide decide cash idea disagree deny teach canvas profit slice beach iron sting warfare slide",
			wantErr:  false,
		},
		{
			name:     "Invalid Mnemonic with too few words",
			mnemonic: "bag wagon luxury iron swarm yellow course merge trumpet wide cash idea disagree deny teach canvas profit iron sting warfare slide",
			wantErr:  true,
			err:      ErrIncorrectWordNumber,
		},
		{
			name:     "Invalid Mnemonic with too many words",
			mnemonic: "bag wagon luxury iron swarm yellow course merge trumpet cash wide decide profit cash idea disagree deny teach canvas profit slice beach iron sting warfare slide",
			wantErr:  true,
			err:      ErrIncorrectWordNumber,
		},
		{
			name:     "Empty Mnemonic",
			mnemonic: "",
			wantErr:  true,
			err:      ErrEmptyMnemonic,
		},
		{
			name:     "Junk Mnemonic",
			mnemonic: "                         a      ",
			wantErr:  true,
			err:      ErrIncorrectWordNumber,
		},
		{
			name:     "Junk Mnemonic ",
			mnemonic: " evilpacket ",
			wantErr:  true,
			err:      ErrIncorrectWordNumber,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateMnemonic(tt.mnemonic)
			if (err != nil) != tt.wantErr || (tt.wantErr && !errors.Is(err, tt.err)) {
				t.Errorf("ValidateMnemonic() error = %v, wantErr %v", err, tt.err)
			}
		})
	}
}
