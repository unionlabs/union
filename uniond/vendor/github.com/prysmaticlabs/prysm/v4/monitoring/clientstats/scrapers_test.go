package clientstats

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/sirupsen/logrus"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func init() {
	logrus.SetLevel(logrus.DebugLevel)
}

type mockRT struct {
	body       string
	status     string
	statusCode int
}

func (rt *mockRT) RoundTrip(_ *http.Request) (*http.Response, error) {
	return &http.Response{
		Status:     http.StatusText(http.StatusOK),
		StatusCode: http.StatusOK,
		Body:       io.NopCloser(strings.NewReader(rt.body)),
	}, nil
}

var _ http.RoundTripper = &mockRT{}

func TestBeaconNodeScraper(t *testing.T) {
	bnScraper := beaconNodeScraper{}
	bnScraper.tripper = &mockRT{body: prometheusTestBody}
	r, err := bnScraper.Scrape()
	require.NoError(t, err, "Unexpected error calling beaconNodeScraper.Scrape")
	bs := &BeaconNodeStats{}
	err = json.NewDecoder(r).Decode(bs)
	require.NoError(t, err, "Unexpected error decoding result of beaconNodeScraper.Scrape")
	// CommonStats
	require.Equal(t, int64(225), bs.CPUProcessSecondsTotal)
	require.Equal(t, int64(1166630912), bs.MemoryProcessBytes)
	require.Equal(t, int64(1619586241), bs.ClientBuild)
	require.Equal(t, "v1.3.8-hotfix+6c0942", bs.ClientVersion)
	require.Equal(t, "prysm", bs.ClientName)

	// BeaconNodeStats
	require.Equal(t, int64(256552), bs.SyncBeaconHeadSlot)
	require.Equal(t, true, bs.SyncEth2Synced)
	require.Equal(t, int64(7365341184), bs.DiskBeaconchainBytesTotal)
	require.Equal(t, int64(37), bs.NetworkPeersConnected)
	require.Equal(t, true, bs.SyncEth1Connected)
}

// helper function to wrap up all the scrape logic so tests can focus on data cases and assertions
func scrapeBeaconNodeStats(body string) (*BeaconNodeStats, error) {
	if !strings.HasSuffix(body, "\n") {
		return nil, fmt.Errorf("bad test fixture -- make sure there is a trailing newline unless you want to waste time debugging tests")
	}
	bnScraper := beaconNodeScraper{}
	bnScraper.tripper = &mockRT{body: body}
	r, err := bnScraper.Scrape()
	if err != nil {
		return nil, err
	}
	bs := &BeaconNodeStats{}
	err = json.NewDecoder(r).Decode(bs)
	return bs, err
}

func TestInvertEth1Metrics(t *testing.T) {
	cases := []struct {
		key  string
		body string
		test func(*BeaconNodeStats) bool
	}{
		{
			key:  "SyncEth1Connected",
			body: strings.Replace(prometheusTestBody, "powchain_sync_eth1_connected 1", "powchain_sync_eth1_connected 0", 1),
			test: func(bs *BeaconNodeStats) bool {
				return bs.SyncEth1Connected == false
			},
		},
	}
	for _, c := range cases {
		bs, err := scrapeBeaconNodeStats(c.body)
		require.NoError(t, err)
		require.Equal(t, true, c.test(bs), "BeaconNodeStats.%s was not false, with prometheus body=%s", c.key, c.body)
	}
}

func TestFalseEth2Synced(t *testing.T) {
	bnScraper := beaconNodeScraper{}
	eth2NotSynced := strings.Replace(prometheusTestBody, "beacon_head_slot 256552", "beacon_head_slot 256559", 1)
	bnScraper.tripper = &mockRT{body: eth2NotSynced}
	r, err := bnScraper.Scrape()
	require.NoError(t, err, "Unexpected error calling beaconNodeScraper.Scrape")

	bs := &BeaconNodeStats{}
	err = json.NewDecoder(r).Decode(bs)
	require.NoError(t, err, "Unexpected error decoding result of beaconNodeScraper.Scrape")

	require.Equal(t, false, bs.SyncEth2Synced)
}

