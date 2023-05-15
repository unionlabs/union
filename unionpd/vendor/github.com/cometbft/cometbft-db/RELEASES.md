# Releases

This document provides a step-by-step guide for creating a release of CometBFT
DB.

1. Create a local branch `release/vX.X.X`, where `vX.X.X` corresponds to the
   version of the release you want to cut.
2. Update and build the changelog on your local release branch.
3. Submit a pull request from your release branch, targeting the `main` branch.
4. Once approved and merged, tag the commit associated with the merged release
   branch.
5. Create a [GitHub release] from the new tag, and include a link from the
   description to the heading associated with the new version in the changelog.

[GitHub release]: https://docs.github.com/en/github/administering-a-repository/releasing-projects-on-github/managing-releases-in-a-repository#creating-a-release
