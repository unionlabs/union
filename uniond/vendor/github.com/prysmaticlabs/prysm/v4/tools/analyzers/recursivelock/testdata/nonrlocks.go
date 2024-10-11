// These are all non recursive rlocks. Testing to make sure there are no false positives
package testdata

func (resource *ProtectResource) NestedRLockWithDefer() string {
	resource.RLock()
	defer resource.RUnlock()
	return resource.GetResource() // want `found recursive read lock call`
}

func (resource *NestedProtectResource) NonNestedRLockDifferentRLocks() {
	resource.RLock()
	resource.GetNestedPResource() // get nested resource uses RLock, but at a deeper level in the struct
	resource.RUnlock()
}

func (resource *ProtectResource) NestedLockWithDefer() string {
	resource.Lock()
	defer resource.Unlock()
	return resource.GetResourceLocked() // want `found recursive lock call`
}

func (resource *NestedProtectResource) NonNestedLockDifferentLocks() {
	resource.Lock()
	resource.GetNestedPResourceLocked() // get nested resource uses RLock, but at a deeper level in the struct
	resource.Unlock()
}
