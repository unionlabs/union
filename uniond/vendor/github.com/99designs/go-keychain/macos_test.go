// +build darwin,!ios

package keychain

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"
	"time"
)

func TestAccess(t *testing.T) {
	var err error

	service, account, label, accessGroup, password := "TestAccess", "test2", "A label", "", "toomanysecrets2"
	item := NewGenericPassword(service, account, label, []byte(password), accessGroup)
	defer func() { _ = DeleteItem(item) }()

	trustedApplications := []string{"/Applications/Mail.app"}
	item.SetAccess(&Access{Label: "Mail", TrustedApplications: trustedApplications})
	err = AddItem(item)
	if err != nil {
		t.Fatal(err)
	}

	_, err = GetGenericPassword(service, account, label, accessGroup)
	if err != nil {
		t.Fatal(err)
	}
}

func TestAccessWithImpliedSelf(t *testing.T) {
	var err error

	service, account, label, accessGroup, password := "TestAccess", "test2", "A label", "", "toomanysecrets2"
	item := NewGenericPassword(service, account, label, []byte(password), accessGroup)
	defer func() { _ = DeleteItem(item) }()

	item.SetAccess(&Access{Label: "Self", TrustedApplications: nil})
	err = AddItem(item)
	if err != nil {
		t.Fatal(err)
	}

	_, err = GetGenericPassword(service, account, label, accessGroup)
	if err != nil {
		t.Fatal(err)
	}
}

func TestAccessWithoutTrust(t *testing.T) {
	var err error

	item := NewGenericPassword("TestAccess", "test2", "A label", []byte("toomanysecrets2"), "")
	defer func() { _ = DeleteItem(item) }()

	trustedApplications := []string{}
	item.SetAccess(&Access{Label: "No Trust", TrustedApplications: trustedApplications})
	err = AddItem(item)
	if err != nil {
		t.Fatal(err)
	}
}

func TestUpdateItem(t *testing.T) {
	var err error

	item := NewGenericPassword("TestAccess", "firsttest", "TestUpdateItem", []byte("toomanysecrets2"), "")
	defer func() { _ = DeleteItem(item) }()
	err = AddItem(item)
	if err != nil {
		t.Fatal(err)
	}

	data1, err := GetGenericPassword("TestAccess", "firsttest", "TestUpdateItem", "")
	if err != nil {
		t.Fatal(err)
	}
	if string(data1) != "toomanysecrets2" {
		t.Fatal("TestUpdateItem: new password does not match")
	}

	updateItem := NewItem()
	updateItem.SetSecClass(SecClassGenericPassword)
	updateItem.SetService("TestAccess")
	updateItem.SetAccount("firsttest")
	updateItem.SetLabel("TestUpdateItem")
	updateItem.SetData([]byte("toomanysecrets3"))
	err = UpdateItem(item, updateItem)
	if err != nil {
		t.Fatal(err)
	}

	data2, err := GetGenericPassword("TestAccess", "firsttest", "TestUpdateItem", "")
	if err != nil {
		t.Fatal(err)
	}
	if string(data2) != "toomanysecrets3" {
		t.Fatal("TestUpdateItem: updated password does not match")
	}
}

func TestAddingAndQueryingNewKeychain(t *testing.T) {
	keychainPath := tempPath(t)
	defer func() { _ = os.Remove(keychainPath) }()

	service, account, label, accessGroup, password := "TestAddingAndQueryingNewKeychain", "test", "", "", "toomanysecrets"

	k, err := NewKeychain(keychainPath, "my password")
	if err != nil {
		t.Fatal(err)
	}

	item := NewGenericPassword(service, account, label, []byte(password), accessGroup)
	item.UseKeychain(k)
	if err = AddItem(item); err != nil {
		t.Fatal(err)
	}

	query := NewItem()
	query.SetSecClass(SecClassGenericPassword)
	query.SetMatchSearchList(k)
	query.SetService(service)
	query.SetAccount(account)
	query.SetLabel(label)
	query.SetAccessGroup(accessGroup)
	query.SetMatchLimit(MatchLimitOne)
	query.SetReturnData(true)

	results, err := QueryItem(query)
	if err != nil {
		t.Fatal(err)
	}

	if len(results) != 1 {
		t.Fatalf("Expected 1 result, got %d", len(results))
	} else if string(results[0].Data) != password {
		t.Fatalf("Expected password to be %s, got %s", password, results[0].Data)
	}

	// Search default keychain to make sure it's not there
	queryDefault := NewItem()
	queryDefault.SetSecClass(SecClassGenericPassword)
	queryDefault.SetService(service)
	queryDefault.SetMatchLimit(MatchLimitOne)
	queryDefault.SetReturnData(true)
	resultsDefault, err := QueryItem(queryDefault)
	if err != nil {
		t.Fatal(err)
	}
	if len(resultsDefault) != 0 {
		t.Fatalf("Expected no results")
	}
}

