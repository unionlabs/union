package cmd

import (
	"context"
	"crypto/tls"
	"fmt"
	"log"
	"os"
	"time"
	provergrpc "unionp/grpc/api/v1"

	"github.com/spf13/cobra"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
)

const (
	flagPath = "path"
)

func GenContract() *cobra.Command {
	var cmd = &cobra.Command{
		Use:  "gen-contract [uri]",
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			tlsEnabled, err := cmd.Flags().GetString(flagTLS)
			if err != nil {
				log.Fatal(err)
			}

			var creds credentials.TransportCredentials
			if tlsEnabled == "yes" {
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

			res, err := client.GenerateContract(ctx, &provergrpc.GenerateContractRequest{})
			if err != nil {
				log.Fatal(err)
			}

			path, err := cmd.Flags().GetString(flagPath)
			if err != nil {
				log.Fatal(err)
			}

			if path == "" {
				fmt.Print(string(res.Content))
			} else {
				err := os.WriteFile(path, res.Content, 0644)
				if err != nil {
					log.Fatal(err)
				}
			}

			return nil
		},
	}
	cmd.Flags().String(flagPath, "", "Path were to write the file. If empty, dump to stdout.")
	cmd.Flags().String(flagTLS, "", "Wether the gRPC endpoint expect TLS.")
	return cmd
}
