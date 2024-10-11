package testdata

import (
	"sync"
)

type ProtectResource struct {
	*sync.RWMutex
	resource string
}

func (r *ProtectResource) GetResource() string {
	defer r.RUnlock()
	r.RLock()
	return r.resource
}

func (r *ProtectResource) GetResourceLocked() string {
	defer r.Unlock()
	r.Lock()
	return r.resource
}

func (r *ProtectResource) GetResourceNested() string {
	return r.GetResource()
}

func (r *ProtectResource) GetResourceNestedGoroutine() {
	go r.GetResource()
}

func (r *ProtectResource) GetResourceNestedLock() string {
	return r.GetResourceLocked()
}

type NestedProtectResource struct {
	*sync.RWMutex
	nestedPR ProtectResource
}

func (r *NestedProtectResource) GetNestedPResource() string {
	defer r.nestedPR.RUnlock()
	r.nestedPR.RLock()
	return r.nestedPR.resource
}

func (r *NestedProtectResource) GetNestedPResourceLocked() string {
	defer r.nestedPR.Unlock()
	r.nestedPR.Lock()
	return r.nestedPR.resource
}

type NotProtected struct {
	resource string
}

func (r *NotProtected) GetResource() string {
	return r.resource
}

type NestedResource struct {
	*NotProtected
	p ProtectResource
}

type ExposedMutex struct {
	lock *sync.RWMutex
}

func (e *ExposedMutex) GetLock() *sync.RWMutex {
	return e.lock
}
