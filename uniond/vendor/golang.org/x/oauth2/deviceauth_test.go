package oauth2

import (
	"context"
	"encoding/json"
	"fmt"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
)

func TestDeviceAuthResponseMarshalJson(t *testing.T) {
	tests := []struct {
		name     string
		response DeviceAuthResponse
		want     string
	}{
		{
			name:     "empty",
			response: DeviceAuthResponse{},
			want:     `{"device_code":"","user_code":"","verification_uri":""}`,
		},
		{
			name: "soon",
			response: DeviceAuthResponse{
				Expiry: time.Now().Add(100*time.Second + 999*time.Millisecond),
			},
			want: `{"expires_in":100,"device_code":"","user_code":"","verification_uri":""}`,
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			begin := time.Now()
			gotBytes, err := json.Marshal(tc.response)
			if err != nil {
				t.Fatal(err)
			}
			if strings.Contains(tc.want, "expires_in") && time.Since(begin) > 999*time.Millisecond {
				t.Skip("test ran too slowly to compare `expires_in`")
			}
			got := string(gotBytes)
			if got != tc.want {
				t.Errorf("want=%s, got=%s", tc.want, got)
			}
		})
	}
}

func TestDeviceAuthResponseUnmarshalJson(t *testing.T) {
	tests := []struct {
		name string
		data string
		want DeviceAuthResponse
	}{
		{
			name: "empty",
			data: `{}`,
			want: DeviceAuthResponse{},
		},
		{
			name: "soon",
			data: `{"expires_in":100}`,
			want: DeviceAuthResponse{Expiry: time.Now().UTC().Add(100 * time.Second)},
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			begin := time.Now()
			got := DeviceAuthResponse{}
			err := json.Unmarshal([]byte(tc.data), &got)
			if err != nil {
				t.Fatal(err)
			}
			if !cmp.Equal(got, tc.want, cmpopts.IgnoreUnexported(DeviceAuthResponse{}), cmpopts.EquateApproxTime(time.Second+time.Since(begin))) {
				t.Errorf("want=%#v, got=%#v", tc.want, got)
			}
		})
	}
}

func ExampleConfig_DeviceAuth() {
	var config Config
	ctx := context.Background()
	response, err := config.DeviceAuth(ctx)
	if err != nil {
		panic(err)
	}
	fmt.Printf("please enter code %s at %s\n", response.UserCode, response.VerificationURI)
	token, err := config.DeviceAccessToken(ctx, response)
	if err != nil {
		panic(err)
	}
	fmt.Println(token)
}
