// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package impl_test

import (
	"reflect"
	"sync"
	"testing"
	"unsafe"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/protobuf/internal/errors"
	"google.golang.org/protobuf/internal/flags"
	"google.golang.org/protobuf/internal/impl"
	"google.golang.org/protobuf/internal/protobuild"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/testing/protocmp"

	lazytestpb "google.golang.org/protobuf/internal/testprotos/lazy"
	"google.golang.org/protobuf/internal/testprotos/messageset/messagesetpb"
	testpb "google.golang.org/protobuf/internal/testprotos/test"
)

func TestLazyExtensions(t *testing.T) {
	checkLazy := func(when string, m *testpb.TestAllExtensions, want bool) {
		xd := testpb.E_OptionalNestedMessage.TypeDescriptor()
		if got := impl.IsLazy(m.ProtoReflect(), xd); got != want {
			t.Errorf("%v: m.optional_nested_message lazy=%v, want %v", when, got, want)
		}
		e := proto.GetExtension(m, testpb.E_OptionalNestedMessage).(*testpb.TestAllExtensions_NestedMessage).Corecursive
		if got := impl.IsLazy(e.ProtoReflect(), xd); got != want {
			t.Errorf("%v: m.optional_nested_message.corecursive.optional_nested_message lazy=%v, want %v", when, got, want)
		}
	}

	m1 := &testpb.TestAllExtensions{}
	protobuild.Message{
		"optional_nested_message": protobuild.Message{
			"a": 1,
			"corecursive": protobuild.Message{
				"optional_nested_message": protobuild.Message{
					"a": 2,
				},
			},
		},
	}.Build(m1.ProtoReflect())
	checkLazy("before unmarshal", m1, false)

	w, err := proto.Marshal(m1)
	if err != nil {
		t.Fatal(err)
	}
	m := &testpb.TestAllExtensions{}
	if err := proto.Unmarshal(w, m); err != nil {
		t.Fatal(err)
	}
	checkLazy("after unmarshal", m, flags.LazyUnmarshalExtensions)
}

func TestMessageSetLazy(t *testing.T) {
	if !flags.LazyUnmarshalExtensions {
		t.Skip("lazy extension unmarshaling disabled; not built with the protolegacy tag")
	}
	h := &lazytestpb.Holder{Data: &messagesetpb.MessageSet{}}

	ext := &lazytestpb.Rabbit{Name: proto.String("Judy")}
	proto.SetExtension(h.GetData(), lazytestpb.E_Rabbit_MessageSetExtension, ext)

	nh := roundtrip(t, h).(*lazytestpb.Holder)

	if extensionIsInitialized(t, nh.GetData(), lazytestpb.E_Rabbit_MessageSetExtension.Field) {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}

	proto.Size(nh)
	if extensionIsInitialized(t, nh.GetData(), lazytestpb.E_Rabbit_MessageSetExtension.Field) {
		t.Errorf("Extension unexpectedly initialized after Size")
	}

	proto.Marshal(nh)
	if extensionIsInitialized(t, nh.GetData(), lazytestpb.E_Rabbit_MessageSetExtension.Field) {
		t.Errorf("Extension unexpectedly initialized after Marshal")
	}

	if !proto.HasExtension(nh.GetData(), lazytestpb.E_Rabbit_MessageSetExtension) {
		t.Fatalf("Can't get extension")
	}
	if extensionIsInitialized(t, nh.GetData(), lazytestpb.E_Rabbit_MessageSetExtension.Field) {
		t.Errorf("Extension unexpectedly initialized after Has")
	}

	nh = roundtrip(t, nh).(*lazytestpb.Holder)
	if extensionIsInitialized(t, nh.GetData(), lazytestpb.E_Rabbit_MessageSetExtension.Field) {
		t.Errorf("Extension unexpectedly initialized after Has")
	}

	if diff := cmp.Diff(h, nh, protocmp.Transform()); diff != "" {
		t.Errorf("Got %+v, want %+v, diff:\n%s", nh, h, diff)
	}
	if got, want := extensionIsInitialized(t, nh.GetData(), lazytestpb.E_Rabbit_MessageSetExtension.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Diff")
	}
	int := proto.GetExtension(nh.GetData(), lazytestpb.E_Rabbit_MessageSetExtension).(*lazytestpb.Rabbit)
	if int.GetName() != "Judy" {
		t.Errorf("Extension value \"Judy\" not retained, got: %v", int.GetName())
	}
	if got, want := extensionIsInitialized(t, nh.GetData(), lazytestpb.E_Rabbit_MessageSetExtension.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Get")
	}
}

