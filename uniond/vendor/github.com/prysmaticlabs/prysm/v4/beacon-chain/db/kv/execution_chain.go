package kv

import (
	"context"
	"errors"

	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	v2 "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	bolt "go.etcd.io/bbolt"
	"go.opencensus.io/trace"
	"google.golang.org/protobuf/proto"
)

// SaveExecutionChainData saves the execution chain data.
func (s *Store) SaveExecutionChainData(ctx context.Context, data *v2.ETH1ChainData) error {
	_, span := trace.StartSpan(ctx, "BeaconDB.SaveExecutionChainData")
	defer span.End()

	if data == nil {
		err := errors.New("cannot save nil eth1data")
		tracing.AnnotateError(span, err)
		return err
	}

	err := s.db.Update(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(powchainBucket)
		enc, err := proto.Marshal(data)
		if err != nil {
			return err
		}
		return bkt.Put(powchainDataKey, enc)
	})
	tracing.AnnotateError(span, err)
	return err
}

// ExecutionChainData retrieves the execution chain data.
func (s *Store) ExecutionChainData(ctx context.Context) (*v2.ETH1ChainData, error) {
	_, span := trace.StartSpan(ctx, "BeaconDB.ExecutionChainData")
	defer span.End()

	var data *v2.ETH1ChainData
	err := s.db.View(func(tx *bolt.Tx) error {
		bkt := tx.Bucket(powchainBucket)
		enc := bkt.Get(powchainDataKey)
		if len(enc) == 0 {
			return nil
		}
		data = &v2.ETH1ChainData{}
		return proto.Unmarshal(enc, data)
	})
	return data, err
}
