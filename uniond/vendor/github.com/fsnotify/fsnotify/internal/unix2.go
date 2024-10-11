//go:build !windows
// +build !windows

package internal

func HasPrivilegesForSymlink() bool {
	return true
}
