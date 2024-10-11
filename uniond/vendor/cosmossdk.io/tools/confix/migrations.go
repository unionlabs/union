package confix

import (
	"context"
	"fmt"
	"strings"

	"github.com/creachadair/tomledit"
	"github.com/creachadair/tomledit/parser"
	"github.com/creachadair/tomledit/transform"
)

const (
	AppConfig        = "app.toml"
	AppConfigType    = "app"
	ClientConfig     = "client.toml"
	ClientConfigType = "client"
	CMTConfig        = "config.toml"
)

// MigrationMap defines a mapping from a version to a transformation plan.
type MigrationMap map[string]func(from *tomledit.Document, to, planType string) (transform.Plan, *tomledit.Document)

// loadDestConfigFile is the function signature to load the destination version
// configuration toml file.
type loadDestConfigFile func(to, planType string) (*tomledit.Document, error)

var Migrations = MigrationMap{
	"v0.45": NoPlan, // Confix supports only the current supported SDK version. So we do not support v0.44 -> v0.45.
	"v0.46": defaultPlanBuilder,
	"v0.47": defaultPlanBuilder,
	"v0.50": defaultPlanBuilder,
	"v0.52": defaultPlanBuilder,
	"v2":    V2PlanBuilder,
	// "v0.xx.x": defaultPlanBuilder, // add specific migration in case of configuration changes in minor versions
}

type v2KeyChangesMap map[string][]string

// list all the keys which are need to be modified in v2
var v2KeyChanges = v2KeyChangesMap{
	"minimum-gas-prices": []string{"server.minimum-gas-prices"},
	"min-retain-blocks":  []string{"comet.min-retain-blocks"},
	"index-events":       []string{"comet.index-events"},
	"halt-height":        []string{"comet.halt-height"},
	"halt-time":          []string{"comet.halt-time"},
	"app-db-backend":     []string{"store.app-db-backend"},
	"pruning-keep-recent": []string{
		"store.options.ss-pruning-option.keep-recent",
		"store.options.sc-pruning-option.keep-recent",
	},
	"pruning-interval": []string{
		"store.options.ss-pruning-option.interval",
		"store.options.sc-pruning-option.interval",
	},
	"iavl-cache-size":       []string{"store.options.iavl-config.cache-size"},
	"iavl-disable-fastnode": []string{"store.options.iavl-config.skip-fast-storage-upgrade"},
	"telemetry.enabled":     []string{"telemetry.enable"},
	"mempool.max-txs":       []string{"comet.mempool.max-txs"},
	// Add other key mappings as needed
}

func defaultPlanBuilder(from *tomledit.Document, to, planType string) (transform.Plan, *tomledit.Document) {
	return PlanBuilder(from, to, planType, LoadLocalConfig)
}

// PlanBuilder is a function that returns a transformation plan for a given diff between two files.
func PlanBuilder(from *tomledit.Document, to, planType string, loadFn loadDestConfigFile) (transform.Plan, *tomledit.Document) {
	plan := transform.Plan{}
	deletedSections := map[string]bool{}

	target, err := loadFn(to, planType)
	if err != nil {
		panic(fmt.Errorf("failed to parse file: %w. This file should have been valid", err))
	}

	diffs := DiffKeys(from, target)
	for _, diff := range diffs {
		kv := diff.KV

		var step transform.Step
		keys := strings.Split(kv.Key, ".")

		if !diff.Deleted {
			switch diff.Type {
			case Section:
				step = transform.Step{
					Desc: fmt.Sprintf("add %s section", kv.Key),
					T: transform.Func(func(_ context.Context, doc *tomledit.Document) error {
						title := fmt.Sprintf("###                    %s Configuration                    ###", strings.Title(kv.Key))
						doc.Sections = append(doc.Sections, &tomledit.Section{
							Heading: &parser.Heading{
								Block: parser.Comments{
									strings.Repeat("#", len(title)),
									title,
									strings.Repeat("#", len(title)),
								},
								Name: keys,
							},
						})
						return nil
					}),
				}
			case Mapping:
				if len(keys) == 1 { // top-level key
					step = transform.Step{
						Desc: fmt.Sprintf("add %s key", kv.Key),
						T: transform.EnsureKey(nil, &parser.KeyValue{
							Block: kv.Block,
							Name:  parser.Key{keys[0]},
							Value: parser.MustValue(kv.Value),
						}),
					}
				} else if len(keys) > 1 {
					step = transform.Step{
						Desc: fmt.Sprintf("add %s key", kv.Key),
						T: transform.EnsureKey(keys[0:len(keys)-1], &parser.KeyValue{
							Block: kv.Block,
							Name:  parser.Key{keys[len(keys)-1]},
							Value: parser.MustValue(kv.Value),
						}),
					}
				}
			default:
				panic(fmt.Errorf("unknown diff type: %s", diff.Type))
			}
		} else {
			if diff.Type == Section {
				deletedSections[kv.Key] = true
				step = transform.Step{
					Desc: fmt.Sprintf("remove %s section", kv.Key),
					T:    transform.Remove(keys),
				}
			} else {
				// when the whole section is deleted we don't need to remove the keys
				if len(keys) > 1 && deletedSections[keys[0]] {
					continue
				}

				step = transform.Step{
					Desc: fmt.Sprintf("remove %s key", kv.Key),
					T:    transform.Remove(keys),
				}
			}
		}

		plan = append(plan, step)
	}

	return plan, from
}

