// Copyright 2019 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"context"
	"errors"
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/credentials"
	"github.com/aws/aws-sdk-go-v2/service/route53"
	"github.com/aws/aws-sdk-go-v2/service/route53/types"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/dnsdisc"
	"github.com/urfave/cli/v2"
	"golang.org/x/exp/slices"
)

const (
	// Route53 limits change sets to 32k of 'RDATA size'. Change sets are also limited to
	// 1000 items. UPSERTs count double.
	// https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/DNSLimitations.html#limits-api-requests-changeresourcerecordsets
	route53ChangeSizeLimit  = 32000
	route53ChangeCountLimit = 1000
	maxRetryLimit           = 60
)

var (
	route53AccessKeyFlag = &cli.StringFlag{
		Name:    "access-key-id",
		Usage:   "AWS Access Key ID",
		EnvVars: []string{"AWS_ACCESS_KEY_ID"},
	}
	route53AccessSecretFlag = &cli.StringFlag{
		Name:    "access-key-secret",
		Usage:   "AWS Access Key Secret",
		EnvVars: []string{"AWS_SECRET_ACCESS_KEY"},
	}
	route53ZoneIDFlag = &cli.StringFlag{
		Name:  "zone-id",
		Usage: "Route53 Zone ID",
	}
	route53RegionFlag = &cli.StringFlag{
		Name:  "aws-region",
		Usage: "AWS Region",
		Value: "eu-central-1",
	}
)

type route53Client struct {
	api    *route53.Client
	zoneID string
}

type recordSet struct {
	values []string
	ttl    int64
}

// newRoute53Client sets up a Route53 API client from command line flags.
func newRoute53Client(ctx *cli.Context) *route53Client {
	akey := ctx.String(route53AccessKeyFlag.Name)
	asec := ctx.String(route53AccessSecretFlag.Name)
	if akey == "" || asec == "" {
		exit(errors.New("need Route53 Access Key ID and secret to proceed"))
	}
	creds := aws.NewCredentialsCache(credentials.NewStaticCredentialsProvider(akey, asec, ""))
	cfg, err := config.LoadDefaultConfig(context.Background(), config.WithCredentialsProvider(creds))
	if err != nil {
		exit(fmt.Errorf("can't initialize AWS configuration: %v", err))
	}
	cfg.Region = ctx.String(route53RegionFlag.Name)
	return &route53Client{
		api:    route53.NewFromConfig(cfg),
		zoneID: ctx.String(route53ZoneIDFlag.Name),
	}
}

// deploy uploads the given tree to Route53.
func (c *route53Client) deploy(name string, t *dnsdisc.Tree) error {
	if err := c.checkZone(name); err != nil {
		return err
	}

	// Compute DNS changes.
	existing, err := c.collectRecords(name)
	if err != nil {
		return err
	}
	log.Info(fmt.Sprintf("Found %d TXT records", len(existing)))
	records := t.ToTXT(name)
	changes := c.computeChanges(name, records, existing)

	// Submit to API.
	comment := fmt.Sprintf("enrtree update of %s at seq %d", name, t.Seq())
	return c.submitChanges(changes, comment)
}

// deleteDomain removes all TXT records of the given domain.
func (c *route53Client) deleteDomain(name string) error {
	if err := c.checkZone(name); err != nil {
		return err
	}

	// Compute DNS changes.
	existing, err := c.collectRecords(name)
	if err != nil {
		return err
	}
	log.Info(fmt.Sprintf("Found %d TXT records", len(existing)))
	changes := makeDeletionChanges(existing, nil)

	// Submit to API.
	comment := "enrtree delete of " + name
	return c.submitChanges(changes, comment)
}

// submitChanges submits the given DNS changes to Route53.
func (c *route53Client) submitChanges(changes []types.Change, comment string) error {
	if len(changes) == 0 {
		log.Info("No DNS changes needed")
		return nil
	}

	var err error
	batches := splitChanges(changes, route53ChangeSizeLimit, route53ChangeCountLimit)
	changesToCheck := make([]*route53.ChangeResourceRecordSetsOutput, len(batches))
	for i, changes := range batches {
		log.Info(fmt.Sprintf("Submitting %d changes to Route53", len(changes)))
		batch := &types.ChangeBatch{
			Changes: changes,
			Comment: aws.String(fmt.Sprintf("%s (%d/%d)", comment, i+1, len(batches))),
		}
		req := &route53.ChangeResourceRecordSetsInput{HostedZoneId: &c.zoneID, ChangeBatch: batch}
		changesToCheck[i], err = c.api.ChangeResourceRecordSets(context.TODO(), req)
		if err != nil {
			return err
		}
	}

	// Wait for all change batches to propagate.
	for _, change := range changesToCheck {
		log.Info(fmt.Sprintf("Waiting for change request %s", *change.ChangeInfo.Id))
		wreq := &route53.GetChangeInput{Id: change.ChangeInfo.Id}
		var count int
		for {
			wresp, err := c.api.GetChange(context.TODO(), wreq)
			if err != nil {
				return err
			}

			count++

			if wresp.ChangeInfo.Status == types.ChangeStatusInsync || count >= maxRetryLimit {
				break
			}

			time.Sleep(30 * time.Second)
		}
	}
	return nil
}

