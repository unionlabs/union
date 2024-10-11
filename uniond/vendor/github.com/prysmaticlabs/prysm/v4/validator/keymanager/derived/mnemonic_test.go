package derived

import (
	"testing"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/tyler-smith/go-bip39"
	"github.com/tyler-smith/go-bip39/wordlists"
)

func TestMnemonic_Generate_CanRecover(t *testing.T) {
	generator := &MnemonicGenerator{}
	data := make([]byte, 32)
	copy(data, "hello-world")
	phrase, err := generator.Generate(data)
	require.NoError(t, err)
	entropy, err := bip39.EntropyFromMnemonic(phrase)
	require.NoError(t, err)
	assert.DeepEqual(t, data, entropy, "Expected to recover original data")
}

func Test_setBip39Lang(t *testing.T) {
	tests := []struct {
		lang             string
		expectedWordlist []string
		wantErr          error
	}{
		{lang: "english", expectedWordlist: wordlists.English, wantErr: nil},
		{lang: "chinese_traditional", expectedWordlist: wordlists.ChineseTraditional, wantErr: nil},
		{lang: "chinese_simplified", expectedWordlist: wordlists.ChineseSimplified, wantErr: nil},
		{lang: "czech", expectedWordlist: wordlists.Czech, wantErr: nil},
		{lang: "french", expectedWordlist: wordlists.French, wantErr: nil},
		{lang: "japanese", expectedWordlist: wordlists.Japanese, wantErr: nil},
		{lang: "korean", expectedWordlist: wordlists.Korean, wantErr: nil},
		{lang: "italian", expectedWordlist: wordlists.Italian, wantErr: nil},
		{lang: "spanish", expectedWordlist: wordlists.Spanish, wantErr: nil},
		{lang: "undefined", expectedWordlist: []string{}, wantErr: errors.Wrapf(ErrUnsupportedMnemonicLanguage, "%s", "undefined")},
	}
	for _, tt := range tests {
		t.Run(tt.lang, func(t *testing.T) {
			err := setBip39Lang(tt.lang)
			if err != nil {
				assert.Equal(t, tt.wantErr.Error(), err.Error())
			} else {
				assert.Equal(t, tt.wantErr, err)
				wordlist := bip39.GetWordList()
				assert.DeepEqual(t, tt.expectedWordlist, wordlist, "Expected wordlist to match")
			}
		})
	}
}