var (
	// Trees are fully lazy
	treeTemplate = &lazytestpb.Tree{
		Eucalyptus: proto.Bool(true),
	}

	spGH = lazytestpb.FlyingFoxSpecies_GREY_HEADED
	spP  = lazytestpb.FlyingFoxSpecies_SPECTACLED
	spLE = lazytestpb.FlyingFoxSpecies_LARGE_EARED
	spBB = lazytestpb.FlyingFoxSpecies_BARE_BACKED
	spF  = lazytestpb.PipistrelleSpecies_FOREST
	spR  = lazytestpb.PipistrelleSpecies_RUSTY
)

func TestExtensionLazy(t *testing.T) {
	if !flags.LazyUnmarshalExtensions {
		t.Skip("lazy extension unmarshaling disabled; not built with the protolegacy tag")
	}

	tree := proto.Clone(treeTemplate)
	proto.SetExtension(tree, lazytestpb.E_Bat, &lazytestpb.FlyingFox{Species: &spGH})
	proto.SetExtension(tree, lazytestpb.E_BatPup, &lazytestpb.FlyingFox{Species: &spP})

	nt := roundtrip(t, tree).(*lazytestpb.Tree)

	if extensionIsInitialized(t, nt, lazytestpb.E_Bat.Field) {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}
	proto.Size(nt)
	if extensionIsInitialized(t, nt, lazytestpb.E_Bat.Field) {
		t.Errorf("Extension unexpectedly initialized after Size")
	}

	gb, err := proto.Marshal(nt)
	if err != nil {
		t.Fatalf("proto.Marshal(%+v) failed: %v", nt, err)
	}
	if extensionIsInitialized(t, nt, lazytestpb.E_Bat.Field) {
		t.Errorf("Extension unexpectedly initialized after Marshal")
	}

	fox := proto.GetExtension(nt, lazytestpb.E_Bat).(*lazytestpb.FlyingFox)
	if got, want := fox.GetSpecies(), spGH; want != got {
		t.Errorf("Extension's Speices field not retained, want: %v, got: %v", want, got)
	}
	if got, want := extensionIsInitialized(t, nt, lazytestpb.E_Bat.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Get")
	}
	if extensionIsInitialized(t, nt, lazytestpb.E_BatPup.Field) {
		t.Errorf("Extension unexpectedly initialized after Get")
	}
	foxPup := proto.GetExtension(nt, lazytestpb.E_BatPup).(*lazytestpb.FlyingFox)
	if got, want := foxPup.GetSpecies(), spP; want != got {
		t.Errorf("Extension's Speices field not retained, want: %v, got: %v", want, got)
	}
	if got, want := extensionIsInitialized(t, nt, lazytestpb.E_Bat.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Get")
	}
	if got, want := extensionIsInitialized(t, nt, lazytestpb.E_BatPup.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Get")
	}

	rt := &lazytestpb.Tree{}
	if err := proto.Unmarshal(gb, rt); err != nil {
		t.Fatalf("Can't unmarshal pb.Tree: %v", err)
	}

	if diff := cmp.Diff(tree, rt, protocmp.Transform()); diff != "" {
		t.Errorf("Got %+v, want %+v, diff:\n%s", rt, tree, diff)
	}
	if got, want := extensionIsInitialized(t, rt, lazytestpb.E_Bat.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Diff")
	}

	nt = roundtrip(t, tree).(*lazytestpb.Tree)

	proto.ClearExtension(nt, lazytestpb.E_Bat)
	proto.ClearExtension(nt, lazytestpb.E_BatPup)
	if proto.HasExtension(nt, lazytestpb.E_Bat) {
		t.Fatalf("Extension not cleared in (%+v)", nt)
	}
	if proto.HasExtension(nt, lazytestpb.E_BatPup) {
		t.Fatalf("Extension not cleared in (%+v)", nt)
	}
}

