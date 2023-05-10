package cmd

import (
	provergrpc "cometbls-prover/grpc/api/v1"
	"github.com/spf13/cobra"
	"google.golang.org/grpc"
	"log"
	"net"
)

const (
	flagR1CS = "cs-path"
	flagPK   = "pk-path"
	flagVK   = "vk-path"
)

func ServeCmd() *cobra.Command {
	var cmd = &cobra.Command{
		Use:  "serve [uri]",
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			uri := args[0]
			lis, err := net.Listen("tcp", uri)
			if err != nil {
				return err
			}
			var opts []grpc.ServerOption
			grpcServer := grpc.NewServer(opts...)
			r1csPath, err := cmd.Flags().GetString(flagR1CS)
			if err != nil {
				return err
			}
			pkPath, err := cmd.Flags().GetString(flagPK)
			if err != nil {
				return err
			}
			vkPath, err := cmd.Flags().GetString(flagVK)
			if err != nil {
				return err
			}
			server, err := provergrpc.NewProverServer(r1csPath, pkPath, vkPath)
			if err != nil {
				return err
			}
			provergrpc.RegisterUnionProverAPIServer(grpcServer, server)
			log.Println("Serving...")
			return grpcServer.Serve(lis)
		},
	}
	cmd.Flags().String(flagR1CS, "r1cs.bin", "Path to the compiled R1CS circuit.")
	cmd.Flags().String(flagPK, "pk.bin", "Path to the proving key.")
	cmd.Flags().String(flagVK, "vk.bin", "Path to the verifying key.")
	return cmd
}
