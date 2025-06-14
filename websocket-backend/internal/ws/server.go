package ws

import (
	"context"
	"fmt"
	"hash/fnv"
	"log"
	"net"
	"net/http"
	"os"
	"os/signal"
	"runtime"
	"sort"
	"sync"
	"sync/atomic"
	"syscall"
	"time"
	"unsafe"

	"websocket-backend/internal/config"
	"websocket-backend/internal/graphql"
	"websocket-backend/internal/models"
	"websocket-backend/internal/stats"
	"websocket-backend/internal/utils"
)

const (
	futureBufferSeconds = 1
	spreadTimeMs       = 3000
	
	// Optimization constants for 100k connections
	maxWorkers         = 32    // Number of broadcast workers
	broadcastBuffer    = 10000 // Buffer size for broadcast channels
	clientShards       = 64    // Number of client shards to reduce lock contention
	maxMessageQueue    = 1000  // Max queued messages per client
)

var startTime = time.Now()

// ClientShard represents a shard of clients to reduce lock contention
type ClientShard struct {
	clients map[*Client]bool
	mu      sync.RWMutex
}

// StatsCollector interface for stats collection
type StatsCollector interface {
	UpdateTransferStats([]models.Transfer)
	GetChartData() stats.ChartData
}

// Server represents the WebSocket server optimized for high concurrency
type Server struct {
	config         *config.Config
	graphql        *graphql.Client
	statsCollector StatsCollector
	logger         *utils.Logger
	
	// Client management with sharding
	clientShards   []*ClientShard
	clientCount    int64 // atomic counter
	
	// Broadcasting system
	broadcastWorkers []chan BroadcastMessage
	
	// Transfer management - timestamp-based buckets
	transferBuckets map[int64][]models.Transfer // Key: Unix timestamp (minute precision)
	transferBucketsMu sync.RWMutex
	
	// Legacy transfer management (for backward compatibility)
	chains         []models.Chain
	lastSortOrder  string
	isInitialFetch bool
	scheduledTransfers []ScheduledTransfer
	transferMu     sync.RWMutex
	
	// Enhanced scheduling
	enhancedScheduler *EnhancedScheduler
	
	// Simple status tracking
	startTime        time.Time
	
	// Graceful shutdown
	shutdown chan struct{}
	wg       sync.WaitGroup
}

// NewServer creates a new WebSocket server optimized for high concurrency
func NewServer(cfg *config.Config, gql *graphql.Client) *Server {
	// Validate configuration
	if err := cfg.ValidateAndWarn(); err != nil {
		utils.ServerLogger.Fatal("Configuration validation failed", map[string]interface{}{
			"error": err.Error(),
		})
	}
	
	// Initialize client shards
	shards := make([]*ClientShard, clientShards)
	for i := range shards {
		shards[i] = &ClientShard{
			clients: make(map[*Client]bool),
		}
	}
	
	// Initialize broadcast workers
	workers := make([]chan BroadcastMessage, maxWorkers)
	for i := range workers {
		workers[i] = make(chan BroadcastMessage, broadcastBuffer)
	}
	
	// Initialize stats collector and server state
	var statsCollector *stats.EnhancedCollector = stats.NewEnhancedCollector()
	var lastSortOrder string = cfg.LastSortOrder
	var isInitialFetch bool = cfg.LastSortOrder == ""
	
	utils.ServerLogger.Info("Server starting fresh", map[string]interface{}{
		"lastSortOrder": lastSortOrder,
		"isInitialFetch": isInitialFetch,
	})
	
	server := &Server{
		config:           cfg,
		graphql:          gql,
		statsCollector:   statsCollector,
		logger:           utils.ServerLogger,
		clientShards:     shards,
		broadcastWorkers: workers,
		transferBuckets:  make(map[int64][]models.Transfer),
		scheduledTransfers: make([]ScheduledTransfer, 0),
		isInitialFetch:   isInitialFetch,
		lastSortOrder:    lastSortOrder,
		shutdown:         make(chan struct{}),
		startTime:        time.Now(),
	}
	
	// Always initialize enhanced scheduler
	server.enhancedScheduler = NewEnhancedScheduler(server)
	
	// Log final configuration
	utils.ServerLogger.Info("Server initialized", map[string]interface{}{
		"lastSortOrder": lastSortOrder,
		"isInitialFetch": isInitialFetch,
	})
	
	return server
}