func TestValidatorScraper(t *testing.T) {
	vScraper := validatorScraper{}
	vScraper.tripper = &mockRT{body: statusFixtureOneOfEach + prometheusTestBody}
	r, err := vScraper.Scrape()
	require.NoError(t, err, "Unexpected error calling validatorScraper.Scrape")
	vs := &ValidatorStats{}
	err = json.NewDecoder(r).Decode(vs)
	require.NoError(t, err, "Unexpected error decoding result of validatorScraper.Scrape")
	// CommonStats
	require.Equal(t, int64(225), vs.CPUProcessSecondsTotal)
	require.Equal(t, int64(1166630912), vs.MemoryProcessBytes)
	require.Equal(t, int64(1619586241), vs.ClientBuild)
	require.Equal(t, "v1.3.8-hotfix+6c0942", vs.ClientVersion)
	require.Equal(t, "prysm", vs.ClientName)
	require.Equal(t, int64(7), vs.ValidatorTotal)
	require.Equal(t, int64(1), vs.ValidatorActive)
}

func TestValidatorScraperAllActive(t *testing.T) {
	vScraper := validatorScraper{}
	vScraper.tripper = &mockRT{body: statusFixtureAllActive + prometheusTestBody}
	r, err := vScraper.Scrape()
	require.NoError(t, err, "Unexpected error calling validatorScraper.Scrape")
	vs := &ValidatorStats{}
	err = json.NewDecoder(r).Decode(vs)
	require.NoError(t, err, "Unexpected error decoding result of validatorScraper.Scrape")
	// CommonStats
	require.Equal(t, int64(4), vs.ValidatorTotal)
	require.Equal(t, int64(4), vs.ValidatorActive)
}

func TestValidatorScraperNoneActive(t *testing.T) {
	vScraper := validatorScraper{}
	vScraper.tripper = &mockRT{body: statusFixtureNoneActive + prometheusTestBody}
	r, err := vScraper.Scrape()
	require.NoError(t, err, "Unexpected error calling validatorScraper.Scrape")
	vs := &ValidatorStats{}
	err = json.NewDecoder(r).Decode(vs)
	require.NoError(t, err, "Unexpected error decoding result of validatorScraper.Scrape")
	// CommonStats
	require.Equal(t, int64(6), vs.ValidatorTotal)
	require.Equal(t, int64(0), vs.ValidatorActive)
}

func mockNowFunc(fixedTime time.Time) func() time.Time {
	return func() time.Time {
		return fixedTime
	}
}

func TestValidatorAPIMessageDefaults(t *testing.T) {
	now = mockNowFunc(time.Unix(1619811114, 123456789))
	// 1+e6 ns per ms, so 123456789 ns rounded down should be 123 ms
	nowMillis := int64(1619811114123)
	vScraper := validatorScraper{}
	vScraper.tripper = &mockRT{body: statusFixtureOneOfEach + prometheusTestBody}
	r, err := vScraper.Scrape()
	require.NoError(t, err, "unexpected error from validatorScraper.Scrape()")

	vs := &ValidatorStats{}
	err = json.NewDecoder(r).Decode(vs)
	require.NoError(t, err, "Unexpected error decoding result of validatorScraper.Scrape")

	// CommonStats
	require.Equal(t, nowMillis, vs.Timestamp, "Unexpected 'timestamp' in client-stats APIMessage struct")
	require.Equal(t, APIVersion, vs.APIVersion, "Unexpected 'version' in client-stats APIMessage struct")
	require.Equal(t, ValidatorProcessName, vs.ProcessName, "Unexpected value for 'process' in client-stats APIMessage struct")
}

func TestBeaconNodeAPIMessageDefaults(t *testing.T) {
	now = mockNowFunc(time.Unix(1619811114, 123456789))
	// 1+e6 ns per ms, so 123456789 ns rounded down should be 123 ms
	nowMillis := int64(1619811114123)
	bScraper := beaconNodeScraper{}
	bScraper.tripper = &mockRT{body: prometheusTestBody}
	r, err := bScraper.Scrape()
	require.NoError(t, err, "unexpected error from beaconNodeScraper.Scrape()")

	vs := &BeaconNodeStats{}
	err = json.NewDecoder(r).Decode(vs)
	require.NoError(t, err, "Unexpected error decoding result of beaconNodeScraper.Scrape")

	// CommonStats
	require.Equal(t, nowMillis, vs.Timestamp, "Unexpected 'timestamp' in client-stats APIMessage struct")
	require.Equal(t, APIVersion, vs.APIVersion, "Unexpected 'version' in client-stats APIMessage struct")
	require.Equal(t, BeaconNodeProcessName, vs.ProcessName, "Unexpected value for 'process' in client-stats APIMessage struct")
}

