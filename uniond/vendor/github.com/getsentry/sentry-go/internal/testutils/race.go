//go:build !race

package testutils

func IsRaceTest() bool {
	return false
}