// Simple logging methods
func (s *Server) logQueueStatus(queueLen int) {
	if queueLen > 0 {
		fmt.Printf("[QUEUE] %d transfers waiting to be processed\n", queueLen)
	}
}

func (s *Server) logTransferFlow(enhanced, streamed, broadcasted, clientCount int) {
	fmt.Printf("[FLOW] Enhanced %d → Streamed %d → Broadcasted %d transfers to %d clients\n", 
		enhanced, streamed, broadcasted, clientCount)
}

func (s *Server) logNewTransfersFetched(count int) {
	if count > 0 {
		fmt.Printf("[FETCH] Retrieved %d new transfers from GraphQL\n", count)
	}
}

func (s *Server) logClientConnection(connected bool, count int) {
	action := "connected"
	if !connected {
		action = "disconnected"
	}
	fmt.Printf("[CLIENT] Client %s (total: %d)\n", action, count)
}

func (s *Server) logChainCount(count int) {
	fmt.Printf("[CHAINS] Loaded chains: %d\n", count)
}

func (s *Server) logPollingStatus(interval int, isActive bool) {
	status := "STOPPED"
	if isActive {
		status = "ACTIVE"
	}
	fmt.Printf("[POLLING] Status: %s, Interval: %dms\n", status, interval)
}

func (s *Server) logError(consecutiveErrors int, lastError string) {
	if lastError != "" {
		fmt.Printf("[ERROR] Consecutive errors: %d, Last: %s\n", consecutiveErrors, lastError)
	} else if consecutiveErrors == 0 {
		fmt.Printf("[INFO] Polling recovered\n")
	}
}

func (s *Server) logChainRefresh(success bool) {
	status := "SUCCESS"
	if !success {
		status = "FAILED"
	}
	fmt.Printf("[CHAINS] Refresh: %s\n", status)
}

// Run starts the server with optimized architecture
func (s *Server) Run() {
	// Initialize logging based on environment
	isDevelopment := s.config.GraphQLEndpoint == "https://staging.graphql.union.build/v1/graphql"
	utils.InitializeLogging(isDevelopment)
	utils.SetIncludeStackTrace(isDevelopment)
	
	// Set GOMAXPROCS to use all available cores
	runtime.GOMAXPROCS(runtime.NumCPU())
	s.logger.Info("Server starting", map[string]interface{}{
		"cpuCores": runtime.NumCPU(),
		"port": s.config.Port,
		"environment": map[string]interface{}{
			"isDevelopment": isDevelopment,
			"graphqlEndpoint": s.config.GraphQLEndpoint,
			"pollInterval": s.config.PollInterval,
			"pollLimit": s.config.PollLimit,
			"mainnetOnly": s.config.MainnetOnly,
			"lastSortOrder": s.config.LastSortOrder,
			"enhancedMode": true, // Always enabled
		},
	})
	
	// Create HTTP server with optimized settings
	mux := http.NewServeMux()
	mux.HandleFunc("/ws", s.handleWebSocket) // WebSocket doesn't use recovery middleware
	mux.HandleFunc("/health", s.recoverMiddleware(s.handleHealth))
	mux.HandleFunc("/stats", s.recoverMiddleware(s.handleStats))
	mux.HandleFunc("/stats/enhanced", s.recoverMiddleware(s.handleEnhancedStats))
	mux.HandleFunc("/scheduler", s.recoverMiddleware(s.handleSchedulerStats))

	mux.HandleFunc("/debug", s.recoverMiddleware(s.handleDebug))
	mux.HandleFunc("/buckets", s.recoverMiddleware(s.handleBuckets))

	server := &http.Server{
		Addr:         ":" + s.config.Port,
		Handler:      mux,
		ReadTimeout:  30 * time.Second,
		WriteTimeout: 30 * time.Second,
		IdleTimeout:  120 * time.Second,
	}

	// Create context for graceful shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Log initial status
	fmt.Printf("[SERVER] Union WebSocket Server starting\n")
	fmt.Printf("[SERVER] Started at: %s\n", s.startTime.Format("2006-01-02 15:04:05"))
	s.logPollingStatus(s.config.PollInterval, true)
	
	// Start broadcast workers
	for i := 0; i < maxWorkers; i++ {
		s.wg.Add(1)
		go s.broadcastWorker(i, s.broadcastWorkers[i])
	}
	// Remove the old log message - status display will show this info

	// Start polling for transfers
	s.wg.Add(1)
	go func() {
		defer s.wg.Done()
		s.logger.Info("Starting polling goroutine")
		s.pollForTransfers(ctx)
	}()

	// Start enhanced scheduler processing
	s.wg.Add(1)
	go func() {
		defer s.wg.Done()
		s.logger.Info("Starting enhanced scheduler processing goroutine")
		s.enhancedScheduler.processScheduledTransfers()
	}()

	// Start chart data broadcaster
	s.wg.Add(1)
	go func() {
		defer s.wg.Done()
		s.chartDataBroadcaster()
	}()



	// Setup graceful shutdown
	go s.setupGracefulShutdown(server, cancel)

	s.logger.Info("WebSocket server running", map[string]interface{}{
		"port": s.config.Port,
		"optimizedFor": "100k connections",
	})

	// Start HTTP server
	if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
		utils.LogError(utils.NetworkError("SERVER_START_FAILED", "Failed to start HTTP server", "SERVER"), s.logger, map[string]interface{}{
			"port": s.config.Port,
		})
	}
}



