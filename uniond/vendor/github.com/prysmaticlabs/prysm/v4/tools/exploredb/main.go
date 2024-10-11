/**
 * Explore DB contents
 *
 * Given a beacon-chain DB, This tool provides many option to
 * inspect and explore it. For every non-empty bucket, print
 * the number of rows, bucket size,min/average/max size of values
 */

package main

import (
	"bytes"
	"context"
	"flag"
	"os"
	"path/filepath"
	"sync"
	"time"

	"github.com/dustin/go-humanize"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/kv"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	log "github.com/sirupsen/logrus"
	"github.com/status-im/keycard-go/hexutils"
	bolt "go.etcd.io/bbolt"
)

const (
	MaxUint64         = ^uint64(0)
	maxSlotsToDisplay = 2000000
)

var (
	datadir       = flag.String("datadir", "", "Path to data directory.")
	dbName        = flag.String("dbname", "", "database name.")
	command       = flag.String("command", "", "command to execute.")
	bucketName    = flag.String("bucket-name", "", "bucket to show contents.")
	rowLimit      = flag.Uint64("limit", 10, "limit to rows.")
	migrationName = flag.String("migration", "", "migration to cross check.")
	destDatadir   = flag.String("dest-datadir", "", "Path to destination data directory.")
)

// used to parallelize all the bucket stats
type bucketStat struct {
	bucketName     string
	noOfRows       uint64
	totalKeySize   uint64
	totalValueSize uint64
	minKeySize     uint64
	maxKeySize     uint64
	minValueSize   uint64
	maxValueSize   uint64
}

// used to parallelize state bucket processing
type modifiedState struct {
	state     state.BeaconState
	key       []byte
	valueSize uint64
	rowCount  uint64
}

// used to parallelize state summary bucket processing
type modifiedStateSummary struct {
	slot      primitives.Slot
	root      []byte
	key       []byte
	valueSize uint64
	rowCount  uint64
}

func main() {
	flag.Parse()

	// Check for the mandatory flags.
	if *datadir == "" {
		log.Fatal("Please specify --datadir <db path> to read the database")
	}
	if *dbName == "" {
		log.Fatal("Please specify --dbname <db file name> to specify the database file.")
	}

	// check if the database file is present.
	dbNameWithPath := filepath.Join(*datadir, *dbName)
	if _, err := os.Stat(dbNameWithPath); os.IsNotExist(err) {
		log.WithError(err).WithField("path", dbNameWithPath).Fatal("could not locate database file")
	}

	switch *command {
	case "bucket-stats":
		printBucketStats(dbNameWithPath)
	case "bucket-content":
		switch *bucketName {
		case "state",
			"state-summary":
			printBucketContents(dbNameWithPath, *rowLimit, *bucketName)
		default:
			log.Fatal("Oops, given bucket is supported for now.")
		}
	case "migration-check":
		destDbNameWithPath := filepath.Join(*destDatadir, *dbName)
		if _, err := os.Stat(destDbNameWithPath); os.IsNotExist(err) {
			log.WithError(err).WithField("path", destDbNameWithPath).Fatal("could not locate database file")
		}
		switch *migrationName {
		case "validator-entries":
			checkValidatorMigration(dbNameWithPath, destDbNameWithPath)
		default:
			log.Fatal("Oops, given migration is not supported for now.")
		}
	}
}

func printBucketStats(dbNameWithPath string) {
	groupSize := uint64(128)
	doneC := make(chan bool)
	statsC := make(chan *bucketStat, groupSize)
	go readBucketStat(dbNameWithPath, statsC)
	go printBucketStat(statsC, doneC)
	<-doneC
}

