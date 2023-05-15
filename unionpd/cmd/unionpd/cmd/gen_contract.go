package cmd

import (
	"context"
	"fmt"
	"io/ioutil"
	"log"
	"time"
	provergrpc "unionp/grpc/api/v1"

	"github.com/spf13/cobra"
	"google.golang.org/grpc"
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
			uri := args[0]
			conn, err := grpc.Dial(uri, grpc.WithTransportCredentials(insecure.NewCredentials()))
			if err != nil {
				log.Fatalf("Failed to dial: %v", err)
			}
			defer conn.Close()
			client := provergrpc.NewUnionProverAPIClient(conn)
			ctx, cancel := context.WithTimeout(context.Background(), 1*time.Hour)
			defer cancel()

			res, err := client.GenerateContract(ctx, &provergrpc.GenerateContractRequest{})
			if err != nil {
				return err
			}

			path, err := cmd.Flags().GetString(flagPath)
			if err != nil {
				return err
			}

			if path == "" {
				fmt.Print(string(res.Content))
			} else {
				err := ioutil.WriteFile(path, res.Content, 0)
				if err != nil {
					return err
				}
			}

			return nil
		},
	}
	cmd.Flags().String(flagPath, "", "Path were to write the file. If empty, dump to stdout.")
	return cmd
}
