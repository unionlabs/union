/*
 *
 * Copyright 2023 gRPC authors.
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

package reflection

import (
	v1reflectiongrpc "google.golang.org/grpc/reflection/grpc_reflection_v1"
	v1reflectionpb "google.golang.org/grpc/reflection/grpc_reflection_v1"
	v1alphareflectiongrpc "google.golang.org/grpc/reflection/grpc_reflection_v1alpha"
	v1alphareflectionpb "google.golang.org/grpc/reflection/grpc_reflection_v1alpha"
)

// asV1Alpha returns an implementation of the v1alpha version of the reflection
// interface that delegates all calls to the given v1 version.
func asV1Alpha(svr v1reflectiongrpc.ServerReflectionServer) v1alphareflectiongrpc.ServerReflectionServer {
	return v1AlphaServerImpl{svr: svr}
}

type v1AlphaServerImpl struct {
	svr v1reflectiongrpc.ServerReflectionServer
}

func (s v1AlphaServerImpl) ServerReflectionInfo(stream v1alphareflectiongrpc.ServerReflection_ServerReflectionInfoServer) error {
	return s.svr.ServerReflectionInfo(v1AlphaServerStreamAdapter{stream})
}

type v1AlphaServerStreamAdapter struct {
	v1alphareflectiongrpc.ServerReflection_ServerReflectionInfoServer
}

func (s v1AlphaServerStreamAdapter) Send(response *v1reflectionpb.ServerReflectionResponse) error {
	return s.ServerReflection_ServerReflectionInfoServer.Send(v1ToV1AlphaResponse(response))
}

func (s v1AlphaServerStreamAdapter) Recv() (*v1reflectionpb.ServerReflectionRequest, error) {
	resp, err := s.ServerReflection_ServerReflectionInfoServer.Recv()
	if err != nil {
		return nil, err
	}
	return v1AlphaToV1Request(resp), nil
}

func v1ToV1AlphaResponse(v1 *v1reflectionpb.ServerReflectionResponse) *v1alphareflectionpb.ServerReflectionResponse {
	var v1alpha v1alphareflectionpb.ServerReflectionResponse
	v1alpha.ValidHost = v1.ValidHost
	if v1.OriginalRequest != nil {
		v1alpha.OriginalRequest = v1ToV1AlphaRequest(v1.OriginalRequest)
	}
	switch mr := v1.MessageResponse.(type) {
	case *v1reflectionpb.ServerReflectionResponse_FileDescriptorResponse:
		if mr != nil {
			v1alpha.MessageResponse = &v1alphareflectionpb.ServerReflectionResponse_FileDescriptorResponse{
				FileDescriptorResponse: &v1alphareflectionpb.FileDescriptorResponse{
					FileDescriptorProto: mr.FileDescriptorResponse.GetFileDescriptorProto(),
				},
			}
		}
	case *v1reflectionpb.ServerReflectionResponse_AllExtensionNumbersResponse:
		if mr != nil {
			v1alpha.MessageResponse = &v1alphareflectionpb.ServerReflectionResponse_AllExtensionNumbersResponse{
				AllExtensionNumbersResponse: &v1alphareflectionpb.ExtensionNumberResponse{
					BaseTypeName:    mr.AllExtensionNumbersResponse.GetBaseTypeName(),
					ExtensionNumber: mr.AllExtensionNumbersResponse.GetExtensionNumber(),
				},
			}
		}
	case *v1reflectionpb.ServerReflectionResponse_ListServicesResponse:
		if mr != nil {
			svcs := make([]*v1alphareflectionpb.ServiceResponse, len(mr.ListServicesResponse.GetService()))
			for i, svc := range mr.ListServicesResponse.GetService() {
				svcs[i] = &v1alphareflectionpb.ServiceResponse{
					Name: svc.GetName(),
				}
			}
			v1alpha.MessageResponse = &v1alphareflectionpb.ServerReflectionResponse_ListServicesResponse{
				ListServicesResponse: &v1alphareflectionpb.ListServiceResponse{
					Service: svcs,
				},
			}
		}
	case *v1reflectionpb.ServerReflectionResponse_ErrorResponse:
		if mr != nil {
			v1alpha.MessageResponse = &v1alphareflectionpb.ServerReflectionResponse_ErrorResponse{
				ErrorResponse: &v1alphareflectionpb.ErrorResponse{
					ErrorCode:    mr.ErrorResponse.GetErrorCode(),
					ErrorMessage: mr.ErrorResponse.GetErrorMessage(),
				},
			}
		}
	default:
		// no value set
	}
	return &v1alpha
}

func v1AlphaToV1Request(v1alpha *v1alphareflectionpb.ServerReflectionRequest) *v1reflectionpb.ServerReflectionRequest {
	var v1 v1reflectionpb.ServerReflectionRequest
	v1.Host = v1alpha.Host
	switch mr := v1alpha.MessageRequest.(type) {
	case *v1alphareflectionpb.ServerReflectionRequest_FileByFilename:
		v1.MessageRequest = &v1reflectionpb.ServerReflectionRequest_FileByFilename{
			FileByFilename: mr.FileByFilename,
		}
	case *v1alphareflectionpb.ServerReflectionRequest_FileContainingSymbol:
		v1.MessageRequest = &v1reflectionpb.ServerReflectionRequest_FileContainingSymbol{
			FileContainingSymbol: mr.FileContainingSymbol,
		}
	case *v1alphareflectionpb.ServerReflectionRequest_FileContainingExtension:
		if mr.FileContainingExtension != nil {
			v1.MessageRequest = &v1reflectionpb.ServerReflectionRequest_FileContainingExtension{
				FileContainingExtension: &v1reflectionpb.ExtensionRequest{
					ContainingType:  mr.FileContainingExtension.GetContainingType(),
					ExtensionNumber: mr.FileContainingExtension.GetExtensionNumber(),
				},
			}
		}
	case *v1alphareflectionpb.ServerReflectionRequest_AllExtensionNumbersOfType:
		v1.MessageRequest = &v1reflectionpb.ServerReflectionRequest_AllExtensionNumbersOfType{
			AllExtensionNumbersOfType: mr.AllExtensionNumbersOfType,
		}
	case *v1alphareflectionpb.ServerReflectionRequest_ListServices:
		v1.MessageRequest = &v1reflectionpb.ServerReflectionRequest_ListServices{
			ListServices: mr.ListServices,
		}
	default:
		// no value set
	}
	return &v1
}

func v1ToV1AlphaRequest(v1 *v1reflectionpb.ServerReflectionRequest) *v1alphareflectionpb.ServerReflectionRequest {
	var v1alpha v1alphareflectionpb.ServerReflectionRequest
	v1alpha.Host = v1.Host
	switch mr := v1.MessageRequest.(type) {
	case *v1reflectionpb.ServerReflectionRequest_FileByFilename:
		if mr != nil {
			v1alpha.MessageRequest = &v1alphareflectionpb.ServerReflectionRequest_FileByFilename{
				FileByFilename: mr.FileByFilename,
			}
		}
	case *v1reflectionpb.ServerReflectionRequest_FileContainingSymbol:
		if mr != nil {
			v1alpha.MessageRequest = &v1alphareflectionpb.ServerReflectionRequest_FileContainingSymbol{
				FileContainingSymbol: mr.FileContainingSymbol,
			}
		}
	case *v1reflectionpb.ServerReflectionRequest_FileContainingExtension:
		if mr != nil {
			v1alpha.MessageRequest = &v1alphareflectionpb.ServerReflectionRequest_FileContainingExtension{
				FileContainingExtension: &v1alphareflectionpb.ExtensionRequest{
					ContainingType:  mr.FileContainingExtension.GetContainingType(),
					ExtensionNumber: mr.FileContainingExtension.GetExtensionNumber(),
				},
			}
		}
	case *v1reflectionpb.ServerReflectionRequest_AllExtensionNumbersOfType:
		if mr != nil {
			v1alpha.MessageRequest = &v1alphareflectionpb.ServerReflectionRequest_AllExtensionNumbersOfType{
				AllExtensionNumbersOfType: mr.AllExtensionNumbersOfType,
			}
		}
	case *v1reflectionpb.ServerReflectionRequest_ListServices:
		if mr != nil {
			v1alpha.MessageRequest = &v1alphareflectionpb.ServerReflectionRequest_ListServices{
				ListServices: mr.ListServices,
			}
		}
	default:
		// no value set
	}
	return &v1alpha
}
