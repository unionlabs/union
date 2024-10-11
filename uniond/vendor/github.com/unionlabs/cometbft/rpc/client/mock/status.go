package mock

import (
	"context"

	"github.com/cometbft/cometbft/rpc/client"
	ctypes "github.com/cometbft/cometbft/rpc/core/types"
)

// StatusMock returns the result specified by the Call.
type StatusMock struct {
	Call
}

var (
	_ client.StatusClient = (*StatusMock)(nil)
	_ client.StatusClient = (*StatusRecorder)(nil)
)

func (m *StatusMock) Status(context.Context) (*ctypes.ResultStatus, error) {
	res, err := m.GetResponse(nil)
	if err != nil {
		return nil, err
	}
	return res.(*ctypes.ResultStatus), nil
}

// StatusRecorder can wrap another type (StatusMock, full client)
// and record the status calls.
type StatusRecorder struct {
	Client client.StatusClient
	Calls  []Call
}

func NewStatusRecorder(client client.StatusClient) *StatusRecorder {
	return &StatusRecorder{
		Client: client,
		Calls:  []Call{},
	}
}

func (r *StatusRecorder) addCall(call Call) {
	r.Calls = append(r.Calls, call)
}

func (r *StatusRecorder) Status(ctx context.Context) (*ctypes.ResultStatus, error) {
	res, err := r.Client.Status(ctx)
	r.addCall(Call{
		Name:     "status",
		Response: res,
		Error:    err,
	})
	return res, err
}
