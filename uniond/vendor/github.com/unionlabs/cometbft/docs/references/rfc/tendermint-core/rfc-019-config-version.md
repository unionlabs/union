<!-- markdown-link-check-disable -->
# RFC 019: Configuration File Versioning

## Changelog

- 19-Apr-2022: Initial draft (@creachadair)
- 20-Apr-2022: Updates from review feedback (@creachadair)

## Abstract

Updating configuration settings is an essential part of upgrading an existing
node to a new version of the Tendermint software.  Unfortunately, it is also
currently a very manual process. This document discusses some of the history of
changes to the config format, actions we've taken to improve the tooling for
configuration upgrades, and additional steps we may want to consider.

## Background

A Tendermint node reads configuration settings at startup from a TOML formatted
text file, typically named `config.toml`. The contents of this file are defined
by the [`github.com/tendermint/tendermint/config`][config-pkg].

Although many settings in this file remain valid from one version of Tendermint
to the next, new versions of Tendermint often add, update, and remove settings.
These changes often require manual intervention by operators who are upgrading
their nodes.

I propose we should provide better tools and documentation to help operators
make configuration changes correctly during version upgrades.  Ideally, as much
as possible of any configuration file update should be automated, and where
that is not possible or practical, we should provide clear, explicit directions
for what steps need to be taken manually. Moreover, when the node discovers
incorrect or invalid configuration, we should improve the diagnostics it emits
so that the operator can quickly and easily find the relevant documentation,
without having to grep through source code.

## Discussion

By convention, we are supposed to document required changes to the config file
in the `UPGRADING.md` file for the release that introduces them.  Although we
have mostly done this, the level of detail in the upgrading instructions is
often insufficient for an operator to correctly update their file.

The updates vary widely in complexity: Operators may need to add new required
settings, update obsolete values for existing settings, move or rename existing
settings within the file, or remove obsolete settings (which are thus invalid).
Here are a few examples of each of these cases:

- **New required settings:** Tendermint v0.35 added a new top-level `mode`
  setting that determines whether a node runs as a validator, a full node, or a
  seed node.  The default value is `"full"`, which means the operator of a
  validator must manually add `mode = "validator"` (or set the `--mode` flag on
  the command line) for their node to come up in the correct mode.

- **Updated obsolete values:** Tendermint v0.35 removed support for versions
  `"v1"` and `"v2"` of the blocksync (formerly "fastsync") protocol, requiring
  any node using either of those values to update to `"v0"`.

- **Moved/renamed settings:** Version v0.34 moved the top-level `pprof_laddr`
  setting under the `[rpc]` section.

  Version v0.35 renamed every setting in the file from `snake_case` to
  `kebab-case`, moved the top-level `fast_sync` setting into the `[blocksync]`
  section as (itself renamed from `[fastsync]`), and moved all the top-level
  `priv-validator-*` settings under a new `[priv-validator]` section with their
  prefix trimmed off.

- **Removed obsolete settings:** Version v0.34 removed the `index_all_keys` and
  `index_keys` settings from the `[tx_index]` section; version v0.35 removed
  the `wal-dir` setting from the `[mempool]` section, and version v0.36 removed
  the `[blocksync]` section entirely.

