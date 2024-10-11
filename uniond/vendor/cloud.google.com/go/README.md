# Google Cloud Client Libraries for Go

[![Go Reference](https://pkg.go.dev/badge/cloud.google.com/go.svg)](https://pkg.go.dev/cloud.google.com/go)

Go packages for [Google Cloud Platform](https://cloud.google.com) services.

``` go
import "cloud.google.com/go"
```

To install the packages on your system, *do not clone the repo*. Instead:

1. Change to your project directory: `cd /my/cloud/project`
1. Get the package you want to use. Some products have their own module, so it's
   best to `go get` the package(s) you want to use:

```bash
go get cloud.google.com/go/firestore # Replace with the package you want to use.
```

**NOTE:** Some of these packages are under development, and may occasionally
make backwards-incompatible changes.

## Supported APIs

For an updated list of all of our released APIs please see our
[reference docs](https://cloud.google.com/go/docs/reference).

## [Go Versions Supported](#supported-versions)

Our libraries are compatible with at least the three most recent, major Go
releases. They are currently compatible with:

- Go 1.22
- Go 1.21
- Go 1.20

## Authorization

By default, each API will use [Google Application Default Credentials](https://developers.google.com/identity/protocols/application-default-credentials)
for authorization credentials used in calling the API endpoints. This will allow your
application to run in many environments without requiring explicit configuration.

```go
client, err := storage.NewClient(ctx)
```

To authorize using a
[JSON key file](https://cloud.google.com/iam/docs/managing-service-account-keys),
pass
[`option.WithCredentialsFile`](https://pkg.go.dev/google.golang.org/api/option#WithCredentialsFile)
to the `NewClient` function of the desired package. For example:

```go
client, err := storage.NewClient(ctx, option.WithCredentialsFile("path/to/keyfile.json"))
```

You can exert more control over authorization by using the
[`golang.org/x/oauth2`](https://pkg.go.dev/golang.org/x/oauth2) package to
create an `oauth2.TokenSource`. Then pass
[`option.WithTokenSource`](https://pkg.go.dev/google.golang.org/api/option#WithTokenSource)
to the `NewClient` function:

```go
tokenSource := ...
client, err := storage.NewClient(ctx, option.WithTokenSource(tokenSource))
```

## Contributing

Contributions are welcome. Please, see the
[CONTRIBUTING](https://github.com/GoogleCloudPlatform/google-cloud-go/blob/main/CONTRIBUTING.md)
document for details.

Please note that this project is released with a Contributor Code of Conduct.
By participating in this project you agree to abide by its terms.
See [Contributor Code of Conduct](https://github.com/GoogleCloudPlatform/google-cloud-go/blob/main/CONTRIBUTING.md#contributor-code-of-conduct)
for more information.

## Links

- [Go on Google Cloud](https://cloud.google.com/go/home)
- [Getting started with Go on Google Cloud](https://cloud.google.com/go/getting-started)
- [App Engine Quickstart](https://cloud.google.com/appengine/docs/standard/go/quickstart)
- [Cloud Functions Quickstart](https://cloud.google.com/functions/docs/quickstart-go)
- [Cloud Run Quickstart](https://cloud.google.com/run/docs/quickstarts/build-and-deploy#go)
