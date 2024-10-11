# Google APIs Client Library for Go

## Getting Started

```shell
$ go get google.golang.org/api/tasks/v1
$ go get google.golang.org/api/moderator/v1
$ go get google.golang.org/api/urlshortener/v1
... etc ...
```

and using:

```go
package main

import (
        "context"
        "net/http"

        "google.golang.org/api/urlshortener/v1"
)

func main() {
        ctx := context.Background()
        svc, err := urlshortener.NewService(ctx)
        // ...
}
```

* For a longer tutorial, see the [Getting Started guide](https://github.com/google/google-api-go-client/blob/main/GettingStarted.md).
* For examples, see the [examples directory](https://github.com/google/google-api-go-client/tree/main/examples).
* For support, use the [golang-nuts](https://groups.google.com/group/golang-nuts) mailing list.

## Status

[![Go Reference](https://pkg.go.dev/badge/google.golang.org/api.svg)](https://pkg.go.dev/google.golang.org/api)

These are auto-generated Go libraries from the Google Discovery Service's JSON description files.

Due to the auto-generated nature of this collection of libraries they may contain breaking changes from one release to
the next. The generator itself and the code it produces are considered beta for this reason.

These client libraries are officially supported by Google.  However, the libraries are considered complete and are in
maintenance mode. This means that we will address critical bugs and security issues but will not add any new features.

If you're working with Google Cloud Platform APIs such as Datastore or Pub/Sub, please use the
[Cloud Client Libraries for Go](https://github.com/googleapis/google-cloud-go) instead. These are the new and idiomatic
Go libraries targeted specifically at Google Cloud Platform Services.

## Authorization

By default, each API will use [Google Application Default Credentials](https://developers.google.com/identity/protocols/application-default-credentials)
for authorization credentials used in calling the API endpoints. This will allow your application to run in many
environments without requiring explicit configuration.

```go
// import "google.golang.org/api/sheets/v4"
client, err := sheets.NewService(ctx)
```

To authorize using a [JSON key file](https://cloud.google.com/iam/docs/managing-service-account-keys), pass
[`option.WithCredentialsFile`](https://pkg.go.dev/google.golang.org/api/option#WithCredentialsFile) to the `NewService`
function of the desired package. For example:

```go
client, err := sheets.NewService(ctx, option.WithCredentialsFile("path/to/keyfile.json"))
```

You can exert more control over authorization by using the [`golang.org/x/oauth2`](https://pkg.go.dev/golang.org/x/oauth2)
package to create an `oauth2.TokenSource`. Then pass [`option.WithTokenSource`](https://pkg.go.dev/google.golang.org/api/option#WithTokenSource)
to the `NewService` function:

```go
tokenSource := ...
svc, err := sheets.NewService(ctx, option.WithTokenSource(tokenSource))
```

## More information

For some more information related to all of the generated clients please read through our
[package documentation](https://pkg.go.dev/google.golang.org/api#section-documentation).
