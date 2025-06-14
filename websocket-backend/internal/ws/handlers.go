package ws

import (
    "encoding/json"
    "fmt"
    "net/http"
    "runtime"
    "runtime/debug"
    "strconv"
    "sync/atomic"
    "time"

    "github.com/gorilla/websocket"
    "websocket-backend/internal/models"
    "websocket-backend/internal/stats"
)

var upgrader = websocket.Upgrader{
    CheckOrigin: func(r *http.Request) bool {
        return true // Allow all origins in development
    },
    ReadBufferSize:  1024,
    WriteBufferSize: 1024,
}

// recoverMiddleware wraps handlers with panic recovery
func (s *Server) recoverMiddleware(handler http.HandlerFunc) http.HandlerFunc {
    return func(w http.ResponseWriter, r *http.Request) {
        defer func() {
            if err := recover(); err != nil {
                s.logger.Error("Handler panic recovered", map[string]interface{}{
                    "error": fmt.Sprintf("%v", err),
                    "path":  r.URL.Path,
                    "stack": string(debug.Stack()),
                })
                
                w.Header().Set("Content-Type", "application/json")
                w.WriteHeader(http.StatusInternalServerError)
                json.NewEncoder(w).Encode(map[string]interface{}{
                    "error":     "Internal server error",
                    "timestamp": time.Now().UnixMilli(),
                    "path":      r.URL.Path,
                })
            }
        }()
        handler(w, r)
    }
}

// handleWebSocket handles WebSocket connections
func (s *Server) handleWebSocket(w http.ResponseWriter, r *http.Request) {
    defer func() {
        if err := recover(); err != nil {
            s.logger.Error("WebSocket handler panic", map[string]interface{}{
                "error": fmt.Sprintf("%v", err),
                "stack": string(debug.Stack()),
            })
        }
    }()

    conn, err := upgrader.Upgrade(w, r, nil)
    if err != nil {
        s.logger.Error("Failed to upgrade WebSocket connection", map[string]interface{}{
            "error": err.Error(),
        })
        return
    }

    client := NewClient(conn, s.removeClient)
    s.addClient(client)

    // Send initial data with error handling
    if err := client.SendConnected(); err != nil {
        s.logger.Warn("Failed to send connected message", map[string]interface{}{
            "error": err.Error(),
        })
    }
    
    s.transferMu.RLock()
    chains := s.chains
    s.transferMu.RUnlock()
    
    if err := client.SendChains(chains); err != nil {
        s.logger.Warn("Failed to send chains data", map[string]interface{}{
            "error": err.Error(),
        })
    }
    
    if err := client.SendServerInfo(); err != nil {
        s.logger.Warn("Failed to send server info", map[string]interface{}{
            "error": err.Error(),
        })
    }

    // Start client goroutines
    go client.WritePump()
    go client.ReadPump(s.handleClientMessage)
}

// ClientMessage represents a validated client message
type ClientMessage struct {
    Type string `json:"type"`
    Data struct {
        FromChain string `json:"fromChain"`
        ToChain   string `json:"toChain"`
    } `json:"data"`
}

// validateClientMessage validates and parses client messages
func (s *Server) validateClientMessage(message []byte) (*ClientMessage, error) {
    if len(message) == 0 {
        return nil, fmt.Errorf("empty message")
    }
    
    if len(message) > 1024 { // Prevent large messages
        return nil, fmt.Errorf("message too large: %d bytes", len(message))
    }

    var data ClientMessage
    if err := json.Unmarshal(message, &data); err != nil {
        return nil, fmt.Errorf("invalid JSON: %v", err)
    }

    // Validate message type
    switch data.Type {
    case "setChainFilter":
        // Validate chain IDs (basic validation)
        if len(data.Data.FromChain) > 100 || len(data.Data.ToChain) > 100 {
            return nil, fmt.Errorf("chain ID too long")
        }
    case "ping":
        // Allow ping messages
    default:
        return nil, fmt.Errorf("unknown message type: %s", data.Type)
    }

    return &data, nil
}

