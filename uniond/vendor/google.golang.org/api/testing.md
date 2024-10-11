# Testing Code that depends on google.golang.org/api

The client libraries generated as a part of `google.golang.org/api` all take
the approach of returning concrete types instead of interfaces. That way, new
fields and methods can be added to the libraries without breaking users. This
document will go over some patterns that can be used to test code that depends
on the these libraries.

## Testing HTTP services using fakes

*Note*: You can see the full
[example code using a fake here](https://github.com/googleapis/google-api-go-client/tree/main/internal/examples/fake).

The services found in `google.golang.org/api` are all HTTP based.
Interactions with HTTP services can be faked by serving up your own in-memory
server within your test. One benefit of using this approach is that you don’t
need to define an interface in your runtime code; you can keep using the
concrete struct types returned by the client library. For example, take a look
at the following function:

```go
import (
    "fmt"
    "os"

    "google.golang.org/api/translate/v3"
)

// TranslateText translates text to the given language using the provided
// service.
func TranslateText(service *translate.Service, text, language string) (string, error) {
    parent := fmt.Sprintf("projects/%s/locations/global", os.Getenv("GOOGLE_CLOUD_PROJECT"))
    req := &translate.TranslateTextRequest{
        TargetLanguageCode: language,
        Contents:           []string{text},
    }
    resp, err := service.Projects.Locations.TranslateText(parent, req).Do()
    if err != nil {
        return "", fmt.Errorf("unable to translate text: %v", err)
    }
    return resp.Translations[0].TranslatedText, nil
}
```

To fake HTTP interactions we can make use of the `httptest` package found in the
standard library. The server URL obtained from creating the test server can be
passed to the service constructor with the use of `option.WithEndpoint`. This
instructs the service to route all traffic to your test sever rather than the
live service. Here is an example of what doing this looks like when calling the
above function:

```go
import (
    "context"
    "encoding/json"
    "net/http"
    "net/http/httptest"
    "testing"

    "google.golang.org/api/option"
    "google.golang.org/api/translate/v3"
)

func TestTranslateText(t *testing.T) {
    ctx := context.Background()
    ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        resp := &translate.TranslateTextResponse{
            Translations: []*translate.Translation{
                {TranslatedText: "Hello World"},
            },
        }
        b, err := json.Marshal(resp)
        if err != nil {
            http.Error(w, "unable to marshal request: "+err.Error(), http.StatusBadRequest)
            return
        }
        w.Write(b)
    }))
    defer ts.Close()
    svc, err := translate.NewService(ctx, option.WithoutAuthentication(), option.WithEndpoint(ts.URL))
    if err != nil {
        t.Fatalf("unable to create client: %v", err)
    }
    text, err := TranslateText(svc, "Hola Mundo", "en-US")
    if err != nil {
        t.Fatal(err)
    }
    if text != "Hello World" {
        t.Fatalf("got %q, want Hello World", text)
    }
}
```

## Testing using mocks

*Note*: You can see the full
[example code using a mocks here](https://github.com/googleapis/google-api-go-client/tree/main/internal/examples/mock).

When mocking code you need to work with interfaces. Because the services in
`google.golang.org/api` use the builder pattern to construct and execute
requests it can be tedious to create low-level interfaces that match methods
found on the services directly. Although this can be done and you can find
examples of this in full code linked above. Another approach that will keep your
interfaces cleaner is to create a high-level interface that accepts all of the
required input and returns the desired outputs. This hides the complexity of the
services builder pattern. This concept is sometimes referred to as the facade
pattern. Here is an example of a high level interface for the `TranslateText`
method:

```go
// TranslateService is a facade of a `translate.Service`, specifically used to
// for translating text.
type TranslateService interface {
    TranslateText(text, language string) (string, error)
}

// TranslateTextHighLevel translates text to the given language using the
// provided service.
func TranslateTextHighLevel(service TranslateService, text, language string) (string, error) {
    return service.TranslateText(text, language)
}
```

This interface allows a concrete `translate.Service` to be wrapped and passed to
the function in production and for a mock implementation to be passed in during
testing. Here is what it would look like to create a wrapper around a `translate.Service`
to fullfil the `TranslateService` interface:

```go
import (
    "context"
    "fmt"
    "log"
    "os"

    "google.golang.org/api/option"
    "google.golang.org/api/translate/v3"
)

type translateService struct {
    svc *translate.Service
}

// NewTranslateService creates a TranslateService.
func NewTranslateService(ctx context.Context, opts ...option.ClientOption) TranslateService {
    svc, err := translate.NewService(ctx, opts...)
    if err != nil {
        log.Fatalf("unable to create translate service, shutting down: %v", err)
    }
    return &translateService{svc}
}

func (t *translateService) TranslateText(text, language string) (string, error) {
    parent := fmt.Sprintf("projects/%s/locations/global", os.Getenv("GOOGLE_CLOUD_PROJECT"))
    resp, err := t.svc.Projects.Locations.TranslateText(parent, &translate.TranslateTextRequest{
        TargetLanguageCode: language,
        Contents:           []string{text},
    }).Do()
    if err != nil {
        return "", fmt.Errorf("unable to translate text: %v", err)
    }
    return resp.Translations[0].TranslatedText, nil
}
```

Let’s take a look at what it might look like to define a lightweight mock for
the `TranslateService` interface.

```go
import "testing"

// mockService fulfills the TranslateService interface.
type mockService struct{}

func (*mockService) TranslateText(text, language string) (string, error) {
    return "Hello World", nil
}
func TestTranslateTextHighLevel(t *testing.T) {
    svc := &mockService{}
    text, err := TranslateTextHighLevel(svc, "Hola Mundo", "en-US")
    if err != nil {
        t.Fatal(err)
    }
    if text != "Hello World" {
        t.Fatalf("got %q, want Hello World", text)
    }
}
```

If you prefer to not write your own mocks there are mocking frameworks such as
[golang/mock](https://github.com/golang/mock) which can generate mocks for you
from an interface. As a word of caution though, try to not
[overuse mocks](https://testing.googleblog.com/2013/05/testing-on-toilet-dont-overuse-mocks.html).
