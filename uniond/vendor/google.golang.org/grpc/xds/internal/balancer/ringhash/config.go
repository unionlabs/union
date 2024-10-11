/*
 *
 * Copyright 2021 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package ringhash

import (
	"encoding/json"
	"fmt"

	"google.golang.org/grpc/internal/envconfig"
	"google.golang.org/grpc/serviceconfig"
)

// LBConfig is the balancer config for ring_hash balancer.
type LBConfig struct {
	serviceconfig.LoadBalancingConfig `json:"-"`

	MinRingSize uint64 `json:"minRingSize,omitempty"`
	MaxRingSize uint64 `json:"maxRingSize,omitempty"`
}

const (
	defaultMinSize         = 1024
	defaultMaxSize         = 4096
	ringHashSizeUpperBound = 8 * 1024 * 1024 // 8M
)

func parseConfig(c json.RawMessage) (*LBConfig, error) {
	var cfg LBConfig
	if err := json.Unmarshal(c, &cfg); err != nil {
		return nil, err
	}
	if cfg.MinRingSize > ringHashSizeUpperBound {
		return nil, fmt.Errorf("min_ring_size value of %d is greater than max supported value %d for this field", cfg.MinRingSize, ringHashSizeUpperBound)
	}
	if cfg.MaxRingSize > ringHashSizeUpperBound {
		return nil, fmt.Errorf("max_ring_size value of %d is greater than max supported value %d for this field", cfg.MaxRingSize, ringHashSizeUpperBound)
	}
	if cfg.MinRingSize == 0 {
		cfg.MinRingSize = defaultMinSize
	}
	if cfg.MaxRingSize == 0 {
		cfg.MaxRingSize = defaultMaxSize
	}
	if cfg.MinRingSize > cfg.MaxRingSize {
		return nil, fmt.Errorf("min %v is greater than max %v", cfg.MinRingSize, cfg.MaxRingSize)
	}
	if cfg.MinRingSize > envconfig.RingHashCap {
		cfg.MinRingSize = envconfig.RingHashCap
	}
	if cfg.MaxRingSize > envconfig.RingHashCap {
		cfg.MaxRingSize = envconfig.RingHashCap
	}
	return &cfg, nil
}
