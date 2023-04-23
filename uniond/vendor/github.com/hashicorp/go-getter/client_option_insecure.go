package getter

// WithInsecure allows for a user to avoid
// checking certificates (not recommended).
// For example, when connecting on HTTPS where an
// invalid certificate is presented.
// User assumes all risk.
// Not all getters have support for insecure mode yet.
func WithInsecure() func(*Client) error {
	return func(c *Client) error {
		c.Insecure = true
		return nil
	}
}
