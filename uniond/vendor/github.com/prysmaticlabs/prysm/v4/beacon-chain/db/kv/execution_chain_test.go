package kv

import (
	"context"
	"testing"

	v2 "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

func TestStore_SavePowchainData(t *testing.T) {
	type args struct {
		data *v2.ETH1ChainData
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "nil data",
			args: args{
				data: nil,
			},
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			store := setupDB(t)
			if err := store.SaveExecutionChainData(context.Background(), tt.args.data); (err != nil) != tt.wantErr {
				t.Errorf("SaveExecutionChainData() error = %v, wantErr %v", err, tt.wantErr)
			}
		})
	}
}