// handleClientMessage handles client messages with validation
func (s *Server) handleClientMessage(client *Client, message []byte) {
    defer func() {
        if err := recover(); err != nil {
            s.logger.Error("Client message handler panic", map[string]interface{}{
                "error": fmt.Sprintf("%v", err),
                "stack": string(debug.Stack()),
            })
        }
    }()

    data, err := s.validateClientMessage(message)
    if err != nil {
        s.logger.Warn("Invalid client message", map[string]interface{}{
            "error":   err.Error(),
            "message": string(message),
        })
        
        // Send error response to client
        client.Send(map[string]interface{}{
            "type":      "error",
            "message":   "Invalid message format",
            "timestamp": time.Now().UnixMilli(),
        })
        return
    }

    switch data.Type {
    case "setChainFilter":
        client.SetFilter(&models.ChainFilter{
            FromChain: data.Data.FromChain,
            ToChain:   data.Data.ToChain,
        })

        if err := client.SendFilterSet(data.Data.FromChain, data.Data.ToChain); err != nil {
            s.logger.Warn("Failed to send filter confirmation", map[string]interface{}{
                "error": err.Error(),
            })
        }
    case "ping":
        // Respond to ping
        client.Send(map[string]interface{}{
            "type":      "pong",
            "timestamp": time.Now().UnixMilli(),
        })
    }
}

// handleHealth handles health check requests
func (s *Server) handleHealth(w http.ResponseWriter, r *http.Request) {
    defer func() {
        if err := recover(); err != nil {
            s.logger.Error("Health handler panic", map[string]interface{}{
                "error": fmt.Sprintf("%v", err),
            })
            http.Error(w, "Internal Server Error", http.StatusInternalServerError)
        }
    }()

    s.transferMu.RLock()
    lastSortOrder := s.lastSortOrder
    s.transferMu.RUnlock()

    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(struct {
        Status       string  `json:"status"`
        LastSortOrder string  `json:"lastSortOrder"`
        Uptime       float64 `json:"uptime"`
        Timestamp    int64   `json:"timestamp"`
    }{
        Status:       "healthy",
        LastSortOrder: lastSortOrder,
        Uptime:       time.Since(startTime).Seconds(),
        Timestamp:    time.Now().UnixMilli(),
    })
}