func TestExtensionNestedScopeLazy(t *testing.T) {
	if !flags.LazyUnmarshalExtensions {
		t.Skip("lazy extension unmarshaling disabled; not built with the protolegacy tag")
	}

	tree := proto.Clone(treeTemplate)

	proto.SetExtension(tree, lazytestpb.E_BatNest_Bat, &lazytestpb.FlyingFox{Species: &spGH})

	nt := roundtrip(t, tree).(*lazytestpb.Tree)

	if extensionIsInitialized(t, nt, lazytestpb.E_BatNest_Bat.Field) {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}
	proto.Size(nt)
	if extensionIsInitialized(t, nt, lazytestpb.E_BatNest_Bat.Field) {
		t.Errorf("Extension unexpectedly initialized after Size")
	}

	gb, err := proto.Marshal(nt)
	if err != nil {
		t.Fatalf("proto.Marshal(%+v) failed: %v", nt, err)
	}
	if extensionIsInitialized(t, nt, lazytestpb.E_BatNest_Bat.Field) {
		t.Errorf("Extension unexpectedly initialized after Marshal")
	}

	fox := proto.GetExtension(nt, lazytestpb.E_BatNest_Bat).(*lazytestpb.FlyingFox)
	if got, want := fox.GetSpecies(), spGH; want != got {
		t.Errorf("Extension's Speices field not retained, want: %v, got: %v", want, got)
	}
	if got, want := extensionIsInitialized(t, nt, lazytestpb.E_BatNest_Bat.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Get")
	}

	rt := &lazytestpb.Tree{}
	if err := proto.Unmarshal(gb, rt); err != nil {
		t.Fatalf("Can't unmarshal pb.Tree: %v", err)
	}

	if diff := cmp.Diff(tree, rt, protocmp.Transform()); diff != "" {
		t.Errorf("Got %+v, want %+v, diff:\n%s", rt, tree, diff)
	}
	if got, want := extensionIsInitialized(t, rt, lazytestpb.E_BatNest_Bat.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Diff")
	}
}

func TestExtensionRepeatedMessageLazy(t *testing.T) {
	if !flags.LazyUnmarshalExtensions {
		t.Skip("lazy extension unmarshaling disabled; not built with the protolegacy tag")
	}

	posse := []*lazytestpb.FlyingFox{
		{Species: &spLE},
		{Species: &spBB},
	}
	m := proto.Clone(treeTemplate)
	proto.SetExtension(m, lazytestpb.E_BatPosse, posse)
	if got, want := proto.HasExtension(m, lazytestpb.E_BatPosse), true; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
	mr := roundtrip(t, m).(*lazytestpb.Tree)

	if extensionIsInitialized(t, mr, lazytestpb.E_BatPosse.Field) {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}

	mrr := roundtrip(t, mr).(*lazytestpb.Tree)
	if got, want := proto.HasExtension(mrr, lazytestpb.E_BatPosse), true; got != want {
		t.Errorf("Extension is not present after setting: got %v, want %v", got, want)
	}

	if extensionIsInitialized(t, mrr, lazytestpb.E_BatPosse.Field) {
		t.Errorf("Extension unexpectedly initialized after Has")
	}

	mrr = roundtrip(t, mr).(*lazytestpb.Tree)
	foxPosse := proto.GetExtension(mrr, lazytestpb.E_BatPosse).([]*lazytestpb.FlyingFox)
	if got, want := foxPosse[0].GetSpecies(), spLE; got != want {
		t.Errorf("Extension's Speices field, want: %v, got: %v", want, got)
	}
	if got, want := extensionIsInitialized(t, mrr, lazytestpb.E_BatPosse.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Get")
	}

	// Set to empty slice instead
	m = proto.Clone(treeTemplate)
	proto.SetExtension(m, lazytestpb.E_BatPosse, []*lazytestpb.FlyingFox{})
	if got, want := proto.HasExtension(m, lazytestpb.E_BatPosse), false; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
	mr = roundtrip(t, m).(*lazytestpb.Tree)

	if got, want := extensionIsInitialized(t, mr, lazytestpb.E_BatPosse.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}

	if got, want := proto.HasExtension(mr, lazytestpb.E_BatPosse), false; got != want {
		t.Errorf("Extension is not present after setting: got %v, want %v", got, want)
	}
	if got, want := extensionIsInitialized(t, mr, lazytestpb.E_BatPosse.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Has")
	}

	mrr = roundtrip(t, mr).(*lazytestpb.Tree)
	if got, want := proto.HasExtension(mrr, lazytestpb.E_BatPosse), false; got != want {
		t.Errorf("Extension is not present after setting: got %v, want %v", got, want)
	}
	if got, want := extensionIsInitialized(t, mrr, lazytestpb.E_BatPosse.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Has")
	}

	foxPosse = proto.GetExtension(mrr, lazytestpb.E_BatPosse).([]*lazytestpb.FlyingFox)
	if got, want := len(foxPosse), 0; got != want {
		t.Errorf("Extension field length, want: %v, got: %v", want, got)
	}
	if got, want := extensionIsInitialized(t, mrr, lazytestpb.E_BatPosse.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Get")
	}
}

