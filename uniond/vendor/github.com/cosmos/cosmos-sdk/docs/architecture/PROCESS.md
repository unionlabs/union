# ADR Creation Process

1. Copy the `adr-template.md` file. Use the following filename pattern: `adr-next_number-title.md`
2. Create a draft Pull Request if you want to get an early feedback.
3. Make sure the context and a solution is clear and well documented.
4. Add an entry to a list in the [README](./README.md) file.
5. Create a Pull Request to propose a new ADR.

## ADR life cycle

ADR creation is an **iterative** process. Instead of trying to solve all decisions in a single ADR pull request, we MUST firstly understand the problem and collect feedback through a GitHub Issue.

1. Every proposal SHOULD start with a new GitHub Issue or be a result of existing Issues. The Issue should contain just a brief proposal summary.

2. Once the motivation is validated, a GitHub Pull Request (PR) is created with a new document based on the `adr-template.md`.

3. An ADR doesn't have to arrive to `main` with an _accepted_ status in a single PR. If the motivation is clear and the solution is sound, we SHOULD be able to merge it and keep a _proposed_ status. It's preferable to have an iterative approach rather than long, not merged Pull Requests.

4. If a _proposed_ ADR is merged, then it should clearly document outstanding issues either in ADR document notes or in a GitHub Issue.

5. The PR SHOULD always be merged. In the case of a faulty ADR, we still prefer to  merge it with a _rejected_ status. The only time the ADR SHOULD NOT be merged is if the author abandons it.

6. Merged ADRs SHOULD NOT be pruned.

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

* `DRAFT`: [optional] an ADR which is work in progress, not being ready for a general review. This is to present an early work and get an early feedback in a Draft Pull Request form.
* `PROPOSED`: an ADR covering a full solution architecture and still in the review - project stakeholders haven't reached an agreed yet.
* `LAST CALL <date for the last call>`: [optional] clear notify that we are close to accept updates. Changing a status to `LAST CALL` means that social consensus (of Cosmos SDK maintainers) has been reached and we still want to give it a time to let the community react or analyze.
* `ACCEPTED`: ADR which will represent a currently implemented or to be implemented architecture design.
* `REJECTED`: ADR can go from PROPOSED or ACCEPTED to rejected if the consensus among project stakeholders will decide so.
* `SUPERSEEDED by ADR-xxx`: ADR which has been superseded by a new ADR.
* `ABANDONED`: the ADR is no longer pursued by the original authors.

## Language used in ADR

* The context/background should be written in the present tense.
* Avoid using a first, personal form.
