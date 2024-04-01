/*
 *
 * Copyright 2016 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

/*
Package reflection implements server reflection service.

The service implemented is defined in:
https://github.com/grpc/grpc/blob/master/src/proto/grpc/reflection/v1alpha/reflection.proto.

To register server reflection on a gRPC server:

	import "google.golang.org/grpc/reflection"

	s := grpc.NewServer()
	pb.RegisterYourOwnServer(s, &server{})

	// Register reflection service on gRPC server.
	reflection.Register(s)

	s.Serve(lis)
*/
package reflection // import "google.golang.org/grpc/reflection"

import (
	"io"
	"sort"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protodesc"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"

	v1reflectiongrpc "google.golang.org/grpc/reflection/grpc_reflection_v1"
	v1reflectionpb "google.golang.org/grpc/reflection/grpc_reflection_v1"
	v1alphareflectiongrpc "google.golang.org/grpc/reflection/grpc_reflection_v1alpha"
)

// GRPCServer is the interface provided by a gRPC server. It is implemented by
// *grpc.Server, but could also be implemented by other concrete types. It acts
// as a registry, for accumulating the services exposed by the server.
type GRPCServer interface {
	grpc.ServiceRegistrar
	ServiceInfoProvider
}

var _ GRPCServer = (*grpc.Server)(nil)

// Register registers the server reflection service on the given gRPC server.
// Both the v1 and v1alpha versions are registered.
func Register(s GRPCServer) {
	svr := NewServerV1(ServerOptions{Services: s})
	v1alphareflectiongrpc.RegisterServerReflectionServer(s, asV1Alpha(svr))
	v1reflectiongrpc.RegisterServerReflectionServer(s, svr)
}

// RegisterV1 registers only the v1 version of the server reflection service
// on the given gRPC server. Many clients may only support v1alpha so most
// users should use Register instead, at least until clients have upgraded.
func RegisterV1(s GRPCServer) {
	svr := NewServerV1(ServerOptions{Services: s})
	v1reflectiongrpc.RegisterServerReflectionServer(s, svr)
}

// ServiceInfoProvider is an interface used to retrieve metadata about the
// services to expose.
//
// The reflection service is only interested in the service names, but the
// signature is this way so that *grpc.Server implements it. So it is okay
// for a custom implementation to return zero values for the
// grpc.ServiceInfo values in the map.
//
// # Experimental
//
// Notice: This type is EXPERIMENTAL and may be changed or removed in a
// later release.
type ServiceInfoProvider interface {
	GetServiceInfo() map[string]grpc.ServiceInfo
}

// ExtensionResolver is the interface used to query details about extensions.
// This interface is satisfied by protoregistry.GlobalTypes.
//
// # Experimental
//
// Notice: This type is EXPERIMENTAL and may be changed or removed in a
// later release.
type ExtensionResolver interface {
	protoregistry.ExtensionTypeResolver
	RangeExtensionsByMessage(message protoreflect.FullName, f func(protoreflect.ExtensionType) bool)
}

// ServerOptions represents the options used to construct a reflection server.
//
// # Experimental
//
// Notice: This type is EXPERIMENTAL and may be changed or removed in a
// later release.
type ServerOptions struct {
	// The source of advertised RPC services. If not specified, the reflection
	// server will report an empty list when asked to list services.
	//
	// This value will typically be a *grpc.Server. But the set of advertised
	// services can be customized by wrapping a *grpc.Server or using an
	// alternate implementation that returns a custom set of service names.
	Services ServiceInfoProvider
	// Optional resolver used to load descriptors. If not specified,
	// protoregistry.GlobalFiles will be used.
	DescriptorResolver protodesc.Resolver
	// Optional resolver used to query for known extensions. If not specified,
	// protoregistry.GlobalTypes will be used.
	ExtensionResolver ExtensionResolver
}

// NewServer returns a reflection server implementation using the given options.
// This can be used to customize behavior of the reflection service. Most usages
// should prefer to use Register instead. For backwards compatibility reasons,
// this returns the v1alpha version of the reflection server. For a v1 version
// of the reflection server, see NewServerV1.
//
// # Experimental
//
// Notice: This function is EXPERIMENTAL and may be changed or removed in a
// later release.
func NewServer(opts ServerOptions) v1alphareflectiongrpc.ServerReflectionServer {
	return asV1Alpha(NewServerV1(opts))
}

