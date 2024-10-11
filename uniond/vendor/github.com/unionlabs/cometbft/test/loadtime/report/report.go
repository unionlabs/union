package report

import (
	"math"
	"sort"
	"sync"
	"time"

	"github.com/google/uuid"
	"gonum.org/v1/gonum/stat"

	"github.com/cometbft/cometbft/test/loadtime/payload"
	"github.com/cometbft/cometbft/types"
)

// BlockStore defines the set of methods needed by the report generator from
// CometBFT's store.Blockstore type. Using an interface allows for tests to
// more easily simulate the required behavior without having to use the more
// complex real API.
type BlockStore interface {
	Height() int64
	Base() int64
	LoadBlock(height int64) (*types.Block, *types.BlockMeta)
}

// DataPoint contains the set of data collected for each transaction.
type DataPoint struct {
	Duration  time.Duration
	BlockTime time.Time
	Hash      []byte
}

// Report contains the data calculated from reading the timestamped transactions
// of each block found in the blockstore.
type Report struct {
	ID                      uuid.UUID
	Rate, Connections, Size uint64
	Max, Min, Avg, StdDev   time.Duration

	// NegativeCount is the number of negative durations encountered while
	// reading the transaction data. A negative duration means that
	// a transaction timestamp was greater than the timestamp of the block it
	// was included in and likely indicates an issue with the experimental
	// setup.
	NegativeCount int

	// All contains all data points gathered from all valid transactions.
	// The order of the contents of All is not guaranteed to be match the order of transactions
	// in the chain.
	All []DataPoint

	// used for calculating average during report creation.
	sum int64
}

type Reports struct {
	s map[uuid.UUID]Report
	l []Report

	// errorCount is the number of parsing errors encountered while reading the
	// transaction data. Parsing errors may occur if a transaction not generated
	// by the payload package is submitted to the chain.
	errorCount int
}

func (rs *Reports) List() []Report {
	return rs.l
}

func (rs *Reports) ErrorCount() int {
	return rs.errorCount
}

func (rs *Reports) addDataPoint(id uuid.UUID, l time.Duration, bt time.Time, hash []byte, conns, rate, size uint64) {
	r, ok := rs.s[id]
	if !ok {
		r = Report{
			Max:         0,
			Min:         math.MaxInt64,
			ID:          id,
			Connections: conns,
			Rate:        rate,
			Size:        size,
		}
		rs.s[id] = r
	}
	r.All = append(r.All, DataPoint{Duration: l, BlockTime: bt, Hash: hash})
	if l > r.Max {
		r.Max = l
	}
	if l < r.Min {
		r.Min = l
	}
	if int64(l) < 0 {
		r.NegativeCount++
	}
	// Using an int64 here makes an assumption about the scale and quantity of the data we are processing.
	// If all latencies were 2 seconds, we would need around 4 billion records to overflow this.
	// We are therefore assuming that the data does not exceed these bounds.
	r.sum += int64(l)
	rs.s[id] = r
}

func (rs *Reports) calculateAll() {
	rs.l = make([]Report, 0, len(rs.s))
	for _, r := range rs.s {
		if len(r.All) == 0 {
			r.Min = 0
			rs.l = append(rs.l, r)
			continue
		}
		r.Avg = time.Duration(r.sum / int64(len(r.All)))
		r.StdDev = time.Duration(int64(stat.StdDev(toFloat(r.All), nil)))
		rs.l = append(rs.l, r)
	}
	sort.Slice(rs.l, func(i, j int) bool {
		if rs.l[i].Connections == rs.l[j].Connections {
			return rs.l[i].Rate < rs.l[j].Rate
		}
		return rs.l[i].Connections < rs.l[j].Connections
	})
}

func (rs *Reports) addError() {
	rs.errorCount++
}

// GenerateFromBlockStore creates a Report using the data in the provided
// BlockStore.
func GenerateFromBlockStore(s BlockStore) (*Reports, error) {
	type payloadData struct {
		id                      uuid.UUID
		l                       time.Duration
		bt                      time.Time
		hash                    []byte
		connections, rate, size uint64
		err                     error
	}
	type txData struct {
		tx types.Tx
		bt time.Time
	}
	reports := &Reports{
		s: make(map[uuid.UUID]Report),
	}

	// Deserializing to proto can be slow but does not depend on other data
	// and can therefore be done in parallel.
	// Deserializing in parallel does mean that the resulting data is
	// not guaranteed to be delivered in the same order it was given to the
	// worker pool.
	const poolSize = 16

	txc := make(chan txData)
	pdc := make(chan payloadData, poolSize)

	wg := &sync.WaitGroup{}
	wg.Add(poolSize)
	for i := 0; i < poolSize; i++ {
		go func() {
			defer wg.Done()
			for b := range txc {
				p, err := payload.FromBytes(b.tx)
				if err != nil {
					pdc <- payloadData{err: err}
					continue
				}

				l := b.bt.Sub(p.GetTime().AsTime())
				idb := (*[16]byte)(p.GetId())
				pdc <- payloadData{
					l:           l,
					bt:          b.bt,
					hash:        b.tx.Hash(),
					id:          uuid.UUID(*idb),
					connections: p.GetConnections(),
					rate:        p.GetRate(),
					size:        p.GetSize(),
				}
			}
		}()
	}
	go func() {
		wg.Wait()
		close(pdc)
	}()

	go func() {
		base, height := s.Base(), s.Height()
		prev, _ := s.LoadBlock(base)
		for i := base + 1; i < height; i++ {
			// Data from two adjacent block are used here simultaneously,
			// blocks of height H and H+1. The transactions of the block of
			// height H are used with the timestamp from the block of height
			// H+1. This is done because the timestamp from H+1 is calculated
			// by using the precommits submitted at height H. The timestamp in
			// block H+1 represents the time at which block H was committed.
			//
			// In the (very unlikely) event that the very last block of the
			// chain contains payload transactions, those transactions will not
			// be used in the latency calculations because the last block whose
			// transactions are used is the block one before the last.
			cur, _ := s.LoadBlock(i)
			for _, tx := range prev.Data.Txs {
				txc <- txData{tx: tx, bt: cur.Time}
			}
			prev = cur
		}
		close(txc)
	}()
	for pd := range pdc {
		if pd.err != nil {
			reports.addError()
			continue
		}
		reports.addDataPoint(pd.id, pd.l, pd.bt, pd.hash, pd.connections, pd.rate, pd.size)
	}
	reports.calculateAll()
	return reports, nil
}

func toFloat(in []DataPoint) []float64 {
	r := make([]float64, len(in))
	for i, v := range in {
		r[i] = float64(int64(v.Duration))
	}
	return r
}
