package cmd

import (
	"context"
	"fmt"
	provergrpc "github.com/unionlabs/union/galoisd/grpc/api/v3"
	"log"
	"os"

	"github.com/spf13/cobra"
)

const (
	flagPath = "path"
)

func GenContract() *cobra.Command {
	var cmd = &cobra.Command{
		Short: "Generate a solidity verifier. Note that the output require further manual modifications before being usable",
		Use:   "gen-contract [uri]",
		Args:  cobra.ExactArgs(1),
		RunE: MakeCobra(func(ctx context.Context, client provergrpc.UnionProverAPIClient, cmd *cobra.Command, args []string) error {
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
		}),
	}
	cmd.Flags().String(flagPath, "", "Path were to write the file. If empty, dump to stdout.")
	cmd.Flags().String(flagTLS, "", "Wether the gRPC endpoint expect TLS.")
	return cmd
}
