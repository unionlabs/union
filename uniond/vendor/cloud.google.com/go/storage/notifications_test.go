// Copyright 2017 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package storage

import (
	"context"
	"testing"

	"cloud.google.com/go/internal/testutil"
	raw "google.golang.org/api/storage/v1"
)

func TestParseNotificationTopic(t *testing.T) {
	for _, test := range []struct {
		in            string
		wantProjectID string
		wantTopicID   string
	}{
		{"", "?", "?"},
		{"foobar", "?", "?"},
		{"//pubsub.googleapis.com/projects/foo", "?", "?"},
		{"//pubsub.googleapis.com/projects/my-project/topics/my-topic",
			"my-project", "my-topic"},
	} {
		gotProjectID, gotTopicID := parseNotificationTopic(test.in)
		if gotProjectID != test.wantProjectID || gotTopicID != test.wantTopicID {
			t.Errorf("%q: got (%q, %q), want (%q, %q)",
				test.in, gotProjectID, gotTopicID, test.wantProjectID, test.wantTopicID)
		}
	}

}

func TestConvertNotification(t *testing.T) {
	want := &Notification{
		ID:               "id",
		TopicProjectID:   "my-project",
		TopicID:          "my-topic",
		EventTypes:       []string{ObjectFinalizeEvent},
		ObjectNamePrefix: "prefix",
		CustomAttributes: map[string]string{"a": "b"},
		PayloadFormat:    JSONPayload,
	}
	got := toNotification(toRawNotification(want))
	if diff := testutil.Diff(got, want); diff != "" {
		t.Errorf("got=-, want=+:\n%s", diff)
	}
}

func TestNotificationsToMap(t *testing.T) {
	got := notificationsToMap(nil)
	want := map[string]*Notification{}
	if !testutil.Equal(got, want) {
		t.Errorf("got %+v, want %+v", got, want)
	}

	in := []*raw.Notification{
		{Id: "a", Topic: "//pubsub.googleapis.com/projects/P1/topics/T1"},
		{Id: "b", Topic: "//pubsub.googleapis.com/projects/P2/topics/T2"},
		{Id: "c", Topic: "//pubsub.googleapis.com/projects/P3/topics/T3"},
	}
	got = notificationsToMap(in)
	want = map[string]*Notification{
		"a": {ID: "a", TopicProjectID: "P1", TopicID: "T1"},
		"b": {ID: "b", TopicProjectID: "P2", TopicID: "T2"},
		"c": {ID: "c", TopicProjectID: "P3", TopicID: "T3"},
	}
	if diff := testutil.Diff(got, want); diff != "" {
		t.Errorf("got=-, want=+:\n%s", diff)
	}
}

func TestAddNotificationsErrors(t *testing.T) {
	c := &Client{}
	b := c.Bucket("b")
	for _, n := range []*Notification{
		{ID: "foo", TopicProjectID: "p", TopicID: "t"}, // has ID
		{TopicProjectID: "p"},                          // missing TopicID
		{TopicID: "t"},                                 // missing TopicProjectID
	} {
		_, err := b.AddNotification(context.Background(), n)
		if err == nil {
			t.Errorf("%+v: got nil, want error", n)
		}
	}
}
