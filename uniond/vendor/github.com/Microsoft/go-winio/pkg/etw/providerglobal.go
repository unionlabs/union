//go:build windows
// +build windows

package etw

import (
	"sync"
)

// Because the provider callback function needs to be able to access the
// provider data when it is invoked by ETW, we need to keep provider data stored
// in a global map based on an index. The index is passed as the callback
// context to ETW.
type providerMap struct {
	m    map[uint]*Provider
	i    uint
	lock sync.Mutex
}

var providers = providerMap{
	m: make(map[uint]*Provider),
}

func (p *providerMap) newProvider() *Provider {
	p.lock.Lock()
	defer p.lock.Unlock()

	i := p.i
	p.i++

	provider := &Provider{
		index: i,
	}

	p.m[i] = provider
	return provider
}

func (p *providerMap) removeProvider(provider *Provider) {
	p.lock.Lock()
	defer p.lock.Unlock()

	delete(p.m, provider.index)
}

func (p *providerMap) getProvider(index uint) *Provider {
	p.lock.Lock()
	defer p.lock.Unlock()

	return p.m[index]
}

//todo: combine these into struct, so that "globalProviderCallback" is guaranteed to be initialized through method access

var providerCallbackOnce sync.Once
var globalProviderCallback uintptr
