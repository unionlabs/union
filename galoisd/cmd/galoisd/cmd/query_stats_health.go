package cmd

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"sync/atomic"

	provergrpc "galois/grpc/api/v3"

	"github.com/spf13/cobra"
)

func QueryStatsHealth() *cobra.Command {
	var cmd = &cobra.Command{
		Short: "Service which query circuit statistics and expose a health endpoint based on the results",
		Use:   "query-stats-health [uri]",
		Args:  cobra.ExactArgs(1),
		RunE: MakeCobra(func(ctx context.Context, client provergrpc.UnionProverAPIClient, cmd *cobra.Command, args []string) error {
			var status int32 = 200

			// Function to query stats and update status
			updateStatus := func() {
				res, err := client.QueryStats(ctx, &provergrpc.QueryStatsRequest{})
				if err != nil {
					log.Println("Error querying stats:", err)
					atomic.StoreInt32(&status, 500)
					return
				}
				bz, err := json.Marshal(res)
				if err != nil {
					log.Println("Error marshaling response:", err)
					atomic.StoreInt32(&status, 500)
					return
				}
				fmt.Println(string(bz))
				atomic.StoreInt32(&status, 200)
			}

			// HTTP server
			http.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
				updateStatus()
				if atomic.LoadInt32(&status) == 200 {
					w.WriteHeader(http.StatusOK)
					w.Write([]byte("Healthy"))
				} else {
					w.WriteHeader(http.StatusInternalServerError)
					w.Write([]byte("Unhealthy"))
				}
			})

			server := &http.Server{Addr: ":9999"}
			go func() {
				if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
					log.Fatalf("Could not listen on :9999: %v\n", err)
				}
			}()

			// Wait for context cancellation
			<-ctx.Done()
			if err := server.Shutdown(context.Background()); err != nil {
				log.Fatalf("HTTP server Shutdown: %v", err)
			}

			return nil
		}),
	}
	cmd.Flags().String(flagTLS, "", "Whether the gRPC endpoint expects TLS.")
	return cmd
}
