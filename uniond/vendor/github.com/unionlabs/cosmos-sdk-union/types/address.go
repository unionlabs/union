package types

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"sync"
	"sync/atomic"

	"github.com/hashicorp/golang-lru/simplelru"
	"sigs.k8s.io/yaml"

	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/internal/conv"
	"github.com/cosmos/cosmos-sdk/types/bech32"
)

const (
	// Constants defined here are the defaults value for address.
	// You can use the specific values for your project.
	// Add the follow lines to the `main()` of your server.
	//
	//	config := sdk.GetConfig()
	//	config.SetBech32PrefixForAccount(yourBech32PrefixAccAddr, yourBech32PrefixAccPub)
	//	config.SetBech32PrefixForValidator(yourBech32PrefixValAddr, yourBech32PrefixValPub)
	//	config.SetBech32PrefixForConsensusNode(yourBech32PrefixConsAddr, yourBech32PrefixConsPub)
	//	config.SetPurpose(yourPurpose)
	//	config.Seal()

	// Bech32MainPrefix defines the main SDK Bech32 prefix of an account's address
	Bech32MainPrefix = "cosmos"

	// Purpose is the ATOM purpose as defined in SLIP44 (https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
	Purpose = 44

	// CoinType is the ATOM coin type as defined in SLIP44 (https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
	CoinType = 118

	// FullFundraiserPath is the parts of the BIP44 HD path that are fixed by
	// what we used during the ATOM fundraiser.
	FullFundraiserPath = "m/44'/118'/0'/0/0"

	// PrefixAccount is the prefix for account keys
	PrefixAccount = "acc"
	// PrefixValidator is the prefix for validator keys
	PrefixValidator = "val"
	// PrefixConsensus is the prefix for consensus keys
	PrefixConsensus = "cons"
	// PrefixPublic is the prefix for public keys
	PrefixPublic = "pub"
	// PrefixOperator is the prefix for operator keys
	PrefixOperator = "oper"

	// PrefixAddress is the prefix for addresses
	PrefixAddress = "addr"

	// Bech32PrefixAccAddr defines the Bech32 prefix of an account's address
	Bech32PrefixAccAddr = Bech32MainPrefix
	// Bech32PrefixAccPub defines the Bech32 prefix of an account's public key
	Bech32PrefixAccPub = Bech32MainPrefix + PrefixPublic
	// Bech32PrefixValAddr defines the Bech32 prefix of a validator's operator address
	Bech32PrefixValAddr = Bech32MainPrefix + PrefixValidator + PrefixOperator
	// Bech32PrefixValPub defines the Bech32 prefix of a validator's operator public key
	Bech32PrefixValPub = Bech32MainPrefix + PrefixValidator + PrefixOperator + PrefixPublic
	// Bech32PrefixConsAddr defines the Bech32 prefix of a consensus node address
	Bech32PrefixConsAddr = Bech32MainPrefix + PrefixValidator + PrefixConsensus
	// Bech32PrefixConsPub defines the Bech32 prefix of a consensus node public key
	Bech32PrefixConsPub = Bech32MainPrefix + PrefixValidator + PrefixConsensus + PrefixPublic
)

// GetBech32PrefixAccPub returns the Bech32 prefix of an account's public key.
func GetBech32PrefixAccPub(mainPrefix string) string {
	return mainPrefix + PrefixPublic
}

// GetBech32PrefixValAddr returns the Bech32 prefix of a validator's operator address.
func GetBech32PrefixValAddr(mainPrefix string) string {
	return mainPrefix + PrefixValidator + PrefixOperator
}

// GetBech32PrefixValPub returns the Bech32 prefix of a validator's operator public key.
func GetBech32PrefixValPub(mainPrefix string) string {
	return mainPrefix + PrefixValidator + PrefixOperator + PrefixPublic
}

// GetBech32PrefixConsAddr returns the Bech32 prefix of a consensus node address.
func GetBech32PrefixConsAddr(mainPrefix string) string {
	return mainPrefix + PrefixValidator + PrefixConsensus
}

// GetBech32PrefixConsPub returns the Bech32 prefix of a consensus node public key.
func GetBech32PrefixConsPub(mainPrefix string) string {
	return mainPrefix + PrefixValidator + PrefixConsensus + PrefixPublic
}

