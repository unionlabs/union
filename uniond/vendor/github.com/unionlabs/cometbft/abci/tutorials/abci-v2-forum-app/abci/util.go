package abci

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"strings"

	"github.com/dgraph-io/badger/v4"

	"github.com/cometbft/cometbft/abci/tutorials/abci-v2-forum-app/model"
	"github.com/cometbft/cometbft/abci/types"
	cryptoencoding "github.com/cometbft/cometbft/crypto/encoding"
)

func isBanTx(tx []byte) bool {
	return strings.Contains(string(tx), "username")
}

func (app *ForumApp) updateValidator(v types.ValidatorUpdate) error {
	pubKey, err := cryptoencoding.PubKeyFromTypeAndBytes(v.PubKeyType, v.PubKeyBytes)
	if err != nil {
		return fmt.Errorf("can't decode public key: %w", err)
	}
	key := []byte("val" + string(pubKey.Bytes()))

	// add or update validator
	value := bytes.NewBuffer(make([]byte, 0))
	if err := types.WriteMessage(&v, value); err != nil {
		return err
	}
	if err = app.state.DB.Set(key, value.Bytes()); err != nil {
		return err
	}
	app.valAddrToPubKeyMap[string(pubKey.Address())] = pubKey
	return nil
}

func hasCurseWord(word string, curseWords string) bool {
	return strings.Contains(curseWords, strings.ToLower(word))
}

const (
	CodeTypeOK              uint32 = 0
	CodeTypeEncodingError   uint32 = 1
	CodeTypeInvalidTxFormat uint32 = 2
	CodeTypeBanned          uint32 = 3
)

func UpdateOrSetUser(db *model.DB, uname string, toBan bool, txn *badger.Txn) error {
	var u *model.User
	u, err := db.FindUserByName(uname)
	if errors.Is(err, badger.ErrKeyNotFound) {
		u = new(model.User)
		u.Name = uname
		u.Banned = toBan
	} else {
		if err != nil {
			return errors.New("not able to process user")
		}
		u.Banned = toBan
	}
	userBytes, err := json.Marshal(u)
	if err != nil {
		return fmt.Errorf("error marshaling user: %w", err)
	}
	return txn.Set([]byte(uname), userBytes)
}

func DeduplicateCurseWords(inWords string) string {
	curseWordMap := make(map[string]struct{})
	for _, word := range strings.Split(inWords, "|") {
		curseWordMap[word] = struct{}{}
	}
	deduplicatedWords := ""
	for word := range curseWordMap {
		if deduplicatedWords == "" {
			deduplicatedWords = word
		} else {
			deduplicatedWords += "|" + word
		}
	}
	return deduplicatedWords
}
