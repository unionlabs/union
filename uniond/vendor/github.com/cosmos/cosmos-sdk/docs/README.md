# Updating the docs

If you want to open a PR in Cosmos SDK to update the documentation, please follow the guidelines in [`CONTRIBUTING.md`](https://github.com/cosmos/cosmos-sdk/tree/main/CONTRIBUTING.md#updating-documentation) and the [Documentation Writing Guidelines](./DOC_WRITING_GUIDELINES.md).

## Stack

The documentation for Cosmos SDK is hosted at https://docs.cosmos.network and built from the files in the `/docs` directory.
It is built using the following stack:

* [Docusaurus 2](https://docusaurus.io)
* Vuepress (pre v0.47)
* [Algolia DocSearch](https://docsearch.algolia.com/)

  ```js
      algolia: {
        appId: "QLS2QSP47E",
        apiKey: "067b84458bfa80c295e1d4f12c461911",
        indexName: "cosmos_network",
        contextualSearch: false,
      },
  ```

* GitHub Pages

## Docs Build Workflow

The docs are built and deployed automatically on GitHub Pages by a [GitHub Action workflow](../.github/workflows/deploy-docs.yml).
The workflow is triggered on every push to the `main` and `release/v**` branches, every time documentations or specs are modified.

### How It Works

There is a GitHub Action listening for changes in the `/docs` directory for the `main` branch and each supported version branch (e.g. `release/v0.46.x`). Any updates to files in the `/docs` directory will automatically trigger a website deployment. Under the hood, the private website repository has a `make build-docs` target consumed by a Github Action within that repository.

## How to Build the Docs Locally

Go to the `docs` directory and run the following commands:

```shell
cd docs
npm install
```

For starting only the current documentation, run:

```shell
npm start
```

It runs `pre.sh` scripts to get all the docs that are not already in the `docs/docs` folder.
It also runs `post.sh` scripts to clean up the docs and remove unnecessary files when quitting.

Note, the command above only build the docs for the current versions.
With the drawback that none of the redirections works. So, you'll need to go to /main to see the docs.

To build all the docs (including versioned documentation), run:

```shell
make build-docs
```

## What to for new major SDK versions

When a new major version of the SDK is released, the following steps should be taken:

* On the `release/vX.Y.Z` branch, remove the deploy action (`.github/workflows/deploy-docs.yml`), for avoiding deploying the docs from the release branches
* Each time a new version is released (on docusaurus), drop support from the oldest versions.
    * If the old version is still running vuepress (v0.45, v0.46), remove its line from `vuepress_versions`
    * If any, remove the outdated redirections from `docusaurus.config.js` and add the base version redirection (`/vX.XX`) to `/main`.

      ```js
        {
          from: ["/", "/master", "/v0.43", "/v0.44", "/v0.XX"], // here add the deprecated version
          to: "/main",
        },
      ```

* Add the new version sidebar to the list of versionned sidebar and add the version to `versions`

Learn more about [versioning](https://docusaurus.io/docs/versioning) in Docusaurus.
