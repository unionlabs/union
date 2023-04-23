# Go Keychain

[![Travis CI](https://travis-ci.org/keybase/go-keychain.svg?branch=master)](https://travis-ci.org/keybase/go-keychain)

A library for accessing the Keychain for macOS, iOS, and Linux in Go (golang).

Requires macOS 10.9 or greater and iOS 8 or greater. On Linux, communicates to
a provider of the DBUS SecretService spec like gnome-keyring or ksecretservice.

```go
import "github.com/keybase/go-keychain"
```


## Mac/iOS Usage

The API is meant to mirror the macOS/iOS Keychain API and is not necessarily idiomatic go.

#### Add Item

```go
item := keychain.NewItem()
item.SetSecClass(keychain.SecClassGenericPassword)
item.SetService("MyService")
item.SetAccount("gabriel")
item.SetLabel("A label")
item.SetAccessGroup("A123456789.group.com.mycorp")
item.SetData([]byte("toomanysecrets"))
item.SetSynchronizable(keychain.SynchronizableNo)
item.SetAccessible(keychain.AccessibleWhenUnlocked)
err := keychain.AddItem(item)

if err == keychain.ErrorDuplicateItem {
  // Duplicate
}
```

#### Query Item

Query for multiple results, returning attributes:

```go
query := keychain.NewItem()
query.SetSecClass(keychain.SecClassGenericPassword)
query.SetService(service)
query.SetAccount(account)
query.SetAccessGroup(accessGroup)
query.SetMatchLimit(keychain.MatchLimitAll)
query.SetReturnAttributes(true)
results, err := keychain.QueryItem(query)
if err != nil {
  // Error
} else {
  for _, r := range results {
    fmt.Printf("%#v\n", r)
  }
}
```

Query for a single result, returning data:

```go
query := keychain.NewItem()
query.SetSecClass(keychain.SecClassGenericPassword)
query.SetService(service)
query.SetAccount(account)
query.SetAccessGroup(accessGroup)
query.SetMatchLimit(keychain.MatchLimitOne)
query.SetReturnData(true)
results, err := keychain.QueryItem(query)
if err != nil {
  // Error
} else if len(results) != 1 {
  // Not found
} else {
  password := string(results[0].Data)
}
```

#### Delete Item

Delete a generic password item with service and account:

```go
item := keychain.NewItem()
item.SetSecClass(keychain.SecClassGenericPassword)
item.SetService(service)
item.SetAccount(account)
err := keychain.DeleteItem(item)
```

### Other

There are some convenience methods for generic password:

```go
// Create generic password item with service, account, label, password, access group
item := keychain.NewGenericPassword("MyService", "gabriel", "A label", []byte("toomanysecrets"), "A123456789.group.com.mycorp")
item.SetSynchronizable(keychain.SynchronizableNo)
item.SetAccessible(keychain.AccessibleWhenUnlocked)
err := keychain.AddItem(item)
if err == keychain.ErrorDuplicateItem {
  // Duplicate
}

accounts, err := keychain.GetGenericPasswordAccounts("MyService")
// Should have 1 account == "gabriel"

err := keychain.DeleteGenericPasswordItem("MyService", "gabriel")
if err == keychain.ErrorNotFound {
  // Not found
}
```

### OS X

Creating a new keychain and add an item to it:

```go

// Add a new key chain into ~/Application Support/Keychains, with the provided password
k, err := keychain.NewKeychain("mykeychain.keychain", "my keychain password")
if err != nil {
  // Error creating
}

// Create generic password item with service, account, label, password, access group
item := keychain.NewGenericPassword("MyService", "gabriel", "A label", []byte("toomanysecrets"), "A123456789.group.com.mycorp")
item.UseKeychain(k)
err := keychain.AddItem(item)
if err != nil {
  // Error creating
}
```

Using a Keychain at path:

```go
k, err := keychain.NewWithPath("mykeychain.keychain")
```

Set a trusted applications for item (OS X only):

```go
item := keychain.NewGenericPassword("MyService", "gabriel", "A label", []byte("toomanysecrets"), "A123456789.group.com.mycorp")
trustedApplications := []string{"/Applications/Mail.app"}
item.SetAccess(&keychain.Access{Label: "Mail", TrustedApplications: trustedApplications})
err := keychain.AddItem(item)
```

## iOS

Bindable package in `bind`. iOS project in `ios`. Run that project to test iOS.

To re-generate framework:

```
(cd bind && gomobile bind -target=ios -tags=ios -o ../ios/bind.framework)
```
