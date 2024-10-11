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
	"strings"

	"github.com/cloudflare/cloudflare-go"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/dnsdisc"
	"github.com/urfave/cli/v2"
)

var (
	cloudflareTokenFlag = &cli.StringFlag{
		Name:    "token",
		Usage:   "CloudFlare API token",
		EnvVars: []string{"CLOUDFLARE_API_TOKEN"},
	}
	cloudflareZoneIDFlag = &cli.StringFlag{
		Name:  "zoneid",
		Usage: "CloudFlare Zone ID (optional)",
	}
)

type cloudflareClient struct {
	*cloudflare.API
	zoneID string
}

// newCloudflareClient sets up a CloudFlare API client from command line flags.
func newCloudflareClient(ctx *cli.Context) *cloudflareClient {
	token := ctx.String(cloudflareTokenFlag.Name)
	if token == "" {
		exit(errors.New("need cloudflare API token to proceed"))
	}
	api, err := cloudflare.NewWithAPIToken(token)
	if err != nil {
		exit(fmt.Errorf("can't create Cloudflare client: %v", err))
	}
	return &cloudflareClient{
		API:    api,
		zoneID: ctx.String(cloudflareZoneIDFlag.Name),
	}
}

// deploy uploads the given tree to CloudFlare DNS.
func (c *cloudflareClient) deploy(name string, t *dnsdisc.Tree) error {
	if err := c.checkZone(name); err != nil {
		return err
	}
	records := t.ToTXT(name)
	return c.uploadRecords(name, records)
}

// checkZone verifies permissions on the CloudFlare DNS Zone for name.
func (c *cloudflareClient) checkZone(name string) error {
	if c.zoneID == "" {
		log.Info(fmt.Sprintf("Finding CloudFlare zone ID for %s", name))
		id, err := c.ZoneIDByName(name)
		if err != nil {
			return err
		}
		c.zoneID = id
	}
	log.Info(fmt.Sprintf("Checking Permissions on zone %s", c.zoneID))
	zone, err := c.ZoneDetails(context.Background(), c.zoneID)
	if err != nil {
		return err
	}
	if !strings.HasSuffix(name, "."+zone.Name) {
		return fmt.Errorf("CloudFlare zone name %q does not match name %q to be deployed", zone.Name, name)
	}
	needPerms := map[string]bool{"#zone:edit": false, "#zone:read": false}
	for _, perm := range zone.Permissions {
		if _, ok := needPerms[perm]; ok {
			needPerms[perm] = true
		}
	}
	for _, ok := range needPerms {
		if !ok {
			return fmt.Errorf("wrong permissions on zone %s: %v", c.zoneID, needPerms)
		}
	}
	return nil
}

// uploadRecords updates the TXT records at a particular subdomain. All non-root records
// will have a TTL of "infinity" and all existing records not in the new map will be
// nuked!
func (c *cloudflareClient) uploadRecords(name string, records map[string]string) error {
	// Convert all names to lowercase.
	lrecords := make(map[string]string, len(records))
	for name, r := range records {
		lrecords[strings.ToLower(name)] = r
	}
	records = lrecords

	log.Info(fmt.Sprintf("Retrieving existing TXT records on %s", name))
	entries, _, err := c.ListDNSRecords(context.Background(), cloudflare.ZoneIdentifier(c.zoneID), cloudflare.ListDNSRecordsParams{Type: "TXT"})
	if err != nil {
		return err
	}
	existing := make(map[string]cloudflare.DNSRecord)
	for _, entry := range entries {
		if !strings.HasSuffix(entry.Name, name) {
			continue
		}
		existing[strings.ToLower(entry.Name)] = entry
	}

	// Iterate over the new records and inject anything missing.
	log.Info("Updating DNS entries")
	created := 0
	updated := 0
	skipped := 0
	for path, val := range records {
		old, exists := existing[path]
		if !exists {
			// Entry is unknown, push a new one to Cloudflare.
			log.Debug(fmt.Sprintf("Creating %s = %q", path, val))
			created++
			ttl := rootTTL
			if path != name {
				ttl = treeNodeTTLCloudflare // Max TTL permitted by Cloudflare
			}
			record := cloudflare.CreateDNSRecordParams{Type: "TXT", Name: path, Content: val, TTL: ttl}
			_, err = c.CreateDNSRecord(context.Background(), cloudflare.ZoneIdentifier(c.zoneID), record)
		} else if old.Content != val {
			// Entry already exists, only change its content.
			log.Info(fmt.Sprintf("Updating %s from %q to %q", path, old.Content, val))
			updated++

			record := cloudflare.UpdateDNSRecordParams{
				Type:     old.Type,
				Name:     old.Name,
				Content:  val,
				Data:     old.Data,
				ID:       old.ID,
				Priority: old.Priority,
				TTL:      old.TTL,
				Proxied:  old.Proxied,
				Tags:     old.Tags,
			}
			_, err = c.UpdateDNSRecord(context.Background(), cloudflare.ZoneIdentifier(c.zoneID), record)
		} else {
			skipped++
			log.Debug(fmt.Sprintf("Skipping %s = %q", path, val))
		}
		if err != nil {
			return fmt.Errorf("failed to publish %s: %v", path, err)
		}
	}
	log.Info("Updated DNS entries", "new", created, "updated", updated, "untouched", skipped)
	// Iterate over the old records and delete anything stale.
	deleted := 0
	log.Info("Deleting stale DNS entries")
	for path, entry := range existing {
		if _, ok := records[path]; ok {
			continue
		}
		// Stale entry, nuke it.
		log.Debug(fmt.Sprintf("Deleting %s = %q", path, entry.Content))
		deleted++
		if err := c.DeleteDNSRecord(context.Background(), cloudflare.ZoneIdentifier(c.zoneID), entry.ID); err != nil {
			return fmt.Errorf("failed to delete %s: %v", path, err)
		}
	}
	log.Info("Deleted stale DNS entries", "count", deleted)
	return nil
}
