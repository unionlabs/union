package v2

import (
	bytes "bytes"
	fmt "fmt"
	io "io"
	math_bits "math/bits"

	proto "github.com/cosmos/gogoproto/proto"
	"gopkg.in/yaml.v2"
)

// AccessType permission types
type AccessType int32

const (
	// AccessTypeUnspecified placeholder for empty value
	AccessTypeUnspecified AccessType = 0
	// AccessTypeNobody forbidden
	AccessTypeNobody AccessType = 1
	// AccessTypeOnlyAddress restricted to a single address
	// Deprecated: use AccessTypeAnyOfAddresses instead
	AccessTypeOnlyAddress AccessType = 2
	// AccessTypeEverybody unrestricted
	AccessTypeEverybody AccessType = 3
	// AccessTypeAnyOfAddresses allow any of the addresses
	AccessTypeAnyOfAddresses AccessType = 4
)

var AccessType_name = map[int32]string{
	0: "ACCESS_TYPE_UNSPECIFIED",
	1: "ACCESS_TYPE_NOBODY",
	2: "ACCESS_TYPE_ONLY_ADDRESS",
	3: "ACCESS_TYPE_EVERYBODY",
	4: "ACCESS_TYPE_ANY_OF_ADDRESSES",
}

var AccessType_value = map[string]int32{
	"ACCESS_TYPE_UNSPECIFIED":      0,
	"ACCESS_TYPE_NOBODY":           1,
	"ACCESS_TYPE_ONLY_ADDRESS":     2,
	"ACCESS_TYPE_EVERYBODY":        3,
	"ACCESS_TYPE_ANY_OF_ADDRESSES": 4,
}

func (AccessType) EnumDescriptor() ([]byte, []int) {
	return fileDescriptor_e6155d98fa173e02, []int{0}
}

// AccessTypeParam
type AccessTypeParam struct {
	Value AccessType `protobuf:"varint,1,opt,name=value,proto3,enum=cosmwasm.wasm.v1.AccessType" json:"value,omitempty" yaml:"value"`
}

func (m *AccessTypeParam) Reset()         { *m = AccessTypeParam{} }
func (m *AccessTypeParam) String() string { return proto.CompactTextString(m) }
func (*AccessTypeParam) ProtoMessage()    {}
func (*AccessTypeParam) Descriptor() ([]byte, []int) {
	return fileDescriptor_e6155d98fa173e02, []int{0}
}

func (m *AccessTypeParam) XXX_Unmarshal(b []byte) error {
	return m.Unmarshal(b)
}

func (m *AccessTypeParam) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	if deterministic {
		return xxx_messageInfo_AccessTypeParam.Marshal(b, m, deterministic)
	} else {
		b = b[:cap(b)]
		n, err := m.MarshalToSizedBuffer(b)
		if err != nil {
			return nil, err
		}
		return b[:n], nil
	}
}

func (m *AccessTypeParam) XXX_Merge(src proto.Message) {
	xxx_messageInfo_AccessTypeParam.Merge(m, src)
}

func (m *AccessTypeParam) XXX_Size() int {
	return m.Size()
}

func (m *AccessTypeParam) XXX_DiscardUnknown() {
	xxx_messageInfo_AccessTypeParam.DiscardUnknown(m)
}

var xxx_messageInfo_AccessTypeParam proto.InternalMessageInfo

// AccessConfig access control type.
type AccessConfig struct {
	Permission AccessType `protobuf:"varint,1,opt,name=permission,proto3,enum=cosmwasm.wasm.v1.AccessType" json:"permission,omitempty" yaml:"permission"`
	// Address
	// Deprecated: replaced by addresses
	Address   string   `protobuf:"bytes,2,opt,name=address,proto3" json:"address,omitempty" yaml:"address"`
	Addresses []string `protobuf:"bytes,3,rep,name=addresses,proto3" json:"addresses,omitempty" yaml:"addresses"`
}

func (m *AccessConfig) Reset()         { *m = AccessConfig{} }
func (m *AccessConfig) String() string { return proto.CompactTextString(m) }
func (*AccessConfig) ProtoMessage()    {}
func (*AccessConfig) Descriptor() ([]byte, []int) {
	return fileDescriptor_e6155d98fa173e02, []int{1}
}

func (m *AccessConfig) XXX_Unmarshal(b []byte) error {
	return m.Unmarshal(b)
}

func (m *AccessConfig) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	if deterministic {
		return xxx_messageInfo_AccessConfig.Marshal(b, m, deterministic)
	} else {
		b = b[:cap(b)]
		n, err := m.MarshalToSizedBuffer(b)
		if err != nil {
			return nil, err
		}
		return b[:n], nil
	}
}

func (m *AccessConfig) XXX_Merge(src proto.Message) {
	xxx_messageInfo_AccessConfig.Merge(m, src)
}

func (m *AccessConfig) XXX_Size() int {
	return m.Size()
}

func (m *AccessConfig) XXX_DiscardUnknown() {
	xxx_messageInfo_AccessConfig.DiscardUnknown(m)
}

var xxx_messageInfo_AccessConfig proto.InternalMessageInfo

// Params defines the set of wasm parameters.
type Params struct {
	CodeUploadAccess             AccessConfig `protobuf:"bytes,1,opt,name=code_upload_access,json=codeUploadAccess,proto3" json:"code_upload_access" yaml:"code_upload_access"`
	InstantiateDefaultPermission AccessType   `protobuf:"varint,2,opt,name=instantiate_default_permission,json=instantiateDefaultPermission,proto3,enum=cosmwasm.wasm.v1.AccessType" json:"instantiate_default_permission,omitempty" yaml:"instantiate_default_permission"`
}

