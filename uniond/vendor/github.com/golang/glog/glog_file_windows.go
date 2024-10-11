//go:build windows

package glog

import (
	"syscall"
)

// This follows the logic in the standard library's user.Current() function, except
// that it leaves out the potentially expensive calls required to look up the user's
// display name in Active Directory.
func lookupUser() string {
	token, err := syscall.OpenCurrentProcessToken()
	if err != nil {
		return ""
	}
	defer token.Close()
	tokenUser, err := token.GetTokenUser()
	if err != nil {
		return ""
	}
	username, _, accountType, err := tokenUser.User.Sid.LookupAccount("")
	if err != nil {
		return ""
	}
	if accountType != syscall.SidTypeUser {
		return ""
	}
	return username
}
