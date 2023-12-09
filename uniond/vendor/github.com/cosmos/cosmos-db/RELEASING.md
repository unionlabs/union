# How To Release cosmos-db

This document provides a step-by-step guide for creating a release of cosmos-db.

## 1. Update the changelog

Open the `CHANGELOG.md` at the root of the repository. 
Amend the top of this file with a section for the latest version (eg: v1.0.x).
Be sure to include any bug fixes, improvements, dependency upgrades, and breaking changes included in this version. 
(It's OK to exclude changes to tooling dependencies, like updates to Github Actions.)
Finally, create a pull request for the changelog update.
Once the tests pass and the pull request is approved, merge the change into master.

## 2. Tag the latest commit with the latest version

cosmos-db is provided as a golang [module](https://blog.golang.org/publishing-go-modules), which rely on git tags for versioning information. 

Tag the changelog commit in master created in step 1 with the latest version.
Be sure to prefix the version tag with `v`. For example, `v1.0.0` for version 1.0.0.
This tagging can be done [using github](https://docs.github.com/en/desktop/contributing-and-collaborating-using-github-desktop/managing-commits/managing-tags#creating-a-tag) or [using git](https://git-scm.com/book/en/v2/Git-Basics-Tagging) on the command line. 

Note that the golang modules tooling expects tags to be immutable. 
If you make a mistake after pushing a tag, make a new tag and start over rather than fix and re-push the old tag.
## 3. Create a github release

Finally, create a github release.
To create a github release, follow the steps in the [github release documentation](https://docs.github.com/en/github/administering-a-repository/releasing-projects-on-github/managing-releases-in-a-repository#creating-a-release).

When creating the github release, select the `Tag version` created in step 2. 
Use the version tag as the release title and paste in the changelog information for this release in the `Describe this release` section.
