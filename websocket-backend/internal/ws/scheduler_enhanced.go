package ws

import (
	"context"
	"fmt"
	"log"
	"math/rand"
	"sort"
	"strings"
	"sync"
	"time"

	"websocket-backend/internal/models"
	"websocket-backend/internal/utils"
)

// ScheduledTransfer represents a transfer scheduled for delivery
type ScheduledTransfer struct {
	transfer      models.Transfer
	scheduledTime int64
}

// Enhanced scheduling constants for more natural feel
const (
	minSpreadTime = 500  // Minimum spread time in milliseconds
	maxSpreadTime = 3000 // Maximum spread time in milliseconds
)

// EnhancedScheduler provides intelligent transfer scheduling based on actual timestamps
type EnhancedScheduler struct {
	server           *Server
	recentActivity   []int64  // Recent transfer timestamps for activity detection
	lastBurstTime    int64    // Last time we had a burst
	activityMu       sync.RWMutex
}

// NewEnhancedScheduler creates a new enhanced scheduler
func NewEnhancedScheduler(server *Server) *EnhancedScheduler {
	return &EnhancedScheduler{
		server:         server,
		recentActivity: make([]int64, 0),
	}
}

// scheduleTransfers schedules transfers for smooth streaming delivery (JavaScript-style approach)
func (es *EnhancedScheduler) scheduleTransfers(transfers []models.Transfer) []ScheduledTransfer {
	if len(transfers) == 0 {
		return nil
	}

	now := time.Now()
	scheduled := make([]ScheduledTransfer, 0, len(transfers))
	
	// JavaScript-style streaming parameters
	const FUTURE_BUFFER_SECONDS = 1
	const SPREAD_TIME_MS = 3000 // 3 seconds spread like the JS version
	
	futureBaseTime := now.UnixMilli() + (FUTURE_BUFFER_SECONDS * 1000)
	
	// Sort transfers by timestamp for chronological streaming (like JS version)
	sortedTransfers := make([]models.Transfer, len(transfers))
	copy(sortedTransfers, transfers)
	sort.Slice(sortedTransfers, func(i, j int) bool {
		return sortedTransfers[i].TransferSendTimestamp.Before(sortedTransfers[j].TransferSendTimestamp)
	})
	
	// Schedule transfers with spread and jitter (exactly like JS version)
	for index, transfer := range sortedTransfers {
		// Calculate spread across the time window
		var spread int64
		if len(sortedTransfers) > 1 {
			spread = int64(float64(index) / float64(len(sortedTransfers)-1) * SPREAD_TIME_MS)
		} else {
			spread = 0
		}
		
		// Add jitter: (Math.random() - 0.5) * 500 equivalent
		jitter := int64((rand.Float64() - 0.5) * 500)
		
		scheduledTime := futureBaseTime + spread + jitter
		
		scheduled = append(scheduled, ScheduledTransfer{
			transfer:      transfer,
			scheduledTime: scheduledTime,
		})
		
		// Debug logging removed - status display shows queue info
	}
	
	// Sort by scheduled time (like JS version)
	sort.Slice(scheduled, func(i, j int) bool {
		return scheduled[i].scheduledTime < scheduled[j].scheduledTime
	})
	
	// Verbose logging removed - status display shows transfer info
	
	return scheduled
}

