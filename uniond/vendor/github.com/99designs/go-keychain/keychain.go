// +build darwin

package keychain

// See https://developer.apple.com/library/ios/documentation/Security/Reference/keychainservices/index.html for the APIs used below.

// Also see https://developer.apple.com/library/ios/documentation/Security/Conceptual/keychainServConcepts/01introduction/introduction.html .

/*
#cgo LDFLAGS: -framework CoreFoundation -framework Security

#include <CoreFoundation/CoreFoundation.h>
#include <Security/Security.h>
*/
import "C"
import (
	"fmt"
	"time"
)

// Error defines keychain errors
type Error int

var (
	// ErrorUnimplemented corresponds to errSecUnimplemented result code
	ErrorUnimplemented = Error(C.errSecUnimplemented)
	// ErrorParam corresponds to errSecParam result code
	ErrorParam = Error(C.errSecParam)
	// ErrorAllocate corresponds to errSecAllocate result code
	ErrorAllocate = Error(C.errSecAllocate)
	// ErrorNotAvailable corresponds to errSecNotAvailable result code
	ErrorNotAvailable = Error(C.errSecNotAvailable)
	// ErrorAuthFailed corresponds to errSecAuthFailed result code
	ErrorAuthFailed = Error(C.errSecAuthFailed)
	// ErrorDuplicateItem corresponds to errSecDuplicateItem result code
	ErrorDuplicateItem = Error(C.errSecDuplicateItem)
	// ErrorItemNotFound corresponds to errSecItemNotFound result code
	ErrorItemNotFound = Error(C.errSecItemNotFound)
	// ErrorInteractionNotAllowed corresponds to errSecInteractionNotAllowed result code
	ErrorInteractionNotAllowed = Error(C.errSecInteractionNotAllowed)
	// ErrorDecode corresponds to errSecDecode result code
	ErrorDecode = Error(C.errSecDecode)
	// ErrorNoSuchKeychain corresponds to errSecNoSuchKeychain result code
	ErrorNoSuchKeychain = Error(C.errSecNoSuchKeychain)
	// ErrorNoAcccessForItem corresponds to errSecNoAccessForItem result code
	ErrorNoAccessForItem = Error(C.errSecNoAccessForItem)
)

func checkError(errCode C.OSStatus) error {
	if errCode == C.errSecSuccess {
		return nil
	}
	return Error(errCode)
}

func (k Error) Error() (msg string) {
	// SecCopyErrorMessageString is only available on OSX, so derive manually.
	// Messages derived from `$ security error $errcode`.
	switch k {
	case ErrorUnimplemented:
		msg = "Function or operation not implemented."
	case ErrorParam:
		msg = "One or more parameters passed to the function were not valid."
	case ErrorAllocate:
		msg = "Failed to allocate memory."
	case ErrorNotAvailable:
		msg = "No keychain is available. You may need to restart your computer."
	case ErrorAuthFailed:
		msg = "The user name or passphrase you entered is not correct."
	case ErrorDuplicateItem:
		msg = "The specified item already exists in the keychain."
	case ErrorItemNotFound:
		msg = "The specified item could not be found in the keychain."
	case ErrorInteractionNotAllowed:
		msg = "User interaction is not allowed."
	case ErrorDecode:
		msg = "Unable to decode the provided data."
	case ErrorNoSuchKeychain:
		msg = "The specified keychain could not be found."
	case ErrorNoAccessForItem:
		msg = "The specified item has no access control."
	default:
		msg = "Keychain Error."
	}
	return fmt.Sprintf("%s (%d)", msg, k)
}

// SecClass is the items class code
type SecClass int

// Keychain Item Classes
var (
	/*
		kSecClassGenericPassword item attributes:
		 kSecAttrAccess (OS X only)
		 kSecAttrAccessGroup (iOS; also OS X if kSecAttrSynchronizable specified)
		 kSecAttrAccessible (iOS; also OS X if kSecAttrSynchronizable specified)
		 kSecAttrAccount
		 kSecAttrService
	*/
	SecClassGenericPassword  SecClass = 1
	SecClassInternetPassword SecClass = 2
)

// SecClassKey is the key type for SecClass
var SecClassKey = attrKey(C.CFTypeRef(C.kSecClass))
var secClassTypeRef = map[SecClass]C.CFTypeRef{
	SecClassGenericPassword:  C.CFTypeRef(C.kSecClassGenericPassword),
	SecClassInternetPassword: C.CFTypeRef(C.kSecClassInternetPassword),
}

