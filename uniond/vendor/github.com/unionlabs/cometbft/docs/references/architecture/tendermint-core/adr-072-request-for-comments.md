# ADR 72: Restore Requests for Comments

## Changelog

- 20-Aug-2021: Initial draft (@creachadair)

## Status

Implemented

## Context

In the past, we kept a collection of Request for Comments (RFC) documents in `docs/rfc`.
Prior to the creation of the ADR process, these documents were used to document
design and implementation decisions about Tendermint Core. The RFC directory
was removed in favor of ADRs, in commit 3761aa69 (PR
[\#6345](https://github.com/tendermint/tendermint/pull/6345)).

For issues where an explicit design decision or implementation change is
required, an ADR is generally preferable to an open-ended RFC: An ADR is
relatively narrowly-focused, identifies a specific design or implementation
question, and documents the consensus answer to that question.

Some discussions are more open-ended, however, or don't require a specific
decision to be made (yet). Such conversations are still valuable to document,
and several members of the Tendermint team have been doing so by writing gists
or Google docs to share them around. That works well enough in the moment, but
gists do not support any kind of collaborative editing, and both gists and docs
are hard to discover after the fact. Google docs have much better collaborative
editing, but are worse for discoverability, especially when contributors span
different Google accounts.

Discoverability is important, because these kinds of open-ended discussions are
useful to people who come later -- either as new team members or as outside
contributors seeking to use and understand the thoughts behind our designs and
the architectural decisions that arose from those discussion.

With these in mind, I propose that:

-  We re-create a new, initially empty `docs/rfc` directory in the repository,
   and use it to capture these kinds of open-ended discussions in supplement to
   ADRs.

-  Unlike in the previous RFC scheme, documents in this new directory will
   _not_ be used directly for decision-making. This is the key difference
   between an RFC and an ADR.

   Instead, an RFC will exist to document background, articulate general
   principles, and serve as a historical record of discussion and motivation.

   In this system, an RFC may _only_ result in a decision indirectly, via ADR
   documents created in response to the RFC.

   **In short:** If a decision is required, write an ADR; otherwise if a
   sufficiently broad discussion is needed, write an RFC.

Just so that there is a consistent format, I also propose that:

-  RFC files are named `rfc-XXX-title.{md,rst,txt}` and are written in plain
   text, Markdown, or ReStructured Text.

-  Like an ADR, an RFC should include a high-level change log at the top of the
   document, and sections for:

     * Abstract: A brief, high-level synopsis of the topic.
     * Background: Any background necessary to understand the topic.
     * Discussion: Detailed discussion of the issue being considered.

-  Unlike an ADR, an RFC does _not_ include sections for Decisions, Detailed
   Design, or evaluation of proposed solutions. If an RFC leads to a proposal
   for an actual architectural change, that must be recorded in an ADR in the
   usual way, and may refer back to the RFC in its References section.

## Alternative Approaches

Leaving aside implementation details, the main alternative to this proposal is
to leave things as they are now, with ADRs as the only log of record and other
discussions being held informally in whatever medium is convenient at the time.

## Decision

(pending)

## Detailed Design

- Create a new `docs/rfc` directory in the `tendermint` repository. Note that
  this proposal intentionally does _not_ pull back the previous contents of
  that path from Git history, as those documents were appropriately merged into
  the ADR process.

- Create a `README.md` for RFCs that explains the rules and their relationship
  to ADRs.

- Create an `rfc-template.md` file for RFC files.

## Consequences

### Positive

- We will have a more discoverable place to record open-ended discussions that
  do not immediately result in a design change.

### Negative

- Potentially some people could be confused about the RFC/ADR distinction.