func TestExtensionIntegerLazy(t *testing.T) {
	if !flags.LazyUnmarshalExtensions {
		t.Skip("lazy extension unmarshaling disabled; not built with the protolegacy tag")
	}

	var iBat uint32 = 4711
	m := proto.Clone(treeTemplate)
	proto.SetExtension(m, lazytestpb.E_IntegerBat, iBat)
	if got, want := proto.HasExtension(m, lazytestpb.E_IntegerBat), true; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
	mr := roundtrip(t, m).(*lazytestpb.Tree)

	if got, want := extensionIsInitialized(t, mr, lazytestpb.E_IntegerBat.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}

	if got, want := proto.HasExtension(mr, lazytestpb.E_IntegerBat), true; got != want {
		t.Errorf("Extension is not present after setting: got %v, want %v", got, want)
	}
	if got, want := extensionIsInitialized(t, mr, lazytestpb.E_IntegerBat.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Has")
	}

	mr = roundtrip(t, m).(*lazytestpb.Tree)
	if got, want := proto.GetExtension(mr, lazytestpb.E_IntegerBat).(uint32), iBat; got != want {
		t.Errorf("Extension's integer field, want: %v, got: %v", want, got)
	}
	if got, want := extensionIsInitialized(t, mr, lazytestpb.E_IntegerBat.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Get")
	}
}

func TestExtensionBinaryLazy(t *testing.T) {
	if !flags.LazyUnmarshalExtensions {
		t.Skip("lazy extension unmarshaling disabled; not built with the protolegacy tag")
	}

	m := proto.Clone(treeTemplate)
	bBat := []byte("I'm a bat")
	proto.SetExtension(m, lazytestpb.E_BinaryBat, bBat)
	if got, want := proto.HasExtension(m, lazytestpb.E_BinaryBat), true; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
	mr := roundtrip(t, m).(*lazytestpb.Tree)
	// A binary extension is never kept lazy
	if got, want := extensionIsInitialized(t, mr, lazytestpb.E_BinaryBat.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}
	if got, want := proto.HasExtension(mr, lazytestpb.E_BinaryBat), true; got != want {
		t.Errorf("Extension present after roundtrip: got %v, want %v", got, want)
	}

	m = proto.Clone(treeTemplate)
	proto.SetExtension(m, lazytestpb.E_BinaryBat, []byte{})
	// An empty binary is also a binary
	if got, want := proto.HasExtension(m, lazytestpb.E_BinaryBat), true; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
	mr = roundtrip(t, m).(*lazytestpb.Tree)
	if got, want := extensionIsInitialized(t, mr, lazytestpb.E_BinaryBat.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}
	if got, want := proto.HasExtension(mr, lazytestpb.E_BinaryBat), true; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
}

