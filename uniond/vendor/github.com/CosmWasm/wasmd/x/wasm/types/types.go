package types

import (
	"fmt"
	"reflect"

	errorsmod "cosmossdk.io/errors"
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/gogoproto/proto"
)

const (
	defaultMemoryCacheSize    uint32 = 100 // in MiB
	defaultSmartQueryGasLimit uint64 = 3_000_000
	defaultContractDebugMode         = false

	// ContractAddrLen defines a valid address length for contracts
	ContractAddrLen = 32
	// SDKAddrLen defines a valid address length that was used in sdk address generation
	SDKAddrLen = 20
)

func (m Model) ValidateBasic() error {
	if len(m.Key) == 0 {
		return errorsmod.Wrap(ErrEmpty, "key")
	}
	return nil
}

func (c CodeInfo) ValidateBasic() error {
	if len(c.CodeHash) == 0 {
		return errorsmod.Wrap(ErrEmpty, "code hash")
	}
	if _, err := sdk.AccAddressFromBech32(c.Creator); err != nil {
		return errorsmod.Wrap(err, "creator")
	}
	if err := c.InstantiateConfig.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "instantiate config")
	}
	return nil
}

// NewCodeInfo fills a new CodeInfo struct
func NewCodeInfo(codeHash []byte, creator sdk.AccAddress, instantiatePermission AccessConfig) CodeInfo {
	return CodeInfo{
		CodeHash:          codeHash,
		Creator:           creator.String(),
		InstantiateConfig: instantiatePermission,
	}
}

var AllCodeHistoryTypes = []ContractCodeHistoryOperationType{ContractCodeHistoryOperationTypeGenesis, ContractCodeHistoryOperationTypeInit, ContractCodeHistoryOperationTypeMigrate}

// NewContractInfo creates a new instance of a given WASM contract info
func NewContractInfo(codeID uint64, creator, admin sdk.AccAddress, label string, createdAt *AbsoluteTxPosition) ContractInfo {
	var adminAddr string
	if !admin.Empty() {
		adminAddr = admin.String()
	}
	return ContractInfo{
		CodeID:  codeID,
		Creator: creator.String(),
		Admin:   adminAddr,
		Label:   label,
		Created: createdAt,
	}
}

// validatable is an optional interface that can be implemented by an ContractInfoExtension to enable validation
type validatable interface {
	ValidateBasic() error
}

// ValidateBasic does syntax checks on the data. If an extension is set and has the `ValidateBasic() error` method, then
// the method is called as well. It is recommend to implement `ValidateBasic` so that the data is verified in the setter
// but also in the genesis import process.
func (c *ContractInfo) ValidateBasic() error {
	if c.CodeID == 0 {
		return errorsmod.Wrap(ErrEmpty, "code id")
	}
	if _, err := sdk.AccAddressFromBech32(c.Creator); err != nil {
		return errorsmod.Wrap(err, "creator")
	}
	if len(c.Admin) != 0 {
		if _, err := sdk.AccAddressFromBech32(c.Admin); err != nil {
			return errorsmod.Wrap(err, "admin")
		}
	}
	if err := ValidateLabel(c.Label); err != nil {
		return errorsmod.Wrap(err, "label")
	}
	if c.Extension == nil {
		return nil
	}

	e, ok := c.Extension.GetCachedValue().(validatable)
	if !ok {
		return nil
	}
	if err := e.ValidateBasic(); err != nil {
		return errorsmod.Wrap(err, "extension")
	}
	return nil
}

// SetExtension set new extension data. Calls `ValidateBasic() error` on non nil values when method is implemented by
// the extension.
func (c *ContractInfo) SetExtension(ext ContractInfoExtension) error {
	if ext == nil {
		c.Extension = nil
		return nil
	}
	if e, ok := ext.(validatable); ok {
		if err := e.ValidateBasic(); err != nil {
			return err
		}
	}
	codecAny, err := codectypes.NewAnyWithValue(ext)
	if err != nil {
		return errorsmod.Wrap(sdkerrors.ErrPackAny, err.Error())
	}

	c.Extension = codecAny
	return nil
}

// ReadExtension copies the extension value to the pointer passed as argument so that there is no need to cast
// For example with a custom extension of type `MyContractDetails` it will look as following:
//
//	var d MyContractDetails
//	if err := info.ReadExtension(&d); err != nil {
//		return nil, errorsmod.Wrap(err, "extension")
//	}
func (c *ContractInfo) ReadExtension(e ContractInfoExtension) error {
	rv := reflect.ValueOf(e)
	if rv.Kind() != reflect.Ptr || rv.IsNil() {
		return errorsmod.Wrap(sdkerrors.ErrInvalidType, "not a pointer")
	}
	if c.Extension == nil {
		return nil
	}

	cached := c.Extension.GetCachedValue()
	elem := reflect.ValueOf(cached).Elem()
	if !elem.Type().AssignableTo(rv.Elem().Type()) {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidType, "extension is of type %s but argument of %s", elem.Type(), rv.Elem().Type())
	}
	rv.Elem().Set(elem)
	return nil
}