// handleStats handles the statistics endpoint
func (s *Server) handleStats(w http.ResponseWriter, r *http.Request) {
    defer func() {
        if err := recover(); err != nil {
            s.logger.Error("Stats handler panic", map[string]interface{}{
                "error": fmt.Sprintf("%v", err),
                "stack": string(debug.Stack()),
            })
            
            w.Header().Set("Content-Type", "application/json")
            w.WriteHeader(http.StatusInternalServerError)
            json.NewEncoder(w).Encode(map[string]interface{}{
                "error":     "Failed to generate stats",
                "timestamp": time.Now().UnixMilli(),
            })
        }
    }()

    s.transferMu.RLock()
    lastSortOrder := s.lastSortOrder
    isInitialFetch := s.isInitialFetch
    scheduledTransfers := len(s.scheduledTransfers)
    s.transferMu.RUnlock()

    clientCount := atomic.LoadInt64(&s.clientCount)
    
    // Safely get chart data
    var chartData interface{}
    func() {
        defer func() {
            if err := recover(); err != nil {
                s.logger.Error("Chart data panic", map[string]interface{}{
                    "error": fmt.Sprintf("%v", err),
                })
                chartData = nil
            }
        }()
        chartData = s.getChartData()
    }()
    
    var m runtime.MemStats
    runtime.ReadMemStats(&m)

    // Handle both enhanced and basic chart data safely
    var transferRates stats.TransferRates
    var popularRoutes []*stats.RouteStats
    var activeSenders []*stats.WalletStats
    var activeReceivers []*stats.WalletStats

    if chartData != nil {
        if enhancedData, ok := chartData.(stats.EnhancedChartData); ok {
            // Enhanced chart data
            transferRates = enhancedData.CurrentRates
            popularRoutes = enhancedData.PopularRoutes
            activeSenders = enhancedData.ActiveSenders
            activeReceivers = enhancedData.ActiveReceivers
        } else if basicData, ok := chartData.(stats.ChartData); ok {
            // Basic chart data - convert to enhanced format
            transferRates = basicData.CurrentRates
            popularRoutes = basicData.PopularRoutes
            activeSenders = basicData.ActiveSenders
            activeReceivers = basicData.ActiveReceivers
        }
    }

    w.Header().Set("Content-Type", "application/json")
    if err := json.NewEncoder(w).Encode(struct {
        LastSortOrder      string   `json:"lastSortOrder"`
        IsInitialFetch     bool     `json:"isInitialFetch"`
        ScheduledTransfers int      `json:"scheduledTransfers"`
        ClientCount        int64    `json:"clientCount"`
        TransferRates      stats.TransferRates `json:"transferRates"`
        PopularRoutes      []*stats.RouteStats `json:"popularRoutes"`
        ActiveSenders      []*stats.WalletStats `json:"activeSenders"`
        ActiveReceivers    []*stats.WalletStats `json:"activeReceivers"`
        Performance        struct {
            Goroutines     int     `json:"goroutines"`
            MemoryMB       float64 `json:"memoryMB"`
            GCCount        uint32  `json:"gcCount"`
            BroadcastWorkers int   `json:"broadcastWorkers"`
            ClientShards   int     `json:"clientShards"`
        } `json:"performance"`
        Config            struct {
            PollInterval    int    `json:"POLL_INTERVAL"`
            MainnetOnly     bool   `json:"MAINNET_ONLY"`
            GraphQLEndpoint string `json:"GRAPHQL_ENDPOINT"`
            SpreadTimeMs    int    `json:"SPREAD_TIME_MS"`
            FutureBuffer    int    `json:"FUTURE_BUFFER_SECONDS"`
            MaxWorkers      int    `json:"MAX_WORKERS"`
            ClientShards    int    `json:"CLIENT_SHARDS"`
        } `json:"config"`
        Uptime    float64 `json:"uptime"`
        Timestamp int64   `json:"timestamp"`
    }{
        LastSortOrder:      lastSortOrder,
        IsInitialFetch:     isInitialFetch,
        ScheduledTransfers: scheduledTransfers,
        ClientCount:        clientCount,
        TransferRates:      transferRates,
        PopularRoutes:      popularRoutes,
        ActiveSenders:      activeSenders,
        ActiveReceivers:    activeReceivers,
        Performance: struct {
            Goroutines     int     `json:"goroutines"`
            MemoryMB       float64 `json:"memoryMB"`
            GCCount        uint32  `json:"gcCount"`
            BroadcastWorkers int   `json:"broadcastWorkers"`
            ClientShards   int     `json:"clientShards"`
        }{
            Goroutines:     runtime.NumGoroutine(),
            MemoryMB:       float64(m.Alloc)/1024/1024,
            GCCount:        m.NumGC,
            BroadcastWorkers: maxWorkers,
            ClientShards:   clientShards,
        },
        Config: struct {
            PollInterval    int    `json:"POLL_INTERVAL"`
            MainnetOnly     bool   `json:"MAINNET_ONLY"`
            GraphQLEndpoint string `json:"GRAPHQL_ENDPOINT"`
            SpreadTimeMs    int    `json:"SPREAD_TIME_MS"`
            FutureBuffer    int    `json:"FUTURE_BUFFER_SECONDS"`
            MaxWorkers      int    `json:"MAX_WORKERS"`
            ClientShards    int    `json:"CLIENT_SHARDS"`
        }{
            PollInterval:    s.config.PollInterval,
            MainnetOnly:     s.config.MainnetOnly,
            GraphQLEndpoint: s.config.GraphQLEndpoint,
            SpreadTimeMs:    spreadTimeMs,
            FutureBuffer:    futureBufferSeconds,
            MaxWorkers:      maxWorkers,
            ClientShards:    clientShards,
        },
        Uptime:    time.Since(startTime).Seconds(),
        Timestamp: time.Now().UnixMilli(),
    }); err != nil {
        s.logger.Error("Failed to encode stats response", map[string]interface{}{
            "error": err.Error(),
        })
    }
}