func tempPath(t *testing.T) string {
	temp, err := RandomID("go-keychain-test-")
	if err != nil {
		t.Fatal(err)
	}
	return filepath.Join(os.TempDir(), temp+".keychain")
}

func TestNewWithPath(t *testing.T) {
	path := tempPath(t)
	defer func() { _ = os.Remove(path) }()
	kc, newErr := NewKeychain(path, "testkeychainpassword")
	if newErr != nil {
		t.Fatal(newErr)
	}

	item := NewGenericPassword("MyService2", "gabriel2", "", []byte("toomanysecrets2"), "")
	item.UseKeychain(kc)
	if err := AddItem(item); err != nil {
		t.Fatal(err)
	}

	kc2 := NewWithPath(path)

	if lockErr := LockAtPath(path); lockErr != nil {
		t.Fatal(lockErr)
	}

	if unlockErr := UnlockAtPath(path, "testkeychainpassword"); unlockErr != nil {
		t.Fatal(unlockErr)
	}

	query := NewItem()
	query.SetMatchSearchList(kc2)
	query.SetService("MyService2")
	query.SetAccount("gabriel2")
	query.SetSecClass(SecClassGenericPassword)
	query.SetMatchLimit(MatchLimitOne)
	query.SetReturnAttributes(true)
	query.SetReturnData(true)
	results, err := QueryItem(query)
	if err != nil {
		t.Fatal(err)
	}
	if len(results) != 1 {
		t.Fatalf("Should have 1 result, had %d", len(results))
	}
	if string(results[0].Data) != "toomanysecrets2" {
		t.Fatalf("Invalid password: %s", results[0].Data)
	}
	if results[0].CreationDate.Equal(time.Time{}) {
		t.Fatal("Got null creation date")
	}
	if results[0].ModificationDate.Equal(time.Time{}) {
		t.Fatal("Got null creation date")
	}
}

func TestStatus(t *testing.T) {
	path := tempPath(t)
	defer func() { _ = os.Remove(path) }()
	k, newErr := NewKeychain(path, "testkeychainpassword")
	if newErr != nil {
		t.Fatal(newErr)
	}

	if err := k.Status(); err != nil {
		t.Fatal(err)
	}

	nonexistent := NewWithPath(fmt.Sprintf("this_shouldnt_exist_%s", time.Now()))
	if err := nonexistent.Status(); err != ErrorNoSuchKeychain {
		t.Fatalf("Expected %v, get %v", ErrorNoSuchKeychain, err)
	}
}

func TestGenericPasswordRef(t *testing.T) {
	service, account, label, accessGroup, password := "TestGenericPasswordRef", "test", "", "", "toomanysecrets"

	item := NewGenericPassword(service, account, label, []byte(password), accessGroup)
	defer func() { _ = DeleteItem(item) }()
	err := AddItem(item)
	if err != nil {
		t.Fatal(err)
	}

	// Query reference and delete by reference
	query := NewItem()
	query.SetSecClass(SecClassGenericPassword)
	query.SetService(service)
	query.SetAccount(account)
	query.SetMatchLimit(MatchLimitOne)
	query.SetReturnRef(true)
	ref, err := QueryItemRef(query)
	if err != nil {
		t.Fatal(err)
	} else if ref == 0 {
		t.Fatal("Missing result")
	} else {
		err = DeleteItemRef(ref)
		if err != nil {
			t.Fatal(err)
		}
		Release(ref)
	}

	passwordAfter, err := GetGenericPassword(service, account, label, accessGroup)
	if err != nil {
		t.Fatal(err)
	}
	if passwordAfter != nil {
		t.Fatal("Shouldn't have password")
	}
}

func TestInternetPassword(t *testing.T) {
	query := NewItem()
	query.SetSecClass(SecClassInternetPassword)
	query.SetLabel("github.com")
	query.SetMatchLimit(MatchLimitOne)
	query.SetReturnAttributes(true)
	results, err := QueryItem(query)
	if err != nil {
		// Error
		t.Errorf("Query Error: %v", err)
	} else {
		for _, r := range results {
			fmt.Printf("%#v\n", r.Account)
		}
	}
}