// cache variables
var (
	// AccAddress.String() is expensive and if unoptimized dominantly showed up in profiles,
	// yet has no mechanisms to trivially cache the result given that AccAddress is a []byte type.
	accAddrMu     sync.Mutex
	accAddrCache  *simplelru.LRU
	consAddrMu    sync.Mutex
	consAddrCache *simplelru.LRU
	valAddrMu     sync.Mutex
	valAddrCache  *simplelru.LRU

	isCachingEnabled atomic.Bool
)

// sentinel errors
var (
	ErrEmptyHexAddress = errors.New("decoding address from hex string failed: empty address")
)

func init() {
	var err error
	SetAddrCacheEnabled(true)

	// in total the cache size is 61k entries. Key is 32 bytes and value is around 50-70 bytes.
	// That will make around 92 * 61k * 2 (LRU) bytes ~ 11 MB
	if accAddrCache, err = simplelru.NewLRU(60000, nil); err != nil {
		panic(err)
	}
	if consAddrCache, err = simplelru.NewLRU(500, nil); err != nil {
		panic(err)
	}
	if valAddrCache, err = simplelru.NewLRU(500, nil); err != nil {
		panic(err)
	}
}

// SetAddrCacheEnabled enables or disables accAddrCache, consAddrCache, and valAddrCache. By default, caches are enabled.
func SetAddrCacheEnabled(enabled bool) {
	isCachingEnabled.Store(enabled)
}

// IsAddrCacheEnabled returns if the address caches are enabled.
func IsAddrCacheEnabled() bool {
	return isCachingEnabled.Load()
}

// Address is a common interface for different types of addresses used by the SDK
type Address interface {
	Equals(Address) bool
	Empty() bool
	Marshal() ([]byte, error)
	MarshalJSON() ([]byte, error)
	Bytes() []byte
	// Deprecated: Use an address.Codec to convert addresses from and to string/bytes.
	String() string
	Format(s fmt.State, verb rune)
}

// Ensure that different address types implement the interface
var (
	_ Address = AccAddress{}
	_ Address = ValAddress{}
	_ Address = ConsAddress{}
)

// ----------------------------------------------------------------------------
// account
// ----------------------------------------------------------------------------

// AccAddress a wrapper around bytes meant to represent an account address.
// When marshaled to a string or JSON, it uses Bech32.
type AccAddress []byte

// AccAddressFromHexUnsafe creates an AccAddress from a HEX-encoded string.
//
// Note, this function is considered unsafe as it may produce an AccAddress from
// otherwise invalid input, such as a transaction hash. Please use
// AccAddressFromBech32.
func AccAddressFromHexUnsafe(address string) (addr AccAddress, err error) {
	bz, err := addressBytesFromHexString(address)
	return AccAddress(bz), err
}

// MustAccAddressFromBech32 calls AccAddressFromBech32 and panics on error.
// Deprecated: Use an address.Codec to convert addresses from and to string/bytes.
func MustAccAddressFromBech32(address string) AccAddress {
	addr, err := AccAddressFromBech32(address)
	if err != nil {
		panic(err)
	}

	return addr
}

// AccAddressFromBech32 creates an AccAddress from a Bech32 string.
// Deprecated: Use an address.Codec to convert addresses from and to string/bytes.
func AccAddressFromBech32(address string) (addr AccAddress, err error) {
	bech32PrefixAccAddr := GetConfig().GetBech32AccountAddrPrefix()

	addrCdc := addresscodec.NewBech32Codec(bech32PrefixAccAddr)
	return addrCdc.StringToBytes(address)
}

// Returns boolean for whether two AccAddresses are Equal
func (aa AccAddress) Equals(aa2 Address) bool {
	if aa.Empty() && aa2.Empty() {
		return true
	}

	return bytes.Equal(aa.Bytes(), aa2.Bytes())
}

// Returns boolean for whether an AccAddress is empty
func (aa AccAddress) Empty() bool {
	return len(aa) == 0
}

// Marshal returns the raw address bytes. It is needed for protobuf
// compatibility.
func (aa AccAddress) Marshal() ([]byte, error) {
	return aa, nil
}

// Unmarshal sets the address to the given data. It is needed for protobuf
// compatibility.
func (aa *AccAddress) Unmarshal(data []byte) error {
	*aa = data
	return nil
}

// MarshalJSON marshals to JSON using Bech32.
func (aa AccAddress) MarshalJSON() ([]byte, error) {
	return json.Marshal(aa.String())
}