func (c ContractInfo) InitialHistory(initMsg []byte) ContractCodeHistoryEntry {
	return ContractCodeHistoryEntry{
		Operation: ContractCodeHistoryOperationTypeInit,
		CodeID:    c.CodeID,
		Updated:   c.Created,
		Msg:       initMsg,
	}
}

func (c *ContractInfo) AddMigration(ctx sdk.Context, codeID uint64, msg []byte) ContractCodeHistoryEntry {
	h := ContractCodeHistoryEntry{
		Operation: ContractCodeHistoryOperationTypeMigrate,
		CodeID:    codeID,
		Updated:   NewAbsoluteTxPosition(ctx),
		Msg:       msg,
	}
	c.CodeID = codeID
	return h
}

// AdminAddr convert into sdk.AccAddress or nil when not set
func (c *ContractInfo) AdminAddr() sdk.AccAddress {
	if c.Admin == "" {
		return nil
	}
	admin, err := sdk.AccAddressFromBech32(c.Admin)
	if err != nil { // should never happen
		panic(err.Error())
	}
	return admin
}

// ContractInfoExtension defines the extension point for custom data to be stored with a contract info
type ContractInfoExtension interface {
	proto.Message
	String() string
}

var _ codectypes.UnpackInterfacesMessage = &ContractInfo{}

// UnpackInterfaces implements codectypes.UnpackInterfaces
func (c *ContractInfo) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	var details ContractInfoExtension
	if err := unpacker.UnpackAny(c.Extension, &details); err != nil {
		return err
	}
	return codectypes.UnpackInterfaces(details, unpacker)
}

// NewAbsoluteTxPosition gets a block position from the context
func NewAbsoluteTxPosition(ctx sdk.Context) *AbsoluteTxPosition {
	// we must safely handle nil gas meters
	var index uint64
	meter := ctx.BlockGasMeter()
	if meter != nil {
		index = meter.GasConsumed()
	}
	height := ctx.BlockHeight()
	if height < 0 {
		panic(fmt.Sprintf("unsupported height: %d", height))
	}
	return &AbsoluteTxPosition{
		BlockHeight: uint64(height),
		TxIndex:     index,
	}
}

// LessThan can be used to sort
func (a *AbsoluteTxPosition) LessThan(b *AbsoluteTxPosition) bool {
	if a == nil {
		return true
	}
	if b == nil {
		return false
	}
	return a.BlockHeight < b.BlockHeight || (a.BlockHeight == b.BlockHeight && a.TxIndex < b.TxIndex)
}

// AbsoluteTxPositionLen number of elements in byte representation
const AbsoluteTxPositionLen = 16

// Bytes encodes the object into a 16 byte representation with big endian block height and tx index.
func (a *AbsoluteTxPosition) Bytes() []byte {
	if a == nil {
		panic("object must not be nil")
	}
	r := make([]byte, AbsoluteTxPositionLen)
	copy(r[0:], sdk.Uint64ToBigEndian(a.BlockHeight))
	copy(r[8:], sdk.Uint64ToBigEndian(a.TxIndex))
	return r
}

// ValidateBasic syntax checks
func (c ContractCodeHistoryEntry) ValidateBasic() error {
	var found bool
	for _, v := range AllCodeHistoryTypes {
		if c.Operation == v {
			found = true
			break
		}
	}
	if !found {
		return ErrInvalid.Wrap("operation")
	}
	if c.CodeID == 0 {
		return ErrEmpty.Wrap("code id")
	}
	if c.Updated == nil {
		return ErrEmpty.Wrap("updated")
	}
	return errorsmod.Wrap(c.Msg.ValidateBasic(), "msg")
}

// NewEnv initializes the environment for a contract instance
func NewEnv(ctx sdk.Context, contractAddr sdk.AccAddress) wasmvmtypes.Env {
	// safety checks before casting below
	if ctx.BlockHeight() < 0 {
		panic("Block height must never be negative")
	}
	nano := ctx.BlockTime().UnixNano()
	if nano < 1 {
		panic("Block (unix) time must never be empty or negative ")
	}

	env := wasmvmtypes.Env{
		Block: wasmvmtypes.BlockInfo{
			Height:  uint64(ctx.BlockHeight()),
			Time:    uint64(nano),
			ChainID: ctx.ChainID(),
		},
		Contract: wasmvmtypes.ContractInfo{
			Address: contractAddr.String(),
		},
	}
	if txCounter, ok := TXCounter(ctx); ok {
		env.Transaction = &wasmvmtypes.TransactionInfo{Index: txCounter}
	}
	return env
}

// NewInfo initializes the MessageInfo for a contract instance
func NewInfo(creator sdk.AccAddress, deposit sdk.Coins) wasmvmtypes.MessageInfo {
	return wasmvmtypes.MessageInfo{
		Sender: creator.String(),
		Funds:  NewWasmCoins(deposit),
	}
}

