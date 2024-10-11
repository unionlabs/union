package derived

import (
	"fmt"
	"os"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	"github.com/prysmaticlabs/prysm/v4/io/prompt"
	"github.com/tyler-smith/go-bip39"
	"github.com/tyler-smith/go-bip39/wordlists"
)

const confirmationText = "Confirm you have written down the recovery words somewhere safe (offline) [y|Y]"

// MnemonicGenerator implements methods for creating
// mnemonic seed phrases in english using a given
// source of entropy such as a private key.
type MnemonicGenerator struct {
	skipMnemonicConfirm bool
}

// ErrUnsupportedMnemonicLanguage is returned when trying to use an unsupported mnemonic language.
var (
	DefaultMnemonicLanguage        = "english"
	ErrUnsupportedMnemonicLanguage = errors.New("unsupported mnemonic language")
)

// GenerateAndConfirmMnemonic requires confirming the generated mnemonics.
func GenerateAndConfirmMnemonic(mnemonicLanguage string, skipMnemonicConfirm bool) (string, error) {
	mnemonicRandomness := make([]byte, 32)
	if _, err := rand.NewGenerator().Read(mnemonicRandomness); err != nil {
		return "", errors.Wrap(err, "could not initialize mnemonic source of randomness")
	}
	err := setBip39Lang(mnemonicLanguage)
	if err != nil {
		return "", err
	}
	m := &MnemonicGenerator{
		skipMnemonicConfirm: skipMnemonicConfirm,
	}
	phrase, err := m.Generate(mnemonicRandomness)
	if err != nil {
		return "", errors.Wrap(err, "could not generate wallet seed")
	}
	if err := m.ConfirmAcknowledgement(phrase); err != nil {
		return "", errors.Wrap(err, "could not confirm mnemonic acknowledgement")
	}
	return phrase, nil
}

// Generate a mnemonic seed phrase in english using a source of
// entropy given as raw bytes.
func (_ *MnemonicGenerator) Generate(data []byte) (string, error) {
	return bip39.NewMnemonic(data)
}

// ConfirmAcknowledgement displays the mnemonic phrase to the user
// and confirms the user has written down the phrase securely offline.
func (m *MnemonicGenerator) ConfirmAcknowledgement(phrase string) error {
	log.Info(
		"Write down the sentence below, as it is your only " +
			"means of recovering your wallet",
	)
	fmt.Printf(
		`=================Wallet Seed Recovery Phrase====================

%s

===================================================================`,
		phrase)
	fmt.Println("")
	if m.skipMnemonicConfirm {
		return nil
	}
	// Confirm the user has written down the mnemonic phrase offline.
	_, err := prompt.ValidatePrompt(os.Stdin, confirmationText, prompt.ValidateConfirmation)
	if err != nil {
		log.Errorf("Could not confirm acknowledgement of userprompt, please enter y")
	}
	return nil
}

// Uses the provided mnemonic seed phrase to generate the
// appropriate seed file for recovering a derived wallets.
func seedFromMnemonic(mnemonic, mnemonicLanguage, mnemonicPassphrase string) ([]byte, error) {
	err := setBip39Lang(mnemonicLanguage)
	if err != nil {
		return nil, err
	}
	if ok := bip39.IsMnemonicValid(mnemonic); !ok {
		return nil, bip39.ErrInvalidMnemonic
	}
	return bip39.NewSeed(mnemonic, mnemonicPassphrase), nil
}

func setBip39Lang(lang string) error {
	var wordlist []string
	allowedLanguages := map[string][]string{
		"chinese_simplified":  wordlists.ChineseSimplified,
		"chinese_traditional": wordlists.ChineseTraditional,
		"czech":               wordlists.Czech,
		"english":             wordlists.English,
		"french":              wordlists.French,
		"japanese":            wordlists.Japanese,
		"korean":              wordlists.Korean,
		"italian":             wordlists.Italian,
		"spanish":             wordlists.Spanish,
	}

	if wl, ok := allowedLanguages[lang]; ok {
		wordlist = wl
	} else {
		return errors.Wrapf(ErrUnsupportedMnemonicLanguage, "%s", lang)
	}
	bip39.SetWordList(wordlist)
	return nil
}
