# RFC 023: Semi-permanent Testnet

## Changelog

- 2022-07-28: Initial draft (@mark-rushakoff)
- 2022-07-29: Renumber to 023, minor clarifications (@mark-rushakoff)

## Abstract

This RFC discusses a long-lived testnet, owned and operated by the Tendermint engineers.
By owning and operating a production-like testnet,
the team who develops Tendermint becomes more capable of discovering bugs that
only arise in production-like environments.
They also build expertise in operating Tendermint;
this will help guide the development of Tendermint towards operator-friendly design.

The RFC details a rough roadmap towards a semi-permanent testnet, some of the considered tradeoffs,
and the expected outcomes from following this roadmap.

## Background

The author's understanding -- which is limited as a new contributor to the Tendermint project --
is that Tendermint development has been largely treated as a library for other projects to consume.
Of course effort has been spent on unit tests, end-to-end tests, and integration tests.
But whether developing a library or an application,
there is no substitute for putting the software under a production-like load.

First, there are classes of bugs that are unrealistic to discover in environments
that do not resemble production.
But perhaps more importantly, there are "operational features" that are best designed
by the authors of a given piece of software.
For instance, does the software have sufficient observability built-in?
Are the reported metrics useful?
Are the log messages clear and sufficiently detailed, without being too noisy?

Furthermore, if the library authors are not only building --
but also maintaining and operating -- an application built on top of their library,
the authors will have a greatly increased confidence that their library's API
is appropriate for other application authors.

Once the decision has been made to run and operate a service,
one of the next strategic questions is that of deploying said service.
The author strongly holds the opinion that, when possible,
a continuous delivery model offers the most compelling set of advantages:

- The code on a particular branch (likely `main` or `master`) is exactly what is,
  or what will very soon be, running in production
- There are no manual steps involved in deploying -- other than merging your pull request,
  which you had to do anyway
- A bug discovered in production can be rapidly confirmed as fixed in production

In summary, if the tendermint authors build, maintain, and continuously deliver an application
intended to serve as a long-lived testnet, they will be able to state with confidence:

- We operate the software in a production-like environment and we have observed it to be
  stable and performant to our requirements
- We have discovered issues in production before any external parties have consumed our software,
  and we have addressed said issues
- We have successfully used the observability tooling built into our software
  (perhaps in conjunction with other off-the-shelf tooling)
  to diagnose and debug issues in production

## Discussion

The Discussion Section proposes a variety of aspects of maintaining a testnet for Tendermint.

### Number of testnets

There should probably be one testnet per maintained branch of Tendermint,
i.e. one for the `main` branch
and one per `v0.N.x` branch that the authors maintain.

There may also exist testnets for long-lived feature branches.

We may eventually discover that there is good reason to run more than one testnet for a branch,
perhaps due to a significant configuration variation.

### Testnet lifecycle

The document has used the terms "long-lived" and "semi-permanent" somewhat interchangeably.
The intent of the testnet being discussed in this RFC is to exist indefinitely;
but there is a practical understanding that there will be testnet instances
which will be retired due to a variety of reasons.
For instance, once a release branch is no longer supported,
its corresponding testnet should be torn down.

In general, new commits to branches with corresponding testnets
should result in an in-place upgrade of all nodes in the testnet
without any data loss and without requiring new configuration.
The mechanism for achieving this is outside the scope of this RFC.

However, it is also expected that there will be
breaking changes during the development of the `main` branch.
For instance, suppose there is an unreleased feature involving storage on disk,
and the developers need to change the storage format.
It should be at the developers' discretion whether it is feasible and worthwhile
to introduce an intermediate commit that translates the old format to the new format,
or if it would be preferable to just destroy the testnet and start from scratch
without any data in the old format.

Similarly, if a developer inadvertently pushed a breaking change to an unreleased feature,
they are free to make a judgement call between reverting the change,
adding a commit to allow a forward migration,
or simply forcing the testnet to recreate.