// processScheduledTransfers processes scheduled transfers for smooth streaming
func (es *EnhancedScheduler) processScheduledTransfers() {
	ticker := time.NewTicker(50 * time.Millisecond)
	defer ticker.Stop()

	// Removed verbose startup log - status display shows this info

	for {
		select {
		case <-es.server.shutdown:
			// Removed verbose shutdown log
			return
		case <-ticker.C:
			func() {
				defer func() {
					if err := recover(); err != nil {
						log.Printf("[STREAM] Enhanced transfer processor panic recovered: %v", err)
					}
				}()
				
				es.server.transferMu.Lock()
				queueLength := len(es.server.scheduledTransfers)
				if queueLength == 0 {
					es.server.transferMu.Unlock()
					return
				}

				now := time.Now().UnixMilli()
				readyTransfers := make([]models.Transfer, 0)
				remainingTransfers := make([]ScheduledTransfer, 0)

				// Process ready transfers
				readyCount := 0
				for _, scheduled := range es.server.scheduledTransfers {
					if scheduled.scheduledTime <= now {
						readyTransfers = append(readyTransfers, scheduled.transfer)
						readyCount++
					} else {
						remainingTransfers = append(remainingTransfers, scheduled)
					}
				}

				es.server.scheduledTransfers = remainingTransfers
				es.server.transferMu.Unlock()

				if len(readyTransfers) > 0 {
					// Limit batch size for live feel - smaller batches for more responsive streaming
					maxBatchSize := 5 // Process max 5 transfers at once for optimal live feel
					
					if len(readyTransfers) <= maxBatchSize {
						// Small batch - process all at once
						// Update statistics BEFORE broadcasting
						es.server.updateTransferStats(readyTransfers)
						
						// Broadcast ready transfers using the new system
						es.server.broadcastTransfers(readyTransfers)
						
						// Log transfer flow
						clientCount := int(es.server.getClientCount())
						es.server.logTransferFlow(len(readyTransfers), len(readyTransfers), len(readyTransfers), clientCount)
						if len(remainingTransfers) > 0 {
							es.server.logQueueStatus(len(remainingTransfers))
						}
					} else {
						// Large batch - process in smaller chunks for live feel
						batch := readyTransfers[:maxBatchSize]
						remaining := readyTransfers[maxBatchSize:]
						
						// Update statistics BEFORE broadcasting
						es.server.updateTransferStats(batch)
						
						// Broadcast the batch
						es.server.broadcastTransfers(batch)
						
						// Re-schedule remaining transfers with minimal delays for live feel
						now := time.Now().UnixMilli()
						for i, transfer := range remaining {
							remainingTransfers = append(remainingTransfers, ScheduledTransfer{
								transfer:      transfer,
								scheduledTime: now + int64((i+1)*10), // 10ms intervals for live feel
							})
						}
						
						// Log transfer flow
						clientCount := int(es.server.getClientCount())
						es.server.logTransferFlow(len(batch), len(batch), len(batch), clientCount)
						totalRemaining := len(remainingTransfers) + len(remaining)
						if totalRemaining > 0 {
							es.server.logQueueStatus(totalRemaining)
						}
					}
				} else if queueLength > 0 {
					// Log queue status (no new transfers processed)
					es.server.logQueueStatus(queueLength)
				}
			}()
		}
	}
}

// groupTransfersByTimeWindow groups transfers that occurred within the same time window
func (es *EnhancedScheduler) groupTransfersByTimeWindow(transfers []models.Transfer, window time.Duration) [][]models.Transfer {
	if len(transfers) == 0 {
		return [][]models.Transfer{}
	}
	
	var groups [][]models.Transfer
	currentGroup := []models.Transfer{transfers[0]}
	currentGroupTime := transfers[0].TransferSendTimestamp
	
	// Check for zero timestamp
	if currentGroupTime.IsZero() {
		utils.ScheduleLogger.Warn("Transfer has zero timestamp, using current time")
		currentGroupTime = time.Now()
	}
	
	for i := 1; i < len(transfers); i++ {
		// Bounds check
		if i >= len(transfers) {
			utils.ScheduleLogger.Error("Array bounds exceeded in grouping", map[string]interface{}{
				"index": i,
				"length": len(transfers),
			})
			break
		}
		
		transfer := transfers[i]
		
		// Validate transfer timestamp
		if transfer.TransferSendTimestamp.IsZero() {
			utils.ScheduleLogger.Warn("Transfer has zero timestamp, using current time", map[string]interface{}{
				"transferIndex": i,
			})
			transfer.TransferSendTimestamp = time.Now()
		}
		
		// If this transfer is within the time window of the current group, add it
		timeDiff := transfer.TransferSendTimestamp.Sub(currentGroupTime)
		if timeDiff <= window && timeDiff >= -window { // Allow small negative differences for clock skew
			currentGroup = append(currentGroup, transfer)
		} else {
			// Start a new group
			if len(currentGroup) > 0 {
				groups = append(groups, currentGroup)
			}
			currentGroup = []models.Transfer{transfer}
			currentGroupTime = transfer.TransferSendTimestamp
		}
	}
	
	// Add the final group if it has any transfers
	if len(currentGroup) > 0 {
		groups = append(groups, currentGroup)
	}
	
	// Debug logging removed - status display shows transfer info
	
	return groups
}