func printBucketContents(dbNameWithPath string, rowLimit uint64, bucketName string) {
	// get the keys within the supplied limit for the given bucket.
	bucketNameInBytes := []byte(bucketName)
	keys, sizes := keysOfBucket(dbNameWithPath, bucketNameInBytes, rowLimit)

	// create a new KV Store.
	dbDirectory := filepath.Dir(dbNameWithPath)
	db, openErr := kv.NewKVStore(context.Background(), dbDirectory)
	if openErr != nil {
		log.WithError(openErr).Fatal("could not open db")
	}

	// don't forget to close it when ejecting out of this function.
	defer func() {
		closeErr := db.Close()
		if closeErr != nil {
			log.WithError(closeErr).Fatal("could not close db")
		}
	}()

	// retrieve every element for keys in the list and call the respective display function.
	ctx := context.Background()
	groupSize := uint64(128)
	doneC := make(chan bool)
	switch bucketName {
	case "state":
		stateC := make(chan *modifiedState, groupSize)
		go readStates(ctx, db, stateC, keys, sizes)
		go printStates(stateC, doneC)

	case "state-summary":
		stateSummaryC := make(chan *modifiedStateSummary, groupSize)
		go readStateSummary(ctx, db, stateSummaryC, keys, sizes)
		go printStateSummary(stateSummaryC, doneC)
	}
	<-doneC
}

func readBucketStat(dbNameWithPath string, statsC chan<- *bucketStat) {
	// open the raw database file. If the file is busy, then exit.
	db, openErr := bolt.Open(dbNameWithPath, 0600, &bolt.Options{Timeout: 1 * time.Second})
	if openErr != nil {
		log.WithError(openErr).Fatal("could not open db to show bucket stats")
	}

	// make sure we close the database before ejecting out of this function.
	defer func() {
		closeErr := db.Close()
		if closeErr != nil {
			log.WithError(closeErr).Fatalf("could not close db after showing bucket stats")
		}
	}()

	// get a list of all the existing buckets.
	var buckets []string
	var bucketsMut sync.Mutex
	if viewErr1 := db.View(func(tx *bolt.Tx) error {
		return tx.ForEach(func(name []byte, buc *bolt.Bucket) error {
			bucketsMut.Lock()
			buckets = append(buckets, string(name))
			bucketsMut.Unlock()
			return nil
		})
	}); viewErr1 != nil {
		log.WithError(viewErr1).Fatal("could not read buckets from db while getting list of buckets")
	}

	// for every bucket, calculate the stats and send it for printing.
	// calculate the state of all the buckets in parallel.
	var wg sync.WaitGroup
	for _, bName := range buckets {
		wg.Add(1)
		go func(bukName string) {
			defer wg.Done()
			count := uint64(0)
			minValueSize := ^uint64(0)
			maxValueSize := uint64(0)
			totalValueSize := uint64(0)
			minKeySize := ^uint64(0)
			maxKeySize := uint64(0)
			totalKeySize := uint64(0)
			if viewErr2 := db.View(func(tx *bolt.Tx) error {
				b := tx.Bucket([]byte(bukName))
				if forEachErr := b.ForEach(func(k, v []byte) error {
					count++
					valueSize := uint64(len(v))
					if valueSize < minValueSize {
						minValueSize = valueSize
					}
					if valueSize > maxValueSize {
						maxValueSize = valueSize
					}
					totalValueSize += valueSize

					keyize := uint64(len(k))
					if keyize < minKeySize {
						minKeySize = keyize
					}
					if keyize > maxKeySize {
						maxKeySize = keyize
					}
					totalKeySize += uint64(len(k))
					return nil
				}); forEachErr != nil {
					log.WithError(forEachErr).Errorf("could not process row %d for bucket: %s", count, bukName)
					return forEachErr
				}
				return nil
			}); viewErr2 != nil {
				log.WithError(viewErr2).Errorf("could not get stats for bucket: %s", bukName)
				return
			}
			stat := &bucketStat{
				bucketName:     bukName,
				noOfRows:       count,
				totalKeySize:   totalKeySize,
				totalValueSize: totalValueSize,
				minKeySize:     minKeySize,
				maxKeySize:     maxKeySize,
				minValueSize:   minValueSize,
				maxValueSize:   maxValueSize,
			}
			statsC <- stat
		}(bName)
	}
	wg.Wait()
	close(statsC)
}

func readStates(ctx context.Context, db *kv.Store, stateC chan<- *modifiedState, keys [][]byte, sizes []uint64) {
	stateMap := make(map[uint64]*modifiedState)
	for rowCount, key := range keys {
		st, stateErr := db.State(ctx, bytesutil.ToBytes32(key))
		if stateErr != nil {
			log.WithError(stateErr).Errorf("could not get state for key : %s", hexutils.BytesToHex(key))
			continue
		}
		mst := &modifiedState{
			state:     st,
			key:       key,
			valueSize: sizes[rowCount],
			rowCount:  uint64(rowCount),
		}
		stateMap[uint64(st.Slot())] = mst
	}

	for i := uint64(0); i < maxSlotsToDisplay; i++ {
		if _, ok := stateMap[i]; ok {
			stateC <- stateMap[i]
		}
	}
	close(stateC)
}