// NewServerV1 returns a reflection server implementation using the given options.
// This can be used to customize behavior of the reflection service. Most usages
// should prefer to use Register instead.
//
// # Experimental
//
// Notice: This function is EXPERIMENTAL and may be changed or removed in a
// later release.
func NewServerV1(opts ServerOptions) v1reflectiongrpc.ServerReflectionServer {
	if opts.DescriptorResolver == nil {
		opts.DescriptorResolver = protoregistry.GlobalFiles
	}
	if opts.ExtensionResolver == nil {
		opts.ExtensionResolver = protoregistry.GlobalTypes
	}
	return &serverReflectionServer{
		s:            opts.Services,
		descResolver: opts.DescriptorResolver,
		extResolver:  opts.ExtensionResolver,
	}
}

type serverReflectionServer struct {
	v1alphareflectiongrpc.UnimplementedServerReflectionServer
	s            ServiceInfoProvider
	descResolver protodesc.Resolver
	extResolver  ExtensionResolver
}

// fileDescWithDependencies returns a slice of serialized fileDescriptors in
// wire format ([]byte). The fileDescriptors will include fd and all the
// transitive dependencies of fd with names not in sentFileDescriptors.
func (s *serverReflectionServer) fileDescWithDependencies(fd protoreflect.FileDescriptor, sentFileDescriptors map[string]bool) ([][]byte, error) {
	if fd.IsPlaceholder() {
		// If the given root file is a placeholder, treat it
		// as missing instead of serializing it.
		return nil, protoregistry.NotFound
	}
	var r [][]byte
	queue := []protoreflect.FileDescriptor{fd}
	for len(queue) > 0 {
		currentfd := queue[0]
		queue = queue[1:]
		if currentfd.IsPlaceholder() {
			// Skip any missing files in the dependency graph.
			continue
		}
		if sent := sentFileDescriptors[currentfd.Path()]; len(r) == 0 || !sent {
			sentFileDescriptors[currentfd.Path()] = true
			fdProto := protodesc.ToFileDescriptorProto(currentfd)
			currentfdEncoded, err := proto.Marshal(fdProto)
			if err != nil {
				return nil, err
			}
			r = append(r, currentfdEncoded)
		}
		for i := 0; i < currentfd.Imports().Len(); i++ {
			queue = append(queue, currentfd.Imports().Get(i))
		}
	}
	return r, nil
}

// fileDescEncodingContainingSymbol finds the file descriptor containing the
// given symbol, finds all of its previously unsent transitive dependencies,
// does marshalling on them, and returns the marshalled result. The given symbol
// can be a type, a service or a method.
func (s *serverReflectionServer) fileDescEncodingContainingSymbol(name string, sentFileDescriptors map[string]bool) ([][]byte, error) {
	d, err := s.descResolver.FindDescriptorByName(protoreflect.FullName(name))
	if err != nil {
		return nil, err
	}
	return s.fileDescWithDependencies(d.ParentFile(), sentFileDescriptors)
}

// fileDescEncodingContainingExtension finds the file descriptor containing
// given extension, finds all of its previously unsent transitive dependencies,
// does marshalling on them, and returns the marshalled result.
func (s *serverReflectionServer) fileDescEncodingContainingExtension(typeName string, extNum int32, sentFileDescriptors map[string]bool) ([][]byte, error) {
	xt, err := s.extResolver.FindExtensionByNumber(protoreflect.FullName(typeName), protoreflect.FieldNumber(extNum))
	if err != nil {
		return nil, err
	}
	return s.fileDescWithDependencies(xt.TypeDescriptor().ParentFile(), sentFileDescriptors)
}

// allExtensionNumbersForTypeName returns all extension numbers for the given type.
func (s *serverReflectionServer) allExtensionNumbersForTypeName(name string) ([]int32, error) {
	var numbers []int32
	s.extResolver.RangeExtensionsByMessage(protoreflect.FullName(name), func(xt protoreflect.ExtensionType) bool {
		numbers = append(numbers, int32(xt.TypeDescriptor().Number()))
		return true
	})
	sort.Slice(numbers, func(i, j int) bool {
		return numbers[i] < numbers[j]
	})
	if len(numbers) == 0 {
		// maybe return an error if given type name is not known
		if _, err := s.descResolver.FindDescriptorByName(protoreflect.FullName(name)); err != nil {
			return nil, err
		}
	}
	return numbers, nil
}

// listServices returns the names of services this server exposes.
func (s *serverReflectionServer) listServices() []*v1reflectionpb.ServiceResponse {
	serviceInfo := s.s.GetServiceInfo()
	resp := make([]*v1reflectionpb.ServiceResponse, 0, len(serviceInfo))
	for svc := range serviceInfo {
		resp = append(resp, &v1reflectionpb.ServiceResponse{Name: svc})
	}
	sort.Slice(resp, func(i, j int) bool {
		return resp[i].Name < resp[j].Name
	})
	return resp
}