func TestBadInput(t *testing.T) {
	hook := logTest.NewGlobal()
	bnScraper := beaconNodeScraper{}
	bnScraper.tripper = &mockRT{body: ""}
	_, err := bnScraper.Scrape()
	require.NoError(t, err)
	require.LogsContain(t, hook, "Failed to get prysm_version")
}

var prometheusTestBody = `
# HELP process_cpu_seconds_total Total user and system CPU time spent in seconds.
# TYPE process_cpu_seconds_total counter
process_cpu_seconds_total 225.09
# HELP process_resident_memory_bytes Resident memory size in bytes.
# TYPE process_resident_memory_bytes gauge
process_resident_memory_bytes 1.166630912e+09
# HELP prysm_version
# TYPE prysm_version gauge
prysm_version{buildDate="1619586241",commit="51eb1540fa838cdbe467bbeb0e36ee667d449377",version="v1.3.8-hotfix+6c0942"} 1
# HELP validator_count The total number of validators
# TYPE validator_count gauge
validator_count{state="Active"} 210301
validator_count{state="Exited"} 10
validator_count{state="Exiting"} 0
validator_count{state="Pending"} 0
validator_count{state="Slashed"} 0
validator_count{state="Slashing"} 0
# HELP beacon_head_slot Slot of the head block of the beacon chain
# TYPE beacon_head_slot gauge
beacon_head_slot 256552
# HELP beacon_clock_time_slot The current slot based on the genesis time and current clock
# TYPE beacon_clock_time_slot gauge
beacon_clock_time_slot 256552
# HELP bcnode_disk_beaconchain_bytes_total Total hard disk space used by the beaconchain database, in bytes. May include mmap.
# TYPE bcnode_disk_beaconchain_bytes_total gauge
bcnode_disk_beaconchain_bytes_total 7.365341184e+09
# HELP p2p_peer_count The number of peers in a given state.
# TYPE p2p_peer_count gauge
p2p_peer_count{state="Bad"} 1
p2p_peer_count{state="Connected"} 37
p2p_peer_count{state="Connecting"} 0
p2p_peer_count{state="Disconnected"} 62
p2p_peer_count{state="Disconnecting"} 0
# HELP powchain_sync_eth1_connected Boolean indicating whether a fallback eth1 endpoint is currently connected: 0=false, 1=true.
# TYPE powchain_sync_eth1_connected gauge
powchain_sync_eth1_connected 1
# HELP powchain_sync_eth1_fallback_configured Boolean recording whether a fallback eth1 endpoint was configured: 0=false, 1=true.
# TYPE powchain_sync_eth1_fallback_configured gauge
powchain_sync_eth1_fallback_configured 1
# HELP powchain_sync_eth1_fallback_connected Boolean indicating whether a fallback eth1 endpoint is currently connected: 0=false, 1=true.
# TYPE powchain_sync_eth1_fallback_connected gauge
powchain_sync_eth1_fallback_connected 1
`

var statusFixtureOneOfEach = `# HELP validator_statuses validator statuses: 0 UNKNOWN, 1 DEPOSITED, 2 PENDING, 3 ACTIVE, 4 EXITING, 5 SLASHING, 6 EXITED
# TYPE validator_statuses gauge
validator_statuses{pubkey="pk0"} 0
validator_statuses{pubkey="pk1"} 1
validator_statuses{pubkey="pk2"} 2
validator_statuses{pubkey="pk3"} 3
validator_statuses{pubkey="pk4"} 4
validator_statuses{pubkey="pk5"} 5
validator_statuses{pubkey="pk6"} 6
`

var statusFixtureAllActive = `# HELP validator_statuses validator statuses: 0 UNKNOWN, 1 DEPOSITED, 2 PENDING, 3 ACTIVE, 4 EXITING, 5 SLASHING, 6 EXITED
# TYPE validator_statuses gauge
validator_statuses{pubkey="pk0"} 3
validator_statuses{pubkey="pk1"} 3
validator_statuses{pubkey="pk2"} 3
validator_statuses{pubkey="pk3"} 3
`

var statusFixtureNoneActive = `# HELP validator_statuses validator statuses: 0 UNKNOWN, 1 DEPOSITED, 2 PENDING, 3 ACTIVE, 4 EXITING, 5 SLASHING, 6 EXITED
# TYPE validator_statuses gauge
validator_statuses{pubkey="pk0"} 0
validator_statuses{pubkey="pk1"} 1
validator_statuses{pubkey="pk2"} 2
validator_statuses{pubkey="pk3"} 4
validator_statuses{pubkey="pk4"} 5
validator_statuses{pubkey="pk5"} 6
`