func TestExtensionGroupLazy(t *testing.T) {
	if !flags.LazyUnmarshalExtensions {
		t.Skip("lazy extension unmarshaling disabled; not built with the protolegacy tag")
	}

	// Group: should behave like message
	pip := &lazytestpb.Pipistrelle{Species: &spF}
	pips := []*lazytestpb.Pipistrelles{
		{Species: &spF},
		{Species: &spR},
	}
	m := proto.Clone(treeTemplate)
	proto.SetExtension(m, lazytestpb.E_Pipistrelle, pip)
	if got, want := proto.HasExtension(m, lazytestpb.E_Pipistrelle), true; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
	mr := roundtrip(t, m).(*lazytestpb.Tree)

	if extensionIsInitialized(t, mr, lazytestpb.E_Pipistrelle.Field) {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}
	if got, want := proto.HasExtension(mr, lazytestpb.E_Pipistrelle), true; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
	if extensionIsInitialized(t, mr, lazytestpb.E_Pipistrelle.Field) {
		t.Errorf("Extension unexpectedly initialized after Has")
	}
	mrr := roundtrip(t, mr).(*lazytestpb.Tree)
	if extensionIsInitialized(t, mrr, lazytestpb.E_Pipistrelle.Field) {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}
	mrr = roundtrip(t, mr).(*lazytestpb.Tree)
	if extensionIsInitialized(t, mrr, lazytestpb.E_Pipistrelle.Field) {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}
	pipistrelle := proto.GetExtension(mrr, lazytestpb.E_Pipistrelle).(*lazytestpb.Pipistrelle)
	if got, want := pipistrelle.GetSpecies(), spF; got != want {
		t.Errorf("Extension's Speices field, want: %v, got: %v", want, got)
	}
	if got, want := extensionIsInitialized(t, mrr, lazytestpb.E_Pipistrelle.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Get")
	}

	// Group slice, behaves like message slice
	m = proto.Clone(treeTemplate)
	proto.SetExtension(m, lazytestpb.E_Pipistrelles, pips)
	if got, want := proto.HasExtension(m, lazytestpb.E_Pipistrelles), true; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
	mr = roundtrip(t, m).(*lazytestpb.Tree)

	if extensionIsInitialized(t, mr, lazytestpb.E_Pipistrelles.Field) {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}
	if got, want := proto.HasExtension(mr, lazytestpb.E_Pipistrelles), true; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
	if extensionIsInitialized(t, mr, lazytestpb.E_Pipistrelles.Field) {
		t.Errorf("Extension unexpectedly initialized after Has")
	}
	mr = roundtrip(t, m).(*lazytestpb.Tree)
	mrr = roundtrip(t, mr).(*lazytestpb.Tree)
	if got, want := proto.HasExtension(mrr, lazytestpb.E_Pipistrelles), true; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
	if extensionIsInitialized(t, mrr, lazytestpb.E_Pipistrelles.Field) {
		t.Errorf("Extension unexpectedly initialized after Has")
	}
	mrr = roundtrip(t, mr).(*lazytestpb.Tree)
	pipistrelles := proto.GetExtension(mrr, lazytestpb.E_Pipistrelles).([]*lazytestpb.Pipistrelles)
	if got, want := pipistrelles[1].GetSpecies(), spR; got != want {
		t.Errorf("Extension's Speices field, want: %v, got: %v", want, got)
	}
	if got, want := extensionIsInitialized(t, mrr, lazytestpb.E_Pipistrelles.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Get")
	}

	// Setting an empty group slice has no effect
	m = proto.Clone(treeTemplate)
	proto.SetExtension(m, lazytestpb.E_Pipistrelles, []*lazytestpb.Pipistrelles{})
	if got, want := proto.HasExtension(m, lazytestpb.E_Pipistrelles), false; got != want {
		t.Errorf("Extension present after setting empty: got %v, want %v", got, want)
	}
	mr = roundtrip(t, m).(*lazytestpb.Tree)

	if got, want := extensionIsInitialized(t, mr, lazytestpb.E_Pipistrelles.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Unmarshal")
	}
	if got, want := proto.HasExtension(mr, lazytestpb.E_Pipistrelles), false; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
	if got, want := extensionIsInitialized(t, mr, lazytestpb.E_Pipistrelles.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Has")
	}
	mrr = roundtrip(t, mr).(*lazytestpb.Tree)
	if got, want := proto.HasExtension(mrr, lazytestpb.E_Pipistrelles), false; got != want {
		t.Errorf("Extension present after setting: got %v, want %v", got, want)
	}
	if got, want := extensionIsInitialized(t, mrr, lazytestpb.E_Pipistrelles.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Has")
	}
	noPipistrelles := proto.GetExtension(mrr, lazytestpb.E_Pipistrelles).([]*lazytestpb.Pipistrelles)
	if got, want := len(noPipistrelles), 0; got != want {
		t.Errorf("Extension's field length, want: %v, got: %v", want, got)
	}
	if got, want := extensionIsInitialized(t, mrr, lazytestpb.E_Pipistrelles.Field), true; got != want {
		t.Errorf("Extension unexpectedly initialized after Get")
	}
}

