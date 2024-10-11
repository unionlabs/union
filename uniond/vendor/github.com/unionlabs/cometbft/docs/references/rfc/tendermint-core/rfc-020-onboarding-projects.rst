=======================================
RFC 020: Tendermint Onboarding Projects
=======================================

.. contents::
   :backlinks: none

Changelog
---------

- 2022-03-30: Initial draft. (@tychoish)
- 2022-04-25: Imported document to tendermint repository. (@tychoish)

Overview
--------

This document describes a collection of projects that might be good for new
engineers joining the Tendermint Core team. These projects mostly describe
features that we'd be very excited to see land in the code base, but that are
intentionally outside of the critical path of a release on the roadmap, and
have the following properties that we think make good on-boarding projects:

- require relatively little context for the project or its history beyond a
  more isolated area of the code.

- provide exposure to different areas of the codebase, so new team members
  will have reason to explore the code base, build relationships with people
  on the team, and gain experience with more than one area of the system.

- be of moderate size, striking a healthy balance between trivial or
  mechanical changes (which provide little insight) and large intractable
  changes that require deeper insight than is available during onboarding to
  address well. A good size project should have natural touchpoints or
  check-ins.

Projects
--------

Before diving into one of these projects, have a conversation about the
project or aspects of Tendermint that you're excited to work on with your
onboarding buddy. This will help make sure that these issues are still
relevant, help you get any context, underatnding known pitfalls, and to
confirm a high level approach or design (if relevant.) On-boarding buddies
should be prepared to do some design work before someone joins the team.

The descriptions that follow provide some basic background and attempt to
describe the user stories and the potential impact of these project.

E2E Test Systems
~~~~~~~~~~~~~~~~

Tendermint's E2E framework makes it possible to run small test networks with
different Tendermint configurations, and make sure that the system works. The
tests run Tendermint in a separate binary, and the system provides some very
high level protection against making changes that could break Tendermint in
otherwise difficult to detect ways.

Working on the E2E system is a good place to get introduced to the Tendermint
codebase, particularly for developers who are newer to Go, as the E2E
system (generator, runner, etc.) is distinct from the rest of Tendermint and
comparatively quite small, so it may be easier to begin making changes in this
area. At the same time, because the E2E system exercises *all* of Tendermint,
work in this area is a good way to get introduced to various components of the
system.

Configurable E2E Workloads
++++++++++++++++++++++++++

All E2E tests use the same workload (e.g. generated transactions, submitted to
different nodes in the network,) which has been tuned empirically to provide a
gentle but consistent parallel load that all E2E tests can pass. Ideally, the
workload generator could be configurable to have different shapes of work
(bursty, different transaction sizes, weighted to different nodes, etc.) and
even perhaps further parameterized within a basic shape, which would make it
possible to use our existing test infrastructure to answer different questions
about the performance or capability of the system.

The work would involve adding a new parameter to the E2E test manifest, and
creating an option (e.g. "legacy") for the current load generation model,
extract configurations options for the current load generation, and then
prototype implementations of alternate load generation, and also run some
preliminary using the tools.

Byzantine E2E Workloads
+++++++++++++++++++++++

There are two main kinds of integration tests in Tendermint: the E2E test
framework, and then a collection of integration tests that masquerade as
unit-tests. While some of this expansion of test scope is (potentially)
inevitable, the masquerading unit tests (e.g ``consensus.byzantine_test.go``)
end up being difficult to understand, difficult to maintain, and unreliable.

One solution to this, would be to modify the E2E ABCI application to allow it
to inject byzantine behavior, and then have this be a configurable aspect of
a test network to be able to provoke Byzantine behavior in a "real" system and
then observe that evidence is constructed. This would make it possible to
remove the legacy tests entirely once the new tests have proven themselves.

Abstract Orchestration Framework
++++++++++++++++++++++++++++++++

The orchestration of e2e test processes is presently done using docker
compose, which works well, but has proven a bit limiting as all processes need
to run on a single machine, and the log aggregation functions are confusing at
best.

This project would replace the current orchestration with something more
generic, potentially maintaining the current system, but also allowing the e2e
tests to manage processes using k8s. There are a few "local" k8s frameworks
(e.g. kind and k3s,) which might be able to be useful for our current testing
model, but hopefully, we could use this new implementation with other k8s
systems for more flexible distribute test orchestration.

Improve Operationalize Experience of ``run-multiple.sh``
++++++++++++++++++++++++++++++++++++++++++++++++++++++++

