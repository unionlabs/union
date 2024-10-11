package main

import (
	"errors"
	"fmt"
	"math/rand"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/Masterminds/semver/v3"
	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing/object"

	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
	"github.com/cometbft/cometbft/version"
)

var (
	// testnetCombinations defines global testnet options, where we generate a
	// separate testnet for each combination (Cartesian product) of options.
	testnetCombinations = map[string][]any{
		"topology":      {"single", "quad", "large"},
		"initialHeight": {0, 1000},
		"initialState": {
			map[string]string{},
			map[string]string{"initial01": "a", "initial02": "b", "initial03": "c"},
		},
		"validators": {"genesis", "initchain"},
	}
	nodeVersions = weightedChoice{
		"": 2,
	}

	// The following specify randomly chosen values for testnet nodes.
	nodeDatabases = uniformChoice{"goleveldb", "rocksdb", "badgerdb", "pebbledb"}
	ipv6          = uniformChoice{false, true}
	// FIXME: grpc disabled due to https://github.com/tendermint/tendermint/issues/5439
	nodeABCIProtocols     = uniformChoice{"unix", "tcp", "builtin", "builtin_connsync"} // "grpc"
	nodePrivvalProtocols  = uniformChoice{"file", "unix", "tcp"}
	nodeBlockSyncs        = uniformChoice{"v0"} // "v2"
	nodeStateSyncs        = uniformChoice{false, true}
	nodePersistIntervals  = uniformChoice{0, 1, 5}
	nodeSnapshotIntervals = uniformChoice{0, 3}
	nodeRetainBlocks      = uniformChoice{
		0,
		2 * int(e2e.EvidenceAgeHeight),
		4 * int(e2e.EvidenceAgeHeight),
	}
	nodeEnableCompanionPruning = uniformChoice{true, false}
	evidence                   = uniformChoice{0, 1, 10, 20, 200}
	abciDelays                 = uniformChoice{"none", "small", "large"}
	nodePerturbations          = probSetChoice{
		"disconnect": 0.1,
		"pause":      0.1,
		"kill":       0.1,
		"restart":    0.1,
		"upgrade":    0.3,
	}
	lightNodePerturbations = probSetChoice{
		"upgrade": 0.3,
	}
	voteExtensionsUpdateHeight = uniformChoice{int64(-1), int64(0), int64(1)} // -1: genesis, 0: InitChain, 1: (use offset)
	voteExtensionEnabled       = weightedChoice{true: 3, false: 1}
	voteExtensionsHeightOffset = uniformChoice{int64(0), int64(10), int64(100)}
	voteExtensionSize          = uniformChoice{uint(128), uint(512), uint(2048), uint(8192)} // TODO: define the right values depending on experiment results.
	pbtsUpdateHeight           = uniformChoice{int64(-1), int64(0), int64(1)}                // -1: genesis, 0: InitChain, 1: (use offset)
	pbtsEnabled                = weightedChoice{true: 3, false: 1}
	pbtsHeightOffset           = uniformChoice{int64(0), int64(10), int64(100)}
)

type generateConfig struct {
	randSource   *rand.Rand
	outputDir    string
	multiVersion string
	prometheus   bool
	logLevel     string
}

