# google-api-go-generator

This package is the discovery client generator for Go. It is used to generate
all of the client libraries in this repository

## Flags

- `api` (`string`): The API ID to generate, like 'tasks:v1'. A value of `*` means
  all. (default "*")
- `api_json_file` (`string`): If non-empty, the path to a local file on disk
  containing the API to generate. Exclusive with setting --api.
- `api_pkg_base` (`string`):Go package prefix to use for all generated APIs.
  (default "google.golang.org/api")
- `base_url` (`string`): (optional) Override the default service API URL. If empty,
  the service's root URL will be used.
- `build` (`bool`): Compile generated packages.
- `cache` (`bool`): Use cache of discovered Google API discovery documents.
  (default true)
- `copyright_year` (`string`): Year for copyright. (default "2024")
- `discoveryurl` (`string`): URL to root discovery document (default
  "www.googleapis.com/discovery/v1/apis")
- `gendir` (`string`): Directory to use to write out generated Go files
- `gensupport_pkg` (`string`): Go package path of the 'api/internal/gensupport'
  support package. (default "google.golang.org/api/internal/gensupport")
- `googleapi_pkg` (`string`): Go package path of the 'api/googleapi' support
  package. (default "google.golang.org/api/googleapi")
- `header_path` (`string`): If non-empty, prepend the contents of this file to
  generated services.
- `htransport_pkg` (`string`): Go package path of the 'api/transport/http' support
  package. (default "google.golang.org/api/transport/http")
- `install` (`bool`): Install generated packages.
- `internal_pkg` (`string`): Go package path of the 'internal' support package.
  (default "google.golang.org/api/internal")
- `internaloption_pkg` (`string`): Go package path of the
  'api/option/internaloption' support package. (default
  "google.golang.org/api/option/internaloption")
- `option_pkg` (`string`): Go package path of the 'api/option' support package.
  (default "google.golang.org/api/option")
- `output` (`string`): (optional) Path to source output file. If not specified,
  the API name and version are used to construct an output path (e.g. tasks/v1).
- `publiconly` (`bool`): Only build public, released APIs. Only applicable for
  Google employees. (default true)

## Automated usage

This generator runs daily via the automated bash script:
[discogen.sh](../internal/kokoro/discogen.sh).

## Example local usage

### Run same command the automation uses

`make all`

### Generate a client from a local discovery document

`go build -o google-api-go-generator && ./google-api-go-generator -cache=true -install -api_json_file=/path/to/file`

### Refresh an existing client

`go build -o google-api-go-generator && ./google-api-go-generator -cache=false -install -api=sevicename:vsomething -gendir=..`
