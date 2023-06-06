package http

import (
	"context"
	"fmt"
	"math/rand"
	"regexp"
	"strings"
	"time"

	"github.com/cometbft/cometbft/light/provider"
	rpcclient "github.com/cometbft/cometbft/rpc/client"
	rpchttp "github.com/cometbft/cometbft/rpc/client/http"
	"github.com/cometbft/cometbft/types"
)

var (
	// This is very brittle, see: https://github.com/tendermint/tendermint/issues/4740
	regexpMissingHeight = regexp.MustCompile(`height \d+ is not available`)
	regexpTooHigh       = regexp.MustCompile(`height \d+ must be less than or equal to`)
	regexpTimedOut      = regexp.MustCompile(`Timeout exceeded`)

	maxRetryAttempts      = 5
	timeout          uint = 5 // sec.
)

// http provider uses an RPC client to obtain the necessary information.
type http struct {
	chainID string
	client  rpcclient.RemoteClient
}

// New creates a HTTP provider, which is using the rpchttp.HTTP client under
// the hood. If no scheme is provided in the remote URL, http will be used by
// default. The 5s timeout is used for all requests.
func New(chainID, remote string) (provider.Provider, error) {
	// Ensure URL scheme is set (default HTTP) when not provided.
	if !strings.Contains(remote, "://") {
		remote = "http://" + remote
	}

	httpClient, err := rpchttp.NewWithTimeout(remote, "/websocket", timeout)
	if err != nil {
		return nil, err
	}

	return NewWithClient(chainID, httpClient), nil
}

// NewWithClient allows you to provide a custom client.
func NewWithClient(chainID string, client rpcclient.RemoteClient) provider.Provider {
	return &http{
		client:  client,
		chainID: chainID,
	}
}

// ChainID returns a chainID this provider was configured with.
func (p *http) ChainID() string {
	return p.chainID
}

func (p *http) String() string {
	return fmt.Sprintf("http{%s}", p.client.Remote())
}

// LightBlock fetches a LightBlock at the given height and checks the
// chainID matches.
func (p *http) LightBlock(ctx context.Context, height int64) (*types.LightBlock, error) {
	h, err := validateHeight(height)
	if err != nil {
		return nil, provider.ErrBadLightBlock{Reason: err}
	}

	sh, err := p.signedHeader(ctx, h)
	if err != nil {
		return nil, err
	}

	if height != 0 && sh.Height != height {
		return nil, provider.ErrBadLightBlock{
			Reason: fmt.Errorf("height %d responded doesn't match height %d requested", sh.Height, height),
		}
	}

	vs, err := p.validatorSet(ctx, &sh.Height)
	if err != nil {
		return nil, err
	}

	lb := &types.LightBlock{
		SignedHeader: sh,
		ValidatorSet: vs,
	}

	err = lb.ValidateBasic(p.chainID)
	if err != nil {
		return nil, provider.ErrBadLightBlock{Reason: err}
	}

	return lb, nil
}

// ReportEvidence calls `/broadcast_evidence` endpoint.
func (p *http) ReportEvidence(ctx context.Context, ev types.Evidence) error {
	_, err := p.client.BroadcastEvidence(ctx, ev)
	return err
}

func (p *http) validatorSet(ctx context.Context, height *int64) (*types.ValidatorSet, error) {
	// Since the malicious node could report a massive number of pages, making us
	// spend a considerable time iterating, we restrict the number of pages here.
	// => 10000 validators max
	const maxPages = 100

	var (
		perPage = 100
		vals    = []*types.Validator{}
		page    = 1
		total   = -1
	)

OUTER_LOOP:
	for len(vals) != total && page <= maxPages {
		for attempt := 1; attempt <= maxRetryAttempts; attempt++ {
			res, err := p.client.Validators(ctx, height, &page, &perPage)
			switch {
			case err == nil:
				// Validate response.
				if len(res.Validators) == 0 {
					return nil, provider.ErrBadLightBlock{
						Reason: fmt.Errorf("validator set is empty (height: %d, page: %d, per_page: %d)",
							height, page, perPage),
					}
				}
				if res.Total <= 0 {
					return nil, provider.ErrBadLightBlock{
						Reason: fmt.Errorf("total number of vals is <= 0: %d (height: %d, page: %d, per_page: %d)",
							res.Total, height, page, perPage),
					}
				}

				total = res.Total
				vals = append(vals, res.Validators...)
				page++
				continue OUTER_LOOP

			case regexpTooHigh.MatchString(err.Error()):
				return nil, provider.ErrHeightTooHigh

			case regexpMissingHeight.MatchString(err.Error()):
				return nil, provider.ErrLightBlockNotFound

			// if we have exceeded retry attempts then return no response error
			case attempt == maxRetryAttempts:
				return nil, provider.ErrNoResponse

			case regexpTimedOut.MatchString(err.Error()):
				// we wait and try again with exponential backoff
				time.Sleep(backoffTimeout(uint16(attempt)))
				continue

			// context canceled or connection refused we return the error
			default:
				return nil, err
			}

		}
	}

	valSet, err := types.ValidatorSetFromExistingValidators(vals)
	if err != nil {
		return nil, provider.ErrBadLightBlock{Reason: err}
	}
	return valSet, nil
}

func (p *http) signedHeader(ctx context.Context, height *int64) (*types.SignedHeader, error) {
	for attempt := 1; attempt <= maxRetryAttempts; attempt++ {
		commit, err := p.client.Commit(ctx, height)
		switch {
		case err == nil:
			// See https://github.com/cometbft/cometbft/issues/575
			// If the node is starting at a non-zero height, but does not yet
			// have any blocks, it can return an empty signed header without
			// returning an error.
			if commit.SignedHeader.IsEmpty() {
				// Technically this means that the provider still needs to
				// catch up.
				return nil, provider.ErrHeightTooHigh
			}
			return &commit.SignedHeader, nil

		case regexpTooHigh.MatchString(err.Error()):
			return nil, provider.ErrHeightTooHigh

		case regexpMissingHeight.MatchString(err.Error()):
			return nil, provider.ErrLightBlockNotFound

		case regexpTimedOut.MatchString(err.Error()):
			// we wait and try again with exponential backoff
			time.Sleep(backoffTimeout(uint16(attempt)))
			continue

		// either context was canceled or connection refused.
		default:
			return nil, err
		}
	}
	return nil, provider.ErrNoResponse
}

func validateHeight(height int64) (*int64, error) {
	if height < 0 {
		return nil, fmt.Errorf("expected height >= 0, got height %d", height)
	}

	h := &height
	if height == 0 {
		h = nil
	}
	return h, nil
}

// exponential backoff (with jitter)
// 0.5s -> 2s -> 4.5s -> 8s -> 12.5 with 1s variation
func backoffTimeout(attempt uint16) time.Duration {
	//nolint:gosec // G404: Use of weak random number generator
	return time.Duration(500*attempt*attempt)*time.Millisecond + time.Duration(rand.Intn(1000))*time.Millisecond
}