// setupGracefulShutdown handles graceful shutdown
func (s *Server) setupGracefulShutdown(server *http.Server, cancel context.CancelFunc) {
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	
	<-sigChan
	s.logger.Info("Shutdown signal received, starting graceful shutdown")
	
	// Signal all goroutines to stop
	close(s.shutdown)
	
	// Stop HTTP server
	ctx, shutdownCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer shutdownCancel()
	
	if err := server.Shutdown(ctx); err != nil {
		utils.LogError(err, s.logger, map[string]interface{}{
			"component": "graceful_shutdown",
		})
	}
	
	// Cancel polling context
	cancel()
	
	// Wait for all goroutines to finish
	done := make(chan struct{})
	go func() {
		s.wg.Wait()
		close(done)
	}()
	
	select {
	case <-done:
		s.logger.Info("Graceful shutdown completed")
	case <-time.After(30 * time.Second):
		s.logger.Warn("Graceful shutdown timeout, forcing exit")
	}
}

// Client management methods

// getClientShard returns the shard for a client based on pointer hash
func (s *Server) getClientShard(client *Client) *ClientShard {
	hash := uintptr(unsafe.Pointer(client))
	return s.clientShards[hash%uintptr(clientShards)]
}

// addClient adds a new client to the server with proper error handling
func (s *Server) addClient(client *Client) {
	defer func() {
		if err := recover(); err != nil {
			s.logger.Error("addClient panic recovered", map[string]interface{}{
				"error": err,
			})
		}
	}()
	
	if client == nil {
		s.logger.Warn("Attempted to add nil client")
		return
	}
	
	if client.conn == nil {
		s.logger.Warn("Attempted to add client with nil connection")
		return
	}
	
	// Generate a hash based on the client's remote address for sharding
	remoteAddr := client.conn.RemoteAddr()
	if remoteAddr == nil {
		s.logger.Warn("Client has nil remote address, using fallback")
		remoteAddr = &net.TCPAddr{IP: net.IPv4(0, 0, 0, 0), Port: 0}
	}
	
	shardIndex := hash(remoteAddr.String()) % uint32(len(s.clientShards))
	
	// Validate shard index
	if int(shardIndex) >= len(s.clientShards) {
		s.logger.Error("Invalid shard index calculated", map[string]interface{}{
			"shardIndex": shardIndex,
			"maxShards": len(s.clientShards),
		})
		return
	}
	
	shard := s.clientShards[shardIndex]
	if shard == nil {
		s.logger.Error("Shard is nil", map[string]interface{}{
			"shardIndex": shardIndex,
		})
		return
	}
	
	shard.mu.Lock()
	if shard.clients == nil {
		shard.clients = make(map[*Client]bool)
	}
	shard.clients[client] = true
	shard.mu.Unlock()
	
	newCount := atomic.AddInt64(&s.clientCount, 1)
	
	// Log client connection
	s.logClientConnection(true, int(newCount))
	
	s.logger.Info("Client added", map[string]interface{}{
		"remoteAddr": remoteAddr.String(),
		"shardIndex": shardIndex,
		"totalClients": newCount,
	})
}

