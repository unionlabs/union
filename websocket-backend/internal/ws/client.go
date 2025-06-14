package ws

import (
	"encoding/json"
	"errors"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"websocket-backend/internal/models"
	"websocket-backend/internal/utils"
)

// Errors
var (
	ErrClientBufferFull = errors.New("client buffer is full")
)

// Client represents a WebSocket client optimized for high concurrency
type Client struct {
	conn        *websocket.Conn
	send        chan []byte
	filter      *models.ChainFilter
	mu          sync.RWMutex
	cleanup     func(*Client)
	closed      bool
	closeMu     sync.Mutex
}

// NewClient creates a new WebSocket client with cleanup callback
func NewClient(conn *websocket.Conn, cleanup func(*Client)) *Client {
	return &Client{
		conn:    conn,
		send:    make(chan []byte, maxMessageQueue), // Use configurable buffer size
		cleanup: cleanup,
	}
}

// WritePump pumps messages from the client's send channel to the WebSocket connection
func (c *Client) WritePump() {
	defer c.close()

	// Set write deadline for each message
	ticker := time.NewTicker(54 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case message, ok := <-c.send:
			c.conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if !ok {
				c.conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			if err := c.conn.WriteMessage(websocket.TextMessage, message); err != nil {
				utils.WSLogger.Error("Error writing message", map[string]interface{}{
					"error": err.Error(),
				})
				return
			}

		case <-ticker.C:
			c.conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if err := c.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}

// ReadPump pumps messages from the WebSocket connection to the server
func (c *Client) ReadPump(handler func(*Client, []byte)) {
	defer c.close()

	// Set read deadline and pong handler
	c.conn.SetReadDeadline(time.Now().Add(60 * time.Second))
	c.conn.SetPongHandler(func(string) error {
		c.conn.SetReadDeadline(time.Now().Add(60 * time.Second))
		return nil
	})

	for {
		_, message, err := c.conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				utils.WSLogger.Error("Unexpected WebSocket close error", map[string]interface{}{
					"error": err.Error(),
				})
			}
			break
		}

		// Reset read deadline on each message
		c.conn.SetReadDeadline(time.Now().Add(60 * time.Second))
		handler(c, message)
	}
}

// close handles client cleanup
func (c *Client) close() {
	c.closeMu.Lock()
	defer c.closeMu.Unlock()

	if c.closed {
		return
	}
	c.closed = true

	// Close the connection
	c.conn.Close()

	// Close the send channel
	close(c.send)

	// Call cleanup callback
	if c.cleanup != nil {
		c.cleanup(c)
	}
}

// SetFilter sets the client's chain filter
func (c *Client) SetFilter(filter *models.ChainFilter) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.filter = filter
}

// GetFilter gets the client's chain filter
func (c *Client) GetFilter() *models.ChainFilter {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.filter
}

// Send sends a message to the client with non-blocking behavior
func (c *Client) Send(message interface{}) error {
	// Critical nil checks first
	if c == nil {
		return errors.New("client is nil")
	}
	
	c.closeMu.Lock()
	if c.closed {
		c.closeMu.Unlock()
		return errors.New("client is closed")
	}
	
	// Additional safety check
	if c.send == nil {
		c.closeMu.Unlock()
		return errors.New("client send channel is nil")
	}
	c.closeMu.Unlock()

	data, err := json.Marshal(message)
	if err != nil {
		return err
	}

	select {
	case c.send <- data:
		return nil
	default:
		// Channel is full, drop the message to prevent blocking
		utils.WSLogger.Warn("Client buffer full, dropping message")
		return ErrClientBufferFull
	}
}

// SendTransfers sends transfers to the client (optimized version)
func (c *Client) SendTransfers(transfers []models.Transfer) bool {
	if len(transfers) == 0 {
		return false
	}
	
	err := c.Send(struct {
		Type      string           `json:"type"`
		Data      []models.Transfer `json:"data"`
		Timestamp int64            `json:"timestamp"`
	}{
		Type:      "transfers",
		Data:      transfers,
		Timestamp: time.Now().UnixMilli(),
	})
	return err == nil
}

// SendConnected sends a connected message to the client
func (c *Client) SendConnected() error {
	return c.Send(struct {
		Type      string `json:"type"`
		Message   string `json:"message"`
		Timestamp int64  `json:"timestamp"`
	}{
		Type:      "connected",
		Message:   "Connected to Union transfer stream (optimized for 100k connections)",
		Timestamp: time.Now().UnixMilli(),
	})
}

// SendChains sends chain data to the client
func (c *Client) SendChains(chains []models.Chain) error {
	if len(chains) == 0 {
		return errors.New("no chains to send")
	}
	
	return c.Send(struct {
		Type      string        `json:"type"`
		Data      []models.Chain `json:"data"`
		Timestamp int64         `json:"timestamp"`
	}{
		Type:      "chains",
		Data:      chains,
		Timestamp: time.Now().UnixMilli(),
	})
}

// SendServerInfo sends server info to the client
func (c *Client) SendServerInfo() error {
	return c.Send(struct {
		Type      string `json:"type"`
		Data      struct {
			ServerPrecomputation string   `json:"serverPrecomputation"`
			Features            []string `json:"features"`
			Architecture        string   `json:"architecture"`
			MaxConnections      int      `json:"maxConnections"`
		} `json:"data"`
		Timestamp int64 `json:"timestamp"`
	}{
		Type: "serverInfo",
		Data: struct {
			ServerPrecomputation string   `json:"serverPrecomputation"`
			Features            []string `json:"features"`
			Architecture        string   `json:"architecture"`
			MaxConnections      int      `json:"maxConnections"`
		}{
			ServerPrecomputation: "enabled",
			Features: []string{
				"testnet_flags_precomputed",
				"display_names_precomputed",
				"full_addresses_provided",
				"timestamps_formatted",
				"route_keys_generated",
				"chains_data_provided",
				"high_concurrency_optimized",
				"sharded_client_management",
				"parallel_broadcasting",
			},
			Architecture:   "sharded_workers",
			MaxConnections: 100000,
		},
		Timestamp: time.Now().UnixMilli(),
	})
}

// SendFilterSet sends filter set confirmation to the client
func (c *Client) SendFilterSet(fromChain, toChain string) error {
	return c.Send(struct {
		Type string `json:"type"`
		Data struct {
			FromChain string `json:"fromChain"`
			ToChain   string `json:"toChain"`
		} `json:"data"`
	}{
		Type: "filterSet",
		Data: struct {
			FromChain string `json:"fromChain"`
			ToChain   string `json:"toChain"`
		}{
			FromChain: fromChain,
			ToChain:   toChain,
		},
	})
}

// SendChartData sends chart data to the client
func (c *Client) SendChartData(chartData interface{}) error {
	if chartData == nil {
		return errors.New("chart data is nil")
	}
	
	return c.Send(struct {
		Type      string      `json:"type"`
		Data      interface{} `json:"data"`
		Timestamp int64       `json:"timestamp"`
	}{
		Type:      "chartData",
		Data:      chartData,
		Timestamp: time.Now().UnixMilli(),
	})
}

// IsClosed returns whether the client is closed
func (c *Client) IsClosed() bool {
	c.closeMu.Lock()
	defer c.closeMu.Unlock()
	return c.closed
}