var (
	// ServiceKey is for kSecAttrService
	ServiceKey = attrKey(C.CFTypeRef(C.kSecAttrService))
	// LabelKey is for kSecAttrLabel
	LabelKey = attrKey(C.CFTypeRef(C.kSecAttrLabel))
	// AccountKey is for kSecAttrAccount
	AccountKey = attrKey(C.CFTypeRef(C.kSecAttrAccount))
	// AccessGroupKey is for kSecAttrAccessGroup
	AccessGroupKey = attrKey(C.CFTypeRef(C.kSecAttrAccessGroup))
	// DataKey is for kSecValueData
	DataKey = attrKey(C.CFTypeRef(C.kSecValueData))
	// DescriptionKey is for kSecAttrDescription
	DescriptionKey = attrKey(C.CFTypeRef(C.kSecAttrDescription))
	// CreationDateKey is for kSecAttrCreationDate
	CreationDateKey = attrKey(C.CFTypeRef(C.kSecAttrCreationDate))
	// ModificationDateKey is for kSecAttrModificationDate
	ModificationDateKey = attrKey(C.CFTypeRef(C.kSecAttrModificationDate))
)

// Synchronizable is the items synchronizable status
type Synchronizable int

const (
	// SynchronizableDefault is the default setting
	SynchronizableDefault Synchronizable = 0
	// SynchronizableAny is for kSecAttrSynchronizableAny
	SynchronizableAny = 1
	// SynchronizableYes enables synchronization
	SynchronizableYes = 2
	// SynchronizableNo disables synchronization
	SynchronizableNo = 3
)

// SynchronizableKey is the key type for Synchronizable
var SynchronizableKey = attrKey(C.CFTypeRef(C.kSecAttrSynchronizable))
var syncTypeRef = map[Synchronizable]C.CFTypeRef{
	SynchronizableAny: C.CFTypeRef(C.kSecAttrSynchronizableAny),
	SynchronizableYes: C.CFTypeRef(C.kCFBooleanTrue),
	SynchronizableNo:  C.CFTypeRef(C.kCFBooleanFalse),
}

// Accessible is the items accessibility
type Accessible int

const (
	// AccessibleDefault is the default
	AccessibleDefault Accessible = 0
	// AccessibleWhenUnlocked is when unlocked
	AccessibleWhenUnlocked = 1
	// AccessibleAfterFirstUnlock is after first unlock
	AccessibleAfterFirstUnlock = 2
	// AccessibleAlways is always
	AccessibleAlways = 3
	// AccessibleWhenPasscodeSetThisDeviceOnly is when passcode is set
	AccessibleWhenPasscodeSetThisDeviceOnly = 4
	// AccessibleWhenUnlockedThisDeviceOnly is when unlocked for this device only
	AccessibleWhenUnlockedThisDeviceOnly = 5
	// AccessibleAfterFirstUnlockThisDeviceOnly is after first unlock for this device only
	AccessibleAfterFirstUnlockThisDeviceOnly = 6
	// AccessibleAccessibleAlwaysThisDeviceOnly is always for this device only
	AccessibleAccessibleAlwaysThisDeviceOnly = 7
)

// MatchLimit is whether to limit results on query
type MatchLimit int

const (
	// MatchLimitDefault is the default
	MatchLimitDefault MatchLimit = 0
	// MatchLimitOne limits to one result
	MatchLimitOne = 1
	// MatchLimitAll is no limit
	MatchLimitAll = 2
)

// MatchLimitKey is key type for MatchLimit
var MatchLimitKey = attrKey(C.CFTypeRef(C.kSecMatchLimit))
var matchTypeRef = map[MatchLimit]C.CFTypeRef{
	MatchLimitOne: C.CFTypeRef(C.kSecMatchLimitOne),
	MatchLimitAll: C.CFTypeRef(C.kSecMatchLimitAll),
}

// ReturnAttributesKey is key type for kSecReturnAttributes
var ReturnAttributesKey = attrKey(C.CFTypeRef(C.kSecReturnAttributes))

// ReturnDataKey is key type for kSecReturnData
var ReturnDataKey = attrKey(C.CFTypeRef(C.kSecReturnData))

// ReturnRefKey is key type for kSecReturnRef
var ReturnRefKey = attrKey(C.CFTypeRef(C.kSecReturnRef))

// Item for adding, querying or deleting.
type Item struct {
	// Values can be string, []byte, Convertable or CFTypeRef (constant).
	attr map[string]interface{}
}

// SetSecClass sets the security class
func (k *Item) SetSecClass(sc SecClass) {
	k.attr[SecClassKey] = secClassTypeRef[sc]
}

// SetString sets a string attibute for a string key
func (k *Item) SetString(key string, s string) {
	if s != "" {
		k.attr[key] = s
	} else {
		delete(k.attr, key)
	}
}

// SetService sets the service attribute
func (k *Item) SetService(s string) {
	k.SetString(ServiceKey, s)
}