// ServerReflectionInfo is the reflection service handler.
func (s *serverReflectionServer) ServerReflectionInfo(stream v1reflectiongrpc.ServerReflection_ServerReflectionInfoServer) error {
	sentFileDescriptors := make(map[string]bool)
	for {
		in, err := stream.Recv()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return err
		}

		out := &v1reflectionpb.ServerReflectionResponse{
			ValidHost:       in.Host,
			OriginalRequest: in,
		}
		switch req := in.MessageRequest.(type) {
		case *v1reflectionpb.ServerReflectionRequest_FileByFilename:
			var b [][]byte
			fd, err := s.descResolver.FindFileByPath(req.FileByFilename)
			if err == nil {
				b, err = s.fileDescWithDependencies(fd, sentFileDescriptors)
			}
			if err != nil {
				out.MessageResponse = &v1reflectionpb.ServerReflectionResponse_ErrorResponse{
					ErrorResponse: &v1reflectionpb.ErrorResponse{
						ErrorCode:    int32(codes.NotFound),
						ErrorMessage: err.Error(),
					},
				}
			} else {
				out.MessageResponse = &v1reflectionpb.ServerReflectionResponse_FileDescriptorResponse{
					FileDescriptorResponse: &v1reflectionpb.FileDescriptorResponse{FileDescriptorProto: b},
				}
			}
		case *v1reflectionpb.ServerReflectionRequest_FileContainingSymbol:
			b, err := s.fileDescEncodingContainingSymbol(req.FileContainingSymbol, sentFileDescriptors)
			if err != nil {
				out.MessageResponse = &v1reflectionpb.ServerReflectionResponse_ErrorResponse{
					ErrorResponse: &v1reflectionpb.ErrorResponse{
						ErrorCode:    int32(codes.NotFound),
						ErrorMessage: err.Error(),
					},
				}
			} else {
				out.MessageResponse = &v1reflectionpb.ServerReflectionResponse_FileDescriptorResponse{
					FileDescriptorResponse: &v1reflectionpb.FileDescriptorResponse{FileDescriptorProto: b},
				}
			}
		case *v1reflectionpb.ServerReflectionRequest_FileContainingExtension:
			typeName := req.FileContainingExtension.ContainingType
			extNum := req.FileContainingExtension.ExtensionNumber
			b, err := s.fileDescEncodingContainingExtension(typeName, extNum, sentFileDescriptors)
			if err != nil {
				out.MessageResponse = &v1reflectionpb.ServerReflectionResponse_ErrorResponse{
					ErrorResponse: &v1reflectionpb.ErrorResponse{
						ErrorCode:    int32(codes.NotFound),
						ErrorMessage: err.Error(),
					},
				}
			} else {
				out.MessageResponse = &v1reflectionpb.ServerReflectionResponse_FileDescriptorResponse{
					FileDescriptorResponse: &v1reflectionpb.FileDescriptorResponse{FileDescriptorProto: b},
				}
			}
		case *v1reflectionpb.ServerReflectionRequest_AllExtensionNumbersOfType:
			extNums, err := s.allExtensionNumbersForTypeName(req.AllExtensionNumbersOfType)
			if err != nil {
				out.MessageResponse = &v1reflectionpb.ServerReflectionResponse_ErrorResponse{
					ErrorResponse: &v1reflectionpb.ErrorResponse{
						ErrorCode:    int32(codes.NotFound),
						ErrorMessage: err.Error(),
					},
				}
			} else {
				out.MessageResponse = &v1reflectionpb.ServerReflectionResponse_AllExtensionNumbersResponse{
					AllExtensionNumbersResponse: &v1reflectionpb.ExtensionNumberResponse{
						BaseTypeName:    req.AllExtensionNumbersOfType,
						ExtensionNumber: extNums,
					},
				}
			}
		case *v1reflectionpb.ServerReflectionRequest_ListServices:
			out.MessageResponse = &v1reflectionpb.ServerReflectionResponse_ListServicesResponse{
				ListServicesResponse: &v1reflectionpb.ListServiceResponse{
					Service: s.listServices(),
				},
			}
		default:
			return status.Errorf(codes.InvalidArgument, "invalid MessageRequest: %v", in.MessageRequest)
		}

		if err := stream.Send(out); err != nil {
			return err
		}
	}
}
