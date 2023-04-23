package getter

import "net/url"

// RedactURL is a port of url.Redacted from the standard library,
// which is like url.String but replaces any password with "redacted".
// Only the password in u.URL is redacted. This allows the library
// to maintain compatibility with go1.14.
// This port was also extended to redact SSH key from URL query parameter.
func RedactURL(u *url.URL) string {
	if u == nil {
		return ""
	}

	ru := *u
	if _, has := ru.User.Password(); has {
		ru.User = url.UserPassword(ru.User.Username(), "redacted")
	}
	q := ru.Query()
	if q.Get("sshkey") != "" {
		q.Set("sshkey", "redacted")
		ru.RawQuery = q.Encode()
	}
	return ru.String()
}
