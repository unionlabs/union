package testdata

import (
	"context"
)

type MsgServerImpl struct{}

var _ MsgServer = MsgServerImpl{}

// CreateDog implements the MsgServer interface.
func (m MsgServerImpl) CreateDog(_ context.Context, msg *MsgCreateDog) (*MsgCreateDogResponse, error) {
	return &MsgCreateDogResponse{
		Name: msg.Dog.Name,
	}, nil
}
