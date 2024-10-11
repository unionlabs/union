package util

import (
	"runtime/debug"
	"testing"

	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"

	_ "cosmossdk.io/client/v2/internal/testpb"
)

func TestIsSupportedVersion(t *testing.T) {
	mockBuildInfo := &debug.BuildInfo{
		Deps: []*debug.Module{
			{
				Path:    "github.com/cosmos/cosmos-sdk",
				Version: "v0.50.0",
			},
			{
				Path:    "cosmossdk.io/feegrant",
				Version: "v0.1.0",
			},
		},
	}

	cases := []struct {
		messageName string
		expected    bool
	}{
		{
			messageName: "testpb.Msg.Send",
			expected:    true,
		},
		{
			messageName: "testpb.Query.Echo",
			expected:    true,
		},
		{
			messageName: "testpb.Msg.Clawback",
			expected:    false,
		},
	}

	for _, tc := range cases {
		t.Run(tc.messageName, func(t *testing.T) {
			desc, err := protoregistry.GlobalFiles.FindDescriptorByName(protoreflect.FullName(tc.messageName))
			if err != nil {
				t.Fatal(err)
			}

			methodDesc := desc.(protoreflect.MethodDescriptor)
			isSupported := isSupportedVersion(methodDesc, mockBuildInfo)
			if isSupported != tc.expected {
				t.Errorf("expected %v, got %v for %s", tc.expected, isSupported, methodDesc.FullName())
			}
		})
	}
}

func TestParseVersion(t *testing.T) {
	cases := []struct {
		input              string
		expectedModuleName string
		expectedVersion    string
	}{
		{
			input:              "",
			expectedModuleName: "",
			expectedVersion:    "",
		},
		{
			input:              "Cosmos SDK 0.50",
			expectedModuleName: "cosmos-sdk",
			expectedVersion:    "v0.50",
		},
		{
			input:              "cosmos sdk 0.50",
			expectedModuleName: "cosmos-sdk",
			expectedVersion:    "v0.50",
		},
		{
			input:              "Cosmos-SDK 0.50",
			expectedModuleName: "cosmos-sdk",
			expectedVersion:    "v0.50",
		},
		{
			input:              "cosmos-sdk v0.50",
			expectedModuleName: "cosmos-sdk",
			expectedVersion:    "v0.50",
		},
		{
			input:              "cosmos-sdk v0.50.1",
			expectedModuleName: "cosmos-sdk",
			expectedVersion:    "v0.50.1",
		},
		{
			input:              "cosmos-sdk 0.47.0-veronica",
			expectedModuleName: "cosmos-sdk",
			expectedVersion:    "v0.47.0-veronica",
		},
		{
			input:              "x/feegrant v0.1.0",
			expectedModuleName: "feegrant",
			expectedVersion:    "v0.1.0",
		},
		{
			input:              "x/feegrant 0.1",
			expectedModuleName: "feegrant",
			expectedVersion:    "v0.1",
		},
	}

	for _, tc := range cases {
		t.Run(tc.input, func(t *testing.T) {
			moduleName, version := parseVersion(tc.input)
			if moduleName != tc.expectedModuleName {
				t.Errorf("expected module name %s, got %s", tc.expectedModuleName, moduleName)
			}
			if version != tc.expectedVersion {
				t.Errorf("expected version %s, got %s", tc.expectedVersion, version)
			}
		})
	}
}