// removeClient removes a client from the server with proper error handling
func (s *Server) removeClient(client *Client) {
	defer func() {
		if err := recover(); err != nil {
			s.logger.Error("removeClient panic recovered", map[string]interface{}{
				"error": err,
			})
		}
	}()
	
	if client == nil {
		s.logger.Warn("Attempted to remove nil client")
		return
	}
	
	// We need to search all shards since we don't know which one the client is in
	// This is necessary because the client might have been added before proper cleanup
	var found bool
	var clientAddr string
	
	if client.conn != nil && client.conn.RemoteAddr() != nil {
		clientAddr = client.conn.RemoteAddr().String()
	} else {
		clientAddr = "unknown"
	}
	
	for i, shard := range s.clientShards {
		if shard == nil {
			continue
		}
		
		shard.mu.Lock()
		if shard.clients != nil {
			if _, exists := shard.clients[client]; exists {
				delete(shard.clients, client)
				found = true
				s.logger.Debug("Client removed from shard", map[string]interface{}{
					"shardIndex": i,
					"clientAddr": clientAddr,
				})
			}
		}
		shard.mu.Unlock()
		
		if found {
			break
		}
	}
	
	if found {
		newCount := atomic.AddInt64(&s.clientCount, -1)
		
		// Log client disconnection
		s.logClientConnection(false, int(newCount))
		
		s.logger.Info("Client removed", map[string]interface{}{
			"clientAddr": clientAddr,
			"totalClients": newCount,
		})
	} else {
		s.logger.Warn("Client not found in any shard during removal", map[string]interface{}{
			"clientAddr": clientAddr,
		})
	}
}

// getClientCount returns the current client count atomically
func (s *Server) getClientCount() int64 {
	return atomic.LoadInt64(&s.clientCount)
}

// updateTransferStats safely updates transfer statistics
func (s *Server) updateTransferStats(transfers []models.Transfer) {
	if s.statsCollector == nil {
		s.logger.Warn("Stats collector is nil, cannot update transfer stats")
		return
	}
	
	if len(transfers) == 0 {
		return
	}
	
	defer func() {
		if err := recover(); err != nil {
			s.logger.Error("Transfer stats update panic", map[string]interface{}{
				"error": err,
				"transferCount": len(transfers),
			})
		}
	}()
	
	s.statsCollector.UpdateTransferStats(transfers)
}

// getChartData safely retrieves chart data
func (s *Server) getChartData() interface{} {
	defer func() {
		if err := recover(); err != nil {
			s.logger.Error("Chart data generation panic", map[string]interface{}{
				"error": err,
			})
		}
	}()
	
	if s.statsCollector == nil {
		s.logger.Warn("Stats collector is nil")
		return nil
	}
	
	// Safe type assertion to EnhancedCollector
	if enhancedCollector, ok := s.statsCollector.(*stats.EnhancedCollector); ok {
		return enhancedCollector.GetEnhancedChartData()
	} else {
		s.logger.Error("Stats collector is not an EnhancedCollector", map[string]interface{}{
			"actualType": fmt.Sprintf("%T", s.statsCollector),
		})
		// Fallback to basic chart data
		return s.statsCollector.GetChartData()
	}
}

// getBasicChartData returns basic chart data (for backward compatibility)
func (s *Server) getBasicChartData() stats.ChartData {
	return s.statsCollector.GetChartData()
}

// hash generates a hash for client sharding
func hash(s string) uint32 {
	h := fnv.New32a()
	h.Write([]byte(s))
	return h.Sum32()
}

// addTransferToBucket adds a transfer to the appropriate time bucket based on its timestamp
func (s *Server) addTransferToBucket(transfer models.Transfer) {
	// Critical nil checks first
	if s == nil {
		log.Printf("[BUCKET] CRITICAL: Server is nil")
		return
	}
	
	// Validate transfer data integrity
	if transfer.SortOrder == "" {
		log.Printf("[BUCKET] CRITICAL: Transfer missing sort order, hash: %s", transfer.TransferSendTxHash)
		return
	}
	
	if transfer.TransferSendTimestamp.IsZero() {
		log.Printf("[BUCKET] CRITICAL: Transfer has zero timestamp, using current time, hash: %s", transfer.TransferSendTxHash)
		transfer.TransferSendTimestamp = time.Now()
	}
	
	s.transferBucketsMu.Lock()
	defer s.transferBucketsMu.Unlock()
	
	// Additional safety check after acquiring lock
	if s.transferBuckets == nil {
		log.Printf("[BUCKET] CRITICAL: Transfer buckets map is nil, initializing")
		s.transferBuckets = make(map[int64][]models.Transfer)
	}
	
	// Round timestamp to minute for bucketing
	bucketTime := transfer.TransferSendTimestamp.Truncate(time.Minute).Unix()
	
	// Add transfer to bucket
	s.transferBuckets[bucketTime] = append(s.transferBuckets[bucketTime], transfer)
	
	s.logger.Debug("Transfer added to bucket", map[string]interface{}{
		"bucketTime": time.Unix(bucketTime, 0).Format(time.RFC3339),
		"transferHash": transfer.TransferSendTxHash,
		"transferTime": transfer.TransferSendTimestamp.Format(time.RFC3339),
	})
}

