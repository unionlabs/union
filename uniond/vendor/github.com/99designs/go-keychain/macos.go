// +build darwin,!ios

package keychain

/*
#cgo LDFLAGS: -framework CoreFoundation -framework Security
#cgo CFLAGS: -w

#include <CoreFoundation/CoreFoundation.h>
#include <Security/Security.h>
*/
import "C"
import (
	"os"
	"unsafe"
)

// AccessibleKey is key for kSecAttrAccessible
var AccessibleKey = attrKey(C.CFTypeRef(C.kSecAttrAccessible))
var accessibleTypeRef = map[Accessible]C.CFTypeRef{
	AccessibleWhenUnlocked:                   C.CFTypeRef(C.kSecAttrAccessibleWhenUnlocked),
	AccessibleAfterFirstUnlock:               C.CFTypeRef(C.kSecAttrAccessibleAfterFirstUnlock),
	AccessibleAlways:                         C.CFTypeRef(C.kSecAttrAccessibleAlways),
	AccessibleWhenUnlockedThisDeviceOnly:     C.CFTypeRef(C.kSecAttrAccessibleWhenUnlockedThisDeviceOnly),
	AccessibleAfterFirstUnlockThisDeviceOnly: C.CFTypeRef(C.kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly),
	AccessibleAccessibleAlwaysThisDeviceOnly: C.CFTypeRef(C.kSecAttrAccessibleAlwaysThisDeviceOnly),

	// Only available in 10.10
	//AccessibleWhenPasscodeSetThisDeviceOnly:  C.CFTypeRef(C.kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly),
}

var (
	// AccessKey is key for kSecAttrAccess
	AccessKey = attrKey(C.CFTypeRef(C.kSecAttrAccess))
)

// createAccess creates a SecAccessRef as CFTypeRef.
// The returned SecAccessRef, if non-nil, must be released via CFRelease.
func createAccess(label string, trustedApplications []string) (C.CFTypeRef, error) {
	var err error
	var labelRef C.CFStringRef
	if labelRef, err = StringToCFString(label); err != nil {
		return 0, err
	}
	defer C.CFRelease(C.CFTypeRef(labelRef))

	var trustedApplicationsArray C.CFArrayRef
	if trustedApplications != nil {
		if len(trustedApplications) > 0 {
			// Always prepend with empty string which signifies that we
			// include a NULL application, which means ourselves.
			trustedApplications = append([]string{""}, trustedApplications...)
		}

		var trustedApplicationsRefs []C.CFTypeRef
		for _, trustedApplication := range trustedApplications {
			trustedApplicationRef, createErr := createTrustedApplication(trustedApplication)
			if createErr != nil {
				return 0, createErr
			}
			defer C.CFRelease(trustedApplicationRef)
			trustedApplicationsRefs = append(trustedApplicationsRefs, trustedApplicationRef)
		}

		trustedApplicationsArray = ArrayToCFArray(trustedApplicationsRefs)
		defer C.CFRelease(C.CFTypeRef(trustedApplicationsArray))
	}

	var access C.SecAccessRef
	errCode := C.SecAccessCreate(labelRef, trustedApplicationsArray, &access) //nolint
	err = checkError(errCode)
	if err != nil {
		return 0, err
	}

	return C.CFTypeRef(access), nil
}

// createTrustedApplication creates a SecTrustedApplicationRef as a CFTypeRef.
// The returned SecTrustedApplicationRef, if non-nil, must be released via CFRelease.
func createTrustedApplication(trustedApplication string) (C.CFTypeRef, error) {
	var trustedApplicationCStr *C.char
	if trustedApplication != "" {
		trustedApplicationCStr = C.CString(trustedApplication)
		defer C.free(unsafe.Pointer(trustedApplicationCStr))
	}

	var trustedApplicationRef C.SecTrustedApplicationRef
	errCode := C.SecTrustedApplicationCreateFromPath(trustedApplicationCStr, &trustedApplicationRef) //nolint
	err := checkError(errCode)
	if err != nil {
		return 0, err
	}

	return C.CFTypeRef(trustedApplicationRef), nil
}

// Access defines whats applications can use the keychain item
type Access struct {
	Label               string
	TrustedApplications []string
}

// Convert converts Access to CFTypeRef.
// The returned CFTypeRef, if non-nil, must be released via CFRelease.
func (a Access) Convert() (C.CFTypeRef, error) {
	return createAccess(a.Label, a.TrustedApplications)
}

// SetAccess sets Access on Item
func (k *Item) SetAccess(a *Access) {
	if a != nil {
		k.attr[AccessKey] = a
	} else {
		delete(k.attr, AccessKey)
	}
}

// DeleteItemRef deletes a keychain item reference.
func DeleteItemRef(ref C.CFTypeRef) error {
	errCode := C.SecKeychainItemDelete(C.SecKeychainItemRef(ref))
	return checkError(errCode)
}

