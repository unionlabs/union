package testdata

var resource *ProtectResource = &ProtectResource{resource: "protected"}
var nested *NestedResource = &NestedResource{p: ProtectResource{resource: "hello"}}

func NestedRLockWithStructs() {
	resource.RLock()
	resource.GetResource() // want `found recursive read lock call`
	resource.RUnlock()
}

func NestedRLockWithParam(r *ProtectResource) {
	r.RLock()
	r.GetResource() // want `found recursive read lock call`
	r.RUnlock()
}

func NestedRLockWithMoreStructs() {
	nested.p.RLock()
	nested.p.GetResource() // want `found recursive read lock call`
	nested.p.RUnlock()
}

func NestedRLockWithFuncLit() {
	resource.RLock()
	var varFuncLit func() = func() {
		resource.RLock()
	}
	varFuncLit() // want `found recursive read lock call`
	assignFuncLit := func() {
		resource.RLock()
	}
	assignFuncLit() // want `found recursive read lock call`
	obfuscateFuncLit := varFuncLit
	obfuscateFuncLit() // want `found recursive read lock call`
	var multiVarFuncLit1, multiVarFuncLit2 func() = func() {
		resource.RLock()
	}, func() {
		resource.RLock()
	}
	multiVarFuncLit1() // want `found recursive read lock call`
	multiVarFuncLit2() // want `found recursive read lock call`
	multiAssignFuncLit1, multiAssignFuncLit2 := func() {
		resource.RLock()
	}, func() {
		resource.RLock()
	}
	multiAssignFuncLit1() // want `found recursive read lock call`
	multiAssignFuncLit2() // want `found recursive read lock call`
	resource.RUnlock()
}