// addTransfersToBuckets adds multiple transfers to their respective time buckets
func (s *Server) addTransfersToBuckets(transfers []models.Transfer) {
	for _, transfer := range transfers {
		s.addTransferToBucket(transfer)
	}
	
	// Clean up old buckets (keep only last 30 minutes)
	s.cleanupOldBuckets()
}

// getRecentTransfers retrieves transfers from the last N minutes
func (s *Server) getRecentTransfers(minutes int) []models.Transfer {
	s.transferBucketsMu.RLock()
	defer s.transferBucketsMu.RUnlock()
	
	cutoff := time.Now().Add(-time.Duration(minutes) * time.Minute).Truncate(time.Minute).Unix()
	
	var recentTransfers []models.Transfer
	for bucketTime, transfers := range s.transferBuckets {
		if bucketTime >= cutoff {
			recentTransfers = append(recentTransfers, transfers...)
		}
	}
	
	// Sort by timestamp (most recent first)
	sort.Slice(recentTransfers, func(i, j int) bool {
		return recentTransfers[i].TransferSendTimestamp.After(recentTransfers[j].TransferSendTimestamp)
	})
	
	return recentTransfers
}

// cleanupOldBuckets removes buckets older than 30 minutes to prevent memory growth
func (s *Server) cleanupOldBuckets() {
	s.transferBucketsMu.Lock()
	defer s.transferBucketsMu.Unlock()
	
	cutoff := time.Now().Add(-30 * time.Minute).Truncate(time.Minute).Unix()
	
	// Collect keys to delete first to avoid concurrent map modification
	var keysToDelete []int64
	for bucketTime := range s.transferBuckets {
		if bucketTime < cutoff {
			keysToDelete = append(keysToDelete, bucketTime)
		}
	}
	
	// Now safely delete the keys
	for _, key := range keysToDelete {
		delete(s.transferBuckets, key)
	}
	
	bucketsRemoved := len(keysToDelete)
	if bucketsRemoved > 0 {
		s.logger.Debug("Cleaned up old transfer buckets", map[string]interface{}{
			"bucketsRemoved": bucketsRemoved,
			"cutoffTime": time.Unix(cutoff, 0).Format(time.RFC3339),
		})
	}
}

// getTransferBucketStats returns statistics about the current bucket state
func (s *Server) getTransferBucketStats() map[string]interface{} {
	s.transferBucketsMu.RLock()
	defer s.transferBucketsMu.RUnlock()
	
	totalTransfers := 0
	bucketCount := len(s.transferBuckets)
	
	var oldestBucket, newestBucket int64
	for bucketTime, transfers := range s.transferBuckets {
		totalTransfers += len(transfers)
		
		if oldestBucket == 0 || bucketTime < oldestBucket {
			oldestBucket = bucketTime
		}
		if bucketTime > newestBucket {
			newestBucket = bucketTime
		}
	}
	
	stats := map[string]interface{}{
		"bucketCount": bucketCount,
		"totalTransfers": totalTransfers,
	}
	
	if oldestBucket > 0 {
		stats["oldestBucket"] = time.Unix(oldestBucket, 0).Format(time.RFC3339)
		stats["newestBucket"] = time.Unix(newestBucket, 0).Format(time.RFC3339)
		stats["timeSpanMinutes"] = (newestBucket - oldestBucket) / 60
	}
	
	return stats
}

// Shutdown gracefully shuts down the server
func (s *Server) Shutdown() {
	fmt.Printf("[SERVER] Shutting down gracefully...\n")
	
	// Signal shutdown
	close(s.shutdown)
}

 
 