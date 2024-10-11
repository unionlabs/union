//go:build go1.7
// +build go1.7

package lexruntimeservice_test

import (
	"context"
	"encoding/base64"
	"io/ioutil"
	"net/http"
	"strings"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/request"
	"github.com/aws/aws-sdk-go/awstesting/unit"
	"github.com/aws/aws-sdk-go/service/lexruntimeservice"
)

func TestLexRunTimeService_suppressedJSONValue(t *testing.T) {
	sess := unit.Session.Copy()

	client := lexruntimeservice.New(sess, &aws.Config{
		DisableParamValidation: aws.Bool(true),
	})
	expectBase64ActiveContexts := base64.StdEncoding.EncodeToString([]byte(`{}`))

	var actualInputActiveContexts string
	result, err := client.PostContentWithContext(context.Background(),
		&lexruntimeservice.PostContentInput{
			ActiveContexts: aws.String(`{}`),
		},
		func(r *request.Request) {
			r.Handlers.Send.Clear()
			r.Handlers.Send.PushBack(func(r *request.Request) {
				actualInputActiveContexts = r.HTTPRequest.Header.Get("x-amz-lex-active-contexts")
				r.HTTPResponse = &http.Response{
					StatusCode: 200,
					Header: func() http.Header {
						h := http.Header{}
						h.Set("x-amz-lex-active-contexts", expectBase64ActiveContexts)
						return h
					}(),
					ContentLength: 2,
					Body:          ioutil.NopCloser(strings.NewReader(`{}`)),
				}
			})
		},
	)
	if err != nil {
		t.Fatalf("failed to invoke operation, %v", err)
	}

	if e, a := expectBase64ActiveContexts, actualInputActiveContexts; e != a {
		t.Errorf("expect %v input active contexts, got %v", e, a)
	}

	if result.ActiveContexts == nil {
		t.Errorf("expect active contexts, got none")
	}
	if e, a := `{}`, *result.ActiveContexts; e != a {
		t.Errorf("expect %v output active contexts, got %v", e, a)
	}

}
