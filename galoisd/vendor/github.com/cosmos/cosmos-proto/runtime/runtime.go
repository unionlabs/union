package runtime

import (
	"fmt"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/runtime/protoiface"
	"io"
	"math/bits"
)

func Sov(x uint64) (n int) {
	return (bits.Len64(x|1) + 6) / 7
}
func Soz(x uint64) (n int) {
	return Sov((x << 1) ^ uint64(int64(x)>>63))
}

func EncodeVarint(dAtA []byte, offset int, v uint64) int {
	offset -= Sov(v)
	base := offset
	for v >= 1<<7 {
		dAtA[offset] = uint8(v&0x7f | 0x80)
		v >>= 7
		offset++
	}
	dAtA[offset] = uint8(v)
	return base
}

func Skip(dAtA []byte) (n int, err error) {
	l := len(dAtA)
	iNdEx := 0
	depth := 0
	for iNdEx < l {
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return 0, ErrIntOverflow
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
					return 0, ErrIntOverflow
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
					return 0, ErrIntOverflow
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
				return 0, ErrInvalidLength
			}
			iNdEx += length
		case 3:
			depth++
		case 4:
			if depth == 0 {
				return 0, ErrUnexpectedEndOfGroup
			}
			depth--
		case 5:
			iNdEx += 4
		default:
			return 0, fmt.Errorf("proto: illegal wireType %d", wireType)
		}
		if iNdEx < 0 {
			return 0, ErrInvalidLength
		}
		if depth == 0 {
			return iNdEx, nil
		}
	}
	return 0, io.ErrUnexpectedEOF
}

func SizeInputToOptions(input protoiface.SizeInput) proto.MarshalOptions {
	return proto.MarshalOptions{
		NoUnkeyedLiterals: input.NoUnkeyedLiterals,
		AllowPartial:      true,
		Deterministic:     input.Flags&protoiface.MarshalDeterministic != 0,
		UseCachedSize:     input.Flags&protoiface.MarshalUseCachedSize != 0,
	}
}

func MarshalInputToOptions(input protoiface.MarshalInput) proto.MarshalOptions {
	return proto.MarshalOptions{
		NoUnkeyedLiterals: input.NoUnkeyedLiterals,
		AllowPartial:      true, // defaults to true as the required fields check is done after the marshalling
		Deterministic:     input.Flags&protoiface.MarshalDeterministic != 0,
		UseCachedSize:     input.Flags&protoiface.MarshalUseCachedSize != 0,
	}
}

func UnmarshalInputToOptions(input protoiface.UnmarshalInput) proto.UnmarshalOptions {
	return proto.UnmarshalOptions{
		NoUnkeyedLiterals: input.NoUnkeyedLiterals,
		Merge:             false,
		AllowPartial:      true, // defaults to true as the required fields check is done after the unmarshalling
		DiscardUnknown:    input.Flags&protoiface.UnmarshalDiscardUnknown != 0,
		Resolver:          input.Resolver,
	}
}

var (
	ErrInvalidLength        = fmt.Errorf("proto: negative length found during unmarshaling")
	ErrIntOverflow          = fmt.Errorf("proto: integer overflow")
	ErrUnexpectedEndOfGroup = fmt.Errorf("proto: unexpected end of group")
)
