//go:build !windows

package glog

import "os/user"

func lookupUser() string {
	if current, err := user.Current(); err == nil {
		return current.Username
	}
	return ""
}
