//go:build go1.16
// +build go1.16

package crr

import (
	"net/url"
	"reflect"
	"strconv"
	"testing"
	"time"
)

func Test_cloneURL(t *testing.T) {
	tests := []struct {
		value     *url.URL
		wantClone *url.URL
	}{
		{
			value: &url.URL{
				Scheme:      "https",
				Opaque:      "foo",
				User:        nil,
				Host:        "amazonaws.com",
				Path:        "/",
				RawPath:     "/",
				ForceQuery:  true,
				RawQuery:    "thing=value",
				Fragment:    "1234",
				RawFragment: "1234",
			},
			wantClone: &url.URL{
				Scheme:      "https",
				Opaque:      "foo",
				User:        nil,
				Host:        "amazonaws.com",
				Path:        "/",
				RawPath:     "/",
				ForceQuery:  true,
				RawQuery:    "thing=value",
				Fragment:    "1234",
				RawFragment: "1234",
			},
		},
		{
			value: &url.URL{
				Scheme:      "https",
				Opaque:      "foo",
				User:        url.UserPassword("NOT", "VALID"),
				Host:        "amazonaws.com",
				Path:        "/",
				RawPath:     "/",
				ForceQuery:  true,
				RawQuery:    "thing=value",
				Fragment:    "1234",
				RawFragment: "1234",
			},
			wantClone: &url.URL{
				Scheme:      "https",
				Opaque:      "foo",
				User:        url.UserPassword("NOT", "VALID"),
				Host:        "amazonaws.com",
				Path:        "/",
				RawPath:     "/",
				ForceQuery:  true,
				RawQuery:    "thing=value",
				Fragment:    "1234",
				RawFragment: "1234",
			},
		},
	}
	for i, tt := range tests {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			gotClone := cloneURL(tt.value)
			if gotClone == tt.value {
				t.Errorf("expct clone URL to not be same pointer address")
			}
			if tt.value.User != nil {
				if tt.value.User == gotClone.User {
					t.Errorf("expct cloned Userinfo to not be same pointer address")
				}
			}
			if !reflect.DeepEqual(gotClone, tt.wantClone) {
				t.Errorf("cloneURL() = %v, want %v", gotClone, tt.wantClone)
			}
		})
	}
}

func TestEndpoint_Prune(t *testing.T) {
	endpoint := Endpoint{}

	endpoint.Add(WeightedAddress{
		URL:     &url.URL{},
		Expired: time.Now().Add(5 * time.Minute),
	})

	initial := endpoint.Addresses

	if e, a := false, endpoint.Prune(); e != a {
		t.Errorf("expect prune %v, got %v", e, a)
	}

	if e, a := &initial[0], &endpoint.Addresses[0]; e != a {
		t.Errorf("expect slice address to be same")
	}

	endpoint.Add(WeightedAddress{
		URL:     &url.URL{},
		Expired: time.Now().Add(5 * -time.Minute),
	})

	initial = endpoint.Addresses

	if e, a := true, endpoint.Prune(); e != a {
		t.Errorf("expect prune %v, got %v", e, a)
	}

	if e, a := &initial[0], &endpoint.Addresses[0]; e == a {
		t.Errorf("expect slice address to be different")
	}

	if e, a := 1, endpoint.Len(); e != a {
		t.Errorf("expect slice length %v, got %v", e, a)
	}
}
