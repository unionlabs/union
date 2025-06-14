package main

import (
	"log"
	"os"
	"os/signal"
	"syscall"

	"websocket-backend/internal/config"
	"websocket-backend/internal/graphql"
	"websocket-backend/internal/ws"
)

func main() {
	// Load configuration
	cfg := config.New()

	// Create GraphQL client
	client := graphql.New(cfg.GraphQLEndpoint)

	// Create WebSocket server
	server := ws.NewServer(cfg, client)

	// Handle graceful shutdown
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		<-sigChan
		log.Println("[SERVER] Received shutdown signal, shutting down gracefully")
		server.Shutdown()
		os.Exit(0)
	}()

	// Start the server
	server.Run()
} 