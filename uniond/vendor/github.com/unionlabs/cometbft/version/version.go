package version

const (
	// CMTSemVer is the used as the fallback version of CometBFT
	// when not using git describe. It is formatted with semantic versioning.
	CMTSemVer = "1.0.0-rc1"
	// ABCISemVer is the semantic version of the ABCI protocol.
	ABCISemVer  = "2.1.0"
	ABCIVersion = ABCISemVer
	// P2PProtocol versions all p2p behavior and msgs.
	// This includes proposer selection.
	P2PProtocol uint64 = 9

	// BlockProtocol versions all block data structures and processing.
	// This includes validity of blocks and state updates.
	BlockProtocol uint64 = 11
)

// CMTGitCommitHash uses git rev-parse HEAD to find commit hash which is helpful
// for the engineering team when working with the cometbft binary. See Makefile.
var CMTGitCommitHash = ""