// enhanceTransfersWithChains enhances transfers with pre-computed metadata using provided chains
func (es *EnhancedScheduler) enhanceTransfersWithChains(transfers []models.Transfer, chains []models.Chain) []models.Transfer {
	enhanced := make([]models.Transfer, 0, len(transfers))
	
	for _, transfer := range transfers {
		// Create enhanced copy
		enhancedTransfer := transfer
		
		// Get source chain display name
		enhancedTransfer.SourceDisplayName = es.getChainDisplayNameSafe(chains, 
			transfer.SourceChain.UniversalChainID, 
			transfer.SourceChain.DisplayName, 
			transfer.SourceChain.ChainID)
		
		// Get destination chain display name
		enhancedTransfer.DestinationDisplayName = es.getChainDisplayNameSafe(chains, 
			transfer.DestinationChain.UniversalChainID, 
			transfer.DestinationChain.DisplayName, 
			transfer.DestinationChain.ChainID)
		
		// Set testnet flag based on either chain being testnet
		enhancedTransfer.IsTestnetTransfer = transfer.SourceChain.Testnet || transfer.DestinationChain.Testnet
		
		// Format timestamp for frontend display
		enhancedTransfer.FormattedTimestamp = transfer.TransferSendTimestamp.Format("2006-01-02T15:04:05Z07:00")
		
		// Create route key for grouping
		enhancedTransfer.RouteKey = transfer.SourceChain.UniversalChainID + "->" + transfer.DestinationChain.UniversalChainID
		
		// Set display addresses (for now, same as canonical - could be enhanced later with ENS/chain-specific formatting)
		enhancedTransfer.SenderDisplay = transfer.SenderCanonical
		enhancedTransfer.ReceiverDisplay = transfer.ReceiverCanonical
		
		enhanced = append(enhanced, enhancedTransfer)
	}
	
	return enhanced
}

// getChainDisplayNameSafe safely gets the display name for a chain
func (es *EnhancedScheduler) getChainDisplayNameSafe(chains []models.Chain, universalChainID, embeddedDisplayName, embeddedChainID string) string {
	// First try to find by universal chain ID
	for _, chain := range chains {
		if chain.UniversalChainID == universalChainID {
			return chain.DisplayName
		}
	}
	
	// Fallback to embedded display name if available
	if embeddedDisplayName != "" {
		return embeddedDisplayName
	}
	
	// Final fallback to chain ID or universal chain ID
	if embeddedChainID != "" {
		return embeddedChainID
	}
	
	return universalChainID
}

// pollForTransfers polls for new transfers (moved from deleted scheduler.go)
// fetchChains fetches chain information for client initialization
func (s *Server) fetchChains(ctx context.Context) error {
	if s.graphql == nil {
		return fmt.Errorf("graphql client is nil")
	}
	
	chains, err := s.graphql.FetchChains(ctx)
	if err != nil {
		return err
	}

	s.transferMu.Lock()
	s.chains = chains
	s.transferMu.Unlock()

	// Log chain count
	s.logChainCount(len(chains))
	
	return nil
}

