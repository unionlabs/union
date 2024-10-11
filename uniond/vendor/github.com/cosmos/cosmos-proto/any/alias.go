// package any
// deprecated: use anyutil package instead
package any

import "github.com/cosmos/cosmos-proto/anyutil"

var (
	New         = anyutil.New
	MarshalFrom = anyutil.MarshalFrom
	Unpack      = anyutil.Unpack
)