### Testnet maintenance investment

While there is certainly engineering effort required to build the tooling and infrastructure
to get the testnets up and running,
the intent is that a running testnet requires no manual upkeep under normal conditions.

It is expected that a subset of the Tendermint engineers are familiar with and engaged in
writing the software to maintain and build the testnet infrastructure,
but the rest of the team should not need any involvement in authoring that code.

The testnets should be configured to send notifications for events requiring triage,
such as a chain halt or a node OOMing.
The time investment necessary to address the underlying issues for those kind of events
is unpredictable.

Aside from triaging exceptional events, an engineer may choose to spend some time
collecting metrics or profiles from testnet nodes to check performance details
before and after a particular change;
or they may inspect logs associated with an expected behavior change.
But during day-to-day work, engineers are not expected to spend any considerable time
directly interacting with the testnets.

If we discover that there are any routine actions engineers must take against the testnet
that take any substantial focused time,
those actions should be automated to a one-line command as much as is reasonable.

### Testnet MVP

The minimum viable testnet meets this set of features:

- The testnet self-updates following a new commit pushed to Tendermint's `main` branch on GitHub
  (there are some omitted steps here, such as CI building appropriate binaries and
  somehow notifying the testnet that a new build is available)
- The testnet runs the Tendermint KV store for MVP
- The testnet operators are notified if:
    - Any node's process exits for any reason other than a restart for a new binary
    - Any node stops updating blocks, and by extension if a chain halt occurs
    - No other observability will be considered for MVP
- The testnet has a minimum of 1 full node and 3 validators
- The testnet has a reasonably low, constant throughput of transactions -- say 30 tx/min --
  and the testnet operators are notified if that throughput drops below 75% of target
  sustained over 5 minutes
- The testnet only needs to run in a single datacenter/cloud-region for MVP,
  i.e. running in multiple datacenters is out of scope for MVP
- The testnet is running directly on VMs or compute instances;
  while Kubernetes or other orchestration frameworks may offer many significant advantages,
  the Tendermint engineers should not be required to learn those tools in order to
  perform basic debugging

### Testnet medium-term goals

The medium-term goals are intended to be achievable within the 6-12 month time range
following the launch of MVP.
These goals could realistically be roadmapped following the launch of the MVP testnet.

- The `main` testnet has more than 20 nodes (completely arbitrary -- 5x more than 1+3 at MVP)
- In addition to the `main` testnet,
  there is at least one testnet associated with one release branch
- The testnet no longer is simply running the Tendermint KV store;
  now it is built on a more complex, custom application
  that deliberately exercises a greater portion of the Tendermint stack
- Each testnet is spread across at least two cloud providers,
  in order to communicate over a network more closely resembling use of Tendermint in "real" chains
- The node updates have some "jitter",
  with some nodes updating immediately when a new build is available,
  and others delaying up to perhaps 30-60 minutes
- The team has published some form of dashboards that have served well for debugging,
  which external parties can copy/modify to their needs
    - The dashboards must include metrics published by Tendermint nodes;
      there should be both OS- or runtime-level metrics such as memory in use,
      and application-level metrics related to the underlying blockchain
    - "Published" in this context is more in the spirit of "shared with the community",
      not "produced a supported open source tool" --
      this could be published to GitHub with a warning that no support is offered,
      or it could simply be a blog post detailing what has worked for the Tendermint developers
    - The dashboards will likely be implemented on free and open source tooling,
      but that is not a hard requirement if paid software is more appropriate
- The team has produced a reference model of a log aggregation stack that external parties can use
    - Similar to the "published" dashboards, this only needs to be "shared" rather than "supported"
- Chaos engineering has begun being integrated into the testnets
  (this could be periodic CPU limiting or deliberate network interference, etc.
  but it probably would not be filesystem corruption)
