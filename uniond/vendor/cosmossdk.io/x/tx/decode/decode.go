package decode

import (
	"crypto/sha256"
	"errors"
	"fmt"
	"reflect"
	"strings"

	gogoproto "github.com/cosmos/gogoproto/proto"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/dynamicpb"

	v1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	"cosmossdk.io/core/transaction"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/x/tx/signing"
)

// DecodedTx contains the decoded transaction, its signers, and other flags.
type DecodedTx struct {
	DynamicMessages              []proto.Message
	Messages                     []gogoproto.Message
	Tx                           *v1beta1.Tx
	TxRaw                        *v1beta1.TxRaw
	Signers                      [][]byte
	TxBodyHasUnknownNonCriticals bool

	// Cache for hash and full bytes
	cachedHash   [32]byte
	cachedBytes  []byte
	cachedHashed bool
}

var _ transaction.Tx = &DecodedTx{}

type gogoProtoCodec interface {
	Unmarshal([]byte, gogoproto.Message) error
}

// Decoder contains the dependencies required for decoding transactions.
type Decoder struct {
	signingCtx *signing.Context
	codec      gogoProtoCodec
}

// Options are options for creating a Decoder.
type Options struct {
	SigningContext *signing.Context
	ProtoCodec     gogoProtoCodec
}

// NewDecoder creates a new Decoder for decoding transactions.
func NewDecoder(options Options) (*Decoder, error) {
	if options.SigningContext == nil {
		return nil, errors.New("signing context is required")
	}
	if options.ProtoCodec == nil {
		return nil, errors.New("proto codec is required for unmarshalling gogoproto messages")
	}
	return &Decoder{
		signingCtx: options.SigningContext,
		codec:      options.ProtoCodec,
	}, nil
}

// Decode decodes raw protobuf encoded transaction bytes into a DecodedTx.
func (d *Decoder) Decode(txBytes []byte) (*DecodedTx, error) {
	// Make sure txBytes follow ADR-027.
	err := rejectNonADR027TxRaw(txBytes)
	if err != nil {
		return nil, errorsmod.Wrap(ErrTxDecode, err.Error())
	}

	var raw v1beta1.TxRaw

	// reject all unknown proto fields in the root TxRaw
	fileResolver := d.signingCtx.FileResolver()
	err = RejectUnknownFieldsStrict(txBytes, raw.ProtoReflect().Descriptor(), fileResolver)
	if err != nil {
		return nil, errorsmod.Wrap(ErrTxDecode, err.Error())
	}

	err = proto.Unmarshal(txBytes, &raw)
	if err != nil {
		return nil, err
	}

	var body v1beta1.TxBody

	// allow non-critical unknown fields in TxBody
	txBodyHasUnknownNonCriticals, err := RejectUnknownFields(raw.BodyBytes, body.ProtoReflect().Descriptor(), true, fileResolver)
	if err != nil {
		return nil, errorsmod.Wrap(ErrTxDecode, err.Error())
	}

	err = proto.Unmarshal(raw.BodyBytes, &body)
	if err != nil {
		return nil, errorsmod.Wrap(ErrTxDecode, err.Error())
	}

	var authInfo v1beta1.AuthInfo

	// reject all unknown proto fields in AuthInfo
	err = RejectUnknownFieldsStrict(raw.AuthInfoBytes, authInfo.ProtoReflect().Descriptor(), fileResolver)
	if err != nil {
		return nil, errorsmod.Wrap(ErrTxDecode, err.Error())
	}

	err = proto.Unmarshal(raw.AuthInfoBytes, &authInfo)
	if err != nil {
		return nil, errorsmod.Wrap(ErrTxDecode, err.Error())
	}

	theTx := &v1beta1.Tx{
		Body:       &body,
		AuthInfo:   &authInfo,
		Signatures: raw.Signatures,
	}

	var (
		signers     [][]byte
		dynamicMsgs []proto.Message
		msgs        []gogoproto.Message
	)
	seenSigners := map[string]struct{}{}
	for _, anyMsg := range body.Messages {
		typeURL := strings.TrimPrefix(anyMsg.TypeUrl, "/")

		// unmarshal into dynamic message
		msgDesc, err := fileResolver.FindDescriptorByName(protoreflect.FullName(typeURL))
		if err != nil {
			return nil, fmt.Errorf("protoFiles does not have descriptor %s: %w", anyMsg.TypeUrl, err)
		}
		dynamicMsg := dynamicpb.NewMessageType(msgDesc.(protoreflect.MessageDescriptor)).New().Interface()
		err = anyMsg.UnmarshalTo(dynamicMsg)
		if err != nil {
			return nil, err
		}
		dynamicMsgs = append(dynamicMsgs, dynamicMsg)

		// unmarshal into gogoproto message
		gogoType := gogoproto.MessageType(typeURL)
		if gogoType == nil {
			return nil, fmt.Errorf("cannot find type: %s", anyMsg.TypeUrl)
		}
		msg := reflect.New(gogoType.Elem()).Interface().(gogoproto.Message)
		err = d.codec.Unmarshal(anyMsg.Value, msg)
		if err != nil {
			return nil, err
		}
		msgs = append(msgs, msg)

		// fetch signers with dynamic message
		ss, signerErr := d.signingCtx.GetSigners(dynamicMsg)
		if signerErr != nil {
			return nil, errorsmod.Wrap(ErrTxDecode, signerErr.Error())
		}
		for _, s := range ss {
			_, seen := seenSigners[string(s)]
			if seen {
				continue
			}
			signers = append(signers, s)
			seenSigners[string(s)] = struct{}{}
		}
	}

	return &DecodedTx{
		Messages:                     msgs,
		DynamicMessages:              dynamicMsgs,
		Tx:                           theTx,
		TxRaw:                        &raw,
		TxBodyHasUnknownNonCriticals: txBodyHasUnknownNonCriticals,
		Signers:                      signers,
	}, nil
}

// Hash implements the interface for the Tx interface.
func (dtx *DecodedTx) Hash() [32]byte {
	if !dtx.cachedHashed {
		dtx.computeHashAndBytes()
	}
	return dtx.cachedHash
}

func (dtx *DecodedTx) GetGasLimit() (uint64, error) {
	if dtx == nil || dtx.Tx == nil || dtx.Tx.AuthInfo == nil || dtx.Tx.AuthInfo.Fee == nil {
		return 0, errors.New("gas limit not available or one or more required fields are nil")
	}
	return dtx.Tx.AuthInfo.Fee.GasLimit, nil
}

func (dtx *DecodedTx) GetMessages() ([]transaction.Msg, error) {
	if dtx == nil || dtx.Messages == nil {
		return nil, errors.New("messages not available or are nil")
	}

	return dtx.Messages, nil
}

func (dtx *DecodedTx) GetSenders() ([][]byte, error) {
	if dtx == nil || dtx.Signers == nil {
		return nil, errors.New("senders not available or are nil")
	}
	return dtx.Signers, nil
}

func (dtx *DecodedTx) Bytes() []byte {
	if !dtx.cachedHashed {
		dtx.computeHashAndBytes()
	}
	return dtx.cachedBytes
}

func (dtx *DecodedTx) computeHashAndBytes() {
	bz, err := proto.Marshal(dtx.TxRaw)
	if err != nil {
		panic(err)
	}

	dtx.cachedBytes = bz
	dtx.cachedHash = sha256.Sum256(bz)
	dtx.cachedHashed = true
}
