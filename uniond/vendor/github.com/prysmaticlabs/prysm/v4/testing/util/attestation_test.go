package util

import (
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	v1 "github.com/prysmaticlabs/prysm/v4/proto/eth/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestHydrateAttestation(t *testing.T) {
	a := HydrateAttestation(&ethpb.Attestation{})
	_, err := a.HashTreeRoot()
	require.NoError(t, err)
	require.DeepEqual(t, a.Signature, make([]byte, fieldparams.BLSSignatureLength))
}

func TestHydrateAttestationData(t *testing.T) {
	d := HydrateAttestationData(&ethpb.AttestationData{})
	_, err := d.HashTreeRoot()
	require.NoError(t, err)
	require.DeepEqual(t, d.BeaconBlockRoot, make([]byte, fieldparams.RootLength))
	require.DeepEqual(t, d.Target.Root, make([]byte, fieldparams.RootLength))
	require.DeepEqual(t, d.Source.Root, make([]byte, fieldparams.RootLength))
}

func TestHydrateV1Attestation(t *testing.T) {
	a := HydrateV1Attestation(&v1.Attestation{})
	_, err := a.HashTreeRoot()
	require.NoError(t, err)
	require.DeepEqual(t, a.Signature, make([]byte, fieldparams.BLSSignatureLength))
}

func TestHydrateV1AttestationData(t *testing.T) {
	d := HydrateV1AttestationData(&v1.AttestationData{})
	_, err := d.HashTreeRoot()
	require.NoError(t, err)
	require.DeepEqual(t, d.BeaconBlockRoot, make([]byte, fieldparams.RootLength))
	require.DeepEqual(t, d.Target.Root, make([]byte, fieldparams.RootLength))
	require.DeepEqual(t, d.Source.Root, make([]byte, fieldparams.RootLength))
}

func TestHydrateIndexedAttestation(t *testing.T) {
	a := &ethpb.IndexedAttestation{}
	a = HydrateIndexedAttestation(a)
	_, err := a.HashTreeRoot()
	require.NoError(t, err)
	_, err = a.Data.HashTreeRoot()
	require.NoError(t, err)
}

func TestGenerateAttestations_EpochBoundary(t *testing.T) {
	gs, pk := DeterministicGenesisState(t, 32)
	_, err := GenerateAttestations(gs, pk, 1, params.BeaconConfig().SlotsPerEpoch, false)
	require.NoError(t, err)
}