// handleSchedulerStats handles the scheduler statistics endpoint
func (s *Server) handleSchedulerStats(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Content-Type", "application/json")
    
    response := map[string]interface{}{
        "enhancedSchedulerEnabled": true, // Always enabled
        "timestamp": time.Now().UnixMilli(),
    }
    
    // Safely get activity stats
    if s.enhancedScheduler != nil {
        func() {
            defer func() {
                if err := recover(); err != nil {
                    s.logger.Error("Scheduler stats panic", map[string]interface{}{
                        "error": fmt.Sprintf("%v", err),
                    })
                    response["activityStats"] = nil
                    response["error"] = "Failed to get activity stats"
                }
            }()
            response["activityStats"] = s.enhancedScheduler.GetActivityStats()
        }()
    } else {
        response["message"] = "Enhanced scheduler is disabled"
    }
    
    if err := json.NewEncoder(w).Encode(response); err != nil {
        s.logger.Error("Failed to encode scheduler stats", map[string]interface{}{
            "error": err.Error(),
        })
    }
}

// handleEnhancedStats handles the enhanced statistics endpoint
func (s *Server) handleEnhancedStats(w http.ResponseWriter, r *http.Request) {
    s.transferMu.RLock()
    lastSortOrder := s.lastSortOrder
    isInitialFetch := s.isInitialFetch
    scheduledTransfers := len(s.scheduledTransfers)
    s.transferMu.RUnlock()

    clientCount := atomic.LoadInt64(&s.clientCount)
    
    // Check if we have enhanced stats collector
    if enhancedCollector, ok := s.statsCollector.(*stats.EnhancedCollector); ok {
        // Safely get enhanced chart data
        var enhancedChartData stats.EnhancedChartData
        func() {
            defer func() {
                if err := recover(); err != nil {
                    s.logger.Error("Enhanced chart data panic", map[string]interface{}{
                        "error": fmt.Sprintf("%v", err),
                    })
                    // Use empty data as fallback
                    enhancedChartData = stats.EnhancedChartData{}
                }
            }()
            enhancedChartData = enhancedCollector.GetEnhancedChartData()
        }()
        
        var m runtime.MemStats
        runtime.ReadMemStats(&m)

        w.Header().Set("Content-Type", "application/json")
        if err := json.NewEncoder(w).Encode(struct {
            LastSortOrder      string   `json:"lastSortOrder"`
            IsInitialFetch     bool     `json:"isInitialFetch"`
            ScheduledTransfers int      `json:"scheduledTransfers"`
            ClientCount        int64    `json:"clientCount"`
            TransferRates      stats.TransferRates `json:"transferRates"`
            ActiveWalletRates  stats.ActiveWalletRates `json:"activeWalletRates"`
            PopularRoutes      []*stats.RouteStats `json:"popularRoutes"`
            PopularRoutesTimeScale map[string][]*stats.RouteStats `json:"popularRoutesTimeScale"`
            ActiveSenders      []*stats.WalletStats `json:"activeSenders"`
            ActiveReceivers    []*stats.WalletStats `json:"activeReceivers"`
            DataAvailability   struct {
                HasMinute bool `json:"hasMinute"`
                HasHour   bool `json:"hasHour"`
                HasDay    bool `json:"hasDay"`
                Has7Days  bool `json:"has7Days"`
                Has14Days bool `json:"has14Days"`
                Has30Days bool `json:"has30Days"`
            } `json:"dataAvailability"`
            Performance        struct {
                Goroutines     int     `json:"goroutines"`
                MemoryMB       float64 `json:"memoryMB"`
                GCCount        uint32  `json:"gcCount"`
                BroadcastWorkers int   `json:"broadcastWorkers"`
                ClientShards   int     `json:"clientShards"`
            } `json:"performance"`
            Config            struct {
                PollInterval      int    `json:"POLL_INTERVAL"`
                MainnetOnly       bool   `json:"MAINNET_ONLY"`
                GraphQLEndpoint   string `json:"GRAPHQL_ENDPOINT"`
                EnhancedScheduler bool   `json:"ENHANCED_SCHEDULER"`
                EnhancedStats     bool   `json:"ENHANCED_STATS"`
                SpreadTimeMs      int    `json:"SPREAD_TIME_MS"`
                FutureBuffer      int    `json:"FUTURE_BUFFER_SECONDS"`
                MaxWorkers        int    `json:"MAX_WORKERS"`
                ClientShards      int    `json:"CLIENT_SHARDS"`
            } `json:"config"`
            Uptime    float64 `json:"uptime"`
            Timestamp int64   `json:"timestamp"`
        }{
            LastSortOrder:      lastSortOrder,
            IsInitialFetch:     isInitialFetch,
            ScheduledTransfers: scheduledTransfers,
            ClientCount:        clientCount,
            TransferRates:      enhancedChartData.CurrentRates,
            ActiveWalletRates:  enhancedChartData.ActiveWalletRates,
            PopularRoutes:      enhancedChartData.PopularRoutes,
            PopularRoutesTimeScale: enhancedChartData.PopularRoutesTimeScale,
            ActiveSenders:      enhancedChartData.ActiveSenders,
            ActiveReceivers:    enhancedChartData.ActiveReceivers,
            DataAvailability:   enhancedChartData.DataAvailability,
            Performance: struct {
                Goroutines     int     `json:"goroutines"`
                MemoryMB       float64 `json:"memoryMB"`
                GCCount        uint32  `json:"gcCount"`
                BroadcastWorkers int   `json:"broadcastWorkers"`
                ClientShards   int     `json:"clientShards"`
            }{
                Goroutines:     runtime.NumGoroutine(),
                MemoryMB:       float64(m.Alloc)/1024/1024,
                GCCount:        m.NumGC,
                BroadcastWorkers: maxWorkers,
                ClientShards:   clientShards,
            },
            Config: struct {
                PollInterval      int    `json:"POLL_INTERVAL"`
                MainnetOnly       bool   `json:"MAINNET_ONLY"`
                GraphQLEndpoint   string `json:"GRAPHQL_ENDPOINT"`
                EnhancedScheduler bool   `json:"ENHANCED_SCHEDULER"`
                EnhancedStats     bool   `json:"ENHANCED_STATS"`
                SpreadTimeMs      int    `json:"SPREAD_TIME_MS"`
                FutureBuffer      int    `json:"FUTURE_BUFFER_SECONDS"`
                MaxWorkers        int    `json:"MAX_WORKERS"`
                ClientShards      int    `json:"CLIENT_SHARDS"`
            }{
                PollInterval:      s.config.PollInterval,
                MainnetOnly:       s.config.MainnetOnly,
                GraphQLEndpoint:   s.config.GraphQLEndpoint,
                EnhancedScheduler: true, // Always enabled
                EnhancedStats:     true, // Always enabled
                SpreadTimeMs:      spreadTimeMs,
                FutureBuffer:      futureBufferSeconds,
                MaxWorkers:        maxWorkers,
                ClientShards:      clientShards,
            },
            Uptime:    time.Since(startTime).Seconds(),
            Timestamp: time.Now().UnixMilli(),
        }); err != nil {
            s.logger.Error("Failed to encode enhanced stats response", map[string]interface{}{
                "error": err.Error(),
            })
        }
    } else {
        // Fallback to regular stats if enhanced stats not available
        w.Header().Set("Content-Type", "application/json")
        json.NewEncoder(w).Encode(map[string]interface{}{
            "error":     "Enhanced stats not available",
            "timestamp": time.Now().UnixMilli(),
        })
    }
}