The e2e test runner currently runs a single test, and in most cases we manage
the test cases using a shell script that ensure cleanup of entire test
suites. This is a bit difficult to maintain and makes reproduction of test
cases more awkward than it should be. The e2e ``runner`` itself should provide
equivalent functionality to ``run-multiple.sh``: ensure cleanup of test cases,
collect and process output, and be able to manage entire suites of cases.

It might also be useful to implement an e2e test orchestrator that runs all
tendermint instances in a single process, using "real" networks for faster
feedback and iteration during development.

In addition to being a bit easier to maintain, having a more capable runner
implementation would make it easier to collect data from test runs, improve
debugability and reporting.

Fan-Out For CI E2E Tests
++++++++++++++++++++++++

While there are some parallelism in the execution of e2e tests, each e2e test
job must build a tendermint e2e image, which takes about 5 minutes of CPU time
per-task, which given the size of each of the runs.

We'd like to be able to reduce the amount of overhead per-e2e tests while
keeping the cycle time for working with the tests very low, while also
maintaining a reasonable level of test coverage.  This is an impossible
tradeoff, in some ways, and the percentage of overhead at the moment is large
enough that we can make some material progress with a moderate amount of time.

Most of this work has to do with modifying github actions configuration and
e2e artifact (docker) building to reduce redundant work. Eventually, when we
can drop the requirement for CGo storage engines, it will be possible to move
(cross) compile tendermint locally, and then inject the binary into the docker
container, which would reduce a lot of the build-time complexity, although we
can move more in this direction or have runtime flags to disable CGo
dependencies for local development.

Remove Panics
~~~~~~~~~~~~~

There are lots of places in the code base which can panic, and would not be
particularly well handled. While in some cases, panics are the right answer,
in many cases the panics were just added to simplify downstream error
checking, and could easily be converted to errors.

The `Don't Panic RFC
<./rfc-008-do-not-panic.md>`_
covers some of the background and approach.

While the changes are in this project are relatively rote, this will provide
exposure to lots of different areas of the codebase as well as insight into
how different areas of the codebase interact with each other, as well as
experience with the test suites and infrastructure.

Implement more Expressive ABCI Applications
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Tendermint maintains two very simple ABCI applications (a KV application used
for basic testing, and slightly more advanced test application used in the
end-to-end tests). Writing an application would provide a new engineer with
useful experiences using Tendermint that mirrors the experience of downstream
users.

This is more of an exploratory project, but could include providing common
interfaces on top of Tendermint consensus for other well known protocols or
tools (e.g. ``etcd``) or a DNS server or some other tool.

Self-Regulating Reactors
~~~~~~~~~~~~~~~~~~~~~~~~

Currently reactors (the internal processes that are responsible for the higher
level behavior of Tendermint) can be started and stopped, but have no
provision for being paused. These additional semantics may allow Tendermint to
pause reactors (and avoid processing their messhages, etc.) and allow better
coordination in the future.

While this is a big project, it's possible to break this apart into many
smaller projects: make p2p channels pauseable, add pause/UN-pause hooks to the
service implementation and machinery, and finally to modify the reactor
implementations to take advantage of these additional semantics

This project would give an engineer some exposure to the p2p layer of the
code, as well as to various aspects of the reactor implementations.

Metrics
~~~~~~~

Tendermint has a metrics system that is relatively underutilized, and figuring
out ways to capture and organize the metrics to provide value to users might
provide an interesting set of projects for new engineers on Tendermint.

Convert Logs to Metrics
+++++++++++++++++++++++

Because the tendermint logs tend to be quite verbose and not particularly
actionable, most users largely ignore the logging or run at very low
verbosity. While the log statements in the code do describe useful events,
taken as a whole the system is not particularly tractable, and particularly at
the Debug level, not useful. One solution to this problem is to identify log
messages that might be (e.g. increment a counter for certain kinds of errors)

One approach might be to look at various logging statements, particularly
debug statements or errors that are logged but not returned, and see if
they're convertible to counters or other metrics.

Expose Metrics to Tests
+++++++++++++++++++++++

The existing Tendermint test suites replace the metrics infrastructure with
no-op implementations, which means that tests can neither verify that metrics
are ever recorded, nor can tests use metrics to observe events in the
system. Writing an implementation, for testing, that makes it possible to
record metrics and provides an API for introspecting this data, as well as
potentially writing tests that take advantage of this type, could be useful.

Logging Metrics
+++++++++++++++

In some systems, the logging system itself can provide some interesting
insights for operators: having metrics that track the number of messages at
different levels as well as the total number of messages, can act as a canary
for the system as a whole.

This should be achievable by adding an interceptor layer within the logging
package itself that can add metrics to the existing system.
