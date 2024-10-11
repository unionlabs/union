package wincred

import "fmt"

func ExampleList() {
	if creds, err := List(); err == nil {
		for _, cred := range creds {
			fmt.Println(cred.TargetName)
		}
	}
}

func ExampleFilteredList() {
	if creds, err := FilteredList("my*"); err == nil {
		for _, cred := range creds {
			fmt.Println(cred.TargetName)
		}
	}
}

func ExampleGetGenericCredential() {
	if cred, err := GetGenericCredential("myGoApplication"); err == nil {
		fmt.Println(cred.TargetName, string(cred.CredentialBlob))
	}
}

func ExampleGenericCredential_Delete() {
	cred, _ := GetGenericCredential("myGoApplication")
	if err := cred.Delete(); err == nil {
		fmt.Println("Deleted")
	}
}

func ExampleGenericCredential_Write() {
	cred := NewGenericCredential("myGoApplication")
	cred.CredentialBlob = []byte("my secret")
	if err := cred.Write(); err == nil {
		fmt.Println("Created")
	}
}
