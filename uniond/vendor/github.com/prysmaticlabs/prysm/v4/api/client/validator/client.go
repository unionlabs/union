package validator

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api/client"
	"github.com/prysmaticlabs/prysm/v4/validator/rpc"
)

const (
	localKeysPath    = "/eth/v1/keystores"
	remoteKeysPath   = "/eth/v1/remotekeys"
	feeRecipientPath = "/eth/v1/validator/{pubkey}/feerecipient"
)

// Client provides a collection of helper methods for calling the Keymanager API endpoints.
type Client struct {
	*client.Client
}

// NewClient returns a new Client that includes functions for REST calls to keymanager APIs.
func NewClient(host string, opts ...client.ClientOpt) (*Client, error) {
	c, err := client.NewClient(host, opts...)
	if err != nil {
		return nil, err
	}
	return &Client{c}, nil
}

// GetValidatorPubKeys gets the current list of web3signer or the local validator public keys in hex format.
func (c *Client) GetValidatorPubKeys(ctx context.Context) ([]string, error) {
	jsonlocal, err := c.GetLocalValidatorKeys(ctx)
	if err != nil {
		return nil, err
	}
	jsonremote, err := c.GetRemoteValidatorKeys(ctx)
	if err != nil {
		return nil, err
	}
	if len(jsonlocal.Data) == 0 && len(jsonremote.Data) == 0 {
		return nil, errors.New("there are no local keys or remote keys on the validator")
	}

	hexKeys := make(map[string]bool)

	for index := range jsonlocal.Data {
		hexKeys[jsonlocal.Data[index].ValidatingPubkey] = true
	}
	for index := range jsonremote.Data {
		hexKeys[jsonremote.Data[index].Pubkey] = true
	}
	keys := make([]string, 0)
	for k := range hexKeys {
		keys = append(keys, k)
	}
	return keys, nil
}

// GetLocalValidatorKeys calls the keymanager APIs for local validator keys
func (c *Client) GetLocalValidatorKeys(ctx context.Context) (*rpc.ListKeystoresResponse, error) {
	localBytes, err := c.Get(ctx, localKeysPath, client.WithAuthorizationToken(c.Token()))
	if err != nil {
		return nil, err
	}
	jsonlocal := &rpc.ListKeystoresResponse{}
	if err := json.Unmarshal(localBytes, jsonlocal); err != nil {
		return nil, errors.Wrap(err, "failed to parse local keystore list")
	}
	return jsonlocal, nil
}

// GetRemoteValidatorKeys calls the keymanager APIs for web3signer validator keys
func (c *Client) GetRemoteValidatorKeys(ctx context.Context) (*rpc.ListRemoteKeysResponse, error) {
	remoteBytes, err := c.Get(ctx, remoteKeysPath, client.WithAuthorizationToken(c.Token()))
	if err != nil {
		if !strings.Contains(err.Error(), "Prysm Wallet is not of type Web3Signer") {
			return nil, err
		}
	}
	jsonremote := &rpc.ListRemoteKeysResponse{}
	if len(remoteBytes) != 0 {
		if err := json.Unmarshal(remoteBytes, jsonremote); err != nil {
			return nil, errors.Wrap(err, "failed to parse remote keystore list")
		}
	}
	return jsonremote, nil
}

// GetFeeRecipientAddresses takes a list of validators in hex format and returns an equal length list of fee recipients in hex format.
func (c *Client) GetFeeRecipientAddresses(ctx context.Context, validators []string) ([]string, error) {
	feeRecipients := make([]string, len(validators))
	for index, validator := range validators {
		feejson, err := c.GetFeeRecipientAddress(ctx, validator)
		if err != nil {
			return nil, errors.Wrap(err, fmt.Sprintf("keymanager API failed to retrieve fee recipient for validator %s", validators[index]))
		}
		if feejson.Data == nil {
			continue
		}
		feeRecipients[index] = feejson.Data.Ethaddress
	}
	return feeRecipients, nil
}

// GetFeeRecipientAddress takes a public key and calls the keymanager API to return its fee recipient.
func (c *Client) GetFeeRecipientAddress(ctx context.Context, pubkey string) (*rpc.GetFeeRecipientByPubkeyResponse, error) {
	path := strings.Replace(feeRecipientPath, "{pubkey}", pubkey, 1)
	b, err := c.Get(ctx, path, client.WithAuthorizationToken(c.Token()))
	if err != nil {
		return nil, err
	}
	feejson := &rpc.GetFeeRecipientByPubkeyResponse{}
	if err := json.Unmarshal(b, feejson); err != nil {
		return nil, errors.Wrap(err, "failed to parse fee recipient")
	}
	return feejson, nil
}
