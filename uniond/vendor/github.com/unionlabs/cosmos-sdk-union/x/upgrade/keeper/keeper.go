package keeper

import (
	"context"
	"encoding/binary"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strconv"

	"github.com/hashicorp/go-metrics"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/server"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/upgrade/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/telemetry"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/kv"
)

type Keeper struct {
	appmodule.Environment

	homePath           string                          // root directory of app config
	skipUpgradeHeights map[int64]bool                  // map of heights to skip for an upgrade
	cdc                codec.BinaryCodec               // App-wide binary codec
	upgradeHandlers    map[string]types.UpgradeHandler // map of plan name to upgrade handler
	versionModifier    server.VersionModifier          // implements setting the protocol version field on BaseApp
	downgradeVerified  bool                            // tells if we've already sanity checked that this binary version isn't being used against an old state.
	authority          string                          // the address capable of executing and canceling an upgrade. Usually the gov module account
	initVersionMap     appmodule.VersionMap            // the module version map at init genesis

	consensusKeeper types.ConsensusKeeper
}

// NewKeeper constructs an upgrade Keeper which requires the following arguments:
// skipUpgradeHeights - map of heights to skip an upgrade
// storeKey - a store key with which to access upgrade's store
// cdc - the app-wide binary codec
// homePath - root directory of the application's config
// vs - the interface implemented by baseapp which allows setting baseapp's protocol version field
func NewKeeper(
	env appmodule.Environment,
	skipUpgradeHeights map[int64]bool,
	cdc codec.BinaryCodec,
	homePath string,
	vs server.VersionModifier,
	authority string,
	ck types.ConsensusKeeper,
) *Keeper {
	k := &Keeper{
		Environment:        env,
		homePath:           homePath,
		skipUpgradeHeights: skipUpgradeHeights,
		cdc:                cdc,
		upgradeHandlers:    map[string]types.UpgradeHandler{},
		versionModifier:    vs,
		authority:          authority,
		consensusKeeper:    ck,
	}

	if homePath == "" {
		k.Logger.Warn("homePath is empty; upgrade info will be written to the current directory")
	}

	return k
}

// SetInitVersionMap sets the initial version map.
// This is only used in app wiring and should not be used in any other context.
func (k *Keeper) SetInitVersionMap(vm appmodule.VersionMap) {
	k.initVersionMap = vm
}

// GetInitVersionMap gets the initial version map
// This is only used in upgrade InitGenesis and should not be used in any other context.
func (k *Keeper) GetInitVersionMap() appmodule.VersionMap {
	return k.initVersionMap
}

// SetUpgradeHandler sets an UpgradeHandler for the upgrade specified by name. This handler will be called when the upgrade
// with this name is applied. In order for an upgrade with the given name to proceed, a handler for this upgrade
// must be set even if it is a no-op function.
func (k Keeper) SetUpgradeHandler(name string, upgradeHandler types.UpgradeHandler) {
	k.upgradeHandlers[name] = upgradeHandler
}

// SetModuleVersionMap saves a given version map to state
func (k Keeper) SetModuleVersionMap(ctx context.Context, vm appmodule.VersionMap) error {
	if len(vm) > 0 {
		store := runtime.KVStoreAdapter(k.KVStoreService.OpenKVStore(ctx))
		versionStore := prefix.NewStore(store, []byte{types.VersionMapByte})
		// Even though the underlying store (cachekv) store is sorted, we still
		// prefer a deterministic iteration order of the map, to avoid undesired
		// surprises if we ever change stores.
		sortedModNames := make([]string, 0, len(vm))

		for key := range vm {
			sortedModNames = append(sortedModNames, key)
		}
		sort.Strings(sortedModNames)

		for _, modName := range sortedModNames {
			ver := vm[modName]
			nameBytes := []byte(modName)
			verBytes := make([]byte, 8)
			binary.BigEndian.PutUint64(verBytes, ver)
			versionStore.Set(nameBytes, verBytes)
		}
	}

	return nil
}

// GetModuleVersionMap returns a map of key module name and value module consensus version
// as defined in ADR-041.
func (k Keeper) GetModuleVersionMap(ctx context.Context) (appmodule.VersionMap, error) {
	store := k.KVStoreService.OpenKVStore(ctx)
	prefix := []byte{types.VersionMapByte}
	it, err := store.Iterator(prefix, storetypes.PrefixEndBytes(prefix))
	if err != nil {
		return nil, err
	}
	defer it.Close()

	vm := make(appmodule.VersionMap)
	for ; it.Valid(); it.Next() {
		moduleBytes := it.Key()
		// first byte is prefix key, so we remove it here
		name := string(moduleBytes[1:])
		moduleVersion := binary.BigEndian.Uint64(it.Value())
		vm[name] = moduleVersion
	}

	return vm, nil
}

