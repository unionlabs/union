---
order: 1
parent:
  title: CometBFT Quality Assurance
  description: This is a report on the process followed and results obtained when running v0.34.x on testnets
  order: 7
---

# CometBFT Quality Assurance

This directory keeps track of the process followed by the CometBFT team
for Quality Assurance before cutting a release.
This directory is to live in multiple branches. On each release branch,
the contents of this directory reflect the status of the process
at the time the Quality Assurance process was applied for that release.

File [method](method.md) keeps track of the process followed to obtain the results
used to decide if a release is passing the Quality Assurance process.
The results obtained in each release are stored in their own directory.
The following releases have undergone the Quality Assurance process, and the corresponding reports include detailed information on tests and comparison with the baseline.

* [TM v0.34.x](TMCore-QA-34.md) - Tested prior to releasing Tendermint Core v0.34.22.
* [v0.34.x](CometBFT-QA-34.md) - Tested prior to releasing v0.34.27, using TM v0.34.x results as baseline.
* [TM v0.37.x](TMCore-QA-37.md) - Tested prior to releasing TM v0.37.x, using TM v0.34.x results as baseline.
* [v0.37.x](CometBFT-QA-37.md) - Tested on CometBFT v0.37.0-alpha3, using TM v0.37.x results as baseline.
* [v0.38.x](CometBFT-QA-38.md) - Tested on v0.38.0-alpha.2, using v0.37.x results as baseline.
* [v1.x](CometBFT-QA-v1.md) - Tested on v1.0.0-alpha.2, using v0.38.x results as baseline.