// MarshalYAML marshals to YAML using Bech32.
func (aa AccAddress) MarshalYAML() (interface{}, error) {
	return aa.String(), nil
}

// UnmarshalJSON unmarshals from JSON assuming Bech32 encoding.
func (aa *AccAddress) UnmarshalJSON(data []byte) error {
	var s string
	err := json.Unmarshal(data, &s)
	if err != nil {
		return err
	}
	if s == "" {
		*aa = AccAddress{}
		return nil
	}

	aa2, err := AccAddressFromBech32(s)
	if err != nil {
		return err
	}

	*aa = aa2
	return nil
}

// UnmarshalYAML unmarshals from JSON assuming Bech32 encoding.
func (aa *AccAddress) UnmarshalYAML(data []byte) error {
	var s string
	err := yaml.Unmarshal(data, &s)
	if err != nil {
		return err
	}
	if s == "" {
		*aa = AccAddress{}
		return nil
	}

	aa2, err := AccAddressFromBech32(s)
	if err != nil {
		return err
	}

	*aa = aa2
	return nil
}

// Bytes returns the raw address bytes.
func (aa AccAddress) Bytes() []byte {
	return aa
}

// String implements the Stringer interface.
// Deprecated: Use an address.Codec to convert addresses from and to string/bytes.
func (aa AccAddress) String() string {
	if aa.Empty() {
		return ""
	}

	key := conv.UnsafeBytesToStr(aa)

	if IsAddrCacheEnabled() {
		accAddrMu.Lock()
		defer accAddrMu.Unlock()

		addr, ok := accAddrCache.Get(key)
		if ok {
			return addr.(string)
		}
	}
	return cacheBech32Addr(GetConfig().GetBech32AccountAddrPrefix(), aa, accAddrCache, key)
}

// Format implements the fmt.Formatter interface.

func (aa AccAddress) Format(s fmt.State, verb rune) {
	switch verb {
	case 's':
		_, _ = s.Write([]byte(aa.String()))
	case 'p':
		_, _ = s.Write([]byte(fmt.Sprintf("%p", aa)))
	default:
		_, _ = s.Write([]byte(fmt.Sprintf("%X", []byte(aa))))
	}
}

// ----------------------------------------------------------------------------
// validator operator
// ----------------------------------------------------------------------------

// ValAddress defines a wrapper around bytes meant to present a validator's
// operator. When marshaled to a string or JSON, it uses Bech32.
type ValAddress []byte

// ValAddressFromHex creates a ValAddress from a hex string.
func ValAddressFromHex(address string) (addr ValAddress, err error) {
	bz, err := addressBytesFromHexString(address)
	return ValAddress(bz), err
}

// ValAddressFromBech32 creates a ValAddress from a Bech32 string.
// Deprecated: Use an address.Codec to convert addresses from and to string/bytes.
func ValAddressFromBech32(address string) (addr ValAddress, err error) {
	bech32PrefixValAddr := GetConfig().GetBech32ValidatorAddrPrefix()

	addrCdc := addresscodec.NewBech32Codec(bech32PrefixValAddr)
	return addrCdc.StringToBytes(address)
}

// MustValAddressFromBech32 calls ValAddressFromBech32 and panics on error.
// Deprecated: Use an address.Codec to convert addresses from and to string/bytes.
func MustValAddressFromBech32(address string) ValAddress {
	addr, err := ValAddressFromBech32(address)
	if err != nil {
		panic(err)
	}

	return addr
}

// Returns boolean for whether two ValAddresses are Equal
func (va ValAddress) Equals(va2 Address) bool {
	if va.Empty() && va2.Empty() {
		return true
	}

	return bytes.Equal(va.Bytes(), va2.Bytes())
}

// Returns boolean for whether an ValAddress is empty
func (va ValAddress) Empty() bool {
	return len(va) == 0
}

// Marshal returns the raw address bytes. It is needed for protobuf
// compatibility.
func (va ValAddress) Marshal() ([]byte, error) {
	return va, nil
}

// Unmarshal sets the address to the given data. It is needed for protobuf
// compatibility.
func (va *ValAddress) Unmarshal(data []byte) error {
	*va = data
	return nil
}

// MarshalJSON marshals to JSON using Bech32.
func (va ValAddress) MarshalJSON() ([]byte, error) {
	return json.Marshal(va.String())
}

// MarshalYAML marshals to YAML using Bech32.
func (va ValAddress) MarshalYAML() (interface{}, error) {
	return va.String(), nil
}

