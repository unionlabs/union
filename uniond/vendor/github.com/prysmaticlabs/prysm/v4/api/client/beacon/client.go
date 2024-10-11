package beacon

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"
	"path"
	"regexp"
	"sort"
	"strconv"
	"text/template"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api/client"
	"github.com/prysmaticlabs/prysm/v4/api/server"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/config"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	apibeacon "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/prysm/beacon"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	log "github.com/sirupsen/logrus"
)

const (
	getSignedBlockPath       = "/eth/v2/beacon/blocks"
	getBlockRootPath         = "/eth/v1/beacon/blocks/{{.Id}}/root"
	getForkForStatePath      = "/eth/v1/beacon/states/{{.Id}}/fork"
	getWeakSubjectivityPath  = "/prysm/v1/beacon/weak_subjectivity"
	getForkSchedulePath      = "/eth/v1/config/fork_schedule"
	getConfigSpecPath        = "/eth/v1/config/spec"
	getStatePath             = "/eth/v2/debug/beacon/states"
	getNodeVersionPath       = "/eth/v1/node/version"
	changeBLStoExecutionPath = "/eth/v1/beacon/pool/bls_to_execution_changes"
)

// StateOrBlockId represents the block_id / state_id parameters that several of the Eth Beacon API methods accept.
// StateOrBlockId constants are defined for named identifiers, and helper methods are provided
// for slot and root identifiers. Example text from the Eth Beacon Node API documentation:
//
// "Block identifier can be one of: "head" (canonical head in node's view), "genesis", "finalized",
// <slot>, <hex encoded blockRoot with 0x prefix>."
type StateOrBlockId string

const (
	IdGenesis   StateOrBlockId = "genesis"
	IdHead      StateOrBlockId = "head"
	IdFinalized StateOrBlockId = "finalized"
)

// IdFromRoot encodes a block root in the format expected by the API in places where a root can be used to identify
// a BeaconState or SignedBeaconBlock.
func IdFromRoot(r [32]byte) StateOrBlockId {
	return StateOrBlockId(fmt.Sprintf("%#x", r))
}

// IdFromSlot encodes a Slot in the format expected by the API in places where a slot can be used to identify
// a BeaconState or SignedBeaconBlock.
func IdFromSlot(s primitives.Slot) StateOrBlockId {
	return StateOrBlockId(strconv.FormatUint(uint64(s), 10))
}

// idTemplate is used to create template functions that can interpolate StateOrBlockId values.
func idTemplate(ts string) func(StateOrBlockId) string {
	t := template.Must(template.New("").Parse(ts))
	f := func(id StateOrBlockId) string {
		b := bytes.NewBuffer(nil)
		err := t.Execute(b, struct{ Id string }{Id: string(id)})
		if err != nil {
			panic(fmt.Sprintf("invalid idTemplate: %s", ts))
		}
		return b.String()
	}
	// run the template to ensure that it is valid
	// this should happen load time (using package scoped vars) to ensure runtime errors aren't possible
	_ = f(IdGenesis)
	return f
}

func renderGetBlockPath(id StateOrBlockId) string {
	return path.Join(getSignedBlockPath, string(id))
}

// Client provides a collection of helper methods for calling the Eth Beacon Node API endpoints.
type Client struct {
	*client.Client
}

// NewClient returns a new Client that includes functions for rest calls to Beacon API.
func NewClient(host string, opts ...client.ClientOpt) (*Client, error) {
	c, err := client.NewClient(host, opts...)
	if err != nil {
		return nil, err
	}
	return &Client{c}, nil
}

// GetBlock retrieves the SignedBeaconBlock for the given block id.
// Block identifier can be one of: "head" (canonical head in node's view), "genesis", "finalized",
// <slot>, <hex encoded blockRoot with 0x prefix>. Variables of type StateOrBlockId are exported by this package
// for the named identifiers.
// The return value contains the ssz-encoded bytes.
func (c *Client) GetBlock(ctx context.Context, blockId StateOrBlockId) ([]byte, error) {
	blockPath := renderGetBlockPath(blockId)
	b, err := c.Get(ctx, blockPath, client.WithSSZEncoding())
	if err != nil {
		return nil, errors.Wrapf(err, "error requesting state by id = %s", blockId)
	}
	return b, nil
}

var getBlockRootTpl = idTemplate(getBlockRootPath)