// Generate generates random testnets using the given RNG.
func Generate(cfg *generateConfig) ([]e2e.Manifest, error) {
	upgradeVersion := ""

	if cfg.multiVersion != "" {
		var err error
		nodeVersions, upgradeVersion, err = parseWeightedVersions(cfg.multiVersion)
		if err != nil {
			return nil, err
		}
		if _, ok := nodeVersions["local"]; ok {
			nodeVersions[""] = nodeVersions["local"]
			delete(nodeVersions, "local")
			if upgradeVersion == "local" {
				upgradeVersion = ""
			}
		}
		if _, ok := nodeVersions["latest"]; ok {
			latestVersion, err := gitRepoLatestReleaseVersion(cfg.outputDir)
			if err != nil {
				return nil, err
			}
			nodeVersions[latestVersion] = nodeVersions["latest"]
			delete(nodeVersions, "latest")
			if upgradeVersion == "latest" {
				upgradeVersion = latestVersion
			}
		}
	}
	fmt.Println("Generating testnet with weighted versions:")
	for ver, wt := range nodeVersions {
		if ver == "" {
			fmt.Printf("- local: %d\n", wt)
		} else {
			fmt.Printf("- %s: %d\n", ver, wt)
		}
	}
	manifests := []e2e.Manifest{}
	for _, opt := range combinations(testnetCombinations) {
		manifest, err := generateTestnet(cfg.randSource, opt, upgradeVersion, cfg.prometheus, cfg.logLevel)
		if err != nil {
			return nil, err
		}
		manifests = append(manifests, manifest)
	}
	return manifests, nil
}

