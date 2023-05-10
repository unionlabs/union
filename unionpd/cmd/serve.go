package cmd

import (
	provergrpc "cometbls-prover/grpc/api/v1"
	"fmt"
	"github.com/spf13/cobra"
	"google.golang.org/grpc"
	"log"
	"net"
)

var ServeCmd = &cobra.Command{
	Use:   "serve",
	Short: "",
	Long:  ``,
	Run: func(cmd *cobra.Command, args []string) {
		lis, err := net.Listen("tcp", fmt.Sprintf("localhost:%d", 9091))
		if err != nil {
			log.Fatalf("failed to listen: %v", err)
		}
		var opts []grpc.ServerOption
		grpcServer := grpc.NewServer(opts...)
		server, err := provergrpc.NewProverServer()
		if err != nil {
			panic(err)
		}
		provergrpc.RegisterUnionProverAPIServer(grpcServer, server)
		fmt.Println("Serving...")
		grpcServer.Serve(lis)
	},
}