// GetBlockRoot retrieves the hash_tree_root of the BeaconBlock for the given block id.
// Block identifier can be one of: "head" (canonical head in node's view), "genesis", "finalized",
// <slot>, <hex encoded blockRoot with 0x prefix>. Variables of type StateOrBlockId are exported by this package
// for the named identifiers.
func (c *Client) GetBlockRoot(ctx context.Context, blockId StateOrBlockId) ([32]byte, error) {
	rootPath := getBlockRootTpl(blockId)
	b, err := c.Get(ctx, rootPath)
	if err != nil {
		return [32]byte{}, errors.Wrapf(err, "error requesting block root by id = %s", blockId)
	}
	jsonr := &struct{ Data struct{ Root string } }{}
	err = json.Unmarshal(b, jsonr)
	if err != nil {
		return [32]byte{}, errors.Wrap(err, "error decoding json data from get block root response")
	}
	rs, err := hexutil.Decode(jsonr.Data.Root)
	if err != nil {
		return [32]byte{}, errors.Wrap(err, fmt.Sprintf("error decoding hex-encoded value %s", jsonr.Data.Root))
	}
	return bytesutil.ToBytes32(rs), nil
}

var getForkTpl = idTemplate(getForkForStatePath)

// GetFork queries the Beacon Node API for the Fork from the state identified by stateId.
// Block identifier can be one of: "head" (canonical head in node's view), "genesis", "finalized",
// <slot>, <hex encoded blockRoot with 0x prefix>. Variables of type StateOrBlockId are exported by this package
// for the named identifiers.
func (c *Client) GetFork(ctx context.Context, stateId StateOrBlockId) (*ethpb.Fork, error) {
	body, err := c.Get(ctx, getForkTpl(stateId))
	if err != nil {
		return nil, errors.Wrapf(err, "error requesting fork by state id = %s", stateId)
	}
	fr := &shared.Fork{}
	dataWrapper := &struct{ Data *shared.Fork }{Data: fr}
	err = json.Unmarshal(body, dataWrapper)
	if err != nil {
		return nil, errors.Wrap(err, "error decoding json response in GetFork")
	}

	return fr.ToConsensus()
}

// GetForkSchedule retrieve all forks, past present and future, of which this node is aware.
func (c *Client) GetForkSchedule(ctx context.Context) (forks.OrderedSchedule, error) {
	body, err := c.Get(ctx, getForkSchedulePath)
	if err != nil {
		return nil, errors.Wrap(err, "error requesting fork schedule")
	}
	fsr := &forkScheduleResponse{}
	err = json.Unmarshal(body, fsr)
	if err != nil {
		return nil, err
	}
	ofs, err := fsr.OrderedForkSchedule()
	if err != nil {
		return nil, errors.Wrap(err, fmt.Sprintf("problem unmarshaling %s response", getForkSchedulePath))
	}
	return ofs, nil
}

// GetConfigSpec retrieve the current configs of the network used by the beacon node.
func (c *Client) GetConfigSpec(ctx context.Context) (*config.GetSpecResponse, error) {
	body, err := c.Get(ctx, getConfigSpecPath)
	if err != nil {
		return nil, errors.Wrap(err, "error requesting configSpecPath")
	}
	fsr := &config.GetSpecResponse{}
	err = json.Unmarshal(body, fsr)
	if err != nil {
		return nil, err
	}
	return fsr, nil
}

type NodeVersion struct {
	implementation string
	semver         string
	systemInfo     string
}

var versionRE = regexp.MustCompile(`^(\w+)/(v\d+\.\d+\.\d+[-a-zA-Z0-9]*)\s*/?(.*)$`)

func parseNodeVersion(v string) (*NodeVersion, error) {
	groups := versionRE.FindStringSubmatch(v)
	if len(groups) != 4 {
		return nil, errors.Wrapf(client.ErrInvalidNodeVersion, "could not be parsed: %s", v)
	}
	return &NodeVersion{
		implementation: groups[1],
		semver:         groups[2],
		systemInfo:     groups[3],
	}, nil
}

// GetNodeVersion requests that the beacon node identify information about its implementation in a format
// similar to a HTTP User-Agent field. ex: Lighthouse/v0.1.5 (Linux x86_64)
func (c *Client) GetNodeVersion(ctx context.Context) (*NodeVersion, error) {
	b, err := c.Get(ctx, getNodeVersionPath)
	if err != nil {
		return nil, errors.Wrap(err, "error requesting node version")
	}
	d := struct {
		Data struct {
			Version string `json:"version"`
		} `json:"data"`
	}{}
	err = json.Unmarshal(b, &d)
	if err != nil {
		return nil, errors.Wrapf(err, "error unmarshaling response body: %s", string(b))
	}
	return parseNodeVersion(d.Data.Version)
}

func renderGetStatePath(id StateOrBlockId) string {
	return path.Join(getStatePath, string(id))
}

// GetState retrieves the BeaconState for the given state id.
// State identifier can be one of: "head" (canonical head in node's view), "genesis", "finalized",
// <slot>, <hex encoded stateRoot with 0x prefix>. Variables of type StateOrBlockId are exported by this package
// for the named identifiers.
// The return value contains the ssz-encoded bytes.
func (c *Client) GetState(ctx context.Context, stateId StateOrBlockId) ([]byte, error) {
	statePath := path.Join(getStatePath, string(stateId))
	b, err := c.Get(ctx, statePath, client.WithSSZEncoding())
	if err != nil {
		return nil, errors.Wrapf(err, "error requesting state by id = %s", stateId)
	}
	return b, nil
}