func (p Params) String() string {
	out, err := yaml.Marshal(p)
	if err != nil {
		panic(err)
	}
	return string(out)
}

func (m *Params) Reset()      { *m = Params{} }
func (*Params) ProtoMessage() {}
func (*Params) Descriptor() ([]byte, []int) {
	return fileDescriptor_e6155d98fa173e02, []int{2}
}

func (m *Params) XXX_Unmarshal(b []byte) error {
	return m.Unmarshal(b)
}

func (m *Params) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	if deterministic {
		return xxx_messageInfo_Params.Marshal(b, m, deterministic)
	} else {
		b = b[:cap(b)]
		n, err := m.MarshalToSizedBuffer(b)
		if err != nil {
			return nil, err
		}
		return b[:n], nil
	}
}

func (m *Params) XXX_Merge(src proto.Message) {
	xxx_messageInfo_Params.Merge(m, src)
}

func (m *Params) XXX_Size() int {
	return m.Size()
}

func (m *Params) XXX_DiscardUnknown() {
	xxx_messageInfo_Params.DiscardUnknown(m)
}

var xxx_messageInfo_Params proto.InternalMessageInfo

// CodeInfo is data for the uploaded contract WASM code
type CodeInfo struct {
	// CodeHash is the unique identifier created by wasmvm
	CodeHash []byte `protobuf:"bytes,1,opt,name=code_hash,json=codeHash,proto3" json:"code_hash,omitempty"`
	// Creator address who initially stored the code
	Creator string `protobuf:"bytes,2,opt,name=creator,proto3" json:"creator,omitempty"`
	// InstantiateConfig access control to apply on contract creation, optional
	InstantiateConfig AccessConfig `protobuf:"bytes,5,opt,name=instantiate_config,json=instantiateConfig,proto3" json:"instantiate_config"`
}

func (m *CodeInfo) Reset()         { *m = CodeInfo{} }
func (m *CodeInfo) String() string { return proto.CompactTextString(m) }
func (*CodeInfo) ProtoMessage()    {}
func (*CodeInfo) Descriptor() ([]byte, []int) {
	return fileDescriptor_e6155d98fa173e02, []int{3}
}

func (m *CodeInfo) XXX_Unmarshal(b []byte) error {
	return m.Unmarshal(b)
}

func (m *CodeInfo) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	if deterministic {
		return xxx_messageInfo_CodeInfo.Marshal(b, m, deterministic)
	} else {
		b = b[:cap(b)]
		n, err := m.MarshalToSizedBuffer(b)
		if err != nil {
			return nil, err
		}
		return b[:n], nil
	}
}

func (m *CodeInfo) XXX_Merge(src proto.Message) {
	xxx_messageInfo_CodeInfo.Merge(m, src)
}

func (m *CodeInfo) XXX_Size() int {
	return m.Size()
}

func (m *CodeInfo) XXX_DiscardUnknown() {
	xxx_messageInfo_CodeInfo.DiscardUnknown(m)
}

var xxx_messageInfo_CodeInfo proto.InternalMessageInfo

