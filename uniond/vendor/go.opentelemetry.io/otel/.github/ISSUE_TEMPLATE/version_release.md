---
name: Version Release
about: Checklist to follow when shipping a new release.
title: 'Release <V1.x.x> Checklist'
labels: ''
assignees: ''

---

<!-- markdownlint-disable MD034 -->
<!--- The current milestones can be found at https://github.com/open-telemetry/opentelemetry-go/milestones -->
- [ ] Complete [Milestone](https://github.com/open-telemetry/opentelemetry-go/milestone/<Release Milesone>)
<!-- markdownlint-enable MD034 -->
- [ ] [Update contrib codebase to support changes about to be released (use a git sha version)](https://github.com/open-telemetry/opentelemetry-go/blob/main/RELEASING.md#verify-changes-for-contrib-repository)
- [ ] [Pre-release](https://github.com/open-telemetry/opentelemetry-go/blob/main/RELEASING.md#pre-release)
- [ ] [Tag](https://github.com/open-telemetry/opentelemetry-go/blob/main/RELEASING.md#tag)
- [ ] [Release](https://github.com/open-telemetry/opentelemetry-go/blob/main/RELEASING.md#release)
- [ ] [Check examples](https://github.com/open-telemetry/opentelemetry-go/blob/main/RELEASING.md#verify-examples)
- [ ] [Sync with Contrib](https://github.com/open-telemetry/opentelemetry-go-contrib/blob/main/RELEASING.md#upgrade-goopentelemetryiootel-packages)
- [ ] [Release contrib](https://github.com/open-telemetry/opentelemetry-go-contrib/blob/main/RELEASING.md#release-process)
- [ ] [Sync website docs](https://github.com/open-telemetry/opentelemetry-go/blob/main/RELEASING.md#website-documentation)
- [ ] Close the milestone
