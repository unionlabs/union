/*
 *
 * Copyright 2016 gRPC authors.
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

package main

import (
	"context"
	"flag"
	"math"
	"math/rand"
	"runtime"
	"sync"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/benchmark"
	"google.golang.org/grpc/benchmark/stats"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/syscall"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/testdata"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"

	_ "google.golang.org/grpc/xds" // To install the xds resolvers and balancers.
)

var caFile = flag.String("ca_file", "", "The file containing the CA root cert file")

type lockingHistogram struct {
	mu        sync.Mutex
	histogram *stats.Histogram
}

func (h *lockingHistogram) add(value int64) {
	h.mu.Lock()
	defer h.mu.Unlock()
	h.histogram.Add(value)
}

// swap sets h.histogram to o and returns its old value.
func (h *lockingHistogram) swap(o *stats.Histogram) *stats.Histogram {
	h.mu.Lock()
	defer h.mu.Unlock()
	old := h.histogram
	h.histogram = o
	return old
}

func (h *lockingHistogram) mergeInto(merged *stats.Histogram) {
	h.mu.Lock()
	defer h.mu.Unlock()
	merged.Merge(h.histogram)
}

type benchmarkClient struct {
	closeConns        func()
	stop              chan bool
	lastResetTime     time.Time
	histogramOptions  stats.HistogramOptions
	lockingHistograms []lockingHistogram
	rusageLastReset   *syscall.Rusage
}

func printClientConfig(config *testpb.ClientConfig) {
	// Some config options are ignored:
	// - client type:
	//     will always create sync client
	// - async client threads.
	// - core list
	logger.Infof(" * client type: %v (ignored, always creates sync client)", config.ClientType)
	logger.Infof(" * async client threads: %v (ignored)", config.AsyncClientThreads)
	// TODO: use cores specified by CoreList when setting list of cores is supported in go.
	logger.Infof(" * core list: %v (ignored)", config.CoreList)

	logger.Infof(" - security params: %v", config.SecurityParams)
	logger.Infof(" - core limit: %v", config.CoreLimit)
	logger.Infof(" - payload config: %v", config.PayloadConfig)
	logger.Infof(" - rpcs per chann: %v", config.OutstandingRpcsPerChannel)
	logger.Infof(" - channel number: %v", config.ClientChannels)
	logger.Infof(" - load params: %v", config.LoadParams)
	logger.Infof(" - rpc type: %v", config.RpcType)
	logger.Infof(" - histogram params: %v", config.HistogramParams)
	logger.Infof(" - server targets: %v", config.ServerTargets)
}

func setupClientEnv(config *testpb.ClientConfig) {
	// Use all cpu cores available on machine by default.
	// TODO: Revisit this for the optimal default setup.
	if config.CoreLimit > 0 {
		runtime.GOMAXPROCS(int(config.CoreLimit))
	} else {
		runtime.GOMAXPROCS(runtime.NumCPU())
	}
}

// createConns creates connections according to given config.
// It returns the connections and corresponding function to close them.
// It returns non-nil error if there is anything wrong.
func createConns(config *testpb.ClientConfig) ([]*grpc.ClientConn, func(), error) {
	opts := []grpc.DialOption{
		grpc.WithWriteBufferSize(128 * 1024),
		grpc.WithReadBufferSize(128 * 1024),
	}

	// Sanity check for client type.
	switch config.ClientType {
	case testpb.ClientType_SYNC_CLIENT:
	case testpb.ClientType_ASYNC_CLIENT:
	default:
		return nil, nil, status.Errorf(codes.InvalidArgument, "unknown client type: %v", config.ClientType)
	}

	// Check and set security options.
	if config.SecurityParams != nil {
		if *caFile == "" {
			*caFile = testdata.Path("ca.pem")
		}
		creds, err := credentials.NewClientTLSFromFile(*caFile, config.SecurityParams.ServerHostOverride)
		if err != nil {
			return nil, nil, status.Errorf(codes.InvalidArgument, "failed to create TLS credentials: %v", err)
		}
		opts = append(opts, grpc.WithTransportCredentials(creds))
	} else {
		opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}

	// Use byteBufCodec if it is required.
	if config.PayloadConfig != nil {
		switch config.PayloadConfig.Payload.(type) {
		case *testpb.PayloadConfig_BytebufParams:
			opts = append(opts, grpc.WithDefaultCallOptions(grpc.CallCustomCodec(byteBufCodec{})))
		case *testpb.PayloadConfig_SimpleParams:
		default:
			return nil, nil, status.Errorf(codes.InvalidArgument, "unknown payload config: %v", config.PayloadConfig)
		}
	}

	// Create connections.
	connCount := int(config.ClientChannels)
	conns := make([]*grpc.ClientConn, connCount)
	for connIndex := 0; connIndex < connCount; connIndex++ {
		conns[connIndex] = benchmark.NewClientConn(config.ServerTargets[connIndex%len(config.ServerTargets)], opts...)
	}

	return conns, func() {
		for _, conn := range conns {
			conn.Close()
		}
	}, nil
}

func performRPCs(config *testpb.ClientConfig, conns []*grpc.ClientConn, bc *benchmarkClient) error {
	// Read payload size and type from config.
	var (
		payloadReqSize, payloadRespSize int
		payloadType                     string
	)
	if config.PayloadConfig != nil {
		switch c := config.PayloadConfig.Payload.(type) {
		case *testpb.PayloadConfig_BytebufParams:
			payloadReqSize = int(c.BytebufParams.ReqSize)
			payloadRespSize = int(c.BytebufParams.RespSize)
			payloadType = "bytebuf"
		case *testpb.PayloadConfig_SimpleParams:
			payloadReqSize = int(c.SimpleParams.ReqSize)
			payloadRespSize = int(c.SimpleParams.RespSize)
			payloadType = "protobuf"
		default:
			return status.Errorf(codes.InvalidArgument, "unknown payload config: %v", config.PayloadConfig)
		}
	}

	// If set, perform an open loop, if not perform a closed loop. An open loop
	// asynchronously starts RPCs based on random start times derived from a
	// Poisson distribution. A closed loop performs RPCs in a blocking manner,
	// and runs the next RPC after the previous RPC completes and returns.
	var poissonLambda *float64
	switch t := config.LoadParams.Load.(type) {
	case *testpb.LoadParams_ClosedLoop:
	case *testpb.LoadParams_Poisson:
		if t.Poisson == nil {
			return status.Errorf(codes.InvalidArgument, "poisson is nil, needs to be set")
		}
		if t.Poisson.OfferedLoad <= 0 {
			return status.Errorf(codes.InvalidArgument, "poisson.offered is <= 0: %v, needs to be >0", t.Poisson.OfferedLoad)
		}
		poissonLambda = &t.Poisson.OfferedLoad
	default:
		return status.Errorf(codes.InvalidArgument, "unknown load params: %v", config.LoadParams)
	}

	rpcCountPerConn := int(config.OutstandingRpcsPerChannel)

	switch config.RpcType {
	case testpb.RpcType_UNARY:
		bc.unaryLoop(conns, rpcCountPerConn, payloadReqSize, payloadRespSize, poissonLambda)
	case testpb.RpcType_STREAMING:
		bc.streamingLoop(conns, rpcCountPerConn, payloadReqSize, payloadRespSize, payloadType, poissonLambda)
	default:
		return status.Errorf(codes.InvalidArgument, "unknown rpc type: %v", config.RpcType)
	}

	return nil
}

func startBenchmarkClient(config *testpb.ClientConfig) (*benchmarkClient, error) {
	printClientConfig(config)

	// Set running environment like how many cores to use.
	setupClientEnv(config)

	conns, closeConns, err := createConns(config)
	if err != nil {
		return nil, err
	}

	rpcCountPerConn := int(config.OutstandingRpcsPerChannel)
	bc := &benchmarkClient{
		histogramOptions: stats.HistogramOptions{
			NumBuckets:     int(math.Log(config.HistogramParams.MaxPossible)/math.Log(1+config.HistogramParams.Resolution)) + 1,
			GrowthFactor:   config.HistogramParams.Resolution,
			BaseBucketSize: (1 + config.HistogramParams.Resolution),
			MinValue:       0,
		},
		lockingHistograms: make([]lockingHistogram, rpcCountPerConn*len(conns)),

		stop:            make(chan bool),
		lastResetTime:   time.Now(),
		closeConns:      closeConns,
		rusageLastReset: syscall.GetRusage(),
	}

	if err = performRPCs(config, conns, bc); err != nil {
		// Close all connections if performRPCs failed.
		closeConns()
		return nil, err
	}

	return bc, nil
}

func (bc *benchmarkClient) unaryLoop(conns []*grpc.ClientConn, rpcCountPerConn int, reqSize int, respSize int, poissonLambda *float64) {
	for ic, conn := range conns {
		client := testgrpc.NewBenchmarkServiceClient(conn)
		// For each connection, create rpcCountPerConn goroutines to do rpc.
		for j := 0; j < rpcCountPerConn; j++ {
			// Create histogram for each goroutine.
			idx := ic*rpcCountPerConn + j
			bc.lockingHistograms[idx].histogram = stats.NewHistogram(bc.histogramOptions)
			// Start goroutine on the created mutex and histogram.
			go func(idx int) {
				// TODO: do warm up if necessary.
				// Now relying on worker client to reserve time to do warm up.
				// The worker client needs to wait for some time after client is created,
				// before starting benchmark.
				if poissonLambda == nil { // Closed loop.
					done := make(chan bool)
					for {
						go func() {
							start := time.Now()
							if err := benchmark.DoUnaryCall(client, reqSize, respSize); err != nil {
								select {
								case <-bc.stop:
								case done <- false:
								}
								return
							}
							elapse := time.Since(start)
							bc.lockingHistograms[idx].add(int64(elapse))
							select {
							case <-bc.stop:
							case done <- true:
							}
						}()
						select {
						case <-bc.stop:
							return
						case <-done:
						}
					}
				} else { // Open loop.
					timeBetweenRPCs := time.Duration((rand.ExpFloat64() / *poissonLambda) * float64(time.Second))
					time.AfterFunc(timeBetweenRPCs, func() {
						bc.poissonUnary(client, idx, reqSize, respSize, *poissonLambda)
					})
				}

			}(idx)
		}
	}
}

func (bc *benchmarkClient) streamingLoop(conns []*grpc.ClientConn, rpcCountPerConn int, reqSize int, respSize int, payloadType string, poissonLambda *float64) {
	var doRPC func(testgrpc.BenchmarkService_StreamingCallClient, int, int) error
	if payloadType == "bytebuf" {
		doRPC = benchmark.DoByteBufStreamingRoundTrip
	} else {
		doRPC = benchmark.DoStreamingRoundTrip
	}
	for ic, conn := range conns {
		// For each connection, create rpcCountPerConn goroutines to do rpc.
		for j := 0; j < rpcCountPerConn; j++ {
			c := testgrpc.NewBenchmarkServiceClient(conn)
			stream, err := c.StreamingCall(context.Background())
			if err != nil {
				logger.Fatalf("%v.StreamingCall(_) = _, %v", c, err)
			}
			idx := ic*rpcCountPerConn + j
			bc.lockingHistograms[idx].histogram = stats.NewHistogram(bc.histogramOptions)
			if poissonLambda == nil { // Closed loop.
				// Start goroutine on the created mutex and histogram.
				go func(idx int) {
					// TODO: do warm up if necessary.
					// Now relying on worker client to reserve time to do warm up.
					// The worker client needs to wait for some time after client is created,
					// before starting benchmark.
					for {
						start := time.Now()
						if err := doRPC(stream, reqSize, respSize); err != nil {
							return
						}
						elapse := time.Since(start)
						bc.lockingHistograms[idx].add(int64(elapse))
						select {
						case <-bc.stop:
							return
						default:
						}
					}
				}(idx)
			} else { // Open loop.
				timeBetweenRPCs := time.Duration((rand.ExpFloat64() / *poissonLambda) * float64(time.Second))
				time.AfterFunc(timeBetweenRPCs, func() {
					bc.poissonStreaming(stream, idx, reqSize, respSize, *poissonLambda, doRPC)
				})
			}
		}
	}
}

func (bc *benchmarkClient) poissonUnary(client testgrpc.BenchmarkServiceClient, idx int, reqSize int, respSize int, lambda float64) {
	go func() {
		start := time.Now()
		if err := benchmark.DoUnaryCall(client, reqSize, respSize); err != nil {
			return
		}
		elapse := time.Since(start)
		bc.lockingHistograms[idx].add(int64(elapse))
	}()
	timeBetweenRPCs := time.Duration((rand.ExpFloat64() / lambda) * float64(time.Second))
	time.AfterFunc(timeBetweenRPCs, func() {
		bc.poissonUnary(client, idx, reqSize, respSize, lambda)
	})
}

func (bc *benchmarkClient) poissonStreaming(stream testgrpc.BenchmarkService_StreamingCallClient, idx int, reqSize int, respSize int, lambda float64, doRPC func(testgrpc.BenchmarkService_StreamingCallClient, int, int) error) {
	go func() {
		start := time.Now()
		if err := doRPC(stream, reqSize, respSize); err != nil {
			return
		}
		elapse := time.Since(start)
		bc.lockingHistograms[idx].add(int64(elapse))
	}()
	timeBetweenRPCs := time.Duration((rand.ExpFloat64() / lambda) * float64(time.Second))
	time.AfterFunc(timeBetweenRPCs, func() {
		bc.poissonStreaming(stream, idx, reqSize, respSize, lambda, doRPC)
	})
}

// getStats returns the stats for benchmark client.
// It resets lastResetTime and all histograms if argument reset is true.
func (bc *benchmarkClient) getStats(reset bool) *testpb.ClientStats {
	var wallTimeElapsed, uTimeElapsed, sTimeElapsed float64
	mergedHistogram := stats.NewHistogram(bc.histogramOptions)

	if reset {
		// Merging histogram may take some time.
		// Put all histograms aside and merge later.
		toMerge := make([]*stats.Histogram, len(bc.lockingHistograms))
		for i := range bc.lockingHistograms {
			toMerge[i] = bc.lockingHistograms[i].swap(stats.NewHistogram(bc.histogramOptions))
		}

		for i := 0; i < len(toMerge); i++ {
			mergedHistogram.Merge(toMerge[i])
		}

		wallTimeElapsed = time.Since(bc.lastResetTime).Seconds()
		latestRusage := syscall.GetRusage()
		uTimeElapsed, sTimeElapsed = syscall.CPUTimeDiff(bc.rusageLastReset, latestRusage)

		bc.rusageLastReset = latestRusage
		bc.lastResetTime = time.Now()
	} else {
		// Merge only, not reset.
		for i := range bc.lockingHistograms {
			bc.lockingHistograms[i].mergeInto(mergedHistogram)
		}

		wallTimeElapsed = time.Since(bc.lastResetTime).Seconds()
		uTimeElapsed, sTimeElapsed = syscall.CPUTimeDiff(bc.rusageLastReset, syscall.GetRusage())
	}

	b := make([]uint32, len(mergedHistogram.Buckets))
	for i, v := range mergedHistogram.Buckets {
		b[i] = uint32(v.Count)
	}
	return &testpb.ClientStats{
		Latencies: &testpb.HistogramData{
			Bucket:       b,
			MinSeen:      float64(mergedHistogram.Min),
			MaxSeen:      float64(mergedHistogram.Max),
			Sum:          float64(mergedHistogram.Sum),
			SumOfSquares: float64(mergedHistogram.SumOfSquares),
			Count:        float64(mergedHistogram.Count),
		},
		TimeElapsed: wallTimeElapsed,
		TimeUser:    uTimeElapsed,
		TimeSystem:  sTimeElapsed,
	}
}

func (bc *benchmarkClient) shutdown() {
	close(bc.stop)
	bc.closeConns()
}
