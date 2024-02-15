package params

import (
	"fmt"

	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

var configs *configset

// All returns a slice of every BeaconChainConfig contained in the configset.
func All() []*BeaconChainConfig {
	return configs.all()
}

// ByName returns the BeaconChainConfig with the matching `ConfigName` field.
// The runtime ensures that each config name uniquely refers to a single BeaconChainConfig.
func ByName(name string) (*BeaconChainConfig, error) {
	return configs.byName(name)
}

// ByVersion returns the BeaconChainConfig that has the given version in its ForkVersionSchedule.
// The configset ensures that each fork version schedule entry uniquely points to a single BeaconChainConfig.
func ByVersion(version [fieldparams.VersionLength]byte) (*BeaconChainConfig, error) {
	return configs.byVersion(version)
}

// SetActive sets the given config as active (the config that will be returned by GetActive).
// SetActive will always overwrite any config with the same ConfigName before setting the updated value to active.
func SetActive(c *BeaconChainConfig) error {
	return configs.setActive(c)
}

// SetActiveWithUndo attempts to set the active config, and if successful,
// returns a callback function that can be used to revert the configset back to its previous state.
func SetActiveWithUndo(c *BeaconChainConfig) (func() error, error) {
	return configs.setActiveWithUndo(c)
}

type configset struct {
	active        *BeaconChainConfig
	versionToName map[[fieldparams.VersionLength]byte]string
	nameToConfig  map[string]*BeaconChainConfig
}

func newConfigset(configs ...*BeaconChainConfig) *configset {
	r := &configset{
		versionToName: make(map[[fieldparams.VersionLength]byte]string),
		nameToConfig:  make(map[string]*BeaconChainConfig),
	}
	for _, c := range configs {
		if err := r.add(c); err != nil {
			panic(err)
		}
	}
	return r
}

var errCannotNullifyActive = errors.New("cannot set a config marked as active to nil")
var errCollisionFork = errors.New("configset cannot add config with conflicting fork version schedule")
var errCollisionName = errors.New("config with conflicting name already exists")
var errConfigNotFound = errors.New("unable to find requested BeaconChainConfig")
var errReplaceNilConfig = errors.New("replace called with a nil value")

func (r *configset) add(c *BeaconChainConfig) error {
	name := c.ConfigName
	if _, exists := r.nameToConfig[name]; exists {
		return errors.Wrapf(errCollisionName, "ConfigName=%s", name)
	}
	c.InitializeForkSchedule()
	for v := range c.ForkVersionSchedule {
		if n, exists := r.versionToName[v]; exists {
			// determine the fork name for the colliding version
			cfv := ConfigForkVersions(c)
			versionId := cfv[v]
			msg := fmt.Sprintf("version %#x for fork %s in config %s conflicts with existing config named=%s",
				v, version.String(versionId), name, n)
			return errors.Wrap(errCollisionFork, msg)
		}
		r.versionToName[v] = name
	}
	r.nameToConfig[name] = c
	return nil
}

func (r *configset) delete(name string) {
	c, exists := r.nameToConfig[name]
	if !exists {
		return
	}
	for v := range c.ForkVersionSchedule {
		delete(r.versionToName, v)
	}
	delete(r.nameToConfig, name)
}

func (r *configset) replace(cfg *BeaconChainConfig) error {
	if cfg == nil {
		return errReplaceNilConfig
	}
	name := cfg.ConfigName
	r.delete(name)
	if err := r.add(cfg); err != nil {
		return err
	}
	if r.active != nil && r.active.ConfigName == name {
		r.active = cfg
	}
	return nil
}

func (r *configset) replaceWithUndo(cfg *BeaconChainConfig) (func() error, error) {
	name := cfg.ConfigName
	prev := r.nameToConfig[name]
	if prev != nil {
		prev = prev.Copy()
	}
	if err := r.replace(cfg); err != nil {
		return nil, err
	}
	return func() error {
		if prev == nil {
			if r.active.ConfigName == name {
				return errors.Wrapf(errCannotNullifyActive, "active config name=%s", name)
			}
			r.delete(name)
			return nil
		}
		return r.replace(prev)
	}, nil
}

func (r *configset) getActive() *BeaconChainConfig {
	return r.active
}

func (r *configset) setActive(c *BeaconChainConfig) error {
	if err := r.replace(c); err != nil {
		return err
	}
	r.active = c
	return nil
}

func (r *configset) setActiveWithUndo(c *BeaconChainConfig) (func() error, error) {
	if r.active == nil {
		return nil, errors.Wrap(errCannotNullifyActive,
			"active config is currently nil, refusing to construct undo method that will leave it nil again")
	}
	active := r.active.Copy()
	r.active = c
	undo, err := r.replaceWithUndo(c)
	if err != nil {
		return nil, err
	}
	return func() error {
		r.active = active
		return undo()
	}, nil
}

func (r *configset) byName(name string) (*BeaconChainConfig, error) {
	c, ok := r.nameToConfig[name]
	if !ok {
		return nil, errors.Wrapf(errConfigNotFound, "name=%s is not a known BeaconChainConfig name", name)
	}
	return c, nil
}

func (r *configset) byVersion(version [fieldparams.VersionLength]byte) (*BeaconChainConfig, error) {
	name, ok := r.versionToName[version]
	if !ok {
		return nil, errors.Wrapf(errConfigNotFound, "version=%#x not found in any known fork choice schedule", version)
	}
	return r.byName(name)
}

func (r *configset) all() []*BeaconChainConfig {
	all := make([]*BeaconChainConfig, 0)
	for _, c := range r.nameToConfig {
		all = append(all, c)
	}
	return all
}