func readStateSummary(ctx context.Context, db *kv.Store, stateSummaryC chan<- *modifiedStateSummary, keys [][]byte, sizes []uint64) {
	for rowCount, key := range keys {
		ss, ssErr := db.StateSummary(ctx, bytesutil.ToBytes32(key))
		if ssErr != nil {
			log.WithError(ssErr).Errorf("could not get state summary for key : %s", hexutils.BytesToHex(key))
			continue
		}
		mst := &modifiedStateSummary{
			slot:      ss.Slot,
			root:      ss.Root,
			key:       key,
			valueSize: sizes[rowCount],
			rowCount:  uint64(rowCount),
		}
		stateSummaryC <- mst
	}
	close(stateSummaryC)
}

func printBucketStat(statsC <-chan *bucketStat, doneC chan<- bool) {
	for stat := range statsC {
		if stat.noOfRows != 0 {
			averageValueSize := stat.totalValueSize / stat.noOfRows
			averageKeySize := stat.totalKeySize / stat.noOfRows
			log.Infof("------ %s ---------", stat.bucketName)
			log.Infof("NumberOfRows     = %d", stat.noOfRows)
			log.Infof("TotalBucketSize  =  %s", humanize.Bytes(stat.totalValueSize+stat.totalKeySize))
			log.Infof("KeySize          =  %s, (min = %s, avg = %s, max = %s)",
				humanize.Bytes(stat.totalKeySize),
				humanize.Bytes(stat.minKeySize),
				humanize.Bytes(averageKeySize),
				humanize.Bytes(stat.maxKeySize))
			log.Infof("ValueSize        = %s, (min = %s, avg = %s, max = %s)",
				humanize.Bytes(stat.totalValueSize),
				humanize.Bytes(stat.minValueSize),
				humanize.Bytes(averageValueSize),
				humanize.Bytes(stat.maxValueSize))
		}
	}
	doneC <- true
}