// checkZone verifies zone information for the given domain.
func (c *route53Client) checkZone(name string) (err error) {
	if c.zoneID == "" {
		c.zoneID, err = c.findZoneID(name)
	}
	return err
}

// findZoneID searches for the Zone ID containing the given domain.
func (c *route53Client) findZoneID(name string) (string, error) {
	log.Info(fmt.Sprintf("Finding Route53 Zone ID for %s", name))
	var req route53.ListHostedZonesByNameInput
	for {
		resp, err := c.api.ListHostedZonesByName(context.TODO(), &req)
		if err != nil {
			return "", err
		}
		for _, zone := range resp.HostedZones {
			if isSubdomain(name, *zone.Name) {
				return *zone.Id, nil
			}
		}
		if !resp.IsTruncated {
			break
		}
		req.DNSName = resp.NextDNSName
		req.HostedZoneId = resp.NextHostedZoneId
	}
	return "", errors.New("can't find zone ID for " + name)
}

// computeChanges creates DNS changes for the given set of DNS discovery records.
// The 'existing' arg is the set of records that already exist on Route53.
func (c *route53Client) computeChanges(name string, records map[string]string, existing map[string]recordSet) []types.Change {
	// Convert all names to lowercase.
	lrecords := make(map[string]string, len(records))
	for name, r := range records {
		lrecords[strings.ToLower(name)] = r
	}
	records = lrecords

	var (
		changes []types.Change
		inserts int
		upserts int
		skips   int
	)

	for path, newValue := range records {
		prevRecords, exists := existing[path]
		prevValue := strings.Join(prevRecords.values, "")

		// prevValue contains quoted strings, encode newValue to compare.
		newValue = splitTXT(newValue)

		// Assign TTL.
		ttl := int64(rootTTL)
		if path != name {
			ttl = int64(treeNodeTTL)
		}

		if !exists {
			// Entry is unknown, push a new one
			log.Debug(fmt.Sprintf("Creating %s = %s", path, newValue))
			changes = append(changes, newTXTChange("CREATE", path, ttl, newValue))
			inserts++
		} else if prevValue != newValue || prevRecords.ttl != ttl {
			// Entry already exists, only change its content.
			log.Info(fmt.Sprintf("Updating %s from %s to %s", path, prevValue, newValue))
			changes = append(changes, newTXTChange("UPSERT", path, ttl, newValue))
			upserts++
		} else {
			log.Debug(fmt.Sprintf("Skipping %s = %s", path, newValue))
			skips++
		}
	}

	// Iterate over the old records and delete anything stale.
	deletions := makeDeletionChanges(existing, records)
	changes = append(changes, deletions...)

	log.Info("Computed DNS changes",
		"changes", len(changes),
		"inserts", inserts,
		"skips", skips,
		"deleted", len(deletions),
		"upserts", upserts)
	// Ensure changes are in the correct order.
	sortChanges(changes)
	return changes
}

// makeDeletionChanges creates record changes which delete all records not contained in 'keep'.
func makeDeletionChanges(records map[string]recordSet, keep map[string]string) []types.Change {
	var changes []types.Change
	for path, set := range records {
		if _, ok := keep[path]; ok {
			continue
		}
		log.Debug(fmt.Sprintf("Deleting %s = %s", path, strings.Join(set.values, "")))
		changes = append(changes, newTXTChange("DELETE", path, set.ttl, set.values...))
	}
	return changes
}

