package authorization

// Method is an authorization method such as 'Basic' or 'Bearer'.
type Method uint8

const (
	// None represents no authorization method.
	None Method = iota
	// Basic represents Basic Authentication.
	Basic
	// Bearer represents Bearer Authentication (token authentication).
	Bearer
)