var fileDescriptor_e6155d98fa173e02 = []byte{
	// 1201 bytes of a gzipped FileDescriptorProto
	0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0x94, 0x56, 0xcf, 0x8f, 0xdb, 0xc4,
	0x17, 0x8f, 0x93, 0xec, 0x8f, 0x4c, 0xf7, 0xdb, 0xaf, 0x3b, 0x6c, 0x69, 0x36, 0xac, 0x92, 0x60,
	0x4a, 0xd9, 0x6e, 0xdb, 0xa4, 0x5d, 0x10, 0x48, 0x3d, 0x54, 0xca, 0x0f, 0xb7, 0xeb, 0x8a, 0x4d,
	0xa2, 0x49, 0x4a, 0x59, 0xa4, 0x62, 0x39, 0xf6, 0x24, 0x6b, 0x35, 0xf1, 0x44, 0x9e, 0xc9, 0x36,
	0xfe, 0x0f, 0x50, 0x24, 0x24, 0x0e, 0x1c, 0xb8, 0x44, 0x42, 0x02, 0x41, 0xb9, 0x71, 0xe0, 0x8f,
	0xa8, 0x40, 0x42, 0x3d, 0x72, 0x8a, 0x60, 0x7b, 0x80, 0x73, 0x8e, 0xe5, 0x82, 0x3c, 0x13, 0xd7,
	0xa6, 0xed, 0x76, 0xc3, 0xc5, 0x9a, 0x79, 0xef, 0x7d, 0x3e, 0xef, 0xbd, 0xcf, 0xcc, 0x3c, 0x19,
	0x6c, 0x9a, 0x84, 0xf6, 0x1f, 0x18, 0xb4, 0x5f, 0xe4, 0x9f, 0xc3, 0x6b, 0x45, 0xe6, 0x0d, 0x30,
	0x2d, 0x0c, 0x5c, 0xc2, 0x08, 0x94, 0x03, 0x6f, 0x81, 0x7f, 0x0e, 0xaf, 0x65, 0x36, 0x7c, 0x0b,
	0xa1, 0x3a, 0xf7, 0x17, 0xc5, 0x46, 0x04, 0x67, 0xd6, 0xbb, 0xa4, 0x4b, 0x84, 0xdd, 0x5f, 0xcd,
	0xad, 0x1b, 0x5d, 0x42, 0xba, 0x3d, 0x5c, 0xe4, 0xbb, 0xf6, 0xb0, 0x53, 0x34, 0x1c, 0x6f, 0xee,
	0x3a, 0x63, 0xf4, 0x6d, 0x87, 0x14, 0xf9, 0x57, 0x98, 0x94, 0x7b, 0xe0, 0xff, 0x25, 0xd3, 0xc4,
	0x94, 0xb6, 0xbc, 0x01, 0x6e, 0x18, 0xae, 0xd1, 0x87, 0x55, 0xb0, 0x74, 0x68, 0xf4, 0x86, 0x38,
	0x2d, 0xe5, 0xa5, 0xad, 0xd3, 0x3b, 0x9b, 0x85, 0xe7, 0x6b, 0x2a, 0x84, 0x88, 0xb2, 0x3c, 0x9b,
	0xe6, 0xd6, 0x3c, 0xa3, 0xdf, 0xbb, 0xae, 0x70, 0x90, 0x82, 0x04, 0xf8, 0x7a, 0xf2, 0xab, 0xaf,
	0x73, 0x92, 0xf2, 0x8b, 0x04, 0xd6, 0x44, 0x74, 0x85, 0x38, 0x1d, 0xbb, 0x0b, 0x9b, 0x00, 0x0c,
	0xb0, 0xdb, 0xb7, 0x29, 0xb5, 0x89, 0xb3, 0x50, 0x86, 0xb3, 0xb3, 0x69, 0xee, 0x8c, 0xc8, 0x10,
	0x22, 0x15, 0x14, 0xa1, 0x81, 0x97, 0xc1, 0x8a, 0x61, 0x59, 0x2e, 0xa6, 0x34, 0x1d, 0xcf, 0x4b,
	0x5b, 0xa9, 0x32, 0x9c, 0x4d, 0x73, 0xa7, 0x05, 0x66, 0xee, 0x50, 0x50, 0x10, 0x02, 0x77, 0x40,
	0x6a, 0xbe, 0xc4, 0x34, 0x9d, 0xc8, 0x27, 0xb6, 0x52, 0xe5, 0xf5, 0xd9, 0x34, 0x27, 0xff, 0x2b,
	0x1e, 0x53, 0x05, 0x85, 0x61, 0xf3, 0x6e, 0xbe, 0x8c, 0x83, 0x65, 0xae, 0x11, 0x85, 0x0c, 0x40,
	0x93, 0x58, 0x58, 0x1f, 0x0e, 0x7a, 0xc4, 0xb0, 0x74, 0x83, 0xd7, 0xcb, 0xfb, 0x39, 0xb5, 0x93,
	0x3d, 0xae, 0x1f, 0xa1, 0x41, 0xf9, 0xc2, 0xa3, 0x69, 0x2e, 0x36, 0x9b, 0xe6, 0x36, 0x44, 0xc6,
	0x17, 0x79, 0x94, 0x87, 0x7f, 0xfe, 0xb8, 0x2d, 0x21, 0xd9, 0xf7, 0xdc, 0xe1, 0x0e, 0x81, 0x87,
	0x9f, 0x4b, 0x20, 0x6b, 0x3b, 0x94, 0x19, 0x0e, 0xb3, 0x0d, 0x86, 0x75, 0x0b, 0x77, 0x8c, 0x61,
	0x8f, 0xe9, 0x11, 0x49, 0xe3, 0x0b, 0x48, 0x7a, 0x71, 0x36, 0xcd, 0xbd, 0x2d, 0x92, 0xbf, 0x9a,
	0x4d, 0x41, 0x9b, 0x91, 0x80, 0xaa, 0xf0, 0x37, 0x9e, 0xb9, 0xb9, 0x2c, 0x31, 0xe5, 0x7b, 0x09,
	0xac, 0x56, 0x88, 0x85, 0x35, 0xa7, 0x43, 0xe0, 0x1b, 0x20, 0xc5, 0x1b, 0x3a, 0x30, 0xe8, 0x01,
	0xd7, 0x63, 0x0d, 0xad, 0xfa, 0x86, 0x5d, 0x83, 0x1e, 0xc0, 0x34, 0x58, 0x31, 0x5d, 0x6c, 0x30,
	0xe2, 0x8a, 0x83, 0x42, 0xc1, 0x16, 0x7e, 0x0c, 0x60, 0xb4, 0x14, 0x93, 0x2b, 0x95, 0x5e, 0x5a,
	0x48, 0xcf, 0x94, 0xaf, 0xa7, 0x90, 0xec, 0x4c, 0x84, 0x44, 0x78, 0x6f, 0x27, 0x57, 0x13, 0x72,
	0xf2, 0x76, 0x72, 0x35, 0x29, 0x2f, 0x29, 0xbf, 0xc6, 0xc1, 0x5a, 0x85, 0x38, 0xcc, 0x35, 0x4c,
	0xc6, 0xab, 0x7d, 0x0b, 0xac, 0xf0, 0x6a, 0x6d, 0x8b, 0xd7, 0x9a, 0x2c, 0x83, 0xa3, 0x69, 0x6e,
	0x99, 0x37, 0x53, 0x45, 0xcb, 0xbe, 0x4b, 0xb3, 0x5e, 0x51, 0xf5, 0x3a, 0x58, 0x32, 0xac, 0xbe,
	0xed, 0xa4, 0x13, 0xdc, 0x2e, 0x36, 0xbe, 0xb5, 0x67, 0xb4, 0x71, 0x2f, 0x9d, 0x14, 0x56, 0xbe,
	0x81, 0x37, 0xe6, 0x2c, 0xd8, 0x9a, 0xb7, 0x75, 0xfe, 0x25, 0x6d, 0xb5, 0x29, 0xe9, 0x0d, 0x19,
	0x6e, 0x8d, 0x1a, 0x84, 0xda, 0xcc, 0x26, 0x0e, 0x0a, 0x40, 0xf0, 0x0a, 0x38, 0x65, 0xb7, 0x4d,
	0x7d, 0x40, 0x5c, 0xe6, 0x97, 0xbb, 0xcc, 0x2f, 0xfa, 0xff, 0x8e, 0xa6, 0xb9, 0x94, 0x56, 0xae,
	0x34, 0x88, 0xcb, 0xb4, 0x2a, 0x4a, 0xd9, 0x6d, 0x93, 0x2f, 0x2d, 0xf8, 0x29, 0x48, 0xe1, 0x11,
	0xc3, 0x0e, 0xbf, 0x14, 0x2b, 0x3c, 0xe1, 0x7a, 0x41, 0x8c, 0x86, 0x42, 0x30, 0x1a, 0x0a, 0x25,
	0xc7, 0x2b, 0x6f, 0xff, 0xfc, 0xd3, 0x95, 0x0b, 0x2f, 0x54, 0x12, 0x55, 0x49, 0x0d, 0x78, 0x50,
	0x48, 0x79, 0x3d, 0xf9, 0x97, 0xff, 0x22, 0xfe, 0x96, 0x40, 0x3a, 0x08, 0xf5, 0x55, 0xdb, 0xb5,
	0x29, 0x23, 0xae, 0xa7, 0x3a, 0xcc, 0xf5, 0x60, 0x03, 0xa4, 0xc8, 0x00, 0xbb, 0x06, 0x0b, 0x9f,
	0xfa, 0x4e, 0xe1, 0xd8, 0x4c, 0x11, 0x78, 0x3d, 0x40, 0xf9, 0xb7, 0x15, 0x85, 0x24, 0xd1, 0xe3,
	0x8a, 0x1f, 0x7b, 0x5c, 0x37, 0xc0, 0xca, 0x70, 0x60, 0x71, 0xa1, 0x13, 0xff, 0x45, 0xe8, 0x39,
	0x08, 0x6e, 0x81, 0x44, 0x9f, 0x76, 0xf9, 0xe1, 0xad, 0x95, 0x5f, 0x7f, 0x3a, 0xcd, 0x41, 0x64,
	0x3c, 0x08, 0xaa, 0xdc, 0xc3, 0x94, 0x1a, 0x5d, 0x8c, 0xfc, 0x10, 0x05, 0x01, 0xf8, 0x22, 0x11,
	0x7c, 0x13, 0xac, 0xb5, 0x7b, 0xc4, 0xbc, 0xaf, 0x1f, 0x60, 0xbb, 0x7b, 0xc0, 0xc4, 0xc5, 0x42,
	0xa7, 0xb8, 0x6d, 0x97, 0x9b, 0xe0, 0x06, 0x58, 0x65, 0x23, 0xdd, 0x76, 0x2c, 0x3c, 0x12, 0x8d,
	0xa0, 0x15, 0x36, 0xd2, 0xfc, 0xad, 0x82, 0xc1, 0xd2, 0x1e, 0xb1, 0x70, 0x0f, 0xde, 0x04, 0x89,
	0xfb, 0xd8, 0x13, 0x4f, 0xa8, 0xfc, 0xde, 0xd3, 0x69, 0xee, 0x6a, 0xd7, 0x66, 0x07, 0xc3, 0x76,
	0xc1, 0x24, 0xfd, 0xa2, 0x49, 0xfa, 0x98, 0xb5, 0x3b, 0x2c, 0x5c, 0xf4, 0xec, 0x36, 0x2d, 0xb6,
	0x3d, 0x86, 0x69, 0x61, 0x17, 0x8f, 0xca, 0xfe, 0x02, 0xf9, 0x04, 0xfe, 0x6d, 0x14, 0xe3, 0x3c,
	0xce, 0x1f, 0xa3, 0xd8, 0x6c, 0xff, 0x10, 0x07, 0x20, 0x9c, 0x08, 0xf0, 0x7d, 0x70, 0xae, 0x54,
	0xa9, 0xa8, 0xcd, 0xa6, 0xde, 0xda, 0x6f, 0xa8, 0xfa, 0x9d, 0x5a, 0xb3, 0xa1, 0x56, 0xb4, 0x9b,
	0x9a, 0x5a, 0x95, 0x63, 0x99, 0x8d, 0xf1, 0x24, 0x7f, 0x36, 0x0c, 0xbe, 0xe3, 0xd0, 0x01, 0x36,
	0xed, 0x8e, 0x8d, 0x2d, 0x78, 0x19, 0xc0, 0x28, 0xae, 0x56, 0x2f, 0xd7, 0xab, 0xfb, 0xb2, 0x94,
	0x59, 0x1f, 0x4f, 0xf2, 0x72, 0x08, 0xa9, 0x91, 0x36, 0xb1, 0x3c, 0xf8, 0x01, 0x48, 0x47, 0xa3,
	0xeb, 0xb5, 0x0f, 0xf7, 0xf5, 0x52, 0xb5, 0x8a, 0xd4, 0x66, 0x53, 0x8e, 0x3f, 0x9f, 0xa6, 0xee,
	0xf4, 0xbc, 0xd2, 0xb3, 0x91, 0x7d, 0x36, 0x0a, 0x54, 0x3f, 0x52, 0xd1, 0x3e, 0xcf, 0x94, 0xc8,
	0x9c, 0x1b, 0x4f, 0xf2, 0xaf, 0x85, 0x28, 0xf5, 0x10, 0xbb, 0x1e, 0x4f, 0x76, 0x03, 0x6c, 0x46,
	0x31, 0xa5, 0xda, 0xbe, 0x5e, 0xbf, 0x19, 0xa4, 0x53, 0x9b, 0x72, 0x32, 0xb3, 0x39, 0x9e, 0xe4,
	0xd3, 0x21, 0xb4, 0xe4, 0x78, 0xf5, 0x4e, 0x29, 0x18, 0xf9, 0x99, 0xd5, 0xcf, 0xbe, 0xc9, 0xc6,
	0x1e, 0x7e, 0x9b, 0x8d, 0x6d, 0x7f, 0x97, 0x00, 0xf9, 0x93, 0x6e, 0x29, 0xc4, 0xe0, 0x6a, 0xa5,
	0x5e, 0x6b, 0xa1, 0x52, 0xa5, 0xa5, 0x57, 0xea, 0x55, 0x55, 0xdf, 0xd5, 0x9a, 0xad, 0x3a, 0xda,
	0xd7, 0xeb, 0x0d, 0x15, 0x95, 0x5a, 0x5a, 0xbd, 0xf6, 0x32, 0x69, 0x8b, 0xe3, 0x49, 0xfe, 0xd2,
	0x49, 0xdc, 0x51, 0xc1, 0xef, 0x82, 0x8b, 0x0b, 0xa5, 0xd1, 0x6a, 0x5a, 0x4b, 0x96, 0x32, 0x5b,
	0xe3, 0x49, 0xfe, 0xfc, 0x49, 0xfc, 0x9a, 0x63, 0x33, 0x78, 0x0f, 0x5c, 0x5e, 0x88, 0x78, 0x4f,
	0xbb, 0x85, 0x4a, 0x2d, 0x55, 0x8e, 0x67, 0x2e, 0x8d, 0x27, 0xf9, 0x77, 0x4e, 0xe2, 0xde, 0xb3,
	0xbb, 0xae, 0xc1, 0xf0, 0xc2, 0xf4, 0xb7, 0xd4, 0x9a, 0xda, 0xd4, 0x9a, 0x72, 0x62, 0x31, 0xfa,
	0x5b, 0xd8, 0xc1, 0xd4, 0xa6, 0x99, 0xa4, 0x7f, 0x58, 0xe5, 0xdd, 0x47, 0x7f, 0x64, 0x63, 0x0f,
	0x8f, 0xb2, 0xd2, 0xa3, 0xa3, 0xac, 0xf4, 0xf8, 0x28, 0x2b, 0xfd, 0x7e, 0x94, 0x95, 0xbe, 0x78,
	0x92, 0x8d, 0x3d, 0x7e, 0x92, 0x8d, 0xfd, 0xf6, 0x24, 0x1b, 0xfb, 0xe4, 0x42, 0xe4, 0x0d, 0x55,
	0x08, 0xed, 0xdf, 0x0d, 0x7e, 0xc4, 0xac, 0xe2, 0x48, 0xfc, 0x90, 0xf1, 0xbf, 0xb1, 0xf6, 0x32,
	0x1f, 0x91, 0xef, 0xfe, 0x13, 0x00, 0x00, 0xff, 0xff, 0x69, 0x83, 0x28, 0x71, 0xae, 0x09, 0x00,
	0x00,
}

