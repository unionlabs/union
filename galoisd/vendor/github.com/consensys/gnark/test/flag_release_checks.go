//go:build (release_checks && !prover_checks) || (release_checks && prover_checks)

package test

const (
	proverTestFlag  = false
	releaseTestFlag = true
)