// periodicChainRefresh refreshes chain data every 5 minutes to catch new chain pairs
func (s *Server) periodicChainRefresh(ctx context.Context) {
	defer func() {
		if err := recover(); err != nil {
			log.Printf("[CHAINS] Chain refresh panic recovered: %v", err)
		}
	}()

	// Create a ticker for 5-minute intervals
	ticker := time.NewTicker(5 * time.Minute)
	defer ticker.Stop()

	// Removed verbose startup log - status display shows this info

	for {
		select {
		case <-ctx.Done():
			return
		case <-s.shutdown:
			return
		case <-ticker.C:
			func() {
				defer func() {
					if err := recover(); err != nil {
						log.Printf("[CHAINS] Chain refresh cycle panic recovered: %v", err)
					}
				}()

				// Retry logic for chain refresh (similar to initial fetch)
				var lastErr error
				for retries := 0; retries < 3; retries++ {
					if err := s.fetchChains(ctx); err != nil {
						lastErr = err
						if retries < 2 {
							// Shorter backoff for periodic refresh
							backoffDuration := time.Duration(500*(retries+1)) * time.Millisecond // 500ms, 1s
							time.Sleep(backoffDuration)
						}
					} else {
						s.logChainRefresh(true)
						lastErr = nil
						break
					}
				}

				if lastErr != nil {
					s.logChainRefresh(false)
				}
			}()
		}
	}
}

// enhanceTransfers adds metadata to transfers
func (s *Server) enhanceTransfers(transfers []models.Transfer) []models.Transfer {
	// Don't use mutex here since we're already inside a mutex from scheduleTransfers
	enhanced := make([]models.Transfer, len(transfers))
	for i, transfer := range transfers {
		enhanced[i] = transfer

		// Set testnet flag based on embedded chain data
		enhanced[i].IsTestnetTransfer = transfer.SourceChain.Testnet || transfer.DestinationChain.Testnet
		
		// Use display names from embedded chain data, with fallbacks to separate chains data
		enhanced[i].SourceDisplayName = s.getChainDisplayName(transfer.SourceChain.UniversalChainID, transfer.SourceChain.DisplayName, transfer.SourceChain.ChainID)
		enhanced[i].DestinationDisplayName = s.getChainDisplayName(transfer.DestinationChain.UniversalChainID, transfer.DestinationChain.DisplayName, transfer.DestinationChain.ChainID)
		
		// Format timestamp
		enhanced[i].FormattedTimestamp = transfer.TransferSendTimestamp.Format("15:04:05")
		
		// Create route key using display names for better readability
		enhanced[i].RouteKey = enhanced[i].SourceDisplayName + "->" + enhanced[i].DestinationDisplayName
		
		// Store full addresses - frontend will handle display truncation
		enhanced[i].SenderDisplay = transfer.SenderCanonical
		enhanced[i].ReceiverDisplay = transfer.ReceiverCanonical
	}
	
	return enhanced
}

// getChainDisplayName gets the best display name for a chain, matching Node.js logic
func (s *Server) getChainDisplayName(universalChainID, embeddedDisplayName, embeddedChainID string) string {
	// Primary: Use separate chains data (like Node.js version)
	for _, chain := range s.chains {
		if chain.UniversalChainID == universalChainID {
			if chain.DisplayName != "" {
				return chain.DisplayName
			}
			if chain.ChainID != "" {
				return chain.ChainID
			}
		}
	}
	
	// Fallback to embedded data from transfer
	if embeddedDisplayName != "" {
		return embeddedDisplayName
	}
	
	if embeddedChainID != "" {
		return embeddedChainID
	}
	
	// Final fallback to universal chain ID
	return universalChainID
}