// GetWeakSubjectivity calls a proposed API endpoint that is unique to prysm
// This api method does the following:
// - computes weak subjectivity epoch
// - finds the highest non-skipped block preceding the epoch
// - returns the htr of the found block and returns this + the value of state_root from the block
func (c *Client) GetWeakSubjectivity(ctx context.Context) (*WeakSubjectivityData, error) {
	body, err := c.Get(ctx, getWeakSubjectivityPath)
	if err != nil {
		return nil, err
	}
	v := &apibeacon.GetWeakSubjectivityResponse{}
	err = json.Unmarshal(body, v)
	if err != nil {
		return nil, err
	}
	epoch, err := strconv.ParseUint(v.Data.WsCheckpoint.Epoch, 10, 64)
	if err != nil {
		return nil, err
	}
	blockRoot, err := hexutil.Decode(v.Data.WsCheckpoint.Root)
	if err != nil {
		return nil, err
	}
	stateRoot, err := hexutil.Decode(v.Data.StateRoot)
	if err != nil {
		return nil, err
	}
	return &WeakSubjectivityData{
		Epoch:     primitives.Epoch(epoch),
		BlockRoot: bytesutil.ToBytes32(blockRoot),
		StateRoot: bytesutil.ToBytes32(stateRoot),
	}, nil
}

// SubmitChangeBLStoExecution calls a beacon API endpoint to set the withdrawal addresses based on the given signed messages.
// If the API responds with something other than OK there will be failure messages associated to the corresponding request message.
func (c *Client) SubmitChangeBLStoExecution(ctx context.Context, request []*shared.SignedBLSToExecutionChange) error {
	u := c.BaseURL().ResolveReference(&url.URL{Path: changeBLStoExecutionPath})
	body, err := json.Marshal(request)
	if err != nil {
		return errors.Wrap(err, "failed to marshal JSON")
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, u.String(), bytes.NewBuffer(body))
	if err != nil {
		return errors.Wrap(err, "invalid format, failed to create new POST request object")
	}
	req.Header.Set("Content-Type", "application/json")
	resp, err := c.Do(req)
	if err != nil {
		return err
	}
	defer func() {
		err = resp.Body.Close()
	}()
	if resp.StatusCode != http.StatusOK {
		decoder := json.NewDecoder(resp.Body)
		decoder.DisallowUnknownFields()
		errorJson := &server.IndexedVerificationFailureError{}
		if err := decoder.Decode(errorJson); err != nil {
			return errors.Wrapf(err, "failed to decode error JSON for %s", resp.Request.URL)
		}
		for _, failure := range errorJson.Failures {
			w := request[failure.Index].Message
			log.WithFields(log.Fields{
				"validator_index":    w.ValidatorIndex,
				"withdrawal_address": w.ToExecutionAddress,
			}).Error(failure.Message)
		}
		return errors.Errorf("POST error %d: %s", errorJson.Code, errorJson.Message)
	}
	return nil
}

// GetBLStoExecutionChanges gets all the set withdrawal messages in the node's operation pool.
// Returns a struct representation of json response.
func (c *Client) GetBLStoExecutionChanges(ctx context.Context) (*beacon.BLSToExecutionChangesPoolResponse, error) {
	body, err := c.Get(ctx, changeBLStoExecutionPath)
	if err != nil {
		return nil, err
	}
	poolResponse := &beacon.BLSToExecutionChangesPoolResponse{}
	err = json.Unmarshal(body, poolResponse)
	if err != nil {
		return nil, err
	}
	return poolResponse, nil
}

type forkScheduleResponse struct {
	Data []shared.Fork
}

func (fsr *forkScheduleResponse) OrderedForkSchedule() (forks.OrderedSchedule, error) {
	ofs := make(forks.OrderedSchedule, 0)
	for _, d := range fsr.Data {
		epoch, err := strconv.Atoi(d.Epoch)
		if err != nil {
			return nil, err
		}
		vSlice, err := hexutil.Decode(d.CurrentVersion)
		if err != nil {
			return nil, err
		}
		if len(vSlice) != 4 {
			return nil, fmt.Errorf("got %d byte version, expected 4 bytes. version hex=%s", len(vSlice), d.CurrentVersion)
		}
		version := bytesutil.ToBytes4(vSlice)
		ofs = append(ofs, forks.ForkScheduleEntry{
			Version: version,
			Epoch:   primitives.Epoch(uint64(epoch)),
		})
	}
	sort.Sort(ofs)
	return ofs, nil
}