// SetAccount sets the account attribute
func (k *Item) SetAccount(a string) {
	k.SetString(AccountKey, a)
}

// SetLabel sets the label attribute
func (k *Item) SetLabel(l string) {
	k.SetString(LabelKey, l)
}

// SetDescription sets the description attribute
func (k *Item) SetDescription(s string) {
	k.SetString(DescriptionKey, s)
}

// SetData sets the data attribute
func (k *Item) SetData(b []byte) {
	if b != nil {
		k.attr[DataKey] = b
	} else {
		delete(k.attr, DataKey)
	}
}

// SetAccessGroup sets the access group attribute
func (k *Item) SetAccessGroup(ag string) {
	k.SetString(AccessGroupKey, ag)
}

// SetSynchronizable sets the synchronizable attribute
func (k *Item) SetSynchronizable(sync Synchronizable) {
	if sync != SynchronizableDefault {
		k.attr[SynchronizableKey] = syncTypeRef[sync]
	} else {
		delete(k.attr, SynchronizableKey)
	}
}

// SetAccessible sets the accessible attribute
func (k *Item) SetAccessible(accessible Accessible) {
	if accessible != AccessibleDefault {
		k.attr[AccessibleKey] = accessibleTypeRef[accessible]
	} else {
		delete(k.attr, AccessibleKey)
	}
}

// SetMatchLimit sets the match limit
func (k *Item) SetMatchLimit(matchLimit MatchLimit) {
	if matchLimit != MatchLimitDefault {
		k.attr[MatchLimitKey] = matchTypeRef[matchLimit]
	} else {
		delete(k.attr, MatchLimitKey)
	}
}

// SetReturnAttributes sets the return value type on query
func (k *Item) SetReturnAttributes(b bool) {
	k.attr[ReturnAttributesKey] = b
}

// SetReturnData enables returning data on query
func (k *Item) SetReturnData(b bool) {
	k.attr[ReturnDataKey] = b
}

// SetReturnRef enables returning references on query
func (k *Item) SetReturnRef(b bool) {
	k.attr[ReturnRefKey] = b
}

// NewItem is a new empty keychain item
func NewItem() Item {
	return Item{make(map[string]interface{})}
}

// NewGenericPassword creates a generic password item with the default keychain. This is a convenience method.
func NewGenericPassword(service string, account string, label string, data []byte, accessGroup string) Item {
	item := NewItem()
	item.SetSecClass(SecClassGenericPassword)
	item.SetService(service)
	item.SetAccount(account)
	item.SetLabel(label)
	item.SetData(data)
	item.SetAccessGroup(accessGroup)
	return item
}

// AddItem adds a Item to a Keychain
func AddItem(item Item) error {
	cfDict, err := ConvertMapToCFDictionary(item.attr)
	if err != nil {
		return err
	}
	defer Release(C.CFTypeRef(cfDict))

	errCode := C.SecItemAdd(cfDict, nil)
	err = checkError(errCode)
	return err
}

// UpdateItem updates the queryItem with the parameters from updateItem
func UpdateItem(queryItem Item, updateItem Item) error {
	cfDict, err := ConvertMapToCFDictionary(queryItem.attr)
	if err != nil {
		return err
	}
	defer Release(C.CFTypeRef(cfDict))
	cfDictUpdate, err := ConvertMapToCFDictionary(updateItem.attr)
	if err != nil {
		return err
	}
	defer Release(C.CFTypeRef(cfDictUpdate))
	errCode := C.SecItemUpdate(cfDict, cfDictUpdate)
	err = checkError(errCode)
	return err
}

// QueryResult stores all possible results from queries.
// Not all fields are applicable all the time. Results depend on query.
type QueryResult struct {
	Service          string
	Account          string
	AccessGroup      string
	Label            string
	Description      string
	Data             []byte
	CreationDate     time.Time
	ModificationDate time.Time
}

// QueryItemRef returns query result as CFTypeRef. You must release it when you are done.
func QueryItemRef(item Item) (C.CFTypeRef, error) {
	cfDict, err := ConvertMapToCFDictionary(item.attr)
	if err != nil {
		return 0, err
	}
	defer Release(C.CFTypeRef(cfDict))

	var resultsRef C.CFTypeRef
	errCode := C.SecItemCopyMatching(cfDict, &resultsRef) //nolint
	if Error(errCode) == ErrorItemNotFound {
		return 0, nil
	}
	err = checkError(errCode)
	if err != nil {
		return 0, err
	}
	return resultsRef, nil
}