// pollForTransfers polls for new transfers
func (s *Server) pollForTransfers(ctx context.Context) {
	defer func() {
		if err := recover(); err != nil {
			log.Printf("[POLL] Polling panic recovered: %v", err)
		}
	}()
	
	// Error tracking for intelligent backoff
	var consecutiveErrors int
	var lastErrorTime time.Time
	const maxConsecutiveErrors = 10
	
	// First, fetch chains data for client initialization with retry
	for retries := 0; retries < 3; retries++ {
		if err := s.fetchChains(ctx); err != nil {
			s.logError(retries+1, fmt.Sprintf("Chain fetch failed: %v", err))
			if retries < 2 {
				backoffDuration := time.Duration(1<<retries) * time.Second // 1s, 2s
				time.Sleep(backoffDuration)
			}
		} else {
			s.logChainRefresh(true)
			break
		}
	}

	// Start periodic chain refresh in a separate goroutine
	go s.periodicChainRefresh(ctx)

	// Log polling status
	s.logPollingStatus(s.config.PollInterval, true)

	for {
		select {
		case <-ctx.Done():
			s.logPollingStatus(s.config.PollInterval, false)
			return
		case <-s.shutdown:
			s.logPollingStatus(s.config.PollInterval, false)
			return
		default:
			// Calculate sleep duration based on error state
			sleepDuration := time.Duration(s.config.PollInterval) * time.Millisecond
			
			// If we have consecutive errors, implement exponential backoff
			if consecutiveErrors > 0 {
				// Exponential backoff: 1s, 2s, 4s, 8s, 16s, max 30s
				backoffSeconds := min(1<<consecutiveErrors, 30)
				sleepDuration = time.Duration(backoffSeconds) * time.Second
				
				// Log error info
				s.logError(consecutiveErrors, "Polling errors - backing off")
			}
			
			func() {
				defer func() {
					if err := recover(); err != nil {
						log.Printf("[POLL] Polling cycle panic recovered: %v", err)
						consecutiveErrors++
					}
				}()
				
				// Get current sort order with read lock
				s.transferMu.RLock()
				lastSortOrder := s.lastSortOrder
				isInitial := s.isInitialFetch
				s.transferMu.RUnlock()

				var err error
				if isInitial || lastSortOrder == "" {
					// Get baseline
					transfers, fetchErr := s.graphql.FetchLatestTransfers(ctx, 1, s.config.GetNetworkFilter())
					if fetchErr != nil {
						err = fetchErr
						s.handlePollingError(err, &consecutiveErrors, &lastErrorTime)
						time.Sleep(sleepDuration)
						return
					}

					if len(transfers) > 0 {
						s.transferMu.Lock()
						s.lastSortOrder = transfers[0].SortOrder
						s.isInitialFetch = false
						s.transferMu.Unlock()
					}
				} else {
					// Poll for new transfers
					transfers, fetchErr := s.graphql.FetchNewTransfers(ctx, lastSortOrder, s.config.PollLimit, s.config.GetNetworkFilter())
					if fetchErr != nil {
						err = fetchErr
						s.handlePollingError(err, &consecutiveErrors, &lastErrorTime)
						time.Sleep(sleepDuration)
						return
					}

					if len(transfers) > 0 {
						// Log fetched transfers
						s.logNewTransfersFetched(len(transfers))
						
						// Update baseline
						sortOrders := make([]string, len(transfers))
						for i, t := range transfers {
							sortOrders[i] = t.SortOrder
						}
						s.transferMu.Lock()
						s.lastSortOrder = sortOrders[len(sortOrders)-1]
						s.transferMu.Unlock()

						// Schedule transfers using enhanced scheduler (always enabled)
						if s.enhancedScheduler != nil {
							s.enhancedScheduler.scheduleTransfersEnhanced(transfers)
						}
					}
				}

				// If we reach here, the polling cycle was successful
				if consecutiveErrors > 0 {
					consecutiveErrors = 0 // Reset error count on success
					s.logError(0, "") // Clear error status
				}

				// Log successful polling
				s.logPollingStatus(s.config.PollInterval, true)
			}()
			
			// Use the calculated sleep duration
			time.Sleep(sleepDuration)
		}
	}
}

