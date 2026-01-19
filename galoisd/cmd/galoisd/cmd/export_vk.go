package cmd

import (
	"context"
	"encoding/json"
	"fmt"
	provergrpc "galois/grpc/api/v3"
	"log"

	"github.com/spf13/cobra"
)

func ExportVk() *cobra.Command {
	var cmd = &cobra.Command{
		Short: "Export the verifying key",
		Use:   "export-vk [uri]",
		Args:  cobra.ExactArgs(1),
		RunE: MakeCobra(func(ctx context.Context, client provergrpc.UnionProverAPIClient, cmd *cobra.Command, args []string) error {
			res, err := client.ExportVk(ctx, &provergrpc.ExportVkRequest{})
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
	cmd.Flags().String(flagTLS, "", "Whether the gRPC endpoint expect TLS.")
	return cmd
}
