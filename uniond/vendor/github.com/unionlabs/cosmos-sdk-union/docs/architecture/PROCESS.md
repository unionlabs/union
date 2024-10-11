# ADR Creation Process

1. Copy the `adr-template.md` file. Use the following filename pattern: `adr-next_number-title.md`
2. Create a draft Pull Request if you want to get early feedback.
3. Make sure the context and solution are clear and well documented.
4. Add an entry to a list in the [README](./README.md) file.
5. Create a Pull Request to propose a new ADR.

## What is an ADR? 

An ADR is a document to document an implementation and design that may or may not have been discussed in an RFC. While an RFC is meant to replace synchronous communication in a distributed environment, an ADR is meant to document an already made decision. An ADR won't come with much of a communication overhead because the discussion was recorded in an RFC or a synchronous discussion. If the consensus came from a synchronous discussion then a short excerpt should be added to the ADR to explain the goals. 

## ADR life cycle

ADR creation is an **iterative** process. Instead of having a high amount of communication overhead, an ADR is used when there is already a decision made and implementation details need to be added. The ADR should document what the collective consensus for the specific issue is and how to solve it. 

1. Every ADR should start with either an RFC or a discussion where consensus has been met. 

2. Once consensus is met, a GitHub Pull Request (PR) is created with a new document based on the `adr-template.md`.

3. If a _proposed_ ADR is merged, then it should clearly document outstanding issues either in ADR document notes or in a GitHub Issue.

4. The PR SHOULD always be merged. In the case of a faulty ADR, we still prefer to  merge it with a _rejected_ status. The only time the ADR SHOULD NOT be merged is if the author abandons it.

5. Merged ADRs SHOULD NOT be pruned.

### ADR status

Status has two components:

```text
{CONSENSUS STATUS} {IMPLEMENTATION STATUS}
```

IMPLEMENTATION STATUS is either `Implemented` or `Not Implemented`.

#### Consensus Status

```text
DRAFT -> PROPOSED -> LAST CALL yyyy-mm-dd -> ACCEPTED | REJECTED -> SUPERSEDED by ADR-xxx
                  \        |
                   \       |
                    v      v
                     ABANDONED
```

* `DRAFT`: [optional] an ADR which is a work in progress, not being ready for a general review. This is to present an early work and get early feedback in a Draft Pull Request form.
* `PROPOSED`: an ADR covering a full solution architecture and still in the review - project stakeholders haven't reached an agreement yet.
* `LAST CALL <date for the last call>`: [optional] Notify that we are close to accepting updates. Changing a status to `LAST CALL` means that social consensus (of Cosmos SDK maintainers) has been reached and we still want to give it a time to let the community react or analyze.
* `ACCEPTED`: ADR which will represent a currently implemented or to be implemented architecture design.
* `REJECTED`: ADR can go from PROPOSED or ACCEPTED to rejected if the consensus among project stakeholders will decide so.
* `SUPERSEDED by ADR-xxx`: ADR which has been superseded by a new ADR.
* `ABANDONED`: the ADR is no longer pursued by the original authors.

## Language used in ADR

* The context/background should be written in the present tense.
* Avoid using a first, personal form.
