package proxy

import (
	"errors"
	"os"
	"os/signal"
	"syscall"
	"testing"
	"time"

	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"

	abcimocks "github.com/cometbft/cometbft/abci/client/mocks"
	"github.com/cometbft/cometbft/proxy/mocks"
)

func TestAppConns_Start_Stop(t *testing.T) {
	quitCh := make(<-chan struct{})

	clientCreatorMock := &mocks.ClientCreator{}

	clientMock := &abcimocks.Client{}
	clientMock.On("SetLogger", mock.Anything).Return().Times(4)
	clientMock.On("Start").Return(nil).Times(4)
	clientMock.On("Stop").Return(nil).Times(4)
	clientMock.On("Quit").Return(quitCh).Times(4)

	clientCreatorMock.On("NewABCIQueryClient").Return(clientMock, nil).Times(4)
	clientCreatorMock.On("NewABCIMempoolClient").Return(clientMock, nil).Times(4)
	clientCreatorMock.On("NewABCISnapshotClient").Return(clientMock, nil).Times(4)
	clientCreatorMock.On("NewABCIConsensusClient").Return(clientMock, nil).Times(4)

	appConns := NewAppConns(clientCreatorMock, NopMetrics())

	err := appConns.Start()
	require.NoError(t, err)

	time.Sleep(100 * time.Millisecond)

	err = appConns.Stop()
	require.NoError(t, err)

	clientMock.AssertExpectations(t)
}

// Upon failure, we call cmtos.Kill.
func TestAppConns_Failure(t *testing.T) {
	ok := make(chan struct{})
	c := make(chan os.Signal, 1)
	signal.Notify(c, syscall.SIGTERM)
	go func() {
		for range c {
			close(ok)
		}
	}()

	quitCh := make(chan struct{})
	var recvQuitCh <-chan struct{} //nolint:gosimple
	recvQuitCh = quitCh

	clientCreatorMock := &mocks.ClientCreator{}

	clientMock := &abcimocks.Client{}
	clientMock.On("SetLogger", mock.Anything).Return()
	clientMock.On("Start").Return(nil)
	clientMock.On("Stop").Return(nil)

	clientMock.On("Quit").Return(recvQuitCh)
	clientMock.On("Error").Return(errors.New("EOF")).Once()

	clientCreatorMock.On("NewABCIQueryClient").Return(clientMock, nil)
	clientCreatorMock.On("NewABCIMempoolClient").Return(clientMock, nil)
	clientCreatorMock.On("NewABCISnapshotClient").Return(clientMock, nil)
	clientCreatorMock.On("NewABCIConsensusClient").Return(clientMock, nil)

	appConns := NewAppConns(clientCreatorMock, NopMetrics())

	err := appConns.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := appConns.Stop(); err != nil {
			t.Error(err)
		}
	})

	// simulate failure
	close(quitCh)

	select {
	case <-ok:
		t.Log("SIGTERM successfully received")
	case <-time.After(5 * time.Second):
		t.Fatal("expected process to receive SIGTERM signal")
	}
}