func printStates(stateC <-chan *modifiedState, doneC chan<- bool) {
	for mst := range stateC {
		st := mst.state
		log.Infof("---- row = %04d, slot = %8d, epoch = %8d, key = %s ----", mst.rowCount, st.Slot(), st.Slot()/params.BeaconConfig().SlotsPerEpoch, hexutils.BytesToHex(mst.key))
		log.Infof("key                           : %s", hexutils.BytesToHex(mst.key))
		log.Infof("value                         : compressed size = %s", humanize.Bytes(mst.valueSize))
		t := time.Unix(int64(st.GenesisTime()), 0) // lint:ignore uintcast -- Genesis time will not exceed int64 in your lifetime.
		log.Infof("genesis_time                  : %s", t.Format(time.UnixDate))
		log.Infof("genesis_validators_root       : %s", hexutils.BytesToHex(st.GenesisValidatorsRoot()))
		log.Infof("slot                          : %d", st.Slot())
		log.Infof("fork                          : previous_version = %b,  current_version = %b", st.Fork().PreviousVersion, st.Fork().CurrentVersion)
		log.Infof("latest_block_header           : sizeSSZ = %s", humanize.Bytes(uint64(st.LatestBlockHeader().SizeSSZ())))
		size, count := sizeAndCountOfByteList(st.BlockRoots())
		log.Infof("block_roots                   : size = %s, count =  %d", humanize.Bytes(size), count)
		size, count = sizeAndCountOfByteList(st.StateRoots())
		log.Infof("state_roots                   : size = %s, count = %d", humanize.Bytes(size), count)
		roots, err := st.HistoricalRoots()
		if err != nil {
			log.WithError(err).Error("could not get historical roots")
		} else {
			size, count = sizeAndCountOfByteList(roots)
			log.Infof("historical_roots              : size = %s, count = %d", humanize.Bytes(size), count)
		}
		log.Infof("eth1_data                     : sizeSSZ = %s", humanize.Bytes(uint64(st.Eth1Data().SizeSSZ())))
		size, count = sizeAndCountGeneric(st.Eth1DataVotes(), nil)
		log.Infof("eth1_data_votes               : sizeSSZ = %s, count = %d", humanize.Bytes(size), count)
		log.Infof("eth1_deposit_index            : %d", st.Eth1DepositIndex())
		size, count = sizeAndCountGeneric(st.Validators(), nil)
		log.Infof("validators                    : sizeSSZ = %s, count = %d", humanize.Bytes(size), count)
		size, count = sizeAndCountOfUin64List(st.Balances())
		log.Infof("balances                      : size = %s, count = %d", humanize.Bytes(size), count)
		size, count = sizeAndCountOfByteList(st.RandaoMixes())
		log.Infof("randao_mixes                  : size = %s, count = %d", humanize.Bytes(size), count)
		size, count = sizeAndCountOfUin64List(st.Slashings())
		log.Infof("slashings                     : size = %s, count = %d", humanize.Bytes(size), count)
		size, count = sizeAndCountGeneric(st.PreviousEpochAttestations())
		log.Infof("previous_epoch_attestations   : sizeSSZ = %s, count = %d", humanize.Bytes(size), count)
		size, count = sizeAndCountGeneric(st.CurrentEpochAttestations())
		log.Infof("current_epoch_attestations    : sizeSSZ = %s, count = %d", humanize.Bytes(size), count)
		justificationBits := st.JustificationBits()
		log.Infof("justification_bits            : size =  %s, count = %d", humanize.Bytes(justificationBits.Len()), justificationBits.Count())
		log.Infof("previous_justified_checkpoint : sizeSSZ = %s", humanize.Bytes(uint64(st.PreviousJustifiedCheckpoint().SizeSSZ())))
		log.Infof("current_justified_checkpoint  : sizeSSZ = %s", humanize.Bytes(uint64(st.CurrentJustifiedCheckpoint().SizeSSZ())))
		log.Infof("finalized_checkpoint          : sizeSSZ = %s", humanize.Bytes(uint64(st.FinalizedCheckpoint().SizeSSZ())))
	}
	doneC <- true
}

func printStateSummary(stateSummaryC <-chan *modifiedStateSummary, doneC chan<- bool) {
	for msts := range stateSummaryC {
		log.Infof("row : %04d, slot : %d, root = %s", msts.rowCount, msts.slot, hexutils.BytesToHex(msts.root))
	}
	doneC <- true
}

func checkValidatorMigration(dbNameWithPath, destDbNameWithPath string) {
	// get the keys within the supplied limit for the given bucket.
	sourceStateKeys, _ := keysOfBucket(dbNameWithPath, []byte("state"), MaxUint64)
	destStateKeys, _ := keysOfBucket(destDbNameWithPath, []byte("state"), MaxUint64)

	if len(destStateKeys) < len(sourceStateKeys) {
		log.Fatalf("destination keys are lesser then source keys (%d/%d)", len(sourceStateKeys), len(destStateKeys))
	}

	// create the source and destination KV stores.
	sourceDbDirectory := filepath.Dir(dbNameWithPath)
	sourceDB, openErr := kv.NewKVStore(context.Background(), sourceDbDirectory)
	if openErr != nil {
		log.WithError(openErr).Fatal("could not open sourceDB")
	}

	destinationDbDirectory := filepath.Dir(destDbNameWithPath)
	destDB, openErr := kv.NewKVStore(context.Background(), destinationDbDirectory)
	if openErr != nil {
		// dirty hack alert: Ignore this prometheus error as we are opening two DB with same metric name
		// if you want to avoid this then we should pass the metric name when opening the DB which touches
		// too many places.
		if openErr.Error() != "duplicate metrics collector registration attempted" {
			log.WithError(openErr).Fatalf("could not open sourceDB")
		}
	}

	// don't forget to close it when ejecting out of this function.
	defer func() {
		closeErr := sourceDB.Close()
		if closeErr != nil {
			log.WithError(closeErr).Fatal("could not close sourceDB")
		}
	}()
	defer func() {
		closeErr := destDB.Close()
		if closeErr != nil {
			log.WithError(closeErr).Fatal("could not close sourceDB")
		}
	}()

	ctx := context.Background()
	failCount := 0
	for rowCount, key := range sourceStateKeys[910:] {
		sourceState, stateErr := sourceDB.State(ctx, bytesutil.ToBytes32(key))
		if stateErr != nil {
			log.WithError(stateErr).WithField("key", hexutils.BytesToHex(key)).Fatalf("could not get from source db, the state for key")
		}
		destinationState, stateErr := destDB.State(ctx, bytesutil.ToBytes32(key))
		if stateErr != nil {
			log.WithError(stateErr).WithField("key", hexutils.BytesToHex(key)).Fatalf("could not get from destination db, the state for key")
		}
		if destinationState == nil {
			log.Infof("could not find state in migrated DB: index = %d, slot = %d, epoch = %d,  numOfValidators = %d, key = %s",
				rowCount, sourceState.Slot(), sourceState.Slot()/params.BeaconConfig().SlotsPerEpoch, sourceState.NumValidators(), hexutils.BytesToHex(key))
			failCount++
			continue
		}

		if len(sourceState.Validators()) != len(destinationState.Validators()) {
			log.Fatalf("validator mismatch : source = %d, dest = %d", len(sourceState.Validators()), len(destinationState.Validators()))
		}
		sourceStateHash, err := sourceState.HashTreeRoot(ctx)
		if err != nil {
			log.WithError(err).Fatal("could not find hash of source state")
		}
		destinationStateHash, err := destinationState.HashTreeRoot(ctx)
		if err != nil {
			log.WithError(err).Fatal("could not find hash of destination state")
		}
		if !bytes.Equal(sourceStateHash[:], destinationStateHash[:]) {
			log.Fatalf("state mismatch : key = %s", hexutils.BytesToHex(key))
		}
	}
	log.Infof("number of state that did not match: %d", failCount)
}