// UnmarshalJSON unmarshals from JSON assuming Bech32 encoding.
func (va *ValAddress) UnmarshalJSON(data []byte) error {
	var s string

	err := json.Unmarshal(data, &s)
	if err != nil {
		return err
	}
	if s == "" {
		*va = ValAddress{}
		return nil
	}

	va2, err := ValAddressFromBech32(s)
	if err != nil {
		return err
	}

	*va = va2
	return nil
}

// UnmarshalYAML unmarshals from YAML assuming Bech32 encoding.
func (va *ValAddress) UnmarshalYAML(data []byte) error {
	var s string

	err := yaml.Unmarshal(data, &s)
	if err != nil {
		return err
	}
	if s == "" {
		*va = ValAddress{}
		return nil
	}

	va2, err := ValAddressFromBech32(s)
	if err != nil {
		return err
	}

	*va = va2
	return nil
}

// Bytes returns the raw address bytes.
func (va ValAddress) Bytes() []byte {
	return va
}

// String implements the Stringer interface.
// Deprecated: Use an address.Codec to convert addresses from and to string/bytes.
func (va ValAddress) String() string {
	if va.Empty() {
		return ""
	}

	key := conv.UnsafeBytesToStr(va)

	if IsAddrCacheEnabled() {
		valAddrMu.Lock()
		defer valAddrMu.Unlock()

		addr, ok := valAddrCache.Get(key)
		if ok {
			return addr.(string)
		}
	}
	return cacheBech32Addr(GetConfig().GetBech32ValidatorAddrPrefix(), va, valAddrCache, key)
}

// Format implements the fmt.Formatter interface.

func (va ValAddress) Format(s fmt.State, verb rune) {
	switch verb {
	case 's':
		_, _ = s.Write([]byte(va.String()))
	case 'p':
		_, _ = s.Write([]byte(fmt.Sprintf("%p", va)))
	default:
		_, _ = s.Write([]byte(fmt.Sprintf("%X", []byte(va))))
	}
}

// ----------------------------------------------------------------------------
// consensus node
// ----------------------------------------------------------------------------

// ConsAddress defines a wrapper around bytes meant to present a consensus node.
// When marshaled to a string or JSON, it uses Bech32.
type ConsAddress []byte

// ConsAddressFromHex creates a ConsAddress from a hex string.
// Deprecated: use ConsensusAddressCodec from Staking keeper
func ConsAddressFromHex(address string) (addr ConsAddress, err error) {
	bz, err := addressBytesFromHexString(address)
	return ConsAddress(bz), err
}

// ConsAddressFromBech32 creates a ConsAddress from a Bech32 string.
// Deprecated: Use an address.Codec to convert addresses from and to string/bytes.
func ConsAddressFromBech32(address string) (addr ConsAddress, err error) {
	bech32PrefixConsAddr := GetConfig().GetBech32ConsensusAddrPrefix()

	addrCdc := addresscodec.NewBech32Codec(bech32PrefixConsAddr)
	return addrCdc.StringToBytes(address)
}

// get ConsAddress from pubkey
func GetConsAddress(pubkey cryptotypes.PubKey) ConsAddress {
	return ConsAddress(pubkey.Address())
}

// Returns boolean for whether two ConsAddress are Equal
func (ca ConsAddress) Equals(ca2 Address) bool {
	if ca.Empty() && ca2.Empty() {
		return true
	}

	return bytes.Equal(ca.Bytes(), ca2.Bytes())
}

// Returns boolean for whether an ConsAddress is empty
func (ca ConsAddress) Empty() bool {
	return len(ca) == 0
}

// Marshal returns the raw address bytes. It is needed for protobuf
// compatibility.
func (ca ConsAddress) Marshal() ([]byte, error) {
	return ca, nil
}

// Unmarshal sets the address to the given data. It is needed for protobuf
// compatibility.
func (ca *ConsAddress) Unmarshal(data []byte) error {
	*ca = data
	return nil
}

// MarshalJSON marshals to JSON using Bech32.
func (ca ConsAddress) MarshalJSON() ([]byte, error) {
	return json.Marshal(ca.String())
}

// MarshalYAML marshals to YAML using Bech32.
func (ca ConsAddress) MarshalYAML() (interface{}, error) {
	return ca.String(), nil
}