While many of these changes are mentioned in the config section of the upgrade
instructions, some are not mentioned at all, or are hidden in other parts of
the doc. For instance, the v0.34 `pprof_laddr` change was documented only as an
RPC flag change. (A savvy reader might realize that the flag `--rpc.pprof_laddr`
implies a corresponding config section, but it omits the related detail that
there was a top-level setting that's been renamed).  The lesson here is not
that the docs are bad, but to point out that prose is not the most efficient
format to convey detailed changes like this. The upgrading instructions are
still valuable for the human reader to understand what to expect.

### Concrete Steps

As part of the v0.36 development cycle, we spent some time reverse-engineering
the configuration changes since the v0.34 release and built an experimental
command-line tool called [`confix`][confix], whose job it is to automatically
update the settings in a `config.toml` file to the latest version.  We also
backported a version of this tool into the v0.35.x branch at release v0.35.4.

This tool should work fine for configuration files created by Tendermint v0.34
and later, but does not (yet) know how to handle changes from prior versions of
Tendermint. Part of the difficulty for older versions is simply logistical: To
figure out which changes to apply, we need to understand something about the
version that made the file, as well as the version we're converting it to.

> **Discussion point:** In the future we might want to consider incorporating
> this into the node CLI directly, but we're keeping it separate for now until
> we can get some feedback from operators.

For the experiment, we handled this by carefully searching the history of
config format changes for shibboleths to bound the version: For example, the
`[fastsync]` section was added in Tendermint v0.32 and renamed `[blocksync]` in
Tendermint v0.35. So if we see a `[fastsync]` section, we have some confidence
that the file was created by v0.32, v0.33, or v0.34.

But such signals are delicate: The `[blocksync]` section was removed in v0.36,
so if we do not find `[fastsync]`, we cannot conclude from that alone that the
file is from v0.31 or earlier -- we have to look for corroborating details.
While such "sniffing" tactics are fine for an experiment, they aren't as robust
as we might like.

This is especially relevant for configuration files that may have already been
manually upgraded across several versions by the time we are asked to update
them again.  Another related concern is that we'd like to make sure conversion
is idempotent, so that it would be safe to rerun the tool over an
already-converted file without breaking anything.

### Config Versioning

One obvious tactic we could use for future releases is add a version marker to
the config file. This would give tools like `confix` (and the node itself) a
way to calibrate their expectations. Rather than being a version for the file
itself, however, this version marker would indicate which version of Tendermint
is needed to read the file.

Provisionally, this might look something like:

```toml
# THe minimum version of Tendermint compatible with the contents of
# this configuration file.
config-version = 'v0.35'
```

When initializing a new node, Tendermint would populate this field with its own
version (e.g., `v0.36`). When conducting an upgrade, tools like `confix` can
then use this to decide which conversions are valid, and then update the value
accordingly. After converting a file marked `'v0.35'` to`'v0.37'`, the
conversion tool sets the file's `config-version` to reflect its compatibility.

> **Discussion point:** This example presumes we would keep config files
> compatible within a given release cycle, e.g., all of v0.36.x. We could also
> use patch numbers here, if we think there's some reason to permit changes
> that would require config file edits at that granularity. I don't think we
> should, but that's a design question to consider.

Upon seeing an up-to-date version marker, the conversion tool can simply exit
with a diagnostic like "this file is already up-to-date", rather than sniffing
the keyspace and potentially introducing errors. In addition, this would let a
tool detect config files that are _newer_ than the one it understands, and
issue a safe diagnostic rather than doing something wrong.  Plus, besides
avoiding potentially unsafe conversions, this would also serve as
human-readable documentation that the file is up-to-date for a given version.

Adding a config version would not address the problem of how to convert files
created by older versions of Tendermint, but it would at least help us build
more robust config tooling going forward.

### Stability and Change

In light of the discussion so far, it is natural to examine why we make so many
changes to the configuration file from one version to the next, and whether we
could reduce friction by being more conservative about what we make
configurable, what config changes we make over time, and how we roll them out.

Some changes, like renaming everything from snake case to kebab case, are
entirely gratuitous. We could safely agree not to make those kinds of changes.
Apart from that obvious case, however, many other configuration settings
provide value to node operators in cases where there is no simple, universal
setting that matches every application.

Taking a high-level view, there are several broad reasons why we might want to
make changes to configuration settings:

- **Lessons learned:** Configuration settings are a good way to try things out
  in production, before making more invasive changes to the consensus protocol.

  For example, up until Tendermint v0.35, consensus timeouts were specified as
  per-node configuration settings (e.g., `timeout-precommit` et al.).  This
  allowed operators to tune these values for the needs of their network, but
  had the downside that individually-misconfigured nodes could stall consensus.

  Based on that experience, these timeouts have been deprecated in Tendermint
  v0.36 and converted to consensus parameters, to be consistent across all
  nodes in the network.

- **Migration & experimentation:** Introducing new features and updating old
  features can complicate migration for existing users of the software.
  Temporary or "experimental" configuration settings can be a valuable way to
  mitigate that friction.

  For example, Tendermint v0.36 introduces a new RPC event subscription
  endpoint (see [ADR 075][adr075]) that will eventually replace the existing
  webwocket-based interface. To give users time to migrate, v0.36 adds an
  `experimental-disable-websocket` setting, defaulted to `false`, that allows
  operators to selectively disable the websocket API for testing purposes
  during the conversion. This setting is designed to be removed in v0.37, when
  the old interface is no longer supported.

- **Ongoing maintenance:** Sometimes configuration settings become obsolete,
  and the cost of removing them trades off against the potential risks of
  leaving a non-functional or deprecated knob hooked up indefinitely.

  For example, Tendermint v0.35 deprecated two alternate implementations of the
  blocksync protocol, one of which was deleted entirely (`v1`) and one of which
  was scheduled for removal (`v2`). The `blocksync.version` setting, which had
  been added as a migration aid, became obsolete and needed to be updated.

  Despite our best intentions, sometimes engineering designs do not work out.
  It's just as important to leave room to back out of changes we have since
  reconsidered, as it is to support migrations forward onto new and improved
  code.

- **Clarity and legibility:** Besides configuring the software, another
  important purpose of a config file is to document intent for the humans who
  operate and maintain the software. Operators need adjust settings to keep the
  node running, and developers need to know what options were in use when
  something goes wrong so they can diagnose and fix bugs.  The legibility of a
  config file as a _human_ artifact is also thus important.

  For example, Tendermint v0.35 moved settings related to validator private
  keys from the top-level section of the configuration file to their own
  designated `[priv-validator]` section. Although this change did not make any
  difference to the meaning of those settings, it made the organization of the
  file easier to understand, and allowed the names of the individual settings
  to be simplified (e.g., `priv-validator-key-file` became simply `key-file` in
  the new section).

  Although such changes are "gratuitous" with respect to the software, there is
  often value in making things more legible for the humans. While there is no
  simple rule to define the line, the Potter Stewart principle can be used with
  due care.

Keeping these examples in mind, we can and should take reasonable steps to
avoid churn in the configuration file across versions where we can. However, we
must also accept that part of the reason for _having_ a config file is to allow
us flexibility elsewhere in the design.  On that basis, we should not attempt
to be too dogmatic about config changes either. Unlike changes in the block
protocol, for example, which affect every user of every network that adopts
them, config changes are relatively self-contained.

There are few guiding principles I think we can use to strike a sensible
balance:

1. **No gratuitous changes.** Aesthetic changes that do not enhance legibility,
   avert confusion, or clarity documentation, should be entirely avoided.

2. **Prefer mechanical changes.** Whenever it is practical, change settings in
   a way that can be updated by a tool without operator judgement. This implies
   finding safe, universal defaults for new settings, and not changing the
   default values of existing settings.

   Even if that means we have to make multiple changes (e.g., add a new setting
   in the current version, deprecate the old one, and remove the old one in the
   next version) it's preferable if we can mechanize each step.

3. **Clearly signal intent.** When adding temporary or experimental settings,
   they should be clearly named and documented as such. Use long names and
   suggestive prefixes (e.g., `experimental-*`) so that they stand out when
   read in the config file or printed in logs.

   Relatedly, using temporary or experimental settings should cause the
   software to emit diagnostic logs at runtime. These log messages should be
   easy to grep for, and should contain pointers to more complete documentation
   (say, issue numbers or URLs) that the operator can read, as well as a hint
   about when the setting is expected to become invalid. For example:

   ```
   WARNING: Websocket RPC access is deprecated and will be removed in
   Tendermint v0.37. See https://tinyurl.com/adr075 for more information.
   ```

4. **Consider both directions.** When adding a configuration setting, take some
   time during the implementation process to think about how the setting could
   be removed, as well as how it will be rolled out. This applies even for
   settings we imagine should be permanent. Experience may cause is to rethink
   our original design intent more broadly than we expected.

   This does not mean we have to spend a long time picking nits over the design
   of every setting; merely that we should convince ourselves we _could_ undo
   it without making too big a mess later. Even a little extra effort up front
   can sometimes save a lot.

## References

- [Tendermint `config` package][config-pkg]
- [`confix` command-line tool][confix]
- [`condiff` command-line tool][condiff]
- [Configuration update plan][plan]
- [ADR 075: RPC Event Subscription Interface][adr075]

[config-pkg]: https://godoc.org/github.com/tendermint/tendermint/config
[confix]: https://github.com/tendermint/tendermint/blob/main/scripts/confix
[condiff]: https://github.com/tendermint/tendermint/blob/main/scripts/confix/condiff
[plan]: https://github.com/tendermint/tendermint/blob/main/scripts/confix/plan.go
[testdata]: https://github.com/tendermint/tendermint/blob/main/scripts/confix/testdata
[adr075]: https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-075-rpc-subscription.md

## Appendix: Research Notes

Discovering when various configuration settings were added, updated, and
removed turns out to be surprisingly tedious.  To solve this puzzle, we had to
answer the following questions:

1. What changes were made between v0.x and v0.y? This is further complicated by
   cases where we have backported config changes into the middle of an earlier
   release cycle (e.g., `psql-conn` from v0.35.x into v0.34.13).

2. When during the development cycle were those changes made? This allows us to
   recognize features that were backported into a previous release.

3. What were the default values of the changed settings, and did they change at
   all during or across the release boundary?

Each step of the [configuration update plan][plan] is commented with a link to
one or more PRs where that change was made. The sections below discuss how we
found these references.

### Tracking Changes Across Releases

To figure out what changed between two releases, we built a tool called
[`condiff`][condiff], which performs a "keyspace" diff of two TOML documents.
This diff respects the structure of the TOML file, but ignores comments, blank
lines, and configuration values, so that we can see what was added and removed.

To use it, run:

```shell
go run ./scripts/confix/condiff old.toml new.toml
```

This tool works on any TOML documents, but for our purposes we needed
Tendermint `config.toml` files. The easiest way to get these is to build the
node binary for your version of interest, run `tendermint init` on a clean home
directory, and copy the generated config file out. The [`testdata`][testdata]
directory for the `confix` tool has configs generated from the heads of each
release branch from v0.31 through v0.35.

If you want to reproduce this yourself, it looks something like this:

```shell
# Example for Tendermint v0.32.
git checkout --track origin/v0.32.x
go get golang.org/x/sys/unix
go mod tidy
make build
rm -fr -- tmhome
./build/tendermint --home=tmhome init
cp tmhome/config/config.toml config-v32.toml
```

Be advised that the further back you go, the more idiosyncrasies you will
encounter. For example, Tendermint v0.31 and earlier predate Go modules (v0.31
used dep), and lack backport branches. And you may need to do some editing of
Makefile rules once you get back into the 20s.

Note that when diffing config files across the v0.34/v0.35 gap, the swap from
`snake_case` to `kebab-case` makes it look like everything changed. The
`condiff` tool has a `-desnake` flag that normalizes all the keys to kebab case
in both inputs before comparison.

### Locating Additions and Deletions

To figure out when a configuration setting was added or removed, your tool of
choice is `git bisect`. The only tricky part is finding the endpoints for the
search.  If the transition happened within a release, you can use that
release's backport branch as the endpoint (if it has one, e.g., `v0.35.x`).

However, the start point can be more problematic. The backport branches are not
ancestors of `master` or of each other, which means you need to find some point
in history _prior_ to the change but still attached to the mainline. For recent
releases there is a dev root (e.g., `v0.35.0-dev`, `v0.34.0-dev1`, etc.). These
are not named consistently, but you can usually grep the output of `git tag` to
find them.

In the worst case you could try starting from the root commit of the repo, but
that turns out not to work in all cases. We've done some branching shenanigans
over the years that mean the root is not a direct ancestor of all our release
branches. When you find this you will probably swear a lot. I did.

Once you have a start and end point (say, `v0.35.0-dev` and `master`), you can
bisect in the usual way. I use `git grep` on the `config` directory to check
whether the case I am looking for is present. For example, to find when the
`[fastsync]` section was removed:

```shell
# Setup:
git checkout master
git bisect start
git bisect bad                 # it's not present on tip of master.
git bisect good v0.34.0-dev1   # it was present at the start of v0.34.
```

```shell
# Now repeat this until it gives you a specific commit:
if git grep -q '\[fastsync\]' config ; then git bisect good ; else git bisect bad ; fi
```

The above example finds where a config was removed: To find where a setting was
added, do the same thing except reverse the sense of the test (`if ! git grep -q
...`).
