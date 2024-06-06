package cmd

import (
	"context"
	"crypto/tls"
	"github.com/spf13/cobra"
	provergrpc "github.com/unionlabs/union/galoisd/grpc/api/v3"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"log"
	"time"
)

const (
	flagTLS = "tls"
)

func MakeCobra(f func(context.Context, provergrpc.UnionProverAPIClient, *cobra.Command, []string) error) func(*cobra.Command, []string) error {
	return func(cmd *cobra.Command, args []string) error {
		tlsEnabled, err := cmd.Flags().GetString(flagTLS)
		if err != nil {
			log.Fatal(err)
		}
		var creds credentials.TransportCredentials
		if tlsEnabled == "yes" || tlsEnabled == "true" || tlsEnabled == "1" {
			creds = credentials.NewTLS(&tls.Config{})
		} else {
			creds = insecure.NewCredentials()
		}
		uri := args[0]
		conn, err := grpc.Dial(uri, grpc.WithTransportCredentials(creds))
		if err != nil {
			log.Fatal(err)
		}
		defer conn.Close()
		client := provergrpc.NewUnionProverAPIClient(conn)
		ctx, cancel := context.WithTimeout(context.Background(), 1*time.Hour)
		defer cancel()
		return f(ctx, client, cmd, args)
	}
}