// generateTestnet generates a single testnet with the given options.
func generateTestnet(r *rand.Rand, opt map[string]any, upgradeVersion string, prometheus bool, logLevel string) (e2e.Manifest, error) {
	manifest := e2e.Manifest{
		IPv6:             ipv6.Choose(r).(bool),
		ABCIProtocol:     nodeABCIProtocols.Choose(r).(string),
		InitialHeight:    int64(opt["initialHeight"].(int)),
		InitialState:     opt["initialState"].(map[string]string),
		Validators:       &map[string]int64{},
		ValidatorUpdates: map[string]map[string]int64{},
		Evidence:         evidence.Choose(r).(int),
		Nodes:            map[string]*e2e.ManifestNode{},
		UpgradeVersion:   upgradeVersion,
		Prometheus:       prometheus,
		LogLevel:         logLevel,
	}

	switch abciDelays.Choose(r).(string) {
	case "none":
	case "small":
		manifest.PrepareProposalDelay = 100 * time.Millisecond
		manifest.ProcessProposalDelay = 100 * time.Millisecond
		manifest.VoteExtensionDelay = 20 * time.Millisecond
		manifest.FinalizeBlockDelay = 200 * time.Millisecond
	case "large":
		manifest.PrepareProposalDelay = 200 * time.Millisecond
		manifest.ProcessProposalDelay = 200 * time.Millisecond
		manifest.CheckTxDelay = 20 * time.Millisecond
		manifest.VoteExtensionDelay = 100 * time.Millisecond
		manifest.FinalizeBlockDelay = 500 * time.Millisecond
	}
	manifest.VoteExtensionsUpdateHeight = voteExtensionsUpdateHeight.Choose(r).(int64)
	if manifest.VoteExtensionsUpdateHeight == 1 {
		manifest.VoteExtensionsUpdateHeight = manifest.InitialHeight + voteExtensionsHeightOffset.Choose(r).(int64)
	}
	if voteExtensionEnabled.Choose(r).(bool) {
		baseHeight := max(manifest.VoteExtensionsUpdateHeight+1, manifest.InitialHeight)
		manifest.VoteExtensionsEnableHeight = baseHeight + voteExtensionsHeightOffset.Choose(r).(int64)
	}

	manifest.VoteExtensionSize = voteExtensionSize.Choose(r).(uint)

	manifest.PbtsUpdateHeight = pbtsUpdateHeight.Choose(r).(int64)
	if manifest.PbtsUpdateHeight == 1 {
		manifest.PbtsUpdateHeight = manifest.InitialHeight + pbtsHeightOffset.Choose(r).(int64)
	}
	if pbtsEnabled.Choose(r).(bool) {
		baseHeight := max(manifest.PbtsUpdateHeight+1, manifest.InitialHeight)
		manifest.PbtsEnableHeight = baseHeight + pbtsHeightOffset.Choose(r).(int64)
	}

	// TODO: Add skew config
	var numSeeds, numValidators, numFulls, numLightClients int
	switch opt["topology"].(string) {
	case "single":
		numValidators = 1
	case "quad":
		numValidators = 4
	case "large":
		// FIXME Networks are kept small since large ones use too much CPU.
		numSeeds = r.Intn(2)
		numLightClients = r.Intn(3)
		numValidators = 4 + r.Intn(4)
		numFulls = r.Intn(4)
	default:
		return manifest, fmt.Errorf("unknown topology %q", opt["topology"])
	}

	// First we generate seed nodes, starting at the initial height.
	for i := 1; i <= numSeeds; i++ {
		manifest.Nodes[fmt.Sprintf("seed%02d", i)] = generateNode(
			r, e2e.ModeSeed, 0, false)
	}

	// Next, we generate validators. We make sure a BFT quorum of validators start
	// at the initial height, and that we have two archive nodes. We also set up
	// the initial validator set, and validator set updates for delayed nodes.
	nextStartAt := manifest.InitialHeight + 5
	quorum := numValidators*2/3 + 1
	var totalWeight int64
	for i := 1; i <= numValidators; i++ {
		startAt := int64(0)
		if i > quorum {
			startAt = nextStartAt
			nextStartAt += 5
		}
		name := fmt.Sprintf("validator%02d", i)
		manifest.Nodes[name] = generateNode(r, e2e.ModeValidator, startAt, i <= 2)

		weight := int64(30 + r.Intn(71))
		if startAt == 0 {
			(*manifest.Validators)[name] = weight
		} else {
			manifest.ValidatorUpdates[strconv.FormatInt(startAt+5, 10)] = map[string]int64{name: weight}
		}
		totalWeight += weight
	}

	// Add clock skew only to processes that accumulate less than 1/3 of voting power.
	var accWeight int64
	for i := 1; i <= numValidators; i++ {
		name := fmt.Sprintf("validator%02d", i)
		startAt := manifest.Nodes[name].StartAt
		var weight int64
		if startAt == 0 {
			weight = (*manifest.Validators)[name]
		} else {
			weight = manifest.ValidatorUpdates[strconv.FormatInt(startAt+5, 10)][name]
		}

		if accWeight > totalWeight*2/3 {
			// Interval: [-500ms, 59s500ms)
			manifest.Nodes[name].ClockSkew = time.Duration(int64(r.Float64()*float64(time.Minute))) - 500*time.Millisecond
		}
		accWeight += weight
	}

	// Move validators to InitChain if specified.
	switch opt["validators"].(string) {
	case "genesis":
	case "initchain":
		manifest.ValidatorUpdates["0"] = *manifest.Validators
		manifest.Validators = &map[string]int64{}
	default:
		return manifest, fmt.Errorf("invalid validators option %q", opt["validators"])
	}

	// Finally, we generate random full nodes.
	for i := 1; i <= numFulls; i++ {
		startAt := int64(0)
		if r.Float64() >= 0.5 {
			startAt = nextStartAt
			nextStartAt += 5
		}
		manifest.Nodes[fmt.Sprintf("full%02d", i)] = generateNode(
			r, e2e.ModeFull, startAt, false)
	}

	// We now set up peer discovery for nodes. Seed nodes are fully meshed with
	// each other, while non-seed nodes either use a set of random seeds or a
	// set of random peers that start before themselves.
	var seedNames, peerNames, lightProviders []string
	for name, node := range manifest.Nodes {
		if node.Mode == string(e2e.ModeSeed) {
			seedNames = append(seedNames, name)
		} else {
			// if the full node or validator is an ideal candidate, it is added as a light provider.
			// There are at least two archive nodes so there should be at least two ideal candidates
			if (node.StartAt == 0 || node.StartAt == manifest.InitialHeight) && node.RetainBlocks == 0 {
				lightProviders = append(lightProviders, name)
			}
			peerNames = append(peerNames, name)
		}
	}

	for _, name := range seedNames {
		for _, otherName := range seedNames {
			if name != otherName {
				manifest.Nodes[name].Seeds = append(manifest.Nodes[name].Seeds, otherName)
			}
		}
	}

	sort.Slice(peerNames, func(i, j int) bool {
		iName, jName := peerNames[i], peerNames[j]
		switch {
		case manifest.Nodes[iName].StartAt < manifest.Nodes[jName].StartAt:
			return true
		case manifest.Nodes[iName].StartAt > manifest.Nodes[jName].StartAt:
			return false
		default:
			return strings.Compare(iName, jName) == -1
		}
	})
	for i, name := range peerNames {
		if len(seedNames) > 0 && (i == 0 || r.Float64() >= 0.5) {
			manifest.Nodes[name].Seeds = uniformSetChoice(seedNames).Choose(r)
		} else if i > 0 {
			manifest.Nodes[name].PersistentPeers = uniformSetChoice(peerNames[:i]).Choose(r)
		}
	}

	// lastly, set up the light clients
	for i := 1; i <= numLightClients; i++ {
		startAt := manifest.InitialHeight + 5
		manifest.Nodes[fmt.Sprintf("light%02d", i)] = generateLightNode(
			r, startAt+(5*int64(i)), lightProviders,
		)
	}

	return manifest, nil
}

