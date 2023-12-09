package sentry

// Based on https://github.com/getsentry/vroom/blob/d11c26063e802d66b9a592c4010261746ca3dfa4/internal/sample/sample.go

import (
	"time"
)

type (
	profileDevice struct {
		Architecture   string `json:"architecture"`
		Classification string `json:"classification"`
		Locale         string `json:"locale"`
		Manufacturer   string `json:"manufacturer"`
		Model          string `json:"model"`
	}

	profileOS struct {
		BuildNumber string `json:"build_number"`
		Name        string `json:"name"`
		Version     string `json:"version"`
	}

	profileRuntime struct {
		Name    string `json:"name"`
		Version string `json:"version"`
	}

	profileSample struct {
		ElapsedSinceStartNS uint64 `json:"elapsed_since_start_ns"`
		StackID             int    `json:"stack_id"`
		ThreadID            uint64 `json:"thread_id"`
	}

	profileThreadMetadata struct {
		Name     string `json:"name,omitempty"`
		Priority int    `json:"priority,omitempty"`
	}

	profileStack []int

	profileTrace struct {
		Frames         []*Frame                          `json:"frames"`
		Samples        []profileSample                   `json:"samples"`
		Stacks         []profileStack                    `json:"stacks"`
		ThreadMetadata map[uint64]*profileThreadMetadata `json:"thread_metadata"`
	}

	profileInfo struct {
		DebugMeta   *DebugMeta         `json:"debug_meta,omitempty"`
		Device      profileDevice      `json:"device"`
		Environment string             `json:"environment,omitempty"`
		EventID     string             `json:"event_id"`
		OS          profileOS          `json:"os"`
		Platform    string             `json:"platform"`
		Release     string             `json:"release"`
		Dist        string             `json:"dist"`
		Runtime     profileRuntime     `json:"runtime"`
		Timestamp   time.Time          `json:"timestamp"`
		Trace       *profileTrace      `json:"profile"`
		Transaction profileTransaction `json:"transaction"`
		Version     string             `json:"version"`
	}

	// see https://github.com/getsentry/vroom/blob/a91e39416723ec44fc54010257020eeaf9a77cbd/internal/transaction/transaction.go
	profileTransaction struct {
		ActiveThreadID uint64  `json:"active_thread_id"`
		DurationNS     uint64  `json:"duration_ns,omitempty"`
		ID             EventID `json:"id"`
		Name           string  `json:"name"`
		TraceID        string  `json:"trace_id"`
	}
)
