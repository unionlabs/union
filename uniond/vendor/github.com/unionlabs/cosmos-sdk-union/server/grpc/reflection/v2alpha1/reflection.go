package v2alpha1

import (
	"context"
	"errors"
	"fmt"

	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx"
)

type Config struct {
	SigningModes      map[string]int32
	ChainID           string
	InterfaceRegistry codectypes.InterfaceRegistry
}

// Register registers the cosmos sdk reflection service
// to the provided *grpc.Server given a Config
func Register(srv *grpc.Server, conf Config) error {
	reflectionServer, err := newReflectionServiceServer(srv, conf)
	if err != nil {
		return err
	}
	RegisterReflectionServiceServer(srv, reflectionServer)
	return nil
}

type reflectionServiceServer struct {
	desc *AppDescriptor
}

func (r reflectionServiceServer) GetAuthnDescriptor(_ context.Context, _ *GetAuthnDescriptorRequest) (*GetAuthnDescriptorResponse, error) {
	return &GetAuthnDescriptorResponse{Authn: r.desc.Authn}, nil
}

func (r reflectionServiceServer) GetChainDescriptor(_ context.Context, _ *GetChainDescriptorRequest) (*GetChainDescriptorResponse, error) {
	return &GetChainDescriptorResponse{Chain: r.desc.Chain}, nil
}

func (r reflectionServiceServer) GetCodecDescriptor(_ context.Context, _ *GetCodecDescriptorRequest) (*GetCodecDescriptorResponse, error) {
	return &GetCodecDescriptorResponse{Codec: r.desc.Codec}, nil
}

func (r reflectionServiceServer) GetConfigurationDescriptor(_ context.Context, _ *GetConfigurationDescriptorRequest) (*GetConfigurationDescriptorResponse, error) {
	return nil, errors.New("this endpoint has been deprecated, please see auth/Bech32Prefix for the data you are seeking")
}

func (r reflectionServiceServer) GetQueryServicesDescriptor(_ context.Context, _ *GetQueryServicesDescriptorRequest) (*GetQueryServicesDescriptorResponse, error) {
	return &GetQueryServicesDescriptorResponse{Queries: r.desc.QueryServices}, nil
}

func (r reflectionServiceServer) GetTxDescriptor(_ context.Context, _ *GetTxDescriptorRequest) (*GetTxDescriptorResponse, error) {
	return &GetTxDescriptorResponse{Tx: r.desc.Tx}, nil
}

func newReflectionServiceServer(grpcSrv *grpc.Server, conf Config) (reflectionServiceServer, error) {
	// set chain descriptor
	chainDescriptor := &ChainDescriptor{Id: conf.ChainID}

	// set codec descriptor
	codecDescriptor, err := newCodecDescriptor(conf.InterfaceRegistry)
	if err != nil {
		return reflectionServiceServer{}, fmt.Errorf("unable to create codec descriptor: %w", err)
	}
	// set query service descriptor
	queryServiceDescriptor := newQueryServiceDescriptor(grpcSrv)
	// set deliver descriptor
	txDescriptor, err := newTxDescriptor(conf.InterfaceRegistry)
	if err != nil {
		return reflectionServiceServer{}, fmt.Errorf("unable to create deliver descriptor: %w", err)
	}
	authnDescriptor := newAuthnDescriptor(conf.SigningModes)
	desc := &AppDescriptor{
		Authn:         authnDescriptor,
		Chain:         chainDescriptor,
		Codec:         codecDescriptor,
		QueryServices: queryServiceDescriptor,
		Tx:            txDescriptor,
	}

	ifaceList := make([]string, len(desc.Codec.Interfaces))
	ifaceImplementers := make(map[string][]string, len(desc.Codec.Interfaces))
	for i, iface := range desc.Codec.Interfaces {
		ifaceList[i] = iface.Fullname
		impls := make([]string, len(iface.InterfaceImplementers))
		for j, impl := range iface.InterfaceImplementers {
			impls[j] = impl.TypeUrl
		}
		ifaceImplementers[iface.Fullname] = impls
	}
	return reflectionServiceServer{
		desc: desc,
	}, nil
}