// GetModuleVersions gets a slice of module consensus versions
func (k Keeper) GetModuleVersions(ctx context.Context) ([]*types.ModuleVersion, error) {
	store := k.KVStoreService.OpenKVStore(ctx)
	prefix := []byte{types.VersionMapByte}
	it, err := store.Iterator(prefix, storetypes.PrefixEndBytes(prefix))
	if err != nil {
		return nil, err
	}
	defer it.Close()

	mv := make([]*types.ModuleVersion, 0)
	for ; it.Valid(); it.Next() {
		moduleBytes := it.Key()
		name := string(moduleBytes[1:])
		moduleVersion := binary.BigEndian.Uint64(it.Value())
		mv = append(mv, &types.ModuleVersion{
			Name:    name,
			Version: moduleVersion,
		})
	}

	return mv, nil
}

// getModuleVersion gets the version for a given module. If it doesn't exist it returns ErrNoModuleVersionFound, other
// errors may be returned if there is an error reading from the store.
func (k Keeper) getModuleVersion(ctx context.Context, name string) (uint64, error) {
	store := k.KVStoreService.OpenKVStore(ctx)
	prefix := []byte{types.VersionMapByte}
	it, err := store.Iterator(prefix, storetypes.PrefixEndBytes(prefix))
	if err != nil {
		return 0, err
	}
	defer it.Close()

	for ; it.Valid(); it.Next() {
		moduleName := string(it.Key()[1:])
		if moduleName == name {
			version := binary.BigEndian.Uint64(it.Value())
			return version, nil
		}
	}

	return 0, types.ErrNoModuleVersionFound
}

// ScheduleUpgrade schedules an upgrade based on the specified plan.
// If there is another Plan already scheduled, it will cancel and overwrite it.
// ScheduleUpgrade will also write the upgraded IBC ClientState to the upgraded client
// path if it is specified in the plan.
func (k Keeper) ScheduleUpgrade(ctx context.Context, plan types.Plan) error {
	if err := plan.ValidateBasic(); err != nil {
		return err
	}

	// NOTE: allow for the possibility of chains to schedule upgrades in begin block of the same block
	// as a strategy for emergency hard fork recoveries
	if plan.Height < k.HeaderService.HeaderInfo(ctx).Height {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "upgrade cannot be scheduled in the past")
	}

	doneHeight, err := k.GetDoneHeight(ctx, plan.Name)
	if err != nil {
		return err
	}

	if doneHeight != 0 {
		return errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "upgrade with name %s has already been completed", plan.Name)
	}

	store := k.KVStoreService.OpenKVStore(ctx)

	// clear any old IBC state stored by previous plan
	oldPlan, err := k.GetUpgradePlan(ctx)
	// if there's an error but it's not ErrNoUpgradePlanFound, return error
	if err != nil && !errors.Is(err, types.ErrNoUpgradePlanFound) {
		return err
	}

	if err == nil {
		err = k.ClearIBCState(ctx, oldPlan.Height)
		if err != nil {
			return err
		}
	}

	bz, err := k.cdc.Marshal(&plan)
	if err != nil {
		return err
	}

	err = store.Set(types.PlanKey(), bz)
	if err != nil {
		return err
	}

	telemetry.SetGaugeWithLabels([]string{"server", "info"}, 1, []metrics.Label{telemetry.NewLabel("upgrade_height", strconv.FormatInt(plan.Height, 10))})

	return nil
}

// SetUpgradedClient sets the expected upgraded client for the next version of this chain at the last height the current chain will commit.
func (k Keeper) SetUpgradedClient(ctx context.Context, planHeight int64, bz []byte) error {
	store := k.KVStoreService.OpenKVStore(ctx)
	return store.Set(types.UpgradedClientKey(planHeight), bz)
}

// GetUpgradedClient gets the expected upgraded client for the next version of this chain. If not found it returns
// ErrNoUpgradedClientFound, but other errors may be returned if there is an error reading from the store.
func (k Keeper) GetUpgradedClient(ctx context.Context, height int64) ([]byte, error) {
	store := k.KVStoreService.OpenKVStore(ctx)
	bz, err := store.Get(types.UpgradedClientKey(height))
	if err != nil {
		return nil, err
	}

	if bz == nil {
		return nil, types.ErrNoUpgradedClientFound
	}

	return bz, nil
}

// SetUpgradedConsensusState sets the expected upgraded consensus state for the next version of this chain
// using the last height committed on this chain.
func (k Keeper) SetUpgradedConsensusState(ctx context.Context, planHeight int64, bz []byte) error {
	store := k.KVStoreService.OpenKVStore(ctx)
	return store.Set(types.UpgradedConsStateKey(planHeight), bz)
}

