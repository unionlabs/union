// recursive read lock calls with methods
package testdata

func (p *ProtectResource) NestedMethod() {
	p.RLock()
	p.GetResource() // want `found recursive read lock call`
	p.RUnlock()
}

func (p *ProtectResource) NestedMethod2() {
	p.RLock()
	p.GetResourceNested() // want `found recursive read lock call`
	p.RUnlock()
}

func (p *ProtectResource) NestedMethodMixedLock() {
	p.Lock()
	p.GetResource() // want `found recursive lock call`
	p.Unlock()
}

func (p *ProtectResource) MixedLock() {
	p.RLock()
	p.Lock() // want `found recursive mixed lock call`
	p.Unlock()
	p.RUnlock()
}

func (p *ProtectResource) NestedMethodGoroutine() {
	p.RLock()
	defer p.RUnlock()
	go p.GetResource()
}

func (p *ProtectResource) NestedResourceGoroutine() {
	p.RLock()
	defer p.RUnlock()
	p.GetResourceNestedGoroutine()
}

func (p *NestedProtectResource) MultiLevelStruct() {
	p.nestedPR.RLock()
	p.nestedPR.GetResource() // want `found recursive read lock call`
	p.nestedPR.RUnlock()
}

func (p *NestedProtectResource) MultiLevelStruct2() {
	p.nestedPR.RLock()
	p.GetNestedPResource() // want `found recursive read lock call`
	p.nestedPR.RUnlock()
}
