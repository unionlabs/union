package main

import (
	"testing"

)

func TestTokens(t *testing.T) {
	testTag := "`protobuf:\"bytes,2004,rep,name=historical_roots,json=historicalRoots,proto3\" json:\"historical_roots,omitempty\" ssz-max:\"16777216\" ssz-size:\"?,32\"`"
	tags, err := GetSSZTags(testTag)
	if err != nil {
		t.Errorf("Unexpected error from GetSSZTags=%s", err)
	}
	sszSize, ok := tags["ssz-size"]
	if !ok {
		t.Errorf("ssz-size tag not present")
	}
	expectedSize := "?,32"
	if sszSize != expectedSize {
		t.Errorf("expected ssz-size value '%s', got '%s'", expectedSize, sszSize)
	}
	sszMax, ok := tags["ssz-max"]
	if !ok {
		t.Errorf("ssz-max tag not present")
	}
	expectedMax := "16777216"
	if sszMax != expectedMax {
		t.Errorf("expected ssz-max value '%s', got '%s'", expectedMax, sszMax)
	}
}

func TestFullTag(t *testing.T) {
	tag := "`protobuf:\"bytes,1002,opt,name=genesis_validators_root,json=genesisValidatorsRoot,proto3\" json:\"genesis_validators_root,omitempty\" ssz-size:\"32\"`"
	_, err := extractSSZDimensions(tag)
	if err != nil {
		t.Errorf("Unexpected error calling extractSSZDimensions: %v", err)
	}
}

func TestListOfVector(t *testing.T) {
	tag := "`protobuf:\"bytes,2004,rep,name=historical_roots,json=historicalRoots,proto3\" json:\"historical_roots,omitempty\" ssz-max:\"16777216\" ssz-size:\"?,32\"`"
	_, err := extractSSZDimensions(tag)
	if err != nil {
		t.Errorf("Unexpected error calling extractSSZDimensions: %v", err)
	}
}

func TestWildcardSSZSize(t *testing.T)  {
	tag := "`ssz-max:\"16777216\" ssz-size:\"?,32\"`"
	dims, err := extractSSZDimensions(tag)
	if err != nil {
		t.Errorf("Unexpected error calling extractSSZDimensions: %v", err)
	}
	expectedDims := 2
	if len(dims) != expectedDims {
		t.Errorf("expected %d dimensions from ssz tags, got %d", expectedDims, len(dims))
	}
	if !dims[0].IsList() {
		t.Errorf("Expected the first dimension to be a list")
	}
	if dims[0].IsVector() {
		t.Errorf("Expected the first dimension to not be a vector")
	}
	if dims[0].ListLen() != 16777216 {
		t.Errorf("Expected max size of list to be %d, got %d", 16777216, dims[0].ListLen())
	}
	if !dims[1].IsVector() {
		t.Errorf("Expected the first dimension to be a vector")
	}
	if dims[1].IsList() {
		t.Errorf("Expected the second dimension to not be a list")
	}
	if dims[1].VectorLen() != 32 {
		t.Errorf("Expected size of vector to be %d, got %d", 32, dims[1].VectorLen())
	}
}

func TestListOfList(t *testing.T) {
	tag := "`protobuf:\"bytes,14,rep,name=transactions,proto3\" json:\"transactions,omitempty\" ssz-max:\"1048576,1073741824\" ssz-size:\"?,?\"`"
	dims, err := extractSSZDimensions(tag)
	if err != nil {
		t.Errorf("Unexpected error calling extractSSZDimensions: %v", err)
	}
	expectedDims := 2
	if len(dims) != expectedDims {
		t.Errorf("expected %d dimensions from ssz tags, got %d", expectedDims, len(dims))
	}
	if !dims[0].IsList() {
		t.Errorf("Expected both dimensions to be lists, but the first dimension is not")
	}
	if !dims[1].IsList() {
		t.Errorf("Expected both dimensions to be lists, but the second dimension is not")
	}
	if dims[0].IsVector() {
		t.Errorf("Expected neither dimension to be vector, but the first dimension is")
	}
	if dims[1].IsVector() {
		t.Errorf("Expected neither dimension to be vector, but the second dimension is")
	}
	if dims[0].ListLen() != 1048576 {
		t.Errorf("Expected ssz-max of first dimension to be %d, got %d", 1048576 , dims[0].ListLen())
	}
	if dims[1].ListLen() != 1073741824 {
		t.Errorf("Expected ssz-max of first dimension to be %d, got %d", 1073741824, dims[1].ListLen())
	}
}

func TestOneDVector(t *testing.T) {
	tag := "`protobuf:\"bytes,1,opt,name=randao_reveal,json=randaoReveal,proto3\" json:\"randao_reveal,omitempty\" ssz-size:\"96\""
	dims, err := extractSSZDimensions(tag)
	if err != nil {
		t.Errorf("Unexpected error calling extractSSZDimensions: %v", err)
	}
	expectedDims := 1
	if len(dims) != expectedDims {
		t.Errorf("expected %d dimensions from ssz tags, got %d", expectedDims, len(dims))
	}
}

func TestOneDList(t *testing.T) {
	tag := "`protobuf:\"bytes,4,rep,name=proposer_slashings,json=proposerSlashings,proto3\" json:\"proposer_slashings,omitempty\" ssz-max:\"16\"`"
	dims, err := extractSSZDimensions(tag)
	if err != nil {
		t.Errorf("Unexpected error calling extractSSZDimensions: %v", err)
	}
	expectedDims := 1
	if len(dims) != expectedDims {
		t.Errorf("expected %d dimensions from ssz tags, got %d", expectedDims, len(dims))
	}
}

func TestNoDims(t *testing.T) {
	tag := "`protobuf:\"bytes,2,opt,name=eth1_data,json=eth1Data,proto3\" json:\"eth1_data,omitempty\"`"
	dims, err := extractSSZDimensions(tag)
	if err == nil {
		t.Errorf("expected error when calling extractSSZDimensions without ssz-size or ssz-max: %v", err)
	}
	expectedDims := 0
	if len(dims) != expectedDims {
		t.Errorf("expected %d dimensions from ssz tags, got %d", expectedDims, len(dims))
	}
}

func TestBitlist(t *testing.T) {
	tag := "`json:\"aggregation_bits\" ssz:\"bitlist\" ssz-max:\"2048\"`"
	dims, err := extractSSZDimensions(tag)
	if err != nil {
		t.Errorf("Unexpected error calling extractSSZDimensions: %v", err)
	}
	expectedDims := 1
	if len(dims) != expectedDims {
		t.Errorf("expected %d dimensions from ssz tags, got %d", expectedDims, len(dims))
	}
	if !dims[0].IsBitlist() {
		t.Error("Expected tag 'ssz:\"bitlist\" to mark field as a bitlist")
	}
}