# RFC Creation Process

1. Copy the `rfc-template.md` file. Use the following filename pattern: `rfc-next_number-title.md`
2. Create a draft Pull Request if you want to get early feedback.
3. Make sure the context and solution are clear and well-documented.
4. Add an entry to the list in the [README](./README.md) file.
5. Create a Pull Request to propose a new RFC.

## What is an RFC?

An RFC is a sort of async whiteboarding session. It is meant to replace the need for a distributed team to come together to decide. Currently, the Cosmos SDK team and contributors are distributed around the world. The team conducts working groups to have a synchronous discussion, and an RFC can be used to capture the discussion for a wider audience to better understand the changes that are coming to the software.

The main difference the Cosmos SDK defines between RFCs and ADRs is that an RFC is to come to consensus and circulate information about a potential change or feature. An ADR is used if there is already consensus on a feature or change and there is no need to articulate the change coming to the software. An ADR will articulate the changes and require less communication.

## RFC life cycle

RFC creation is an **iterative** process. An RFC is meant as a distributed collaboration session, it may have many comments and is usually the by-product of no working group or synchronous communication.

1. Proposals could start with a new GitHub Issue, be a result of existing Issues or a discussion.

2. An RFC doesn't have to arrive to `main` with an _accepted_ status in a single PR. If the motivation is clear and the solution is sound, we SHOULD be able to merge it and keep a _proposed_ status. It's preferable to have an iterative approach rather than long, not merged Pull Requests.

3. If a _proposed_ RFC is merged, then it should clearly document outstanding issues either in the RFC document notes or in a GitHub Issue.

4. The PR SHOULD always be merged. In the case of a faulty RFC, we still prefer to merge it with a _rejected_ status. The only time the RFC SHOULD NOT be merged is if the author abandons it.

5. Merged RFCs SHOULD NOT be pruned.

6. If there is consensus and enough feedback then the RFC can be accepted.

> Note: An RFC is written when there is no working group or team session on the problem. RFCs are meant as a distributed whiteboarding session. If there is a working group on the proposal, there is no need to have an RFC as there is synchronous whiteboarding going on.

### RFC status

Status has two components:

```text
{CONSENSUS STATUS}
```

#### Consensus Status

```text
DRAFT -> PROPOSED -> LAST CALL yyyy-mm-dd -> ACCEPTED | REJECTED -> SUPERSEDED by RFC-xxx
                  \        |
                   \       |
                    v      v
                     ABANDONED
```

* `DRAFT`: [optional] an RFC which is work in progress, not being ready for a general review. This is to present early work and get early feedback in a Draft Pull Request form.
* `PROPOSED`: an RFC covering a full solution architecture and still in review - project stakeholders haven't reached an agreement yet.
* `LAST CALL <date for the last call>`: [optional] clearly notify that we are close to accepting updates. Changing a status to `LAST CALL` means that social consensus (of Cosmos SDK maintainers) has been reached, and we still want to give it time to let the community react or analyze.
* `ACCEPTED`: RFC which will represent a currently implemented or to be implemented architecture design.
* `REJECTED`: RFC can go from PROPOSED or ACCEPTED to rejected if the consensus among project stakeholders decides so.
* `SUPERSEDED by RFC-xxx`: RFC which has been superseded by a new RFC.
* `ABANDONED`: the RFC is no longer pursued by the original authors.

## Language used in RFC

* The background/goal should be written in the present tense.
* Avoid using a first-person form.