// generateNode randomly generates a node, with some constraints to avoid
// generating invalid configurations. We do not set Seeds or PersistentPeers
// here, since we need to know the overall network topology and startup
// sequencing.
func generateNode(
	r *rand.Rand, mode e2e.Mode, startAt int64, forceArchive bool,
) *e2e.ManifestNode {
	node := e2e.ManifestNode{
		Version:                nodeVersions.Choose(r).(string),
		Mode:                   string(mode),
		StartAt:                startAt,
		Database:               nodeDatabases.Choose(r).(string),
		PrivvalProtocol:        nodePrivvalProtocols.Choose(r).(string),
		BlockSyncVersion:       nodeBlockSyncs.Choose(r).(string),
		StateSync:              nodeStateSyncs.Choose(r).(bool) && startAt > 0,
		PersistInterval:        ptrUint64(uint64(nodePersistIntervals.Choose(r).(int))),
		SnapshotInterval:       uint64(nodeSnapshotIntervals.Choose(r).(int)),
		RetainBlocks:           uint64(nodeRetainBlocks.Choose(r).(int)),
		EnableCompanionPruning: false,
		Perturb:                nodePerturbations.Choose(r),
	}

	// If this node is forced to be an archive node, retain all blocks and
	// enable state sync snapshotting.
	if forceArchive {
		node.RetainBlocks = 0
		node.SnapshotInterval = 3
	}

	// If a node which does not persist state also does not retain blocks, randomly
	// choose to either persist state or retain all blocks.
	if node.PersistInterval != nil && *node.PersistInterval == 0 && node.RetainBlocks > 0 {
		if r.Float64() > 0.5 {
			node.RetainBlocks = 0
		} else {
			node.PersistInterval = ptrUint64(node.RetainBlocks)
		}
	}

	// If either PersistInterval or SnapshotInterval are greater than RetainBlocks,
	// expand the block retention time.
	if node.RetainBlocks > 0 {
		if node.PersistInterval != nil && node.RetainBlocks < *node.PersistInterval {
			node.RetainBlocks = *node.PersistInterval
		}
		if node.RetainBlocks < node.SnapshotInterval {
			node.RetainBlocks = node.SnapshotInterval
		}
	}

	// Only randomly enable data companion-related pruning on 50% of the full
	// nodes and validators.
	if mode == e2e.ModeFull || mode == e2e.ModeValidator {
		node.EnableCompanionPruning = nodeEnableCompanionPruning.Choose(r).(bool)
	}

	return &node
}

func generateLightNode(r *rand.Rand, startAt int64, providers []string) *e2e.ManifestNode {
	return &e2e.ManifestNode{
		Mode:            string(e2e.ModeLight),
		Version:         nodeVersions.Choose(r).(string),
		StartAt:         startAt,
		Database:        nodeDatabases.Choose(r).(string),
		PersistInterval: ptrUint64(0),
		PersistentPeers: providers,
		Perturb:         lightNodePerturbations.Choose(r),
	}
}

func ptrUint64(i uint64) *uint64 {
	return &i
}