// NoPlan returns a no-op plan.
func NoPlan(from *tomledit.Document, to, planType string) (transform.Plan, *tomledit.Document) {
	fmt.Printf("no migration needed to %s\n", to)
	return transform.Plan{}, from
}

// V2PlanBuilder is a function that returns a transformation plan to convert to v2 config
func V2PlanBuilder(from *tomledit.Document, to, planType string) (transform.Plan, *tomledit.Document) {
	target, err := LoadLocalConfig(to, planType)
	if err != nil {
		panic(fmt.Errorf("failed to parse file: %w. This file should have been valid", err))
	}

	plan := transform.Plan{}
	plan = updateMatchedKeysPlan(from, target, plan)
	plan = applyKeyChangesPlan(from, plan)

	return plan, target
}

// updateMatchedKeysPlan updates all matched keys with old key values
func updateMatchedKeysPlan(from, target *tomledit.Document, plan transform.Plan) transform.Plan {
	matches := MatchKeys(from, target)
	for oldKey, newKey := range matches {
		oldEntry := getEntry(from, oldKey)
		if oldEntry == nil {
			continue
		}

		// check if the key "app-db-backend" exists and if its value is empty in the existing config
		// If the value is empty, update the key value with the default value
		// of v2 i.e., goleveldb  to prevent network failures.
		if isAppDBBackend(newKey, oldEntry) {
			continue // lets keep app-db-backend with v2 default value
		}

		// update newKey value with old entry value
		step := createUpdateStep(oldKey, newKey, oldEntry)
		plan = append(plan, step)
	}
	return plan
}

// applyKeyChangesPlan checks if key changes are needed with the "to" version and applies them
func applyKeyChangesPlan(from *tomledit.Document, plan transform.Plan) transform.Plan {
	changes := v2KeyChanges
	for oldKey, newKeys := range changes {
		oldEntry := getEntry(from, oldKey)
		if oldEntry == nil {
			continue
		}

		for _, newKey := range newKeys {
			// check if the key "app-db-backend" exists and if its value is empty in the existing config
			// If the value is empty, update the key value with the default value
			// of v2 i.e., goleveldb  to prevent network failures.
			if isAppDBBackend(newKey, oldEntry) {
				continue // lets keep app-db-backend with v2 default value
			}

			// update newKey value with old entry value
			step := createUpdateStep(oldKey, newKey, oldEntry)
			plan = append(plan, step)
		}
	}
	return plan
}

// getEntry retrieves the first entry for the given key from the document
func getEntry(doc *tomledit.Document, key string) *parser.KeyValue {
	splitKeys := strings.Split(key, ".")
	entry := doc.First(splitKeys...)
	if entry == nil || entry.KeyValue == nil {
		return nil
	}
	return entry.KeyValue
}

// isAppDBBackend checks if the key is "store.app-db-backend" and the value is empty
func isAppDBBackend(key string, entry *parser.KeyValue) bool {
	return key == "store.app-db-backend" && entry.Value.String() == `""`
}

// createUpdateStep creates a transformation step to update a key with a new key value
func createUpdateStep(oldKey, newKey string, oldEntry *parser.KeyValue) transform.Step {
	return transform.Step{
		Desc: fmt.Sprintf("updating %s key with %s key", oldKey, newKey),
		T: transform.Func(func(_ context.Context, doc *tomledit.Document) error {
			splitNewKeys := strings.Split(newKey, ".")
			newEntry := doc.First(splitNewKeys...)
			if newEntry == nil || newEntry.KeyValue == nil {
				return nil
			}

			newEntry.KeyValue.Value = oldEntry.Value
			return nil
		}),
	}
}