// newCodecDescriptor describes the codec given the codectypes.InterfaceRegistry
func newCodecDescriptor(ir codectypes.InterfaceRegistry) (*CodecDescriptor, error) {
	registeredInterfaces := ir.ListAllInterfaces()
	interfaceDescriptors := make([]*InterfaceDescriptor, len(registeredInterfaces))

	for i, iface := range registeredInterfaces {
		implementers := ir.ListImplementations(iface)
		interfaceImplementers := make([]*InterfaceImplementerDescriptor, len(implementers))
		for j, implementer := range implementers {
			pb, err := ir.Resolve(implementer)
			if err != nil {
				return nil, fmt.Errorf("unable to resolve implementing type %s for interface %s", implementer, iface)
			}
			pbName := proto.MessageName(pb)
			if pbName == "" {
				return nil, fmt.Errorf("unable to get proto name for implementing type %s for interface %s", implementer, iface)
			}
			interfaceImplementers[j] = &InterfaceImplementerDescriptor{
				Fullname: pbName,
				TypeUrl:  implementer,
			}
		}
		interfaceDescriptors[i] = &InterfaceDescriptor{
			Fullname: iface,
			// NOTE(fdymylja): this could be filled, but it won't be filled as of now
			// doing this would require us to fully rebuild in a (dependency) transitive way the proto
			// registry of the supported proto.Messages for the application, this could be easily
			// done if we weren't relying on gogoproto which does not allow us to iterate over the
			// registry. Achieving this right now would mean to start slowly building descriptors
			// getting their files dependencies, building those dependencies then rebuilding the
			// descriptor builder. It's too much work as of now.
			InterfaceAcceptingMessages: nil,
			InterfaceImplementers:      interfaceImplementers,
		}
	}

	return &CodecDescriptor{
		Interfaces: interfaceDescriptors,
	}, nil
}

func newQueryServiceDescriptor(srv *grpc.Server) *QueryServicesDescriptor {
	svcInfo := srv.GetServiceInfo()
	queryServices := make([]*QueryServiceDescriptor, 0, len(svcInfo))
	for name, info := range svcInfo {
		methods := make([]*QueryMethodDescriptor, len(info.Methods))
		for i, svcMethod := range info.Methods {
			methods[i] = &QueryMethodDescriptor{
				Name:          svcMethod.Name,
				FullQueryPath: fmt.Sprintf("/%s/%s", name, svcMethod.Name),
			}
		}
		queryServices = append(queryServices, &QueryServiceDescriptor{
			Fullname: name,
			Methods:  methods,
		})
	}
	return &QueryServicesDescriptor{QueryServices: queryServices}
}

func newTxDescriptor(ir codectypes.InterfaceRegistry) (*TxDescriptor, error) {
	// get base tx type name
	txPbName := proto.MessageName(&tx.Tx{})
	if txPbName == "" {
		return nil, fmt.Errorf("unable to get *tx.Tx protobuf name")
	}
	// get msgs
	sdkMsgImplementers := ir.ListImplementations(sdk.MsgInterfaceProtoName)

	msgsDesc := make([]*MsgDescriptor, 0, len(sdkMsgImplementers))

	// process sdk.Msg
	for _, msgTypeURL := range sdkMsgImplementers {
		msgsDesc = append(msgsDesc, &MsgDescriptor{
			MsgTypeUrl: msgTypeURL,
		})
	}

	return &TxDescriptor{
		Fullname: txPbName,
		Msgs:     msgsDesc,
	}, nil
}

func newAuthnDescriptor(signingModes map[string]int32) *AuthnDescriptor {
	signModesDesc := make([]*SigningModeDescriptor, 0, len(signingModes))
	for i, m := range signingModes {
		signModesDesc = append(signModesDesc, &SigningModeDescriptor{
			Name:   i,
			Number: m,
			// NOTE(fdymylja): this cannot be filled as of now, auth and the sdk itself don't support as of now
			// a service which allows to get authentication metadata for the provided sign mode.
			AuthnInfoProviderMethodFullname: "",
		})
	}
	return &AuthnDescriptor{SignModes: signModesDesc}
}
