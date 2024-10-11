package db

import (
	"bytes"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	log "github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
	bolt "go.etcd.io/bbolt"
)

var queryFlags = struct {
	Path     string
	Bucket   string
	KeysOnly bool
	Prefix   string
}{}

var queryCmd = &cli.Command{
	Name:  "query",
	Usage: "database query tool",
	Action: func(cliCtx *cli.Context) error {
		if err := queryAction(cliCtx); err != nil {
			log.WithError(err).Fatal("Could not query db")
		}
		return nil
	},
	Flags: []cli.Flag{
		&cli.StringFlag{
			Name:        "bucket",
			Usage:       "boltdb bucket to search",
			Destination: &queryFlags.Bucket,
		},
		&cli.StringFlag{
			Name:        "path",
			Usage:       "path to directory containing beaconchain.db",
			Destination: &queryFlags.Path,
		},
		&cli.StringFlag{
			Name:        "prefix",
			Usage:       "prefix of db record key to match against (eg 0xa1 would match 0xa10, 0xa1f etc)",
			Destination: &queryFlags.Prefix,
		},
		&cli.BoolFlag{
			Name:        "print-keys",
			Usage:       "only display keys, not values",
			Destination: &queryFlags.KeysOnly,
		},
	},
}

func queryAction(_ *cli.Context) error {
	flags := queryFlags
	db, err := getDB(flags.Path)
	if err != nil {
		return err
	}
	if flags.Prefix != "" {
		return prefixScan(db, flags.Bucket, flags.Prefix, flags.KeysOnly)
	}
	return nil
}

func prefixScan(db *bolt.DB, bucket, prefix string, keysOnly bool) error {
	if !keysOnly {
		return errors.New("prefix scan with value display not implemented")
	}
	pb, err := hexutil.Decode(prefix)
	if err != nil {
		return err
	}
	log.Infof("scanning for prefix=%#x", pb)
	return db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket([]byte(bucket))
		c := b.Cursor()
		for k, _ := c.Seek(pb); k != nil && bytes.HasPrefix(k, pb); k, _ = c.Next() {
			fmt.Printf("%#x\n", k)
		}
		return nil
	})
}

func getDB(path string) (*bolt.DB, error) {
	bdb, err := bolt.Open(
		path,
		params.BeaconIoConfig().ReadWritePermissions,
		&bolt.Options{
			Timeout: 1 * time.Second,
		},
	)
	if err != nil {
		if errors.Is(err, bolt.ErrTimeout) {
			return nil, errors.New("cannot obtain database lock, database may be in use by another process")
		}
		return nil, err
	}
	return bdb, nil
}