var (
	// KeychainKey is key for kSecUseKeychain
	KeychainKey = attrKey(C.CFTypeRef(C.kSecUseKeychain))
	// MatchSearchListKey is key for kSecMatchSearchList
	MatchSearchListKey = attrKey(C.CFTypeRef(C.kSecMatchSearchList))
)

// Keychain represents the path to a specific OSX keychain
type Keychain struct {
	path string
}

// NewKeychain creates a new keychain file with a password
func NewKeychain(path string, password string) (Keychain, error) {
	return newKeychain(path, password, false)
}

// NewKeychainWithPrompt creates a new Keychain and prompts user for password
func NewKeychainWithPrompt(path string) (Keychain, error) {
	return newKeychain(path, "", true)
}

func newKeychain(path, password string, promptUser bool) (Keychain, error) {
	pathRef := C.CString(path)
	defer C.free(unsafe.Pointer(pathRef))

	var errCode C.OSStatus
	var kref C.SecKeychainRef

	if promptUser {
		errCode = C.SecKeychainCreate(pathRef, C.UInt32(0), nil, C.Boolean(1), 0, &kref) //nolint
	} else {
		passwordRef := C.CString(password)
		defer C.free(unsafe.Pointer(passwordRef))
		errCode = C.SecKeychainCreate(pathRef, C.UInt32(len(password)), unsafe.Pointer(passwordRef), C.Boolean(0), 0, &kref) //nolint
	}

	if err := checkError(errCode); err != nil {
		return Keychain{}, err
	}

	// TODO: Without passing in kref I get 'One or more parameters passed to the function were not valid (-50)'
	defer Release(C.CFTypeRef(kref))

	return Keychain{
		path: path,
	}, nil
}

// NewWithPath to use an existing keychain
func NewWithPath(path string) Keychain {
	return Keychain{
		path: path,
	}
}

// Status returns the status of the keychain
func (kc Keychain) Status() error {
	// returns no error even if it doesn't exist
	kref, err := openKeychainRef(kc.path)
	if err != nil {
		return err
	}
	defer C.CFRelease(C.CFTypeRef(kref))

	var status C.SecKeychainStatus
	return checkError(C.SecKeychainGetStatus(kref, &status))
}

// The returned SecKeychainRef, if non-nil, must be released via CFRelease.
func openKeychainRef(path string) (C.SecKeychainRef, error) {
	pathName := C.CString(path)
	defer C.free(unsafe.Pointer(pathName))

	var kref C.SecKeychainRef
	if err := checkError(C.SecKeychainOpen(pathName, &kref)); err != nil { //nolint
		return 0, err
	}

	return kref, nil
}

// UnlockAtPath unlocks keychain at path
func UnlockAtPath(path string, password string) error {
	kref, err := openKeychainRef(path)
	defer Release(C.CFTypeRef(kref))
	if err != nil {
		return err
	}
	passwordRef := C.CString(password)
	defer C.free(unsafe.Pointer(passwordRef))
	return checkError(C.SecKeychainUnlock(kref, C.UInt32(len(password)), unsafe.Pointer(passwordRef), C.Boolean(1)))
}

// LockAtPath locks keychain at path
func LockAtPath(path string) error {
	kref, err := openKeychainRef(path)
	defer Release(C.CFTypeRef(kref))
	if err != nil {
		return err
	}
	return checkError(C.SecKeychainLock(kref))
}

// Delete the Keychain
func (kc *Keychain) Delete() error {
	return os.Remove(kc.path)
}

// Convert Keychain to CFTypeRef.
// The returned CFTypeRef, if non-nil, must be released via CFRelease.
func (kc Keychain) Convert() (C.CFTypeRef, error) {
	keyRef, err := openKeychainRef(kc.path)
	return C.CFTypeRef(keyRef), err
}

type keychainArray []Keychain

// Convert the keychainArray to a CFTypeRef.
// The returned CFTypeRef, if non-nil, must be released via CFRelease.
func (ka keychainArray) Convert() (C.CFTypeRef, error) {
	var refs = make([]C.CFTypeRef, len(ka))
	var err error

	for idx, kc := range ka {
		if refs[idx], err = kc.Convert(); err != nil {
			// If we error trying to convert lets release any we converted before
			for _, ref := range refs {
				if ref != 0 {
					Release(ref)
				}
			}
			return 0, err
		}
	}

	return C.CFTypeRef(ArrayToCFArray(refs)), nil
}

// SetMatchSearchList sets match type on keychains
func (k *Item) SetMatchSearchList(karr ...Keychain) {
	k.attr[MatchSearchListKey] = keychainArray(karr)
}

// UseKeychain tells item to use the specified Keychain
func (k *Item) UseKeychain(kc Keychain) {
	k.attr[KeychainKey] = kc
}
