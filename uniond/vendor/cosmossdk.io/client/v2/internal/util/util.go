package util

import (
	"regexp"
	"runtime/debug"
	"strings"

	cosmos_proto "github.com/cosmos/cosmos-proto"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
	"google.golang.org/protobuf/types/dynamicpb"

	"cosmossdk.io/client/v2/internal/strcase"
)

// get build info to verify later if comment is supported
// this is a hack in because of the global api module package
// later versions unsupported by the current version can be added
var buildInfo, _ = debug.ReadBuildInfo()

// DescriptorKebabName returns the name of the descriptor in kebab case.
func DescriptorKebabName(descriptor protoreflect.Descriptor) string {
	return strcase.ToKebab(string(descriptor.Name()))
}

func ResolveMessageType(resolver protoregistry.MessageTypeResolver, descriptor protoreflect.MessageDescriptor) protoreflect.MessageType {
	typ, err := resolver.FindMessageByName(descriptor.FullName())
	if err == nil {
		return typ
	}

	return dynamicpb.NewMessageType(descriptor)
}

// IsSupportedVersion is used to determine in which version of a module / sdk a rpc was introduced.
// It returns false if the rpc has comment for an higher version than the current one.
// It returns true if the method descriptor contains no annotation.
func IsSupportedVersion(methodDesc protoreflect.MethodDescriptor) bool {
	return isSupportedVersion(methodDesc, buildInfo)
}

// isSupportedVersion is used to determine in which version of a module / sdk a rpc was introduced.
// It returns false if the rpc has comment for an higher version than the current one.
// It returns true if the method descriptor contains no annotation.
// It takes a buildInfo as argument to be able to test it.
func isSupportedVersion(methodDesc protoreflect.MethodDescriptor, buildInfo *debug.BuildInfo) bool {
	hasVersion := proto.HasExtension(methodDesc.Options(), cosmos_proto.E_MethodAddedIn)
	if !hasVersion || buildInfo == nil || len(buildInfo.Deps) == 0 {
		return true
	}

	version := proto.GetExtension(methodDesc.Options(), cosmos_proto.E_MethodAddedIn).(string)
	moduleName, version := parseVersion(version)
	if moduleName == "" || version == "" {
		return true // if no comment consider it's supported
	}

	for _, dep := range buildInfo.Deps {
		if !strings.Contains(dep.Path, moduleName) {
			continue
		}

		return version <= dep.Version
	}

	// if cannot find the module consider it isn't supported
	// for instance the x/gov module wasn't extracted in v0.50
	// so it isn't present in the build info, however, that means
	// it isn't supported in v0.50.
	return false
}

var sinceCommentRegex = regexp.MustCompile(`(\S+) (\S+)`)

// parseVersion parses the `cosmos-sdk v0.xx` comment on rpc.
func parseVersion(input string) (string, string) {
	var (
		moduleName string
		version    string
	)

	input = strings.ToLower(input)
	input = strings.ReplaceAll(input, "cosmos sdk", "cosmos-sdk")

	matches := sinceCommentRegex.FindStringSubmatch(input)
	if len(matches) >= 3 {
		moduleName, version = strings.TrimPrefix(matches[1], "x/"), matches[2]

		if !strings.HasPrefix(version, "v") {
			version = "v" + version
		}
	}

	return moduleName, version
}