// GetUpgradedConsensusState gets the expected upgraded consensus state for the next version of this chain. If not found
// it returns ErrNoUpgradedConsensusStateFound, but other errors may be returned if there is an error reading from the store.
func (k Keeper) GetUpgradedConsensusState(ctx context.Context, lastHeight int64) ([]byte, error) {
	store := k.KVStoreService.OpenKVStore(ctx)
	bz, err := store.Get(types.UpgradedConsStateKey(lastHeight))
	if err != nil {
		return nil, err
	}

	if bz == nil {
		return nil, types.ErrNoUpgradedConsensusStateFound
	}

	return bz, nil
}

// GetLastCompletedUpgrade returns the last applied upgrade name and height.
func (k Keeper) GetLastCompletedUpgrade(ctx context.Context) (string, int64, error) {
	store := k.KVStoreService.OpenKVStore(ctx)
	prefix := []byte{types.DoneByte}
	it, err := store.ReverseIterator(prefix, storetypes.PrefixEndBytes(prefix))
	if err != nil {
		return "", 0, err
	}
	defer it.Close()

	if it.Valid() {
		name, height := parseDoneKey(it.Key())
		return name, height, nil
	}

	return "", 0, nil
}

// parseDoneKey - split upgrade name and height from the done key
func parseDoneKey(key []byte) (string, int64) {
	// 1 byte for the DoneByte + 8 bytes height + at least 1 byte for the name
	kv.AssertKeyAtLeastLength(key, 10)
	height := binary.BigEndian.Uint64(key[1:9])
	return string(key[9:]), int64(height)
}

// encodeDoneKey - concatenate DoneByte, height and upgrade name to form the done key
func encodeDoneKey(name string, height int64) []byte {
	key := make([]byte, 9+len(name)) // 9 = donebyte + uint64 len
	key[0] = types.DoneByte
	binary.BigEndian.PutUint64(key[1:9], uint64(height))
	copy(key[9:], name)
	return key
}

// GetDoneHeight returns the height at which the given upgrade was executed
func (k Keeper) GetDoneHeight(ctx context.Context, name string) (int64, error) {
	store := k.KVStoreService.OpenKVStore(ctx)
	prefix := []byte{types.DoneByte}
	it, err := store.Iterator(prefix, storetypes.PrefixEndBytes(prefix))
	if err != nil {
		return 0, err
	}
	defer it.Close()

	for ; it.Valid(); it.Next() {
		upgradeName, height := parseDoneKey(it.Key())
		if upgradeName == name {
			return height, nil
		}
	}

	return 0, nil
}

// ClearIBCState clears any planned IBC state
func (k Keeper) ClearIBCState(ctx context.Context, lastHeight int64) error {
	// delete IBC client and consensus state from store if this is IBC plan
	store := k.KVStoreService.OpenKVStore(ctx)
	err := store.Delete(types.UpgradedClientKey(lastHeight))
	if err != nil {
		return err
	}

	return store.Delete(types.UpgradedConsStateKey(lastHeight))
}

// ClearUpgradePlan clears any schedule upgrade and associated IBC states.
func (k Keeper) ClearUpgradePlan(ctx context.Context) error {
	// clear IBC states every time upgrade plan is removed
	oldPlan, err := k.GetUpgradePlan(ctx)
	if err != nil {
		// if there's no upgrade plan, return nil to match previous behavior
		if errors.Is(err, types.ErrNoUpgradePlanFound) {
			return nil
		}
		return err
	}

	err = k.ClearIBCState(ctx, oldPlan.Height)
	if err != nil {
		return err
	}

	store := k.KVStoreService.OpenKVStore(ctx)
	return store.Delete(types.PlanKey())
}

// GetUpgradePlan returns the currently scheduled Plan if any. If not found it returns
// ErrNoUpgradePlanFound, but other errors may be returned if there is an error reading from the store.
func (k Keeper) GetUpgradePlan(ctx context.Context) (plan types.Plan, err error) {
	store := k.KVStoreService.OpenKVStore(ctx)
	bz, err := store.Get(types.PlanKey())
	if err != nil {
		return plan, err
	}

	if bz == nil {
		return plan, types.ErrNoUpgradePlanFound
	}

	err = k.cdc.Unmarshal(bz, &plan)
	if err != nil {
		return plan, err
	}

	return plan, err
}

