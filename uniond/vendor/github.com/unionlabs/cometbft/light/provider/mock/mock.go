package mock

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/cometbft/cometbft/light/provider"
	"github.com/cometbft/cometbft/types"
)

type Mock struct {
	chainID string

	mtx              sync.Mutex
	headers          map[int64]*types.SignedHeader
	vals             map[int64]*types.ValidatorSet
	evidenceToReport map[string]types.Evidence // hash => evidence
	latestHeight     int64
}

var _ provider.Provider = (*Mock)(nil)

// New creates a mock provider with the given set of headers and validator
// sets.
func New(chainID string, headers map[int64]*types.SignedHeader, vals map[int64]*types.ValidatorSet) *Mock {
	height := int64(0)
	for h := range headers {
		if h > height {
			height = h
		}
	}
	return &Mock{
		chainID:          chainID,
		headers:          headers,
		vals:             vals,
		evidenceToReport: make(map[string]types.Evidence),
		latestHeight:     height,
	}
}

// ChainID returns the blockchain ID.
func (p *Mock) ChainID() string {
	return p.chainID
}

func (p *Mock) String() string {
	var headers strings.Builder
	for _, h := range p.headers {
		fmt.Fprintf(&headers, " %d:%X", h.Height, h.Hash())
	}

	var vals strings.Builder
	for _, v := range p.vals {
		fmt.Fprintf(&vals, " %X", v.Hash())
	}

	return fmt.Sprintf("Mock{headers: %s, vals: %v}", headers.String(), vals.String())
}

func (p *Mock) LightBlock(ctx context.Context, height int64) (*types.LightBlock, error) {
	p.mtx.Lock()
	defer p.mtx.Unlock()

	// allocate a window of time for contexts to be canceled
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	case <-time.After(10 * time.Millisecond):
	}

	var lb *types.LightBlock

	if height > p.latestHeight {
		return nil, provider.ErrHeightTooHigh
	}

	if height == 0 && len(p.headers) > 0 {
		height = p.latestHeight
	}

	if _, ok := p.headers[height]; ok {
		sh := p.headers[height]
		vals := p.vals[height]
		lb = &types.LightBlock{
			SignedHeader: sh,
			ValidatorSet: vals,
		}
	}
	if lb == nil {
		return nil, provider.ErrLightBlockNotFound
	}
	if lb.SignedHeader == nil || lb.ValidatorSet == nil {
		return nil, provider.ErrBadLightBlock{Reason: errors.New("nil header or vals")}
	}
	if err := lb.ValidateBasic(lb.ChainID); err != nil {
		return nil, provider.ErrBadLightBlock{Reason: err}
	}
	return lb, nil
}

func (p *Mock) ReportEvidence(_ context.Context, ev types.Evidence) error {
	p.evidenceToReport[string(ev.Hash())] = ev
	return nil
}

func (p *Mock) HasEvidence(ev types.Evidence) bool {
	_, ok := p.evidenceToReport[string(ev.Hash())]
	return ok
}

func (p *Mock) AddLightBlock(lb *types.LightBlock) {
	p.mtx.Lock()
	defer p.mtx.Unlock()

	if err := lb.ValidateBasic(lb.ChainID); err != nil {
		panic(fmt.Sprintf("unable to add light block, err: %v", err))
	}
	p.headers[lb.Height] = lb.SignedHeader
	p.vals[lb.Height] = lb.ValidatorSet
	if lb.Height > p.latestHeight {
		p.latestHeight = lb.Height
	}
}

func (p *Mock) Copy(id string) *Mock {
	return New(id, p.headers, p.vals)
}