- Each testnet has at least one node running a build with the Go race detector enabled
- The testnet contains some kind of generalized notification system built in:
    - Tendermint code grows "watchdog" systems built in to validate things like
      subsystems have not deadlocked; e.g. if the watchdog can't acquire and immediately release
      a particular mutex once in every 5-minute period, it is near certain that the target
      subsystem has deadlocked, and an alert must be sent to the engineering team.
      (Outside of the testnet, the watchdogs could be disabled, or they could panic on failure.)
    - The notification system does some deduplication to minimize spam on system failure

### Testnet long-term vision

The long-term vision includes goals that are not necessary for short- or medium-term success,
but which would support building an increasingly stable and performant product.
These goals would generally be beyond the one-year plan,
and therefore they would not be part of initial planning.

- There is a centralized dashboard to get a quick overview of all testnets,
  or at least one centralized dashboard per testnet,
  showing TBD basic information
- Testnets include cloud spot instances which periodically and abruptly join and leave the network
- The testnets are a heterogeneous mixture of straight VMs and Docker containers,
  thereby more closely representing production blockchains
- Testnets have some manner of continuous profiling,
  so that we can produce an apples-to-apples comparison of CPU/memory cost of particular operations

### Testnet non-goals

There are some things we are explicitly not trying to achieve with long-lived testnets:

- The Tendermint engineers will NOT be responsible for the testnets' availability
  outside of working hours; there will not be any kind of on-call schedule
- As a result of the 8x5 support noted in the previous point,
  there will be NO guarantee of uptime or availability for any testnet
- The testnets will NOT be used to gate pull requests;
  that responsibility belongs to unit tests, end-to-end tests, and integration tests
- Similarly, the testnet will NOT be used to automate any changes back into Tendermint source code;
  we will not automatically create a revert commit due to a failed rollout, for instance
- The testnets are NOT intended to have participation from machines outside of the
  Tendermint engineering team's control, as the Tendermint engineers are expected
  to have full access to any instance where they may need to debug an issue
- While there will certainly be individuals within the Tendermint engineering team
  who will continue to build out their individual "devops" skills to produce
  the infrastructure for the testnet, it is NOT a goal that every Tendermint engineer
  is even _familiar_ with the tech stack involved, whether it is Ansible, Terraform,
  Kubernetes, etc.
  As a rule of thumb, all engineers should be able to get shell access on any given instance
  and should have access to the instance's logs.
  Little if any further operational skills will be expected.
- The testnets are not intended to be _created_ for one-off experiments.
  While there is nothing wrong with an engineer directly interacting with a testnet
  to try something out,
  a testnet comes with a considerable amount of "baggage", so end-to-end or integration tests
  are closer to the intent for "trying something to see what happens".
  Direct interaction should be limited to standard blockchain operations,
  _not_ modifying configuration of nodes.
- Likewise, the purpose of the testnet is not to run specific "tests" per se,
  but rather to demonstrate that Tendermint blockchains as a whole are stable
  under a production load.
  Of course we will inject faults periodically, but the intent is to observe and prove that
  the testnet is resilient to those faults.
  It would be the responsibility of a lower-level test to demonstrate e.g.
  that the network continues when a single validator disappears without warning.
- The testnet descriptions in this document are scoped only to building directly on Tendermint;
  integrating with the Cosmos SDK, or any other third-party library, is out of scope

### Team outcomes as a result of maintaining and operating a testnet

Finally, this section reiterates what team growth we expect by running semi-permanent testnets.

- Confidence that Tendermint is stable under a particular production-like load
- Familiarity with typical production behavior of Tendermint, e.g. what the logs look like,
  what the memory footprint looks like, and what kind of throughput is reasonable
  for a network of a particular size
- Comfort and familiarity in manually inspecting a misbehaving or failing node
- Confidence that Tendermint ships sufficient tooling for external users
  to operate their nodes
- Confidence that Tendermint exposes useful metrics, and comfort interpreting those metrics
- Produce useful reference documentation that gives operators confidence to run Tendermint nodes
