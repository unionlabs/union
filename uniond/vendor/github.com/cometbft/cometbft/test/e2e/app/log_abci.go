package app

import (
	"encoding/base64"
	"fmt"
	"strings"

	"github.com/cosmos/gogoproto/proto"

	abci "github.com/cometbft/cometbft/abci/types"
)

const AbciReq = "abci-req"

// GetRequestString gets the string representation of the request that will be logged by the application.
func GetABCIRequestString(req *abci.Request) (string, error) {
	b, err := proto.Marshal(req)
	if err != nil {
		return "", err
	}
	reqStr := base64.StdEncoding.EncodeToString(b)
	return AbciReq + reqStr + AbciReq, nil
}

// GetABCIRequestFromString parse string and try to get a string of a Request created by GetRequestString.
func GetABCIRequestFromString(s string) (*abci.Request, error) {
	if !strings.Contains(s, AbciReq) {
		return nil, nil
	}
	parts := strings.Split(s, AbciReq)
	if len(parts) != 3 || len(parts[1]) == 0 {
		return nil, fmt.Errorf("string %q passed to GetRequestFromString does not have a good format", s)
	}
	req := &abci.Request{}
	reqStr := parts[1]
	b, err := base64.StdEncoding.DecodeString(reqStr)
	if err != nil {
		return nil, err
	}
	err = proto.Unmarshal(b, req)
	if err != nil {
		return nil, err
	}
	return req, nil
}