// handleDebug handles debug information endpoint
func (s *Server) handleDebug(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Content-Type", "application/json")
    
    s.transferMu.RLock()
    lastSortOrder := s.lastSortOrder
    isInitialFetch := s.isInitialFetch
    s.transferMu.RUnlock()
    
    debugInfo := map[string]interface{}{
        "timestamp": time.Now().UnixMilli(),
        "config": map[string]interface{}{
            "enhancedStats":   true, // Always enabled
            "pollInterval":    s.config.PollInterval,
            "mainnetOnly":     s.config.MainnetOnly,

            "transferAgeThreshold": s.config.TransferAgeThreshold.String(),
        },
        "server": map[string]interface{}{
            "lastSortOrder":  lastSortOrder,
            "isInitialFetch": isInitialFetch,
        },
        "buckets": s.getTransferBucketStats(),
    }
    
    // Check collector type safely
    if enhancedCollector, ok := s.statsCollector.(*stats.EnhancedCollector); ok {
        debugInfo["collectorType"] = "EnhancedCollector"
        
        // Safely get raw stats for debugging
        func() {
            defer func() {
                if err := recover(); err != nil {
                    s.logger.Error("Debug stats panic", map[string]interface{}{
                        "error": fmt.Sprintf("%v", err),
                    })
                    debugInfo["rawStats"] = map[string]interface{}{
                        "error": "Failed to get debug stats",
                    }
                }
            }()
            
            chartData := enhancedCollector.GetEnhancedChartData()
            debugInfo["rawStats"] = map[string]interface{}{
                "totalTracked":       chartData.CurrentRates.TotalTracked,
                "txPer30Days":        chartData.CurrentRates.TxPer30Days,
                "txPer7Days":         chartData.CurrentRates.TxPer7Days,
                "txPerDay":           chartData.CurrentRates.TxPerDay,
                "popularRoutesCount": len(chartData.PopularRoutes),
                "activeSendersCount": len(chartData.ActiveSenders),
                "activeSenders":      chartData.ActiveWalletRates.SendersLastMin,
                "activeReceivers":    chartData.ActiveWalletRates.ReceiversLastMin,
                "totalActive":        chartData.ActiveWalletRates.TotalLastMin,
            }
            
            // Show top routes if any
            if len(chartData.PopularRoutes) > 0 {
                debugInfo["topRoutes"] = chartData.PopularRoutes[:min(3, len(chartData.PopularRoutes))]
            }
            
            // Show top senders if any
            if len(chartData.ActiveSenders) > 0 {
                debugInfo["topSenders"] = chartData.ActiveSenders[:min(3, len(chartData.ActiveSenders))]
            }
        }()
    } else {
        debugInfo["collectorType"] = "StandardCollector"
        func() {
            defer func() {
                if err := recover(); err != nil {
                    s.logger.Error("Standard debug stats panic", map[string]interface{}{
                        "error": fmt.Sprintf("%v", err),
                    })
                    debugInfo["rawStats"] = map[string]interface{}{
                        "error": "Failed to get standard debug stats",
                    }
                }
            }()
            
            chartData := s.statsCollector.GetChartData()
            debugInfo["rawStats"] = map[string]interface{}{
                "totalTracked":       chartData.CurrentRates.TotalTracked,
                "txPerDay":           chartData.CurrentRates.TxPerDay,
                "popularRoutesCount": len(chartData.PopularRoutes),
                "activeSendersCount": len(chartData.ActiveSenders),
            }
        }()
    }
    
    if err := json.NewEncoder(w).Encode(debugInfo); err != nil {
        s.logger.Error("Failed to encode debug response", map[string]interface{}{
            "error": err.Error(),
        })
    }
}



// handleBuckets handles the bucket information endpoint
func (s *Server) handleBuckets(w http.ResponseWriter, r *http.Request) {
	defer func() {
		if err := recover(); err != nil {
			s.logger.Error("Buckets handler panic", map[string]interface{}{
				"error": err,
			})
			http.Error(w, "Internal server error", http.StatusInternalServerError)
		}
	}()
	
	// Get query parameter for minutes (default to 5)
	minutesStr := r.URL.Query().Get("minutes")
	minutes := 5
	if minutesStr != "" {
		if parsed, err := strconv.Atoi(minutesStr); err == nil && parsed > 0 && parsed <= 60 {
			minutes = parsed
		}
	}
	
	// Get recent transfers from buckets
	recentTransfers := s.getRecentTransfers(minutes)
	bucketStats := s.getTransferBucketStats()
	
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{
		"bucketStats": bucketStats,
		"recentTransfers": map[string]interface{}{
			"minutesRequested": minutes,
			"transferCount": len(recentTransfers),
			"transfers": recentTransfers,
		},
		"timestamp": time.Now().UnixMilli(),
	})
}

// Helper function for min
func min(a, b int) int {
    if a < b {
        return a
    }
    return b
} 
