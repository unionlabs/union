package descriptor

import (
	"github.com/golang/protobuf/proto"
	"google.golang.org/genproto/googleapis/api/annotations"
)

// GrpcAPIService represents a stripped down version of google.api.Service .
// Compare to https://github.com/googleapis/googleapis/blob/master/google/api/service.proto
// The original imports 23 other protobuf files we are not interested in. If a significant
// subset (>50%) of these start being reproduced in this file we should swap to using the
// full generated version instead.
//
// For the purposes of the gateway generator we only consider a small subset of all
// available features google supports in their service descriptions. Thanks to backwards
// compatibility guarantees by protobuf it is safe for us to remove the other fields.
// We also only implement the absolute minimum of protobuf generator boilerplate to use
// our simplified version. These should be pretty stable too.
type GrpcAPIService struct {
	// Http Rule. Named Http in the actual proto. Changed to suppress linter warning.
	HTTP *annotations.Http `protobuf:"bytes,9,opt,name=http" json:"http,omitempty"`
}

// ProtoMessage returns an empty GrpcAPIService element
func (*GrpcAPIService) ProtoMessage() {}

// Reset resets the GrpcAPIService
func (m *GrpcAPIService) Reset() { *m = GrpcAPIService{} }

// String returns the string representation of the GrpcAPIService
func (m *GrpcAPIService) String() string { return proto.CompactTextString(m) }