// Parses strings like "v0.34.21:1,v0.34.22:2" to represent two versions
// ("v0.34.21" and "v0.34.22") with weights of 1 and 2 respectively.
// Versions may be specified as cometbft/e2e-node:v0.34.27-alpha.1:1 or
// ghcr.io/informalsystems/tendermint:v0.34.26:1.
// If only the tag and weight are specified, cometbft/e2e-node is assumed.
// Also returns the last version in the list, which will be used for updates.
func parseWeightedVersions(s string) (weightedChoice, string, error) {
	wc := make(weightedChoice)
	lv := ""
	wvs := strings.Split(strings.TrimSpace(s), ",")
	for _, wv := range wvs {
		parts := strings.Split(strings.TrimSpace(wv), ":")
		var ver string
		switch len(parts) {
		case 2:
			ver = strings.TrimSpace(strings.Join([]string{"cometbft/e2e-node", parts[0]}, ":"))
		case 3:
			ver = strings.TrimSpace(strings.Join([]string{parts[0], parts[1]}, ":"))
		default:
			return nil, "", fmt.Errorf("unexpected weight:version combination: %s", wv)
		}

		wt, err := strconv.Atoi(strings.TrimSpace(parts[len(parts)-1]))
		if err != nil {
			return nil, "", fmt.Errorf("unexpected weight \"%s\": %w", parts[1], err)
		}

		if wt < 1 {
			return nil, "", errors.New("version weights must be >= 1")
		}
		wc[ver] = uint(wt)
		lv = ver
	}
	return wc, lv, nil
}

// Extracts the latest release version from the given Git repository. Uses the
// current version of CometBFT to establish the "major" version
// currently in use.
func gitRepoLatestReleaseVersion(gitRepoDir string) (string, error) {
	opts := &git.PlainOpenOptions{
		DetectDotGit: true,
	}
	r, err := git.PlainOpenWithOptions(gitRepoDir, opts)
	if err != nil {
		return "", err
	}
	tags := make([]string, 0)
	tagObjs, err := r.TagObjects()
	if err != nil {
		return "", err
	}
	err = tagObjs.ForEach(func(tagObj *object.Tag) error {
		tags = append(tags, tagObj.Name)
		return nil
	})
	if err != nil {
		return "", err
	}
	return findLatestReleaseTag(version.CMTSemVer, tags)
}

func findLatestReleaseTag(baseVer string, tags []string) (string, error) {
	baseSemVer, err := semver.NewVersion(strings.Split(baseVer, "-")[0])
	if err != nil {
		return "", fmt.Errorf("failed to parse base version \"%s\": %w", baseVer, err)
	}
	compVer := fmt.Sprintf("%d.%d", baseSemVer.Major(), baseSemVer.Minor())
	// Build our version comparison string
	// See https://github.com/Masterminds/semver#caret-range-comparisons-major for details
	compStr := "^ " + compVer
	verCon, err := semver.NewConstraint(compStr)
	if err != nil {
		return "", err
	}
	var latestVer *semver.Version
	for _, tag := range tags {
		if !strings.HasPrefix(tag, "v") {
			continue
		}
		curVer, err := semver.NewVersion(tag)
		// Skip tags that are not valid semantic versions
		if err != nil {
			continue
		}
		// Skip pre-releases
		if len(curVer.Prerelease()) != 0 {
			continue
		}
		// Skip versions that don't match our constraints
		if !verCon.Check(curVer) {
			continue
		}
		if latestVer == nil || curVer.GreaterThan(latestVer) {
			latestVer = curVer
		}
	}
	// No relevant latest version (will cause the generator to only use the tip
	// of the current branch)
	if latestVer == nil {
		return "", nil
	}
	// Ensure the version string has a "v" prefix, because all CometBFT E2E
	// node Docker images' versions have a "v" prefix.
	vs := latestVer.String()
	if !strings.HasPrefix(vs, "v") {
		return "v" + vs, nil
	}
	return vs, nil
}