// NewWasmCoins translates between Cosmos SDK coins and Wasm coins
func NewWasmCoins(cosmosCoins sdk.Coins) (wasmCoins []wasmvmtypes.Coin) {
	for _, coin := range cosmosCoins {
		wasmCoin := wasmvmtypes.Coin{
			Denom:  coin.Denom,
			Amount: coin.Amount.String(),
		}
		wasmCoins = append(wasmCoins, wasmCoin)
	}
	return wasmCoins
}

// WasmConfig is the extra config required for wasm
type WasmConfig struct {
	// SimulationGasLimit is the max gas to be used in a tx simulation call.
	// When not set the consensus max block gas is used instead
	SimulationGasLimit *uint64 `mapstructure:"simulation_gas_limit"`
	// SmartQueryGasLimit is the max gas to be used in a smart query contract call
	SmartQueryGasLimit uint64 `mapstructure:"query_gas_limit"`
	// MemoryCacheSize in MiB not bytes
	MemoryCacheSize uint32 `mapstructure:"memory_cache_size"`
	// ContractDebugMode log what contract print
	ContractDebugMode bool
}

// DefaultWasmConfig returns the default settings for WasmConfig
func DefaultWasmConfig() WasmConfig {
	return WasmConfig{
		SmartQueryGasLimit: defaultSmartQueryGasLimit,
		MemoryCacheSize:    defaultMemoryCacheSize,
		ContractDebugMode:  defaultContractDebugMode,
	}
}

// DefaultConfigTemplate toml snippet with default values for app.toml
func DefaultConfigTemplate() string {
	return ConfigTemplate(DefaultWasmConfig())
}

// ConfigTemplate toml snippet for app.toml
func ConfigTemplate(c WasmConfig) string {
	simGasLimit := `# simulation_gas_limit =`
	if c.SimulationGasLimit != nil {
		simGasLimit = fmt.Sprintf(`simulation_gas_limit = %d`, *c.SimulationGasLimit)
	}

	return fmt.Sprintf(`
[wasm]
# Smart query gas limit is the max gas to be used in a smart query contract call
query_gas_limit = %d

# in-memory cache for Wasm contracts. Set to 0 to disable.
# The value is in MiB not bytes
memory_cache_size = %d

# Simulation gas limit is the max gas to be used in a tx simulation call.
# When not set the consensus max block gas is used instead
%s
`, c.SmartQueryGasLimit, c.MemoryCacheSize, simGasLimit)
}

// VerifyAddressLen ensures that the address matches the expected length
func VerifyAddressLen() func(addr []byte) error {
	return func(addr []byte) error {
		if len(addr) != ContractAddrLen && len(addr) != SDKAddrLen {
			return sdkerrors.ErrInvalidAddress
		}
		return nil
	}
}

// IsSubset will return true if the caller is the same as the superset,
// or if the caller is more restrictive than the superset.
func (a AccessType) IsSubset(superSet AccessType) bool {
	switch superSet {
	case AccessTypeEverybody:
		// Everything is a subset of this
		return a != AccessTypeUnspecified
	case AccessTypeNobody:
		// Only an exact match is a subset of this
		return a == AccessTypeNobody
	case AccessTypeOnlyAddress, AccessTypeAnyOfAddresses:
		// Nobody or address(es)
		return a == AccessTypeNobody || a == AccessTypeOnlyAddress || a == AccessTypeAnyOfAddresses
	default:
		return false
	}
}

// IsSubset will return true if the caller is the same as the superset,
// or if the caller is more restrictive than the superset.
func (a AccessConfig) IsSubset(superSet AccessConfig) bool {
	switch superSet.Permission {
	case AccessTypeOnlyAddress:
		// An exact match or nobody
		return a.Permission == AccessTypeNobody || (a.Permission == AccessTypeOnlyAddress && a.Address == superSet.Address) ||
			(a.Permission == AccessTypeAnyOfAddresses && isSubset([]string{superSet.Address}, a.Addresses))
	case AccessTypeAnyOfAddresses:
		// An exact match or nobody
		return a.Permission == AccessTypeNobody || (a.Permission == AccessTypeOnlyAddress && isSubset(superSet.Addresses, []string{a.Address})) ||
			a.Permission == AccessTypeAnyOfAddresses && isSubset(superSet.Addresses, a.Addresses)
	case AccessTypeUnspecified:
		return false
	default:
		return a.Permission.IsSubset(superSet.Permission)
	}
}

// return true when all elements in sub are also part of super
func isSubset(super, sub []string) bool {
	if len(sub) == 0 {
		return true
	}
	var matches int
	for _, o := range sub {
		for _, s := range super {
			if o == s {
				matches++
				break
			}
		}
	}
	return matches == len(sub)
}

// AllAuthorizedAddresses returns the list of authorized addresses. Can be empty.
func (a AccessConfig) AllAuthorizedAddresses() []string {
	switch a.Permission {
	case AccessTypeAnyOfAddresses:
		return a.Addresses
	case AccessTypeOnlyAddress:
		return []string{a.Address}
	}
	return []string{}
}