// sortChanges ensures DNS changes are in leaf-added -> root-changed -> leaf-deleted order.
func sortChanges(changes []types.Change) {
	score := map[string]int{"CREATE": 1, "UPSERT": 2, "DELETE": 3}
	slices.SortFunc(changes, func(a, b types.Change) int {
		if a.Action == b.Action {
			return strings.Compare(*a.ResourceRecordSet.Name, *b.ResourceRecordSet.Name)
		}
		if score[string(a.Action)] < score[string(b.Action)] {
			return -1
		}
		if score[string(a.Action)] > score[string(b.Action)] {
			return 1
		}
		return 0
	})
}

// splitChanges splits up DNS changes such that each change batch
// is smaller than the given RDATA limit.
func splitChanges(changes []types.Change, sizeLimit, countLimit int) [][]types.Change {
	var (
		batches    [][]types.Change
		batchSize  int
		batchCount int
	)
	for _, ch := range changes {
		// Start new batch if this change pushes the current one over the limit.
		count := changeCount(ch)
		size := changeSize(ch) * count
		overSize := batchSize+size > sizeLimit
		overCount := batchCount+count > countLimit
		if len(batches) == 0 || overSize || overCount {
			batches = append(batches, nil)
			batchSize = 0
			batchCount = 0
		}
		batches[len(batches)-1] = append(batches[len(batches)-1], ch)
		batchSize += size
		batchCount += count
	}
	return batches
}

// changeSize returns the RDATA size of a DNS change.
func changeSize(ch types.Change) int {
	size := 0
	for _, rr := range ch.ResourceRecordSet.ResourceRecords {
		if rr.Value != nil {
			size += len(*rr.Value)
		}
	}
	return size
}

func changeCount(ch types.Change) int {
	if ch.Action == types.ChangeActionUpsert {
		return 2
	}
	return 1
}

// collectRecords collects all TXT records below the given name.
func (c *route53Client) collectRecords(name string) (map[string]recordSet, error) {
	var req route53.ListResourceRecordSetsInput
	req.HostedZoneId = &c.zoneID
	existing := make(map[string]recordSet)
	log.Info("Loading existing TXT records", "name", name, "zone", c.zoneID)
	for page := 0; ; page++ {
		log.Debug("Loading existing TXT records", "name", name, "zone", c.zoneID, "page", page)
		resp, err := c.api.ListResourceRecordSets(context.TODO(), &req)
		if err != nil {
			return existing, err
		}
		for _, set := range resp.ResourceRecordSets {
			if !isSubdomain(*set.Name, name) || set.Type != types.RRTypeTxt {
				continue
			}
			s := recordSet{ttl: *set.TTL}
			for _, rec := range set.ResourceRecords {
				s.values = append(s.values, *rec.Value)
			}
			name := strings.TrimSuffix(*set.Name, ".")
			existing[name] = s
		}

		if !resp.IsTruncated {
			break
		}
		// Set the cursor to the next batch. From the AWS docs:
		//
		// To display the next page of results, get the values of NextRecordName,
		// NextRecordType, and NextRecordIdentifier (if any) from the response. Then submit
		// another ListResourceRecordSets request, and specify those values for
		// StartRecordName, StartRecordType, and StartRecordIdentifier.
		req.StartRecordIdentifier = resp.NextRecordIdentifier
		req.StartRecordName = resp.NextRecordName
		req.StartRecordType = resp.NextRecordType
	}
	log.Info("Loaded existing TXT records", "name", name, "zone", c.zoneID, "records", len(existing))
	return existing, nil
}

// newTXTChange creates a change to a TXT record.
func newTXTChange(action, name string, ttl int64, values ...string) types.Change {
	r := types.ResourceRecordSet{
		Type: types.RRTypeTxt,
		Name: &name,
		TTL:  &ttl,
	}
	var rrs []types.ResourceRecord
	for _, val := range values {
		var rr types.ResourceRecord
		rr.Value = aws.String(val)
		rrs = append(rrs, rr)
	}

	r.ResourceRecords = rrs

	return types.Change{
		Action:            types.ChangeAction(action),
		ResourceRecordSet: &r,
	}
}

// isSubdomain returns true if name is a subdomain of domain.
func isSubdomain(name, domain string) bool {
	domain = strings.TrimSuffix(domain, ".")
	name = strings.TrimSuffix(name, ".")
	return strings.HasSuffix("."+name, "."+domain)
}

// splitTXT splits value into a list of quoted 255-character strings.
func splitTXT(value string) string {
	var result strings.Builder
	for len(value) > 0 {
		rlen := len(value)
		if rlen > 253 {
			rlen = 253
		}
		result.WriteString(strconv.Quote(value[:rlen]))
		value = value[rlen:]
	}
	return result.String()
}
