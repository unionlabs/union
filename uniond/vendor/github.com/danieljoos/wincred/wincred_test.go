//go:build windows
// +build windows

package wincred

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
)

const (
	testTargetName              = "github.com/danieljoos/wincred/testing"
	testTargetNameMissing       = "github.com/danieljoos/wincred/missing"
	testListFilter              = "github.com/danieljoos*"
	testDomainTargetName        = "emea.acme-corp.net"
	testDomainTargetNameMissing = "unknown-corp.net"
)

func TestGenericCredential_EndToEnd(t *testing.T) {
	// 1. Create new credential `foo`
	cred := NewGenericCredential(testTargetName)
	cred.CredentialBlob = []byte("my secret")
	cred.Persist = PersistSession
	err := cred.Write()
	assert.Nil(t, err)

	// 2. Get the credential from the store
	cred, err = GetGenericCredential(testTargetName)
	assert.Nil(t, err)
	assert.NotNil(t, cred)
	assert.Equal(t, "my secret", string(cred.CredentialBlob))

	// 3. Search it in the list
	creds, err := List()
	assert.Nil(t, err)
	assert.NotNil(t, creds)
	assert.NotEqual(t, 0, len(creds))
	found := false
	for i := range creds {
		found = found || creds[i].TargetName == testTargetName
	}
	assert.True(t, found)

	// 4. Search it also in a filtered list
	creds, err = FilteredList(testListFilter)
	assert.Nil(t, err)
	assert.NotNil(t, creds)
	assert.NotEqual(t, 0, len(creds))
	found = false
	for i := range creds {
		found = found || creds[i].TargetName == testTargetName
	}
	assert.True(t, found)

	// 5. Delete it
	err = cred.Delete()
	assert.Nil(t, err)

	// 6. Search it again in the complete list. It should be gone.
	creds, err = List()
	assert.Nil(t, err)
	assert.NotNil(t, creds)
	found = false
	for i := range creds {
		found = found || creds[i].TargetName == testTargetName
	}
	assert.False(t, found)
}

func TestGetGenericCredential_NotFound(t *testing.T) {
	cred, err := GetGenericCredential(testTargetNameMissing)
	assert.Nil(t, cred)
	assert.NotNil(t, err)
	assert.True(t, errors.Is(err, ErrElementNotFound))
}

func TestGetGenericCredential_Empty(t *testing.T) {
	cred, err := GetGenericCredential("")
	assert.Nil(t, cred)
	assert.NotNil(t, err)
	assert.True(t, errors.Is(err, ErrInvalidParameter))
}

func TestGenericCredential_WriteEmpty(t *testing.T) {
	cred := NewGenericCredential("")
	err := cred.Write()
	assert.NotNil(t, err)
	assert.True(t, errors.Is(err, ErrInvalidParameter))
}

func TestGenericCredential_DeleteNotFound(t *testing.T) {
	cred := NewGenericCredential(testTargetNameMissing)
	err := cred.Delete()
	assert.NotNil(t, err)
	assert.True(t, errors.Is(err, ErrElementNotFound))
}

func TestDomainPassword_EndToEnd(t *testing.T) {
	// 1. Create new domain password credentials
	cred := NewDomainPassword(testDomainTargetName)
	cred.UserName = "johndoe"
	cred.SetPassword("s3cr3t!")
	err := cred.Write()
	assert.Nil(t, err)

	// 2. Get the new credentials from the store.
	// For domain passwords, we're not allowed to fetch the actual password data.
	cred, err = GetDomainPassword(testDomainTargetName)
	assert.NotNil(t, cred)
	assert.Nil(t, err)
	assert.Equal(t, cred.UserName, "johndoe")

	// 3. Search it in the list
	creds, err := List()
	assert.Nil(t, err)
	assert.NotNil(t, creds)
	assert.NotEqual(t, 0, len(creds))
	found := false
	for i := range creds {
		found = found || creds[i].TargetName == testDomainTargetName
	}
	assert.True(t, found)

	// 4. Delete it
	err = cred.Delete()
	assert.Nil(t, err)

	// 5. Try to get it again - should be gone now
	cred, err = GetDomainPassword(testDomainTargetName)
	assert.Nil(t, cred)
	assert.NotNil(t, err)
	assert.True(t, errors.Is(err, ErrElementNotFound))
}

func TestGetDomainPassword_NotFound(t *testing.T) {
	cred, err := GetDomainPassword(testDomainTargetNameMissing)
	assert.NotNil(t, err)
	assert.Nil(t, cred)
	assert.True(t, errors.Is(err, ErrElementNotFound))
}

func TestGetDomainPassword_InvalidTargetName(t *testing.T) {
	invalidTargetNames := []string{"", "Invalid\\Domain\\Target"}
	for _, targetName := range invalidTargetNames {
		t.Run(targetName, func(t *testing.T) {
			cred, err := GetDomainPassword(targetName)
			assert.NotNil(t, err)
			assert.Nil(t, cred)
			assert.True(t, errors.Is(err, ErrInvalidParameter))
		})
	}
}

func TestDomainPassword_WriteEmpty(t *testing.T) {
	cred := NewDomainPassword("")
	err := cred.Write()
	assert.NotNil(t, err)
	assert.True(t, errors.Is(err, ErrInvalidParameter))
}

func TestDomainPassword_BadUsername(t *testing.T) {
	cred := NewDomainPassword(testDomainTargetName)
	cred.UserName = ""
	err := cred.Write()
	assert.NotNil(t, err)
	assert.True(t, errors.Is(err, ErrBadUsername))
}

func TestDomainPassword_DeleteNotFound(t *testing.T) {
	cred := NewDomainPassword(testDomainTargetName)
	err := cred.Delete()
	assert.NotNil(t, err)
	assert.True(t, errors.Is(err, ErrElementNotFound))
}

func TestFilteredList_Empty(t *testing.T) {
	list, err := FilteredList(testDomainTargetNameMissing)
	assert.Nil(t, err)
	assert.Empty(t, list)
}
