package beaconapi

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	log "github.com/sirupsen/logrus"
)

var (
	errEmptyPrysmData      = errors.New("Prysm data is empty")
	errEmptyLighthouseData = errors.New("Lighthouse data is empty")
)

const (
	msgWrongJson          = "JSON response has wrong structure, expected %T, got %T"
	msgRequestFailed      = "%s request failed with response code %d with response body %s"
	msgUnknownNode        = "unknown node type %s"
	msgSSZUnmarshalFailed = "failed to unmarshal SSZ"
)

func doJSONGetRequest(template string, requestPath string, beaconNodeIdx int, resp interface{}, bnType ...string) error {
	if len(bnType) == 0 {
		bnType = []string{"prysm"}
	}

	var port int
	switch bnType[0] {
	case "prysm":
		port = params.TestParams.Ports.PrysmBeaconNodeGatewayPort
	case "lighthouse":
		port = params.TestParams.Ports.LighthouseBeaconNodeHTTPPort
	default:
		return fmt.Errorf(msgUnknownNode, bnType[0])
	}

	basePath := fmt.Sprintf(template, port+beaconNodeIdx)
	httpResp, err := http.Get(
		basePath + requestPath,
	)
	if err != nil {
		return err
	}

	var body interface{}
	if httpResp.StatusCode != http.StatusOK {
		if httpResp.Header.Get("Content-Type") == api.JsonMediaType {
			if err = json.NewDecoder(httpResp.Body).Decode(&body); err != nil {
				return err
			}
		} else {
			body, err = io.ReadAll(httpResp.Body)
			if err != nil {
				return err
			}
		}
		return fmt.Errorf(msgRequestFailed, bnType[0], httpResp.StatusCode, body)
	}
	return json.NewDecoder(httpResp.Body).Decode(&resp)
}

func doSSZGetRequest(template string, requestPath string, beaconNodeIdx int, bnType ...string) ([]byte, error) {
	if len(bnType) == 0 {
		bnType = []string{"prysm"}
	}

	client := &http.Client{}
	var port int
	switch bnType[0] {
	case "prysm":
		port = params.TestParams.Ports.PrysmBeaconNodeGatewayPort
	case "lighthouse":
		port = params.TestParams.Ports.LighthouseBeaconNodeHTTPPort
	default:
		return nil, fmt.Errorf(msgUnknownNode, bnType[0])
	}

	basePath := fmt.Sprintf(template, port+beaconNodeIdx)

	req, err := http.NewRequest(http.MethodGet, basePath+requestPath, nil)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Accept", "application/octet-stream")
	rsp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	if rsp.StatusCode != http.StatusOK {
		var body interface{}
		if err := json.NewDecoder(rsp.Body).Decode(&body); err != nil {
			return nil, err
		}
		return nil, fmt.Errorf(msgRequestFailed, bnType[0], rsp.StatusCode, body)
	}
	defer closeBody(rsp.Body)
	body, err := io.ReadAll(rsp.Body)
	if err != nil {
		return nil, err
	}

	return body, nil
}

func doJSONPostRequest(template string, requestPath string, beaconNodeIdx int, postData, resp interface{}, bnType ...string) error {
	if len(bnType) == 0 {
		bnType = []string{"prysm"}
	}

	var port int
	switch bnType[0] {
	case "prysm":
		port = params.TestParams.Ports.PrysmBeaconNodeGatewayPort
	case "lighthouse":
		port = params.TestParams.Ports.LighthouseBeaconNodeHTTPPort
	default:
		return fmt.Errorf(msgUnknownNode, bnType[0])
	}

	basePath := fmt.Sprintf(template, port+beaconNodeIdx)
	b, err := json.Marshal(postData)
	if err != nil {
		return err
	}
	httpResp, err := http.Post(
		basePath+requestPath,
		"application/json",
		bytes.NewBuffer(b),
	)
	if err != nil {
		return err
	}

	var body interface{}
	if httpResp.StatusCode != http.StatusOK {
		if httpResp.Header.Get("Content-Type") == api.JsonMediaType {
			if err = json.NewDecoder(httpResp.Body).Decode(&body); err != nil {
				return err
			}
		} else {
			body, err = io.ReadAll(httpResp.Body)
			if err != nil {
				return err
			}
		}
		return fmt.Errorf(msgRequestFailed, bnType[0], httpResp.StatusCode, body)
	}
	return json.NewDecoder(httpResp.Body).Decode(&resp)
}

func closeBody(body io.Closer) {
	if err := body.Close(); err != nil {
		log.WithError(err).Error("could not close response body")
	}
}
