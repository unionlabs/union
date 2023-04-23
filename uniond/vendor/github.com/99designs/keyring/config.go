package keyring

// Config contains configuration for keyring.
type Config struct {
	// AllowedBackends is a whitelist of backend providers that can be used. Nil means all available.
	AllowedBackends []BackendType

	// ServiceName is a generic service name that is used by backends that support the concept
	ServiceName string

	// MacOSKeychainNameKeychainName is the name of the macOS keychain that is used
	KeychainName string

	// KeychainTrustApplication is whether the calling application should be trusted by default by items
	KeychainTrustApplication bool

	// KeychainSynchronizable is whether the item can be synchronized to iCloud
	KeychainSynchronizable bool

	// KeychainAccessibleWhenUnlocked is whether the item is accessible when the device is locked
	KeychainAccessibleWhenUnlocked bool

	// KeychainPasswordFunc is an optional function used to prompt the user for a password
	KeychainPasswordFunc PromptFunc

	// FilePasswordFunc is a required function used to prompt the user for a password
	FilePasswordFunc PromptFunc

	// FileDir is the directory that keyring files are stored in, ~/ is resolved to the users' home dir
	FileDir string

	// KeyCtlScope is the scope of the kernel keyring (either "user", "session", "process" or "thread")
	KeyCtlScope string

	// KeyCtlPerm is the permission mask to use for new keys
	KeyCtlPerm uint32

	// KWalletAppID is the application id for KWallet
	KWalletAppID string

	// KWalletFolder is the folder for KWallet
	KWalletFolder string

	// LibSecretCollectionName is the name collection in secret-service
	LibSecretCollectionName string

	// PassDir is the pass password-store directory, ~/ is resolved to the users' home dir
	PassDir string

	// PassCmd is the name of the pass executable
	PassCmd string

	// PassPrefix is a string prefix to prepend to the item path stored in pass
	PassPrefix string

	// WinCredPrefix is a string prefix to prepend to the key name
	WinCredPrefix string
}
