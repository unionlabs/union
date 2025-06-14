package ws

import (
	"log"
	"sync/atomic"
	"time"

	"websocket-backend/internal/models"
	"runtime/debug"
)

// BroadcastMessage represents a message to broadcast
type BroadcastMessage struct {
	transfers []models.Transfer
	timestamp int64
}

// broadcastWorker handles broadcasting messages to clients in assigned shards
func (s *Server) broadcastWorker(workerID int, messages chan BroadcastMessage) {
	defer func() {
		s.wg.Done()
		if err := recover(); err != nil {
			log.Printf("[BROADCAST] Worker %d panic recovered: %v", workerID, err)
			// Log stack trace for debugging
			log.Printf("[BROADCAST] Worker %d stack trace: %s", workerID, debug.Stack())
		}
	}()
	
	// Worker started - no log needed
	
	// Calculate which shards this worker handles
	shardsPerWorker := clientShards / maxWorkers
	startShard := workerID * shardsPerWorker
	endShard := startShard + shardsPerWorker
	if workerID == maxWorkers-1 {
		endShard = clientShards
	}
	
	// Validate shard range
	if startShard < 0 || endShard > clientShards || startShard >= endShard {
		log.Printf("[BROADCAST] Worker %d has invalid shard range [%d, %d), maxShards: %d", 
			workerID, startShard, endShard, clientShards)
		return
	}
	
	for {
		select {
		case <-s.shutdown:
			return
		case msg, ok := <-messages:
			if !ok {
				return
			}
			
			func() {
				defer func() {
					if err := recover(); err != nil {
						log.Printf("[BROADCAST] Worker %d message processing panic recovered: %v", workerID, err)
					}
				}()
				
				if len(msg.transfers) == 0 {
					return
				}
				
				clientsDelivered := 0
				
				// Deliver to all clients in assigned shards
				for i := startShard; i < endShard; i++ {
					// Critical bounds check
					if i >= len(s.clientShards) || i < 0 {
						log.Printf("[BROADCAST] Worker %d: CRITICAL shard index %d out of range (max: %d)", workerID, i, len(s.clientShards))
						break // Exit immediately to prevent crash
					}
					
					shard := s.clientShards[i]
					if shard == nil {
						log.Printf("[BROADCAST] Worker %d: CRITICAL shard %d is nil", workerID, i)
						continue
					}
					
					func() {
						defer func() {
							if err := recover(); err != nil {
								log.Printf("[BROADCAST] Worker %d shard %d iteration panic recovered: %v", workerID, i, err)
							}
						}()
						
						shard.mu.RLock()
						defer shard.mu.RUnlock()
						
						// Additional safety check after acquiring lock
						if shard.clients == nil {
							log.Printf("[BROADCAST] Worker %d: shard %d clients map is nil", workerID, i)
							return
						}
						
						for client := range shard.clients {
							if client == nil {
								continue
							}
							// Check if client is closed before delivery
							if client.IsClosed() {
								continue
							}
							if s.deliverToClient(client, msg.transfers) {
								clientsDelivered++
							}
						}
					}()
				}
				
				// Delivery completed - no log needed
			}()
		}
	}
}

// deliverToClient delivers transfers to a specific client with filtering
func (s *Server) deliverToClient(client *Client, transfers []models.Transfer) bool {
	defer func() {
		if err := recover(); err != nil {
			log.Printf("[BROADCAST] deliverToClient panic recovered: %v", err)
		}
	}()
	
	if client == nil || len(transfers) == 0 {
		return false
	}
	
	// Check if client is closed
	if client.IsClosed() {
		return false
	}
	
	// Apply client-specific filtering
	filteredTransfers := make([]models.Transfer, 0, len(transfers))
	filter := client.GetFilter()
	
	for _, transfer := range transfers {
		// Additional safety check for transfer data
		if transfer.SourceChain.UniversalChainID == "" || transfer.DestinationChain.UniversalChainID == "" {
			log.Printf("[BROADCAST] Skipping transfer with empty chain IDs")
			continue
		}
		
		if filter == nil || transfer.MatchesFilter(filter) {
			filteredTransfers = append(filteredTransfers, transfer)
		}
	}
	
	if len(filteredTransfers) == 0 {
		return false
	}
	
	// Send to client using the client's Send method with error handling
	var sendResult bool
	func() {
		defer func() {
			if err := recover(); err != nil {
				log.Printf("[BROADCAST] Client SendTransfers panic recovered: %v", err)
				sendResult = false
			}
		}()
		
		sendResult = client.SendTransfers(filteredTransfers)
	}()
	
	// Return the actual send result
	return sendResult
}

