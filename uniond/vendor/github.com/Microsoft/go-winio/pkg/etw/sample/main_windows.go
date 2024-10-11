//go:build windows
// +build windows

// Shows a sample usage of the ETW logging package.
package main

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"runtime"

	"github.com/Microsoft/go-winio/pkg/etw"
	"github.com/Microsoft/go-winio/pkg/guid"
)

func callback(sourceID guid.GUID, state etw.ProviderState, level etw.Level, matchAnyKeyword uint64, matchAllKeyword uint64, filterData uintptr) {
	fmt.Printf("Callback: isEnabled=%d, level=%d, matchAnyKeyword=%d\n", state, level, matchAnyKeyword)
}

func main() {
	fmt.Printf("Running on %s/%s\n", runtime.GOOS, runtime.GOARCH)

	group, err := guid.FromString("12341234-abcd-abcd-abcd-123412341234")
	if err != nil {
		log.Fatal(err)
	}

	provider, err := etw.NewProvider("TestProvider", callback)

	if err != nil {
		log.Fatal(err)
	}
	defer func() {
		if err := provider.Close(); err != nil {
			log.Fatal(err)
		}
	}()

	providerWithGroup, err := etw.NewProviderWithOptions("TestProviderWithGroup", etw.WithGroup(group), etw.WithCallback(callback))

	if err != nil {
		log.Fatal(err)
	}
	defer func() {
		if err := providerWithGroup.Close(); err != nil {
			log.Fatal(err)
		}
	}()

	fmt.Printf("Provider ID: %s\n", provider)
	fmt.Printf("Provider w/ Group ID: %s\n", providerWithGroup)

	reader := bufio.NewReader(os.Stdin)

	fmt.Println("Press enter to log events")
	reader.ReadString('\n')

	if err := provider.WriteEvent(
		"TestEvent",
		etw.WithEventOpts(
			etw.WithLevel(etw.LevelInfo),
			etw.WithKeyword(0x140),
		),
		etw.WithFields(
			etw.StringField("TestField", "Foo"),
			etw.StringField("TestField2", "Bar"),
			etw.Struct("TestStruct",
				etw.StringField("Field1", "Value1"),
				etw.StringField("Field2", "Value2")),
			etw.StringArray("TestArray", []string{
				"Item1",
				"Item2",
				"Item3",
				"Item4",
				"Item5",
			})),
	); err != nil {
		log.Fatal(err)
	}

	if err := providerWithGroup.WriteEvent(
		"TestEventWithGroup",
		etw.WithEventOpts(
			etw.WithLevel(etw.LevelInfo),
			etw.WithKeyword(0x140),
		),
		etw.WithFields(
			etw.StringField("TestField", "Foo"),
			etw.StringField("TestField2", "Bar"),
			etw.Struct("TestStruct",
				etw.StringField("Field1", "Value1"),
				etw.StringField("Field2", "Value2")),
			etw.StringArray("TestArray", []string{
				"Item1",
				"Item2",
				"Item3",
				"Item4",
				"Item5",
			})),
	); err != nil {
		log.Fatal(err)
	}
}