// handlePollingError handles different types of polling errors with appropriate strategies
func (s *Server) handlePollingError(err error, consecutiveErrors *int, lastErrorTime *time.Time) {
	const maxConsecutiveErrors = 10
	*consecutiveErrors++
	*lastErrorTime = time.Now()
	
	// Analyze error type for specific handling
	errorStr := err.Error()
	
	// Rate limiting detection
	if strings.Contains(strings.ToLower(errorStr), "rate limit") || 
	   strings.Contains(strings.ToLower(errorStr), "too many requests") ||
	   strings.Contains(errorStr, "429") {
		*consecutiveErrors += 2 // Increase backoff more aggressively for rate limits
		return
	}
	
	// Network/timeout errors
	if strings.Contains(strings.ToLower(errorStr), "timeout") ||
	   strings.Contains(strings.ToLower(errorStr), "connection") ||
	   strings.Contains(strings.ToLower(errorStr), "network") {
		return
	}
	
	// Server errors (5xx)
	if strings.Contains(errorStr, "status code: 5") {
		return
	}
	
	// GraphQL errors
	if strings.Contains(strings.ToLower(errorStr), "graphql") {
		return
	}
	
	// If we have too many consecutive errors, log a warning
	if *consecutiveErrors >= maxConsecutiveErrors {
		log.Printf("[POLL] WARNING: %d consecutive polling errors. Server may be in degraded state.", *consecutiveErrors)
		log.Printf("[POLL] Consider checking network connectivity and server status")
		// Cap the consecutive errors to prevent infinite backoff
		*consecutiveErrors = maxConsecutiveErrors
	}
}

// min function is defined in handlers.go

// Legacy functions for compatibility

// analyzeNetworkActivity determines current network activity level based on actual transfer timestamps
func (es *EnhancedScheduler) analyzeNetworkActivity() float64 {
	es.activityMu.RLock()
	defer es.activityMu.RUnlock()
	
	if len(es.recentActivity) == 0 {
		return 0.1 // Default low activity level
	}
	
	now := time.Now().UnixMilli()
	recentWindow := now - 30000 // Last 30 seconds
	
	recentCount := 0
	for _, timestamp := range es.recentActivity {
		if timestamp > recentWindow {
			recentCount++
		}
	}
	
	// Normalize activity level (0.0 = low, 1.0 = high)
	// Assume 60 transfers per 30 seconds is "high" activity
	activityLevel := float64(recentCount) / 60.0
	if activityLevel > 1.0 {
		activityLevel = 1.0
	}
	
	// Ensure minimum activity level
	if activityLevel < 0.1 {
		activityLevel = 0.1
	}
	
	return activityLevel
}

// updateActivityTrackingWithTimestamps updates the recent activity tracking with actual transfer times
func (es *EnhancedScheduler) updateActivityTrackingWithTimestamps(transfers []models.Transfer) {
	if len(transfers) == 0 {
		return
	}
	
	es.activityMu.Lock()
	defer es.activityMu.Unlock()
	
	now := time.Now().UnixMilli()
	
	// Add current batch to activity tracking
	for _, transfer := range transfers {
		es.recentActivity = append(es.recentActivity, transfer.TransferSendTimestamp.UnixMilli())
	}
	
	// Clean up old activity (keep last hour)
	oneHourAgo := now - 3600000
	filtered := make([]int64, 0, len(es.recentActivity))
	for _, timestamp := range es.recentActivity {
		if timestamp > oneHourAgo {
			filtered = append(filtered, timestamp)
		}
	}
	es.recentActivity = filtered
}

// GetActivityStats returns current activity statistics
func (es *EnhancedScheduler) GetActivityStats() map[string]interface{} {
	es.activityMu.RLock()
	defer es.activityMu.RUnlock()
	
	now := time.Now().UnixMilli()
	
	// Calculate activity for different time windows
	windows := map[string]int64{
		"last30s":  30000,
		"last5min": 300000,
		"last1hr":  3600000,
	}
	
	stats := make(map[string]interface{})
	
	for window, duration := range windows {
		cutoff := now - duration
		count := 0
		for _, timestamp := range es.recentActivity {
			if timestamp > cutoff {
				count++
			}
		}
		stats[window] = count
	}
	
	stats["totalTracked"] = len(es.recentActivity)
	stats["activityLevel"] = es.analyzeNetworkActivity()
	
	return stats
}