func (m *AccessTypeParam) Equal(that interface{}) bool {
	if that == nil {
		return m == nil
	}

	that1, ok := that.(*AccessTypeParam)
	if !ok {
		that2, ok := that.(AccessTypeParam)
		if ok {
			that1 = &that2
		} else {
			return false
		}
	}
	if that1 == nil {
		return m == nil
	} else if m == nil {
		return false
	}
	if m.Value != that1.Value {
		return false
	}
	return true
}

func (m *AccessConfig) Equal(that interface{}) bool {
	if that == nil {
		return m == nil
	}

	that1, ok := that.(*AccessConfig)
	if !ok {
		that2, ok := that.(AccessConfig)
		if ok {
			that1 = &that2
		} else {
			return false
		}
	}
	if that1 == nil {
		return m == nil
	} else if m == nil {
		return false
	}
	if m.Permission != that1.Permission {
		return false
	}
	if m.Address != that1.Address {
		return false
	}
	if len(m.Addresses) != len(that1.Addresses) {
		return false
	}
	for i := range m.Addresses {
		if m.Addresses[i] != that1.Addresses[i] {
			return false
		}
	}
	return true
}

func (m *Params) Equal(that interface{}) bool {
	if that == nil {
		return m == nil
	}

	that1, ok := that.(*Params)
	if !ok {
		that2, ok := that.(Params)
		if ok {
			that1 = &that2
		} else {
			return false
		}
	}
	if that1 == nil {
		return m == nil
	} else if m == nil {
		return false
	}
	if !m.CodeUploadAccess.Equal(&that1.CodeUploadAccess) {
		return false
	}
	if m.InstantiateDefaultPermission != that1.InstantiateDefaultPermission {
		return false
	}
	return true
}

