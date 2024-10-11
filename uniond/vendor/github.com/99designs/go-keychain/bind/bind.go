// +build darwin ios

package bind

import (
	"fmt"
	"reflect"

	"github.com/keybase/go-keychain"
)

// Test is a bind interface for the test
type Test interface {
	Fail(s string)
}

// AddGenericPassword adds generic password
func AddGenericPassword(service string, account string, label string, password string, accessGroup string) error {
	item := keychain.NewGenericPassword(service, account, label, []byte(password), accessGroup)
	return keychain.AddItem(item)
}

// DeleteGenericPassword deletes generic password
func DeleteGenericPassword(service string, account string, accessGroup string) error {
	item := keychain.NewItem()
	item.SetSecClass(keychain.SecClassGenericPassword)
	item.SetService(service)
	item.SetAccount(account)
	item.SetAccessGroup(accessGroup)
	return keychain.DeleteItem(item)
}

// GenericPasswordTest runs test code for generic password keychain item.
// This is here so we can export using gomobile bind and run this method on iOS simulator and device.
// Access groups aren't supported in iOS simulator.
func GenericPasswordTest(t Test, service string, accessGroup string) {
	var err error

	account := "Testing account with unicode テスト"
	item := keychain.NewGenericPassword(service, account, "", []byte("toomanysecrets"), accessGroup)
	item.SetSynchronizable(keychain.SynchronizableNo)
	item.SetAccessible(keychain.AccessibleWhenUnlocked)

	account2 := "Testing account #2"
	item2 := keychain.NewGenericPassword(service, account2, "", []byte("toomanysecrets2"), accessGroup)

	// Cleanup
	defer func() { _ = keychain.DeleteItem(item) }()
	defer func() { _ = keychain.DeleteItem(item2) }()

	// Test account names empty
	accounts, err := keychain.GetGenericPasswordAccounts(service)
	if err != nil {
		t.Fail(err.Error())
	}
	if len(accounts) != 0 {
		t.Fail("Should have no accounts")
	}

	// Test add
	err = keychain.AddItem(item)
	if err != nil {
		t.Fail(err.Error())
	}

	// Test dupe
	err = keychain.AddItem(item)
	if err != keychain.ErrorDuplicateItem {
		t.Fail("Should error with duplicate item")
	}

	// Test add another
	err = keychain.AddItem(item2)
	if err != nil {
		t.Fail(err.Error())
	}

	// Test querying attributes
	query := keychain.NewItem()
	query.SetSecClass(keychain.SecClassGenericPassword)
	query.SetService(service)
	query.SetAccount(account)
	query.SetAccessGroup(accessGroup)
	query.SetMatchLimit(keychain.MatchLimitAll)
	query.SetReturnAttributes(true)
	results, err := keychain.QueryItem(query)
	if err != nil {
		t.Fail(err.Error())
	}

	if len(results) != 1 {
		t.Fail(fmt.Sprintf("Invalid results count: %d", len(results)))
	}

	if results[0].Service != service {
		t.Fail(fmt.Sprintf("Invalid service, %v != %v, %v", results[0].Service, service, results))
	}

	if results[0].Account != account {
		t.Fail(fmt.Sprintf("Invalid account, %v != %v, %v", results[0].Account, account, results))
	}

	if len(results[0].Data) != 0 {
		t.Fail("Password shouldn't come back when returning attributes")
	}

	// Test querying data
	queryData := keychain.NewItem()
	queryData.SetSecClass(keychain.SecClassGenericPassword)
	queryData.SetService(service)
	queryData.SetAccount(account)
	queryData.SetAccessGroup(accessGroup)
	queryData.SetMatchLimit(keychain.MatchLimitOne)
	queryData.SetReturnData(true)
	resultsData, err := keychain.QueryItem(queryData)
	if err != nil {
		t.Fail(err.Error())
	}

	if len(resultsData) != 1 {
		t.Fail("Too many results")
	}

	if string(resultsData[0].Data) != "toomanysecrets" {
		t.Fail("Invalid password")
	}

	// Test account names
	accounts2, err := keychain.GetGenericPasswordAccounts(service)
	if err != nil {
		t.Fail(err.Error())
	}
	if len(accounts2) != 2 {
		t.Fail(fmt.Sprintf("Should have 2 accounts: %v", accounts2))
	}

	if !reflect.DeepEqual(accounts2, []string{account, account2}) {
		t.Fail(fmt.Sprintf("Invalid accounts: %v", accounts2))
	}

	// Remove
	queryDel := keychain.NewItem()
	queryDel.SetSecClass(keychain.SecClassGenericPassword)
	queryDel.SetService(service)
	queryDel.SetAccount(account)
	queryDel.SetAccessGroup(accessGroup)
	err = keychain.DeleteItem(queryDel)
	if err != nil {
		t.Fail(err.Error())
	}

	// Test removed
	query3 := keychain.NewItem()
	query3.SetSecClass(keychain.SecClassGenericPassword)
	query3.SetService(service)
	query3.SetAccount(account)
	query3.SetAccessGroup(accessGroup)
	query3.SetMatchLimit(keychain.MatchLimitAll)
	query3.SetReturnAttributes(true)
	results3, err := keychain.QueryItem(query3)
	if err != nil {
		t.Fail(err.Error())
	}

	if len(results3) != 0 {
		t.Fail("Results should have been empty")
	}

	accounts3, err := keychain.GetGenericPasswordAccounts(service)
	if err != nil {
		t.Fail(err.Error())
	}
	if len(accounts3) != 1 {
		t.Fail("Should have an account")
	}

	// Test remove not found
	err = keychain.DeleteItem(item)
	if err != keychain.ErrorItemNotFound {
		t.Fail("Error should be not found")
	}
}