func keysOfBucket(dbNameWithPath string, bucketName []byte, rowLimit uint64) ([][]byte, []uint64) {
	// open the raw database file. If the file is busy, then exit.
	db, openErr := bolt.Open(dbNameWithPath, 0600, &bolt.Options{Timeout: 1 * time.Second})
	if openErr != nil {
		log.WithError(openErr).Fatal("could not open db while getting keys of a bucket")
	}

	// make sure we close the database before ejecting out of this function.
	defer func() {
		closeErr := db.Close()
		if closeErr != nil {
			log.WithError(closeErr).Fatal("could not close db while getting keys of a bucket")
		}
	}()

	// get all the keys of the given bucket.
	var keys [][]byte
	var sizes []uint64
	if viewErr := db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(bucketName)
		c := b.Cursor()
		count := uint64(0)
		for k, v := c.First(); k != nil; k, v = c.Next() {
			if count >= rowLimit {
				return nil
			}
			actualKey := make([]byte, len(k))
			actualSizes := make([]byte, len(v))
			copy(actualKey, k)
			copy(actualSizes, v)
			keys = append(keys, actualKey)
			sizes = append(sizes, uint64(len(v)))
			count++
		}
		return nil
	}); viewErr != nil {
		log.WithError(viewErr).Fatal("could not read keys of bucket from db")
	}
	return keys, sizes
}

func sizeAndCountOfByteList(list [][]byte) (uint64, uint64) {
	size := uint64(0)
	count := uint64(0)
	for _, root := range list {
		size += uint64(len(root))
		count += 1
	}
	return size, count
}

func sizeAndCountOfUin64List(list []uint64) (uint64, uint64) {
	size := uint64(0)
	count := uint64(0)
	for i := 0; i < len(list); i++ {
		size += uint64(8)
		count += 1
	}
	return size, count
}

func sizeAndCountGeneric(genericItems interface{}, err error) (uint64, uint64) {
	size := uint64(0)
	count := uint64(0)
	if err != nil {
		return size, count
	}

	switch items := genericItems.(type) {
	case []*ethpb.Eth1Data:
		for _, item := range items {
			size += uint64(item.SizeSSZ())
		}
		count = uint64(len(items))
	case []*ethpb.Validator:
		for _, item := range items {
			size += uint64(item.SizeSSZ())
		}
		count = uint64(len(items))
	case []*ethpb.PendingAttestation:
		for _, item := range items {
			size += uint64(item.SizeSSZ())
		}
		count = uint64(len(items))
	default:
		return 0, 0
	}

	return size, count
}