// setDone marks this upgrade name as being done so the name can't be reused accidentally
func (k Keeper) setDone(ctx context.Context, name string) error {
	store := k.KVStoreService.OpenKVStore(ctx)

	k.Logger.Debug("setting done", "height", k.HeaderService.HeaderInfo(ctx).Height, "name", name)

	return store.Set(encodeDoneKey(name, k.HeaderService.HeaderInfo(ctx).Height), []byte{1})
}

// HasHandler returns true iff there is a handler registered for this name
func (k Keeper) HasHandler(name string) bool {
	_, ok := k.upgradeHandlers[name]
	return ok
}

// ApplyUpgrade will execute the handler associated with the Plan and mark the plan as done.
// If successful, it will increment the app version and clear the IBC state
func (k Keeper) ApplyUpgrade(ctx context.Context, plan types.Plan) error {
	handler := k.upgradeHandlers[plan.Name]
	if handler == nil {
		return errors.New("ApplyUpgrade should never be called without first checking HasHandler")
	}

	vm, err := k.GetModuleVersionMap(ctx)
	if err != nil {
		return err
	}

	updatedVM, err := handler(ctx, plan, vm)
	if err != nil {
		return err
	}

	err = k.SetModuleVersionMap(ctx, updatedVM)
	if err != nil {
		return err
	}

	// incremement the app version and set it in state and baseapp
	if k.versionModifier != nil {
		currentAppVersion, err := k.versionModifier.AppVersion(ctx)
		if err != nil {
			return err
		}

		if err := k.versionModifier.SetAppVersion(ctx, currentAppVersion+1); err != nil {
			return err
		}
	}

	// Must clear IBC state after upgrade is applied as it is stored separately from the upgrade plan.
	// This will prevent resubmission of upgrade msg after upgrade is already completed.
	err = k.ClearIBCState(ctx, plan.Height)
	if err != nil {
		return err
	}

	err = k.ClearUpgradePlan(ctx)
	if err != nil {
		return err
	}

	return k.setDone(ctx, plan.Name)
}

// IsSkipHeight checks if the given height is part of skipUpgradeHeights
func (k Keeper) IsSkipHeight(height int64) bool {
	return k.skipUpgradeHeights[height]
}

// DumpUpgradeInfoToDisk writes upgrade information to UpgradeInfoFileName.
func (k Keeper) DumpUpgradeInfoToDisk(height int64, p types.Plan) error {
	upgradeInfoFilePath, err := k.GetUpgradeInfoPath()
	if err != nil {
		return err
	}

	upgradeInfo := types.Plan{
		Name:   p.Name,
		Height: height,
		Info:   p.Info,
	}
	info, err := json.Marshal(upgradeInfo)
	if err != nil {
		return err
	}

	return os.WriteFile(upgradeInfoFilePath, info, 0o600)
}

// GetUpgradeInfoPath returns the upgrade info file path
func (k Keeper) GetUpgradeInfoPath() (string, error) {
	upgradeInfoFileDir := filepath.Join(k.homePath, "data")
	if err := os.MkdirAll(upgradeInfoFileDir, os.ModePerm); err != nil {
		return "", fmt.Errorf("could not create directory %q: %w", upgradeInfoFileDir, err)
	}

	return filepath.Join(upgradeInfoFileDir, types.UpgradeInfoFilename), nil
}

// ReadUpgradeInfoFromDisk returns the name and height of the upgrade which is
// written to disk by the old binary when panicking. An error is returned if
// the upgrade path directory cannot be created or if the file exists and
// cannot be read or if the upgrade info fails to unmarshal.
func (k Keeper) ReadUpgradeInfoFromDisk() (types.Plan, error) {
	var upgradeInfo types.Plan

	upgradeInfoPath, err := k.GetUpgradeInfoPath()
	if err != nil {
		return upgradeInfo, err
	}

	data, err := os.ReadFile(upgradeInfoPath)
	if err != nil {
		// if file does not exist, assume there are no upgrades
		if os.IsNotExist(err) {
			return upgradeInfo, nil
		}

		return upgradeInfo, err
	}

	if err := json.Unmarshal(data, &upgradeInfo); err != nil {
		return upgradeInfo, err
	}

	if err := upgradeInfo.ValidateBasic(); err != nil {
		return upgradeInfo, err
	}

	if upgradeInfo.Height > 0 {
		telemetry.SetGaugeWithLabels([]string{"server", "info"}, 1, []metrics.Label{telemetry.NewLabel("upgrade_height", strconv.FormatInt(upgradeInfo.Height, 10))})
	}

	return upgradeInfo, nil
}

// SetDowngradeVerified updates downgradeVerified.
func (k *Keeper) SetDowngradeVerified(v bool) {
	k.downgradeVerified = v
}

// DowngradeVerified returns downgradeVerified.
func (k Keeper) DowngradeVerified() bool {
	return k.downgradeVerified
}
