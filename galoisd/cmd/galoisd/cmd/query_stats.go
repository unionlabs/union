package cmd

import (
	"context"
	"encoding/json"
	"fmt"
	provergrpc "github.com/unionlabs/union/galoisd/grpc/api/v3"
	"log"

	"github.com/spf13/cobra"
)

func QueryStats() *cobra.Command {
	var cmd = &cobra.Command{
		Short: "Query various circuit statistics (constraints, nb of public/private witnesses etc...)",
		Use:   "query-stats [uri]",
		Args:  cobra.ExactArgs(1),
		RunE: MakeCobra(func(ctx context.Context, client provergrpc.UnionProverAPIClient, cmd *cobra.Command, args []string) error {
			res, err := client.QueryStats(ctx, &provergrpc.QueryStatsRequest{})
			if err != nil {
				log.Fatal(err)
			}
			bz, err := json.Marshal(res)
			if err != nil {
				log.Fatal(err)
			}
			fmt.Println(string(bz))
			return nil
		}),
	}
	cmd.Flags().String(flagTLS, "", "Wether the gRPC endpoint expect TLS.")
	return cmd
}