func (m *CodeInfo) Equal(that interface{}) bool {
	if that == nil {
		return m == nil
	}

	that1, ok := that.(*CodeInfo)
	if !ok {
		that2, ok := that.(CodeInfo)
		if ok {
			that1 = &that2
		} else {
			return false
		}
	}
	if that1 == nil {
		return m == nil
	} else if m == nil {
		return false
	}
	if !bytes.Equal(m.CodeHash, that1.CodeHash) {
		return false
	}
	if m.Creator != that1.Creator {
		return false
	}
	if !m.InstantiateConfig.Equal(&that1.InstantiateConfig) {
		return false
	}
	return true
}

func (m *AccessTypeParam) Marshal() (dAtA []byte, err error) {
	size := m.Size()
	dAtA = make([]byte, size)
	n, err := m.MarshalToSizedBuffer(dAtA[:size])
	if err != nil {
		return nil, err
	}
	return dAtA[:n], nil
}

func (m *AccessTypeParam) MarshalTo(dAtA []byte) (int, error) {
	size := m.Size()
	return m.MarshalToSizedBuffer(dAtA[:size])
}

func (m *AccessTypeParam) MarshalToSizedBuffer(dAtA []byte) (int, error) {
	i := len(dAtA)
	_ = i
	var l int
	_ = l
	if m.Value != 0 {
		i = encodeVarintTypes(dAtA, i, uint64(m.Value))
		i--
		dAtA[i] = 0x8
	}
	return len(dAtA) - i, nil
}