// UnmarshalJSON unmarshals from JSON assuming Bech32 encoding.
func (ca *ConsAddress) UnmarshalJSON(data []byte) error {
	var s string

	err := json.Unmarshal(data, &s)
	if err != nil {
		return err
	}
	if s == "" {
		*ca = ConsAddress{}
		return nil
	}

	ca2, err := ConsAddressFromBech32(s)
	if err != nil {
		return err
	}

	*ca = ca2
	return nil
}

// UnmarshalYAML unmarshals from YAML assuming Bech32 encoding.
func (ca *ConsAddress) UnmarshalYAML(data []byte) error {
	var s string

	err := yaml.Unmarshal(data, &s)
	if err != nil {
		return err
	}
	if s == "" {
		*ca = ConsAddress{}
		return nil
	}

	ca2, err := ConsAddressFromBech32(s)
	if err != nil {
		return err
	}

	*ca = ca2
	return nil
}

// Bytes returns the raw address bytes.
func (ca ConsAddress) Bytes() []byte {
	return ca
}

// String implements the Stringer interface.
// Deprecated: Use an address.Codec to convert addresses from and to string/bytes.
func (ca ConsAddress) String() string {
	if ca.Empty() {
		return ""
	}

	key := conv.UnsafeBytesToStr(ca)

	if IsAddrCacheEnabled() {
		consAddrMu.Lock()
		defer consAddrMu.Unlock()

		addr, ok := consAddrCache.Get(key)
		if ok {
			return addr.(string)
		}
	}
	return cacheBech32Addr(GetConfig().GetBech32ConsensusAddrPrefix(), ca, consAddrCache, key)
}

// Bech32ifyAddressBytes returns a bech32 representation of address bytes.
// Returns an empty string if the byte slice is 0-length. Returns an error if the bech32 conversion
// fails or the prefix is empty.
func Bech32ifyAddressBytes(prefix string, bs []byte) (string, error) {
	if len(bs) == 0 {
		return "", nil
	}
	if len(prefix) == 0 {
		return "", errors.New("prefix cannot be empty")
	}
	return bech32.ConvertAndEncode(prefix, bs)
}

// MustBech32ifyAddressBytes returns a bech32 representation of address bytes.
// Returns an empty string if the byte slice is 0-length. It panics if the bech32 conversion
// fails or the prefix is empty.
func MustBech32ifyAddressBytes(prefix string, bs []byte) string {
	s, err := Bech32ifyAddressBytes(prefix, bs)
	if err != nil {
		panic(err)
	}
	return s
}

// Format implements the fmt.Formatter interface.

func (ca ConsAddress) Format(s fmt.State, verb rune) {
	switch verb {
	case 's':
		_, _ = s.Write([]byte(ca.String()))
	case 'p':
		_, _ = s.Write([]byte(fmt.Sprintf("%p", ca)))
	default:
		_, _ = s.Write([]byte(fmt.Sprintf("%X", []byte(ca))))
	}
}

// ----------------------------------------------------------------------------
// auxiliary
// ----------------------------------------------------------------------------

var errBech32EmptyAddress = errors.New("decoding Bech32 address failed: must provide a non empty address")

// GetFromBech32 decodes a bytestring from a Bech32 encoded string.
func GetFromBech32(bech32str, prefix string) ([]byte, error) {
	if len(bech32str) == 0 {
		return nil, errBech32EmptyAddress
	}

	hrp, bz, err := bech32.DecodeAndConvert(bech32str)
	if err != nil {
		return nil, err
	}

	if hrp != prefix {
		return nil, fmt.Errorf("invalid Bech32 prefix; expected %s, got %s", prefix, hrp)
	}

	return bz, nil
}

func addressBytesFromHexString(address string) ([]byte, error) {
	if len(address) == 0 {
		return nil, ErrEmptyHexAddress
	}

	return hex.DecodeString(address)
}

// cacheBech32Addr is not concurrency safe. Concurrent access to cache causes race condition.
func cacheBech32Addr(prefix string, addr []byte, cache *simplelru.LRU, cacheKey string) string {
	bech32Addr, err := bech32.ConvertAndEncode(prefix, addr)
	if err != nil {
		panic(err)
	}
	if IsAddrCacheEnabled() {
		cache.Add(cacheKey, bech32Addr)
	}
	return bech32Addr
}

// GetFullBIP44Path returns the BIP44Prefix.
func GetFullBIP44Path() string {
	return fmt.Sprintf("m/%d'/%d'/0'/0/0", Purpose, CoinType)
}
