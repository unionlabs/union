
## Note on future releases
** There is no longer an automated PR workflow for releasing the web ui due to its frozen state **
This is due to this PR removal of build content:https://github.com/prysmaticlabs/prysm/pull/12719

in order to update the `site_data.go` follow the following steps to update the specific release of https://github.com/prysmaticlabs/prysm-web-ui/releases
1. download and install https://github.com/kevinburke/go-bindata. (working as of version `4.0.2`) This tool will be used to generate the site_data.go file.
2. download the specific release from https://github.com/prysmaticlabs/prysm-web-ui/releases
3. run `go-bindata -pkg web -nometadata -modtime 0 -o site_data.go  prysm-web-ui/` . `prysm-web-ui/` represents the extracted folder from the release.
4. copy and replace the site_data.go in this package.
5. Open a PR