func (m *AccessConfig) Marshal() (dAtA []byte, err error) {
	size := m.Size()
	dAtA = make([]byte, size)
	n, err := m.MarshalToSizedBuffer(dAtA[:size])
	if err != nil {
		return nil, err
	}
	return dAtA[:n], nil
}

func (m *AccessConfig) MarshalTo(dAtA []byte) (int, error) {
	size := m.Size()
	return m.MarshalToSizedBuffer(dAtA[:size])
}

func (m *AccessConfig) MarshalToSizedBuffer(dAtA []byte) (int, error) {
	i := len(dAtA)
	_ = i
	var l int
	_ = l
	if len(m.Addresses) > 0 {
		for iNdEx := len(m.Addresses) - 1; iNdEx >= 0; iNdEx-- {
			i -= len(m.Addresses[iNdEx])
			copy(dAtA[i:], m.Addresses[iNdEx])
			i = encodeVarintTypes(dAtA, i, uint64(len(m.Addresses[iNdEx])))
			i--
			dAtA[i] = 0x1a
		}
	}
	if len(m.Address) > 0 {
		i -= len(m.Address)
		copy(dAtA[i:], m.Address)
		i = encodeVarintTypes(dAtA, i, uint64(len(m.Address)))
		i--
		dAtA[i] = 0x12
	}
	if m.Permission != 0 {
		i = encodeVarintTypes(dAtA, i, uint64(m.Permission))
		i--
		dAtA[i] = 0x8
	}
	return len(dAtA) - i, nil
}

func (m *Params) Marshal() (dAtA []byte, err error) {
	size := m.Size()
	dAtA = make([]byte, size)
	n, err := m.MarshalToSizedBuffer(dAtA[:size])
	if err != nil {
		return nil, err
	}
	return dAtA[:n], nil
}

func (m *Params) MarshalTo(dAtA []byte) (int, error) {
	size := m.Size()
	return m.MarshalToSizedBuffer(dAtA[:size])
}

func (m *Params) MarshalToSizedBuffer(dAtA []byte) (int, error) {
	i := len(dAtA)
	_ = i
	var l int
	_ = l
	if m.InstantiateDefaultPermission != 0 {
		i = encodeVarintTypes(dAtA, i, uint64(m.InstantiateDefaultPermission))
		i--
		dAtA[i] = 0x10
	}
	{
		size, err := m.CodeUploadAccess.MarshalToSizedBuffer(dAtA[:i])
		if err != nil {
			return 0, err
		}
		i -= size
		i = encodeVarintTypes(dAtA, i, uint64(size))
	}
	i--
	dAtA[i] = 0xa
	return len(dAtA) - i, nil
}

func (m *CodeInfo) Marshal() (dAtA []byte, err error) {
	size := m.Size()
	dAtA = make([]byte, size)
	n, err := m.MarshalToSizedBuffer(dAtA[:size])
	if err != nil {
		return nil, err
	}
	return dAtA[:n], nil
}

func (m *CodeInfo) MarshalTo(dAtA []byte) (int, error) {
	size := m.Size()
	return m.MarshalToSizedBuffer(dAtA[:size])
}

func (m *CodeInfo) MarshalToSizedBuffer(dAtA []byte) (int, error) {
	i := len(dAtA)
	_ = i
	var l int
	_ = l
	{
		size, err := m.InstantiateConfig.MarshalToSizedBuffer(dAtA[:i])
		if err != nil {
			return 0, err
		}
		i -= size
		i = encodeVarintTypes(dAtA, i, uint64(size))
	}
	i--
	dAtA[i] = 0x2a
	if len(m.Creator) > 0 {
		i -= len(m.Creator)
		copy(dAtA[i:], m.Creator)
		i = encodeVarintTypes(dAtA, i, uint64(len(m.Creator)))
		i--
		dAtA[i] = 0x12
	}
	if len(m.CodeHash) > 0 {
		i -= len(m.CodeHash)
		copy(dAtA[i:], m.CodeHash)
		i = encodeVarintTypes(dAtA, i, uint64(len(m.CodeHash)))
		i--
		dAtA[i] = 0xa
	}
	return len(dAtA) - i, nil
}

func encodeVarintTypes(dAtA []byte, offset int, v uint64) int {
	offset -= sovTypes(v)
	base := offset
	for v >= 1<<7 {
		dAtA[offset] = uint8(v&0x7f | 0x80)
		v >>= 7
		offset++
	}
	dAtA[offset] = uint8(v)
	return base
}

func (m *AccessTypeParam) Size() (n int) {
	if m == nil {
		return 0
	}
	var l int
	_ = l
	if m.Value != 0 {
		n += 1 + sovTypes(uint64(m.Value))
	}
	return n
}

func (m *AccessConfig) Size() (n int) {
	if m == nil {
		return 0
	}
	var l int
	_ = l
	if m.Permission != 0 {
		n += 1 + sovTypes(uint64(m.Permission))
	}
	l = len(m.Address)
	if l > 0 {
		n += 1 + l + sovTypes(uint64(l))
	}
	if len(m.Addresses) > 0 {
		for _, s := range m.Addresses {
			l = len(s)
			n += 1 + l + sovTypes(uint64(l))
		}
	}
	return n
}

func (m *Params) Size() (n int) {
	if m == nil {
		return 0
	}
	var l int
	_ = l
	l = m.CodeUploadAccess.Size()
	n += 1 + l + sovTypes(uint64(l))
	if m.InstantiateDefaultPermission != 0 {
		n += 1 + sovTypes(uint64(m.InstantiateDefaultPermission))
	}
	return n
}

