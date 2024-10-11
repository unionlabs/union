package casbin

import (
	"context"
	"testing"

	stdcasbin "github.com/casbin/casbin/v2"
	"github.com/casbin/casbin/v2/model"
	fileadapter "github.com/casbin/casbin/v2/persist/file-adapter"
)

func TestStructBaseContext(t *testing.T) {
	e := func(ctx context.Context, i interface{}) (interface{}, error) { return ctx, nil }

	m := model.NewModel()
	m.AddDef("r", "r", "sub, obj, act")
	m.AddDef("p", "p", "sub, obj, act")
	m.AddDef("e", "e", "some(where (p.eft == allow))")
	m.AddDef("m", "m", "r.sub == p.sub && keyMatch(r.obj, p.obj) && regexMatch(r.act, p.act)")

	a := fileadapter.NewAdapter("testdata/keymatch_policy.csv")

	ctx := context.WithValue(context.Background(), CasbinModelContextKey, m)
	ctx = context.WithValue(ctx, CasbinPolicyContextKey, a)

	// positive case
	middleware := NewEnforcer("alice", "/alice_data/resource1", "GET")(e)
	ctx1, err := middleware(ctx, struct{}{})
	if err != nil {
		t.Fatalf("Enforcer returned error: %s", err)
	}
	_, ok := ctx1.(context.Context).Value(CasbinEnforcerContextKey).(*stdcasbin.Enforcer)
	if !ok {
		t.Fatalf("context should contains the active enforcer")
	}

	// negative case
	middleware = NewEnforcer("alice", "/alice_data/resource2", "POST")(e)
	_, err = middleware(ctx, struct{}{})
	if err == nil {
		t.Fatalf("Enforcer should return error")
	}
}

func TestFileBaseContext(t *testing.T) {
	e := func(ctx context.Context, i interface{}) (interface{}, error) { return ctx, nil }
	ctx := context.WithValue(context.Background(), CasbinModelContextKey, "testdata/basic_model.conf")
	ctx = context.WithValue(ctx, CasbinPolicyContextKey, "testdata/basic_policy.csv")

	// positive case
	middleware := NewEnforcer("alice", "data1", "read")(e)
	_, err := middleware(ctx, struct{}{})
	if err != nil {
		t.Fatalf("Enforcer returned error: %s", err)
	}
}
