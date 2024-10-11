package p2p

import (
	"context"
	"fmt"
	"sync"
	"testing"
	"time"

	"github.com/pkg/errors"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/encoder"
	testp2p "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestService_PublishToTopicConcurrentMapWrite(t *testing.T) {
	cs := startup.NewClockSynchronizer()
	s, err := NewService(context.Background(), &Config{
		StateNotifier: &mock.MockStateNotifier{},
		ClockWaiter:   cs,
	})
	require.NoError(t, err)
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	go s.awaitStateInitialized()
	fd := initializeStateWithForkDigest(ctx, t, cs)

	if !s.isInitialized() {
		t.Fatal("service was not initialized")
	}

	// Set up two connected test hosts.
	p0 := testp2p.NewTestP2P(t)
	p1 := testp2p.NewTestP2P(t)
	p0.Connect(p1)
	s.host = p0.BHost
	s.pubsub = p0.PubSub()

	topic := fmt.Sprintf(BlockSubnetTopicFormat, fd) + "/" + encoder.ProtocolSuffixSSZSnappy

	// Establish the remote peer to be subscribed to the outgoing topic.
	_, err = p1.SubscribeToTopic(topic)
	require.NoError(t, err)

	wg := sync.WaitGroup{}
	wg.Add(10)
	for i := 0; i < 10; i++ {
		go func(i int) {
			assert.NoError(t, s.PublishToTopic(ctx, topic, []byte{}))
			wg.Done()
		}(i)
	}
	wg.Wait()
}

func TestExtractGossipDigest(t *testing.T) {
	tests := []struct {
		name    string
		topic   string
		want    [4]byte
		wantErr bool
		error   error
	}{
		{
			name:    "empty topic",
			topic:   "",
			want:    [4]byte{},
			wantErr: true,
			error:   errors.New("invalid topic format"),
		},
		{
			name:    "too short topic",
			topic:   "/eth2/",
			want:    [4]byte{},
			wantErr: true,
			error:   errors.New("invalid topic format"),
		},
		{
			name:    "bogus topic prefix",
			topic:   "/eth3/b5303f2a/beacon_coin",
			want:    [4]byte{},
			wantErr: true,
			error:   errors.New("invalid topic format"),
		},
		{
			name:    "invalid digest in topic",
			topic:   "/eth2/zzxxyyaa/beacon_block" + "/" + encoder.ProtocolSuffixSSZSnappy,
			want:    [4]byte{},
			wantErr: true,
			error:   errors.New("encoding/hex: invalid byte"),
		},
		{
			name:    "short digest",
			topic:   fmt.Sprintf(BlockSubnetTopicFormat, []byte{0xb5, 0x30, 0x3f}) + "/" + encoder.ProtocolSuffixSSZSnappy,
			want:    [4]byte{},
			wantErr: true,
			error:   errors.New("invalid digest length wanted"),
		},
		{
			name:    "too short topic, missing suffixes",
			topic:   "/eth2/b5303f2a",
			want:    [4]byte{},
			wantErr: true,
			error:   errors.New("invalid topic format"),
		},
		{
			name:    "valid topic",
			topic:   fmt.Sprintf(BlockSubnetTopicFormat, []byte{0xb5, 0x30, 0x3f, 0x2a}) + "/" + encoder.ProtocolSuffixSSZSnappy,
			want:    [4]byte{0xb5, 0x30, 0x3f, 0x2a},
			wantErr: false,
			error:   nil,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := ExtractGossipDigest(tt.topic)
			assert.Equal(t, err != nil, tt.wantErr)
			if tt.wantErr {
				assert.ErrorContains(t, tt.error.Error(), err)
			}
			assert.DeepEqual(t, tt.want, got)
		})
	}
}

func BenchmarkExtractGossipDigest(b *testing.B) {
	topic := fmt.Sprintf(BlockSubnetTopicFormat, []byte{0xb5, 0x30, 0x3f, 0x2a}) + "/" + encoder.ProtocolSuffixSSZSnappy
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := ExtractGossipDigest(topic)
		if err != nil {
			b.Fatal(err)
		}
	}
}
