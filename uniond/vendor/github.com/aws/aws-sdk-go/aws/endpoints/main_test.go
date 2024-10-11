package endpoints

import (
	"fmt"
	"os"
	"path/filepath"
	"testing"
)

// Update all endpoints partition variables data to be the static testdata
// model instead of the dynamic live model. This ensures that endpoint tests
// are tested against static data, and will not break when the live
// endpoints.json model is updated.

func TestMain(m *testing.M) {
	testdataFilename := filepath.Join("testdata", "endpoints.json")

	testdataFile, err := os.Open(testdataFilename)
	if err != nil {
		panic(fmt.Sprintf("failed to open test endpoints model, %v", err))
	}

	resolver, err := DecodeModel(testdataFile)
	if err != nil {
		panic(fmt.Sprintf("failed to decode test endpoints model, %v", err))
	}

	partitions, ok := resolver.(partitions)
	if !ok {
		panic(fmt.Sprintf("expect %T resolver, got %T", partitions, resolver))
	}

	for _, p := range partitions {
		switch p.ID {
		case "aws":
			awsPartition = p
		case "aws-cn":
			awscnPartition = p
		case "aws-us-gov":
			awsusgovPartition = p
		case "aws-iso":
			awsisoPartition = p
		case "aws-iso-b":
			awsisobPartition = p
		default:
			panic("unknown endpoints partition " + p.ID)
		}
	}
}