// QueryItem returns a list of query results.
func QueryItem(item Item) ([]QueryResult, error) {
	resultsRef, err := QueryItemRef(item)
	if err != nil {
		return nil, err
	}
	if resultsRef == 0 {
		return nil, nil
	}
	defer Release(resultsRef)

	results := make([]QueryResult, 0, 1)

	typeID := C.CFGetTypeID(resultsRef)
	if typeID == C.CFArrayGetTypeID() {
		arr := CFArrayToArray(C.CFArrayRef(resultsRef))
		for _, ref := range arr {
			elementTypeID := C.CFGetTypeID(ref)
			if elementTypeID == C.CFDictionaryGetTypeID() {
				item, err := convertResult(C.CFDictionaryRef(ref))
				if err != nil {
					return nil, err
				}
				results = append(results, *item)
			} else {
				return nil, fmt.Errorf("invalid result type (If you SetReturnRef(true) you should use QueryItemRef directly)")
			}
		}
	} else if typeID == C.CFDictionaryGetTypeID() {
		item, err := convertResult(C.CFDictionaryRef(resultsRef))
		if err != nil {
			return nil, err
		}
		results = append(results, *item)
	} else if typeID == C.CFDataGetTypeID() {
		b, err := CFDataToBytes(C.CFDataRef(resultsRef))
		if err != nil {
			return nil, err
		}
		item := QueryResult{Data: b}
		results = append(results, item)
	} else {
		return nil, fmt.Errorf("Invalid result type: %s", CFTypeDescription(resultsRef))
	}

	return results, nil
}

func attrKey(ref C.CFTypeRef) string {
	return CFStringToString(C.CFStringRef(ref))
}

func convertResult(d C.CFDictionaryRef) (*QueryResult, error) {
	m := CFDictionaryToMap(d)
	result := QueryResult{}
	for k, v := range m {
		switch attrKey(k) {
		case ServiceKey:
			result.Service = CFStringToString(C.CFStringRef(v))
		case AccountKey:
			result.Account = CFStringToString(C.CFStringRef(v))
		case AccessGroupKey:
			result.AccessGroup = CFStringToString(C.CFStringRef(v))
		case LabelKey:
			result.Label = CFStringToString(C.CFStringRef(v))
		case DescriptionKey:
			result.Description = CFStringToString(C.CFStringRef(v))
		case DataKey:
			b, err := CFDataToBytes(C.CFDataRef(v))
			if err != nil {
				return nil, err
			}
			result.Data = b
		case CreationDateKey:
			result.CreationDate = CFDateToTime(C.CFDateRef(v))
		case ModificationDateKey:
			result.ModificationDate = CFDateToTime(C.CFDateRef(v))
			// default:
			// fmt.Printf("Unhandled key in conversion: %v = %v\n", cfTypeValue(k), cfTypeValue(v))
		}
	}
	return &result, nil
}

// DeleteGenericPasswordItem removes a generic password item.
func DeleteGenericPasswordItem(service string, account string) error {
	item := NewItem()
	item.SetSecClass(SecClassGenericPassword)
	item.SetService(service)
	item.SetAccount(account)
	return DeleteItem(item)
}

// DeleteItem removes a Item
func DeleteItem(item Item) error {
	cfDict, err := ConvertMapToCFDictionary(item.attr)
	if err != nil {
		return err
	}
	defer Release(C.CFTypeRef(cfDict))

	errCode := C.SecItemDelete(cfDict)
	return checkError(errCode)
}

// GetAccountsForService is deprecated
func GetAccountsForService(service string) ([]string, error) {
	return GetGenericPasswordAccounts(service)
}

// GetGenericPasswordAccounts returns generic password accounts for service. This is a convenience method.
func GetGenericPasswordAccounts(service string) ([]string, error) {
	query := NewItem()
	query.SetSecClass(SecClassGenericPassword)
	query.SetService(service)
	query.SetMatchLimit(MatchLimitAll)
	query.SetReturnAttributes(true)
	results, err := QueryItem(query)
	if err != nil {
		return nil, err
	}

	accounts := make([]string, 0, len(results))
	for _, r := range results {
		accounts = append(accounts, r.Account)
	}

	return accounts, nil
}

// GetGenericPassword returns password data for service and account. This is a convenience method.
// If item is not found returns nil, nil.
func GetGenericPassword(service string, account string, label string, accessGroup string) ([]byte, error) {
	query := NewItem()
	query.SetSecClass(SecClassGenericPassword)
	query.SetService(service)
	query.SetAccount(account)
	query.SetLabel(label)
	query.SetAccessGroup(accessGroup)
	query.SetMatchLimit(MatchLimitOne)
	query.SetReturnData(true)
	results, err := QueryItem(query)
	if err != nil {
		return nil, err
	}
	if len(results) > 1 {
		return nil, fmt.Errorf("Too many results")
	}
	if len(results) == 1 {
		return results[0].Data, nil
	}
	return nil, nil
}