// scheduleTransfersEnhanced uses natural streaming with random intervals for authentic feel
func (es *EnhancedScheduler) scheduleTransfersEnhanced(transfers []models.Transfer) {
	defer func() {
		if err := recover(); err != nil {
			utils.ScheduleLogger.Error("scheduleTransfersEnhanced panic recovered", map[string]interface{}{
				"error": err,
				"transferCount": len(transfers),
			})
		}
	}()
	
	if len(transfers) == 0 {
		utils.ScheduleLogger.Info("No transfers to schedule")
		return
	}
	
	// Critical nil checks
	if es == nil {
		utils.ScheduleLogger.Error("Enhanced scheduler is nil")
		return
	}
	
	if es.server == nil {
		utils.ScheduleLogger.Error("Enhanced scheduler server is nil")
		return
	}
	
	// Validate transfers data integrity
	validTransfers := make([]models.Transfer, 0, len(transfers))
	for i, transfer := range transfers {
		if transfer.SortOrder == "" {
			utils.ScheduleLogger.Warn("Transfer missing sort order", map[string]interface{}{
				"transferIndex": i,
				"transferHash": transfer.TransferSendTxHash,
			})
			continue
		}
		if transfer.SourceChain.UniversalChainID == "" || transfer.DestinationChain.UniversalChainID == "" {
			utils.ScheduleLogger.Warn("Transfer missing chain IDs", map[string]interface{}{
				"transferIndex": i,
				"sourceChain": transfer.SourceChain.UniversalChainID,
				"destChain": transfer.DestinationChain.UniversalChainID,
			})
			continue
		}
		validTransfers = append(validTransfers, transfer)
	}
	
	if len(validTransfers) == 0 {
		utils.ScheduleLogger.Warn("No valid transfers after validation")
		return
	}
	
	utils.ScheduleLogger.Info("Starting natural stream scheduling", map[string]interface{}{
		"originalCount": len(transfers),
		"validCount": len(validTransfers),
	})
	
			es.server.transferMu.Lock()
		defer es.server.transferMu.Unlock()

	// Enhance transfers first with safety check
	var enhancedTransfers []models.Transfer
	var chains []models.Chain
	if es.server.chains != nil {
		chains = make([]models.Chain, len(es.server.chains))
		copy(chains, es.server.chains)
	}
	
	func() {
		defer func() {
			if err := recover(); err != nil {
				utils.ScheduleLogger.Error("Transfer enhancement panic recovered", map[string]interface{}{
					"error": err,
				})
				enhancedTransfers = []models.Transfer{} // Use empty slice as fallback
			}
		}()
		enhancedTransfers = es.enhanceTransfersWithChains(validTransfers, chains)
	}()
	
	if len(enhancedTransfers) == 0 {
		utils.ScheduleLogger.Warn("No enhanced transfers generated")
		return
	}
	
	// Schedule transfers using natural streaming
	scheduled := es.scheduleTransfers(enhancedTransfers)
	
	if len(scheduled) == 0 {
		utils.ScheduleLogger.Warn("No transfers scheduled")
		return
	}
	
	// Add to server's scheduled transfers
	if es.server.scheduledTransfers == nil {
		es.server.scheduledTransfers = make([]ScheduledTransfer, 0)
	}
	es.server.scheduledTransfers = append(es.server.scheduledTransfers, scheduled...)
	
	// Sort by scheduled time to maintain proper delivery order
	sort.Slice(es.server.scheduledTransfers, func(i, j int) bool {
		if i >= len(es.server.scheduledTransfers) || j >= len(es.server.scheduledTransfers) || i < 0 || j < 0 {
			return false
		}
		return es.server.scheduledTransfers[i].scheduledTime < es.server.scheduledTransfers[j].scheduledTime
	})
	
	// Update activity tracking with actual transfer times
	func() {
		defer func() {
			if err := recover(); err != nil {
				utils.ScheduleLogger.Error("Activity tracking update panic recovered", map[string]interface{}{
					"error": err,
				})
			}
		}()
		es.updateActivityTrackingWithTimestamps(enhancedTransfers)
	}()
	
	utils.ScheduleLogger.Info("Natural stream scheduling complete", map[string]interface{}{
		"scheduledCount": len(scheduled),
		"totalQueued":    len(es.server.scheduledTransfers),
		"streamDuration": "natural intervals",
	})
} 