func (m *CodeInfo) Size() (n int) {
	if m == nil {
		return 0
	}
	var l int
	_ = l
	l = len(m.CodeHash)
	if l > 0 {
		n += 1 + l + sovTypes(uint64(l))
	}
	l = len(m.Creator)
	if l > 0 {
		n += 1 + l + sovTypes(uint64(l))
	}
	l = m.InstantiateConfig.Size()
	n += 1 + l + sovTypes(uint64(l))
	return n
}

func sovTypes(x uint64) (n int) {
	return (math_bits.Len64(x|1) + 6) / 7
}

func (m *AccessTypeParam) Unmarshal(dAtA []byte) error {
	l := len(dAtA)
	iNdEx := 0
	for iNdEx < l {
		preIndex := iNdEx
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return ErrIntOverflowTypes
			}
			if iNdEx >= l {
				return io.ErrUnexpectedEOF
			}
			b := dAtA[iNdEx]
			iNdEx++
			wire |= uint64(b&0x7F) << shift
			if b < 0x80 {
				break
			}
		}
		fieldNum := int32(wire >> 3)
		wireType := int(wire & 0x7)
		if wireType == 4 {
			return fmt.Errorf("proto: AccessTypeParam: wiretype end group for non-group")
		}
		if fieldNum <= 0 {
			return fmt.Errorf("proto: AccessTypeParam: illegal tag %d (wire type %d)", fieldNum, wire)
		}
		switch fieldNum {
		case 1:
			if wireType != 0 {
				return fmt.Errorf("proto: wrong wireType = %d for field Value", wireType)
			}
			m.Value = 0
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowTypes
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				m.Value |= AccessType(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
		default:
			iNdEx = preIndex
			skippy, err := skipTypes(dAtA[iNdEx:])
			if err != nil {
				return err
			}
			if (skippy < 0) || (iNdEx+skippy) < 0 {
				return ErrInvalidLengthTypes
			}
			if (iNdEx + skippy) > l {
				return io.ErrUnexpectedEOF
			}
			iNdEx += skippy
		}
	}

	if iNdEx > l {
		return io.ErrUnexpectedEOF
	}
	return nil
}