func TestMarshalMessageSetLazyRace(t *testing.T) {
	if !flags.LazyUnmarshalExtensions {
		t.Skip("lazy extension unmarshaling disabled; not built with the protolegacy tag")
	}

	h := &lazytestpb.Holder{Data: &messagesetpb.MessageSet{}}

	ext := &lazytestpb.Rabbit{Name: proto.String("Judy")}
	proto.SetExtension(h.GetData(), lazytestpb.E_Rabbit_MessageSetExtension, ext)

	b, err := proto.Marshal(h)
	if err != nil {
		t.Fatalf("Could not marshal message: %v", err)
	}
	if err := proto.Unmarshal(b, h); err != nil {
		t.Fatalf("Could not unmarshal message: %v", err)
	}
	// after Unmarshal, the extension is in undecoded form.
	// GetExtension will decode it lazily. Make sure this does
	// not race against Marshal.

	// The following pattern is similar to x/sync/errgroup,
	// but we want to avoid adding that dependencies just for a test.
	var (
		wg       sync.WaitGroup
		errOnce  sync.Once
		groupErr error
	)
	for n := 30; n > 0; n-- {
		wg.Add(2)
		go func() {
			defer wg.Done()
			if err := func() error {
				b, err := proto.Marshal(h)
				if err == nil {
					return proto.Unmarshal(b, &lazytestpb.Rabbit{})
				}
				return err
			}(); err != nil {
				errOnce.Do(func() { groupErr = err })
			}
		}()
		go func() {
			defer wg.Done()
			if err := func() error {
				if !proto.HasExtension(h.GetData(), lazytestpb.E_Rabbit_MessageSetExtension) {
					return errors.New("proto: missing extension")
				}
				return nil
			}(); err != nil {
				errOnce.Do(func() { groupErr = err })
			}
		}()
	}
	wg.Wait()
	if groupErr != nil {
		t.Fatal(groupErr)
	}
}

// Utility functions for the test cases

// Some functions from pointer_unsafe.go
type pointer struct{ p unsafe.Pointer }

func (p pointer) Apply(f uintptr) pointer {
	return pointer{p: unsafe.Pointer(uintptr(p.p) + uintptr(f))}
}

func pointerOfIface(v any) pointer {
	type ifaceHeader struct {
		Type unsafe.Pointer
		Data unsafe.Pointer
	}
	return pointer{p: (*ifaceHeader)(unsafe.Pointer(&v)).Data}
}

func (p pointer) AsValueOf(t reflect.Type) reflect.Value {
	return reflect.NewAt(t, p.p)
}

// Highly implementation dependent - uses unsafe pointers to figure
// out if the lazyExtensionValue is initialized.
func extensionIsInitialized(t *testing.T, data any, fieldNo int32) bool {
	ext, ok := reflect.TypeOf(data).Elem().FieldByName("extensionFields")
	if !ok {
		t.Fatalf("Failed to retrieve offset of field \"extensionFields\".")
	}
	lazy, ok := reflect.TypeOf((*impl.ExtensionField)(nil)).Elem().FieldByName("lazy")
	if !ok {
		t.Fatalf("Failed to retrieve offset of field \"lazy\".")
	}

	pi := pointerOfIface(data)
	m, ok := pi.Apply(ext.Offset).AsValueOf(reflect.TypeOf((map[int32]impl.ExtensionField)(nil))).Interface().(*map[int32]impl.ExtensionField)
	if !ok {
		t.Fatalf("Extension map has unexpected type.")
	}
	f := (*m)[fieldNo]
	// Here we rely on atomicOnce being the first field in the 'lazy' struct.
	app, ok := pointerOfIface(&f).Apply(lazy.Offset).AsValueOf(reflect.TypeOf((*uint32)(nil))).Interface().(**uint32)
	if !ok {
		t.Fatalf("Field atomicOnce does not seem to be the first field, or has changed type.")
	}
	if *app == nil {
		return true // lazy ptr is nil
	}
	return **app > 0
}

func roundtrip(t *testing.T, m proto.Message) proto.Message {
	t.Helper()
	n := m.ProtoReflect().Type().New().Interface()
	b, err := proto.Marshal(m)
	if err != nil {
		t.Fatalf("proto.Marshal(%+v) failed: %v", m, err)
	}
	if err := proto.Unmarshal(b, n); err != nil {
		t.Fatalf("proto.Unmarshal failed: %v", err)
	}
	return n
}
