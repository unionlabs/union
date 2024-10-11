package msgservice

import (
	"errors"
	"fmt"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"

	msg "cosmossdk.io/api/cosmos/msg/v1"
	"cosmossdk.io/x/tx/signing"
)

// ValidateProtoAnnotations validates that the proto annotations are correct.
// More specifically, it verifies:
// - all services named "Msg" have `(cosmos.msg.v1.service) = true`,
//
// More validations can be added here in the future.
//
// If `protoFiles` is nil, then protoregistry.GlobalFile will be used.
func ValidateProtoAnnotations(protoFiles signing.ProtoFileResolver) error {
	if protoFiles == nil {
		protoFiles = protoregistry.GlobalFiles
	}

	var serviceErrs []error
	protoFiles.RangeFiles(func(fd protoreflect.FileDescriptor) bool {
		for i := 0; i < fd.Services().Len(); i++ {
			sd := fd.Services().Get(i)
			if sd.Name() == "Msg" {
				// We use the heuristic that services name Msg are exactly the
				// ones that need the proto annotations check.
				err := validateMsgServiceAnnotations(sd)
				if err != nil {
					serviceErrs = append(serviceErrs, err)
				}
			}
		}

		return true
	})

	return errors.Join(serviceErrs...)
}

// validateMsgServiceAnnotations validates that the service has the
// `(cosmos.msg.v1.service) = true` proto annotation.
func validateMsgServiceAnnotations(sd protoreflect.ServiceDescriptor) error {
	ext := proto.GetExtension(sd.Options(), msg.E_Service)
	isService, ok := ext.(bool)
	if !ok {
		return fmt.Errorf("expected bool, got %T", ext)
	}

	if !isService {
		return fmt.Errorf("service %s does not have cosmos.msg.v1.service proto annotation", sd.FullName())
	}

	return nil
}