// broadcastTransfers distributes transfers across broadcast workers
func (s *Server) broadcastTransfers(transfers []models.Transfer) {
	if len(transfers) == 0 {
		return
	}
	
	msg := BroadcastMessage{
		transfers: transfers,
		timestamp: time.Now().UnixMilli(),
	}
	
	// Distribute to all workers
	for i, worker := range s.broadcastWorkers {
		if worker == nil {
			log.Printf("[BROADCAST] Worker %d channel is nil", i)
			continue
		}
		
		select {
		case worker <- msg:
			// Message sent successfully
		default:
			log.Printf("[BROADCAST] Worker %d channel full, dropping message", i)
		}
	}
	
	// Distribution completed - no log needed
}

// broadcastChartData broadcasts chart data to all clients
func (s *Server) broadcastChartData() {
	defer func() {
		if err := recover(); err != nil {
			log.Printf("[CHART] Chart data broadcast panic recovered: %v", err)
		}
	}()
	
	chartData := s.getChartData()
	if chartData == nil {
		return
	}
	
	// Send chart data directly to all clients without spawning goroutines
	// to prevent goroutine leaks when broadcasting every second
	for i := 0; i < maxWorkers; i++ {
		s.sendChartDataToWorker(i, chartData)
	}
}

// sendChartDataToWorker sends chart data to a specific worker's clients
func (s *Server) sendChartDataToWorker(workerID int, chartData interface{}) {
	defer func() {
		if err := recover(); err != nil {
			log.Printf("[CHART] Worker %d chart data send panic recovered: %v", workerID, err)
		}
	}()
	
	if chartData == nil {
		return
	}
	
	// Calculate which shards this worker handles
	shardsPerWorker := clientShards / maxWorkers
	startShard := workerID * shardsPerWorker
	endShard := startShard + shardsPerWorker
	if workerID == maxWorkers-1 {
		endShard = clientShards
	}
	
	// Send to all clients in assigned shards
	for i := startShard; i < endShard; i++ {
		if i >= len(s.clientShards) {
			continue
		}
		
		shard := s.clientShards[i]
		if shard == nil {
			continue
		}
		
		shard.mu.RLock()
		for client := range shard.clients {
			if client == nil {
				continue
			}
			// Safely send chart data to each client
			func() {
				defer func() {
					if err := recover(); err != nil {
						log.Printf("[CHART] Client chart data send panic recovered: %v", err)
					}
				}()
				client.SendChartData(chartData)
			}()
		}
		shard.mu.RUnlock()
	}
}

// chartDataBroadcaster periodically broadcasts chart data
func (s *Server) chartDataBroadcaster() {
	defer func() {
		if err := recover(); err != nil {
			log.Printf("[CHART] Chart data broadcaster panic recovered: %v", err)
			// Restart the broadcaster after a panic
			time.Sleep(5 * time.Second) // Wait before restarting
			go s.chartDataBroadcaster() // Restart the broadcaster
		}
	}()
	
	ticker := time.NewTicker(1 * time.Second) // Broadcast every 1 second for real-time updates
	defer ticker.Stop()
	
	for {
		select {
		case <-s.shutdown:
			return
		case <-ticker.C:
			func() {
				defer func() {
					if err := recover(); err != nil {
						log.Printf("[CHART] Chart data broadcast cycle panic recovered: %v", err)
					}
				}()
				
				clientCount := atomic.LoadInt64(&s.clientCount)
				if clientCount > 0 {
					s.broadcastChartData()
				}
			}()
		}
	}
} 