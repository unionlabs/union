package logtags

import (
	"context"
	"testing"
)

func TestRemove(t *testing.T) {
	b := &Buffer{}
	b = b.Add("1", nil)
	b = b.Add("2", nil)
	b = b.Add("3", nil)
	ctx := WithTags(context.Background(), b)
	rctx := RemoveTag(ctx, "2")
	if FromContext(rctx).String() != "1,3" {
		t.Fatalf("expected 1,3 got: %s", FromContext(rctx).String())
	}
	rctx = RemoveTag(ctx, "3")
	if FromContext(rctx).String() != "1,2" {
		t.Fatalf("expected 1,2 got: %s", FromContext(rctx).String())
	}
	rctx = RemoveTag(ctx, "4")
	if FromContext(rctx).String() != "1,2,3" {
		t.Fatalf("expected 1,2 got: %s", FromContext(rctx).String())
	}
}