func (m *AccessConfig) Unmarshal(dAtA []byte) error {
	l := len(dAtA)
	iNdEx := 0
	for iNdEx < l {
		preIndex := iNdEx
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return ErrIntOverflowTypes
			}
			if iNdEx >= l {
				return io.ErrUnexpectedEOF
			}
			b := dAtA[iNdEx]
			iNdEx++
			wire |= uint64(b&0x7F) << shift
			if b < 0x80 {
				break
			}
		}
		fieldNum := int32(wire >> 3)
		wireType := int(wire & 0x7)
		if wireType == 4 {
			return fmt.Errorf("proto: AccessConfig: wiretype end group for non-group")
		}
		if fieldNum <= 0 {
			return fmt.Errorf("proto: AccessConfig: illegal tag %d (wire type %d)", fieldNum, wire)
		}
		switch fieldNum {
		case 1:
			if wireType != 0 {
				return fmt.Errorf("proto: wrong wireType = %d for field Permission", wireType)
			}
			m.Permission = 0
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowTypes
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				m.Permission |= AccessType(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
		case 2:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field Address", wireType)
			}
			var stringLen uint64
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowTypes
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				stringLen |= uint64(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			intStringLen := int(stringLen)
			if intStringLen < 0 {
				return ErrInvalidLengthTypes
			}
			postIndex := iNdEx + intStringLen
			if postIndex < 0 {
				return ErrInvalidLengthTypes
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			m.Address = string(dAtA[iNdEx:postIndex])
			iNdEx = postIndex
		case 3:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field Addresses", wireType)
			}
			var stringLen uint64
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowTypes
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				stringLen |= uint64(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			intStringLen := int(stringLen)
			if intStringLen < 0 {
				return ErrInvalidLengthTypes
			}
			postIndex := iNdEx + intStringLen
			if postIndex < 0 {
				return ErrInvalidLengthTypes
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			m.Addresses = append(m.Addresses, string(dAtA[iNdEx:postIndex]))
			iNdEx = postIndex
		default:
			iNdEx = preIndex
			skippy, err := skipTypes(dAtA[iNdEx:])
			if err != nil {
				return err
			}
			if (skippy < 0) || (iNdEx+skippy) < 0 {
				return ErrInvalidLengthTypes
			}
			if (iNdEx + skippy) > l {
				return io.ErrUnexpectedEOF
			}
			iNdEx += skippy
		}
	}

	if iNdEx > l {
		return io.ErrUnexpectedEOF
	}
	return nil
}

func (m *Params) Unmarshal(dAtA []byte) error {
	l := len(dAtA)
	iNdEx := 0
	for iNdEx < l {
		preIndex := iNdEx
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return ErrIntOverflowTypes
			}
			if iNdEx >= l {
				return io.ErrUnexpectedEOF
			}
			b := dAtA[iNdEx]
			iNdEx++
			wire |= uint64(b&0x7F) << shift
			if b < 0x80 {
				break
			}
		}
		fieldNum := int32(wire >> 3)
		wireType := int(wire & 0x7)
		if wireType == 4 {
			return fmt.Errorf("proto: Params: wiretype end group for non-group")
		}
		if fieldNum <= 0 {
			return fmt.Errorf("proto: Params: illegal tag %d (wire type %d)", fieldNum, wire)
		}
		switch fieldNum {
		case 1:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field CodeUploadAccess", wireType)
			}
			var msglen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowTypes
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				msglen |= int(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if msglen < 0 {
				return ErrInvalidLengthTypes
			}
			postIndex := iNdEx + msglen
			if postIndex < 0 {
				return ErrInvalidLengthTypes
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			if err := m.CodeUploadAccess.Unmarshal(dAtA[iNdEx:postIndex]); err != nil {
				return err
			}
			iNdEx = postIndex
		case 2:
			if wireType != 0 {
				return fmt.Errorf("proto: wrong wireType = %d for field InstantiateDefaultPermission", wireType)
			}
			m.InstantiateDefaultPermission = 0
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowTypes
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				m.InstantiateDefaultPermission |= AccessType(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
		default:
			iNdEx = preIndex
			skippy, err := skipTypes(dAtA[iNdEx:])
			if err != nil {
				return err
			}
			if (skippy < 0) || (iNdEx+skippy) < 0 {
				return ErrInvalidLengthTypes
			}
			if (iNdEx + skippy) > l {
				return io.ErrUnexpectedEOF
			}
			iNdEx += skippy
		}
	}

	if iNdEx > l {
		return io.ErrUnexpectedEOF
	}
	return nil
}

func (m *CodeInfo) Unmarshal(dAtA []byte) error {
	l := len(dAtA)
	iNdEx := 0
	for iNdEx < l {
		preIndex := iNdEx
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return ErrIntOverflowTypes
			}
			if iNdEx >= l {
				return io.ErrUnexpectedEOF
			}
			b := dAtA[iNdEx]
			iNdEx++
			wire |= uint64(b&0x7F) << shift
			if b < 0x80 {
				break
			}
		}
		fieldNum := int32(wire >> 3)
		wireType := int(wire & 0x7)
		if wireType == 4 {
			return fmt.Errorf("proto: CodeInfo: wiretype end group for non-group")
		}
		if fieldNum <= 0 {
			return fmt.Errorf("proto: CodeInfo: illegal tag %d (wire type %d)", fieldNum, wire)
		}
		switch fieldNum {
		case 1:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field CodeHash", wireType)
			}
			var byteLen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowTypes
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				byteLen |= int(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if byteLen < 0 {
				return ErrInvalidLengthTypes
			}
			postIndex := iNdEx + byteLen
			if postIndex < 0 {
				return ErrInvalidLengthTypes
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			m.CodeHash = append(m.CodeHash[:0], dAtA[iNdEx:postIndex]...)
			if m.CodeHash == nil {
				m.CodeHash = []byte{}
			}
			iNdEx = postIndex
		case 2:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field Creator", wireType)
			}
			var stringLen uint64
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowTypes
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				stringLen |= uint64(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			intStringLen := int(stringLen)
			if intStringLen < 0 {
				return ErrInvalidLengthTypes
			}
			postIndex := iNdEx + intStringLen
			if postIndex < 0 {
				return ErrInvalidLengthTypes
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			m.Creator = string(dAtA[iNdEx:postIndex])
			iNdEx = postIndex
		case 5:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field InstantiateConfig", wireType)
			}
			var msglen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowTypes
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				msglen |= int(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if msglen < 0 {
				return ErrInvalidLengthTypes
			}
			postIndex := iNdEx + msglen
			if postIndex < 0 {
				return ErrInvalidLengthTypes
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			if err := m.InstantiateConfig.Unmarshal(dAtA[iNdEx:postIndex]); err != nil {
				return err
			}
			iNdEx = postIndex
		default:
			iNdEx = preIndex
			skippy, err := skipTypes(dAtA[iNdEx:])
			if err != nil {
				return err
			}
			if (skippy < 0) || (iNdEx+skippy) < 0 {
				return ErrInvalidLengthTypes
			}
			if (iNdEx + skippy) > l {
				return io.ErrUnexpectedEOF
			}
			iNdEx += skippy
		}
	}

	if iNdEx > l {
		return io.ErrUnexpectedEOF
	}
	return nil
}

func skipTypes(dAtA []byte) (n int, err error) {
	l := len(dAtA)
	iNdEx := 0
	depth := 0
	for iNdEx < l {
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return 0, ErrIntOverflowTypes
			}
			if iNdEx >= l {
				return 0, io.ErrUnexpectedEOF
			}
			b := dAtA[iNdEx]
			iNdEx++
			wire |= (uint64(b) & 0x7F) << shift
			if b < 0x80 {
				break
			}
		}
		wireType := int(wire & 0x7)
		switch wireType {
		case 0:
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return 0, ErrIntOverflowTypes
				}
				if iNdEx >= l {
					return 0, io.ErrUnexpectedEOF
				}
				iNdEx++
				if dAtA[iNdEx-1] < 0x80 {
					break
				}
			}
		case 1:
			iNdEx += 8
		case 2:
			var length int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return 0, ErrIntOverflowTypes
				}
				if iNdEx >= l {
					return 0, io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				length |= (int(b) & 0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if length < 0 {
				return 0, ErrInvalidLengthTypes
			}
			iNdEx += length
		case 3:
			depth++
		case 4:
			if depth == 0 {
				return 0, ErrUnexpectedEndOfGroupTypes
			}
			depth--
		case 5:
			iNdEx += 4
		default:
			return 0, fmt.Errorf("proto: illegal wireType %d", wireType)
		}
		if iNdEx < 0 {
			return 0, ErrInvalidLengthTypes
		}
		if depth == 0 {
			return iNdEx, nil
		}
	}
	return 0, io.ErrUnexpectedEOF
}

var (
	ErrInvalidLengthTypes        = fmt.Errorf("proto: negative length found during unmarshaling")
	ErrIntOverflowTypes          = fmt.Errorf("proto: integer overflow")
	ErrUnexpectedEndOfGroupTypes = fmt.Errorf("proto: unexpected end of group")
)
