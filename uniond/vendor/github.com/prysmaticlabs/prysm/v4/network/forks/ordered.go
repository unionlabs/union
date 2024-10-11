package forks

import (
	"bytes"
	"sort"
	"strings"

	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// ForkScheduleEntry is a Version+Epoch tuple for sorted storage in an OrderedSchedule
type ForkScheduleEntry struct {
	Version [fieldparams.VersionLength]byte
	Epoch   primitives.Epoch
	Name    string
}

// OrderedSchedule provides a type that can be used to sort the fork schedule and find the Version
// the chain should be at for a given epoch (via VersionForEpoch) or name (via VersionForName).
type OrderedSchedule []ForkScheduleEntry

// Len implements the Len method of sort.Interface
func (o OrderedSchedule) Len() int { return len(o) }

// Swap implements the Swap method of sort.Interface
func (o OrderedSchedule) Swap(i, j int) { o[i], o[j] = o[j], o[i] }

// Less implements the Less method of sort.Interface
func (o OrderedSchedule) Less(i, j int) bool {
	if o[i].Epoch == o[j].Epoch {
		return bytes.Compare(o[i].Version[:], o[j].Version[:]) < 0
	}
	return o[i].Epoch < o[j].Epoch
}

// VersionForEpoch finds the Version with the highest epoch <= the given epoch
func (o OrderedSchedule) VersionForEpoch(epoch primitives.Epoch) ([fieldparams.VersionLength]byte, error) {
	for i := len(o) - 1; i >= 0; i-- {
		if o[i].Epoch <= epoch {
			return o[i].Version, nil
		}
	}
	return [fieldparams.VersionLength]byte{}, errors.Wrapf(ErrVersionNotFound, "no epoch in list <= %d", epoch)
}

// VersionForName finds the Version corresponding to the lowercase version of the provided name.
func (o OrderedSchedule) VersionForName(name string) ([fieldparams.VersionLength]byte, error) {
	lower := strings.ToLower(name)
	for _, e := range o {
		if e.Name == lower {
			return e.Version, nil
		}
	}
	return [4]byte{}, errors.Wrapf(ErrVersionNotFound, "no version with name %s", lower)
}

func (o OrderedSchedule) ForkFromVersion(version [fieldparams.VersionLength]byte) (*ethpb.Fork, error) {
	for i := range o {
		e := o[i]
		if e.Version == version {
			f := &ethpb.Fork{Epoch: e.Epoch, CurrentVersion: version[:], PreviousVersion: version[:]}
			if i > 0 {
				f.PreviousVersion = o[i-1].Version[:]
			}
			return f, nil
		}
	}
	return nil, errors.Wrapf(ErrVersionNotFound, "could not determine fork for version %#x", version)
}

func (o OrderedSchedule) Previous(version [fieldparams.VersionLength]byte) ([fieldparams.VersionLength]byte, error) {
	for i := len(o) - 1; i >= 0; i-- {
		if o[i].Version == version {
			if i-1 >= 0 {
				return o[i-1].Version, nil
			} else {
				return [fieldparams.VersionLength]byte{}, errors.Wrapf(ErrNoPreviousVersion, "%#x is the first version", version)
			}
		}
	}
	return [fieldparams.VersionLength]byte{}, errors.Wrapf(ErrVersionNotFound, "no version in list == %#x", version)
}

// NewOrderedSchedule Converts fork version maps into a list of Version+Epoch+Name values, ordered by Epoch from lowest to highest.
// See docs for OrderedSchedule for more detail on what you can do with this type.
func NewOrderedSchedule(b *params.BeaconChainConfig) OrderedSchedule {
	ofs := make(OrderedSchedule, 0)
	for version, epoch := range b.ForkVersionSchedule {
		fse := ForkScheduleEntry{
			Version: version,
			Epoch:   epoch,
			Name:    b.ForkVersionNames[version],
		}
		ofs = append(ofs, fse)
	}
	sort.Sort(ofs)
	return ofs
}
