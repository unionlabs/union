package evaluators

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/policies"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/emptypb"
)

const maxMemStatsBytes = 2000000000 // 2 GiB.

// MetricsCheck performs a check on metrics to make sure caches are functioning, and
// overall health is good. Not checking the first epoch so the sample size isn't too small.
var MetricsCheck = types.Evaluator{
	Name:       "metrics_check_epoch_%d",
	Policy:     policies.AfterNthEpoch(0),
	Evaluation: metricsTest,
}

type equalityTest struct {
	name  string
	topic string
	value int
}

type comparisonTest struct {
	name               string
	topic1             string
	topic2             string
	expectedComparison float64
}

var metricLessThanTests = []equalityTest{
	{
		name:  "memory usage",
		topic: "go_memstats_alloc_bytes",
		value: maxMemStatsBytes,
	},
}

const (
	p2pFailValidationTopic = "p2p_message_failed_validation_total{topic=\"%s/ssz_snappy\"}"
	p2pReceivedTotalTopic  = "p2p_message_received_total{topic=\"%s/ssz_snappy\"}"
)

var metricComparisonTests = []comparisonTest{
	{
		name:               "beacon aggregate and proof",
		topic1:             fmt.Sprintf(p2pFailValidationTopic, p2p.AggregateAndProofSubnetTopicFormat),
		topic2:             fmt.Sprintf(p2pReceivedTotalTopic, p2p.AggregateAndProofSubnetTopicFormat),
		expectedComparison: 0.8,
	},
	{
		name:               "committee index beacon attestations",
		topic1:             fmt.Sprintf(p2pFailValidationTopic, formatTopic(p2p.AttestationSubnetTopicFormat)),
		topic2:             fmt.Sprintf(p2pReceivedTotalTopic, formatTopic(p2p.AttestationSubnetTopicFormat)),
		expectedComparison: 0.15,
	},
	{
		name:               "committee cache",
		topic1:             "committee_cache_miss",
		topic2:             "committee_cache_hit",
		expectedComparison: 0.01,
	},
	{
		name:               "hot state cache",
		topic1:             "hot_state_cache_miss",
		topic2:             "hot_state_cache_hit",
		expectedComparison: 0.01,
	},
}

func metricsTest(_ *types.EvaluationContext, conns ...*grpc.ClientConn) error {
	genesis, err := eth.NewNodeClient(conns[0]).GetGenesis(context.Background(), &emptypb.Empty{})
	if err != nil {
		return err
	}
	forkDigest, err := forks.CreateForkDigest(time.Unix(genesis.GenesisTime.Seconds, 0), genesis.GenesisValidatorsRoot)
	if err != nil {
		return err
	}
	for i := 0; i < len(conns); i++ {
		response, err := http.Get(fmt.Sprintf("http://localhost:%d/metrics", e2e.TestParams.Ports.PrysmBeaconNodeMetricsPort+i))
		if err != nil {
			// Continue if the connection fails, regular flake.
			continue
		}
		dataInBytes, err := io.ReadAll(response.Body)
		if err != nil {
			return err
		}
		pageContent := string(dataInBytes)
		if err = response.Body.Close(); err != nil {
			return err
		}
		time.Sleep(connTimeDelay)

		beaconClient := eth.NewBeaconChainClient(conns[i])
		nodeClient := eth.NewNodeClient(conns[i])
		chainHead, err := beaconClient.GetChainHead(context.Background(), &emptypb.Empty{})
		if err != nil {
			return err
		}
		genesisResp, err := nodeClient.GetGenesis(context.Background(), &emptypb.Empty{})
		if err != nil {
			return err
		}
		timeSlot := slots.SinceGenesis(genesisResp.GenesisTime.AsTime())
		if uint64(chainHead.HeadSlot) != uint64(timeSlot) {
			return fmt.Errorf("expected metrics slot to equal chain head slot, expected %d, received %d", timeSlot, chainHead.HeadSlot)
		}

		for _, test := range metricLessThanTests {
			topic := test.topic
			if strings.Contains(topic, "%x") {
				topic = fmt.Sprintf(topic, forkDigest)
			}
			if err = metricCheckLessThan(pageContent, topic, test.value); err != nil {
				return errors.Wrapf(err, "failed %s check", test.name)
			}
		}
		for _, test := range metricComparisonTests {
			topic1 := test.topic1
			if strings.Contains(topic1, "%x") {
				topic1 = fmt.Sprintf(topic1, forkDigest)
			}
			topic2 := test.topic2
			if strings.Contains(topic2, "%x") {
				topic2 = fmt.Sprintf(topic2, forkDigest)
			}
			if err = metricCheckComparison(pageContent, topic1, topic2, test.expectedComparison); err != nil {
				return err
			}
		}
	}
	return nil
}

func metricCheckLessThan(pageContent, topic string, value int) error {
	topicValue, err := valueOfTopic(pageContent, topic)
	if err != nil {
		return err
	}
	if topicValue >= value {
		return fmt.Errorf(
			"unexpected result for metric %s, expected less than %d, received %d",
			topic,
			value,
			topicValue,
		)
	}
	return nil
}

func metricCheckComparison(pageContent, topic1, topic2 string, comparison float64) error {
	topic2Value, err := valueOfTopic(pageContent, topic2)
	// If we can't find the first topic (error metrics), then assume the test passes.
	if topic2Value != -1 {
		return nil
	}
	if err != nil {
		return err
	}
	topic1Value, err := valueOfTopic(pageContent, topic1)
	if topic1Value != -1 {
		return nil
	}
	if err != nil {
		return err
	}
	topicComparison := float64(topic1Value) / float64(topic2Value)
	if topicComparison >= comparison {
		return fmt.Errorf(
			"unexpected result for comparison between metric %s and metric %s, expected comparison to be %.2f, received %.2f",
			topic1,
			topic2,
			comparison,
			topicComparison,
		)
	}
	return nil
}

func valueOfTopic(pageContent, topic string) (int, error) {
	regexExp, err := regexp.Compile(topic + " ")
	if err != nil {
		return -1, errors.Wrap(err, "could not create regex expression")
	}
	indexesFound := regexExp.FindAllStringIndex(pageContent, 8)
	if indexesFound == nil {
		return -1, fmt.Errorf("no strings found for %s", topic)
	}
	var result float64
	for i, stringIndex := range indexesFound {
		// Only performing every third result found since theres 2 comments above every metric.
		if i == 0 || i%2 != 0 {
			continue
		}
		startOfValue := stringIndex[1]
		endOfValue := strings.Index(pageContent[startOfValue:], "\n")
		if endOfValue == -1 {
			return -1, fmt.Errorf("could not find next space in %s", pageContent[startOfValue:])
		}
		metricValue := pageContent[startOfValue : startOfValue+endOfValue]
		floatResult, err := strconv.ParseFloat(metricValue, 64)
		if err != nil {
			return -1, errors.Wrapf(err, "could not parse %s for int", metricValue)
		}
		result += floatResult
	}
	return int(result), nil
}

func formatTopic(topic string) string {
	replacedD := strings.Replace(topic, "%d", "\\w*", 1)
	return replacedD
}
