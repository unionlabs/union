package types_test

import (
	"encoding/json"
	"reflect"
	"testing"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/math"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type eventsTestSuite struct {
	suite.Suite
}

func TestEventsTestSuite(t *testing.T) {
	suite.Run(t, new(eventsTestSuite))
}

func (s *eventsTestSuite) TestAppendEvents() {
	e1 := sdk.NewEvent("transfer", sdk.NewAttribute("sender", "foo"))
	e2 := sdk.NewEvent("transfer", sdk.NewAttribute("sender", "bar"))
	a := sdk.Events{e1}
	b := sdk.Events{e2}
	c := a.AppendEvents(b)
	s.Require().Equal(c, sdk.Events{e1, e2})
	s.Require().Equal(c, sdk.Events{e1}.AppendEvent(sdk.NewEvent("transfer", sdk.NewAttribute("sender", "bar"))))
	s.Require().Equal(c, sdk.Events{e1}.AppendEvents(sdk.Events{e2}))
}

func (s *eventsTestSuite) TestAppendAttributes() {
	e := sdk.NewEvent("transfer", sdk.NewAttribute("sender", "foo"))
	e = e.AppendAttributes(sdk.NewAttribute("recipient", "bar"))
	s.Require().Len(e.Attributes, 2)
	s.Require().Equal(e, sdk.NewEvent("transfer", sdk.NewAttribute("sender", "foo"), sdk.NewAttribute("recipient", "bar")))
}

func (s *eventsTestSuite) TestGetAttributes() {
	e := sdk.NewEvent("transfer", sdk.NewAttribute("sender", "foo"))
	e = e.AppendAttributes(sdk.NewAttribute("recipient", "bar"))
	attr, found := e.GetAttribute("recipient")
	s.Require().True(found)
	s.Require().Equal(attr, sdk.NewAttribute("recipient", "bar"))
	_, found = e.GetAttribute("foo")
	s.Require().False(found)

	events := sdk.Events{e}.AppendEvent(sdk.NewEvent("message", sdk.NewAttribute("sender", "bar")))
	attrs, found := events.GetAttributes("sender")
	s.Require().True(found)
	s.Require().Len(attrs, 2)
	s.Require().Equal(attrs[0], sdk.NewAttribute("sender", "foo"))
	s.Require().Equal(attrs[1], sdk.NewAttribute("sender", "bar"))
	_, found = events.GetAttributes("foo")
	s.Require().False(found)
}

func (s *eventsTestSuite) TestEmptyEvents() {
	s.Require().Equal(sdk.EmptyEvents(), sdk.Events{})
}

func (s *eventsTestSuite) TestAttributeString() {
	s.Require().Equal("foo: bar", sdk.NewAttribute("foo", "bar").String())
}

func (s *eventsTestSuite) TestToABCIEvents() {
	e := sdk.Events{sdk.NewEvent("transfer", sdk.NewAttribute("sender", "foo"))}
	abciEvents := e.ToABCIEvents()
	s.Require().Len(abciEvents, 1)
	s.Require().Equal(abciEvents[0].Type, e[0].Type)
	s.Require().Equal(abciEvents[0].Attributes, e[0].Attributes)
}

func (s *eventsTestSuite) TestEventManager() {
	em := sdk.NewEventManager()
	event := sdk.NewEvent("reward", sdk.NewAttribute("x", "y"))
	events := sdk.Events{sdk.NewEvent("transfer", sdk.NewAttribute("sender", "foo"))}

	em.EmitEvents(events)
	em.EmitEvent(event)

	s.Require().Len(em.Events(), 2)
	s.Require().Equal(em.Events(), events.AppendEvent(event))
}

func (s *eventsTestSuite) TestEmitTypedEvent() {
	s.Run("deterministic key-value order", func() {
		for i := 0; i < 10; i++ {
			em := sdk.NewEventManager()
			coin := sdk.NewCoin("fakedenom", math.NewInt(1999999))
			s.Require().NoError(em.EmitTypedEvent(&coin))
			s.Require().Len(em.Events(), 1)
			attrs := em.Events()[0].Attributes
			s.Require().Len(attrs, 2)
			s.Require().Equal(attrs[0].Key, "amount")
			s.Require().Equal(attrs[1].Key, "denom")
		}
	})
}

func (s *eventsTestSuite) TestEventManagerTypedEvents() {
	em := sdk.NewEventManager()

	coin := sdk.NewCoin("fakedenom", math.NewInt(1999999))
	cat := testdata.Cat{
		Moniker: "Garfield",
		Lives:   6,
	}
	animal, err := codectypes.NewAnyWithValue(&cat)
	s.Require().NoError(err)
	hasAnimal := testdata.HasAnimal{
		X:      1000,
		Animal: animal,
	}

	s.Require().NoError(em.EmitTypedEvents(&coin))
	s.Require().NoError(em.EmitTypedEvent(&hasAnimal))
	s.Require().Len(em.Events(), 2)

	msg1, err := sdk.ParseTypedEvent(em.Events().ToABCIEvents()[0])
	s.Require().NoError(err)
	s.Require().Equal(coin.String(), msg1.String())
	s.Require().Equal(reflect.TypeOf(&coin), reflect.TypeOf(msg1))

	msg2, err := sdk.ParseTypedEvent(em.Events().ToABCIEvents()[1])
	s.Require().NoError(err)
	s.Require().Equal(reflect.TypeOf(&hasAnimal), reflect.TypeOf(msg2))
	response := msg2.(*testdata.HasAnimal)
	s.Require().Equal(hasAnimal.Animal.String(), response.Animal.String())
}

func (s *eventsTestSuite) TestStringifyEvents() {
	cases := []struct {
		name       string
		events     sdk.Events
		expTxtStr  string
		expJSONStr string
	}{
		{
			name: "default",
			events: sdk.Events{
				sdk.NewEvent("message", sdk.NewAttribute(sdk.AttributeKeySender, "foo")),
				sdk.NewEvent("message", sdk.NewAttribute(sdk.AttributeKeyModule, "bank")),
			},
			expTxtStr:  "\t\t- message\n\t\t\t- sender: foo\n\t\t- message\n\t\t\t- module: bank",
			expJSONStr: "[{\"type\":\"message\",\"attributes\":[{\"key\":\"sender\",\"value\":\"foo\"}]},{\"type\":\"message\",\"attributes\":[{\"key\":\"module\",\"value\":\"bank\"}]}]",
		},
		{
			name: "multiple events with same attributes",
			events: sdk.Events{
				sdk.NewEvent(
					"message",
					sdk.NewAttribute(sdk.AttributeKeyModule, "staking"),
					sdk.NewAttribute(sdk.AttributeKeySender, "cosmos1foo"),
				),
				sdk.NewEvent("message", sdk.NewAttribute(sdk.AttributeKeySender, "foo")),
			},
			expTxtStr:  "\t\t- message\n\t\t\t- module: staking\n\t\t\t- sender: cosmos1foo\n\t\t- message\n\t\t\t- sender: foo",
			expJSONStr: `[{"type":"message","attributes":[{"key":"module","value":"staking"},{"key":"sender","value":"cosmos1foo"}]},{"type":"message","attributes":[{"key":"sender","value":"foo"}]}]`,
		},
	}

	for _, test := range cases {
		se := sdk.StringifyEvents(test.events.ToABCIEvents())
		s.Require().Equal(test.expTxtStr, se.String())
		bz, err := json.Marshal(se)
		s.Require().NoError(err)
		s.Require().Equal(test.expJSONStr, string(bz))
	}
}

func (s *eventsTestSuite) TestMarkEventsToIndex() {
	events := []abci.Event{
		{
			Type: "message",
			Attributes: []abci.EventAttribute{
				{Key: "sender", Value: "foo"},
				{Key: "recipient", Value: "bar"},
			},
		},
		{
			Type: "staking",
			Attributes: []abci.EventAttribute{
				{Key: "deposit", Value: "5"},
				{Key: "unbond", Value: "10"},
			},
		},
	}

	testCases := map[string]struct {
		events   []abci.Event
		indexSet map[string]struct{}
		expected []abci.Event
	}{
		"empty index set": {
			events: events,
			expected: []abci.Event{
				{
					Type: "message",
					Attributes: []abci.EventAttribute{
						{Key: "sender", Value: "foo", Index: true},
						{Key: "recipient", Value: "bar", Index: true},
					},
				},
				{
					Type: "staking",
					Attributes: []abci.EventAttribute{
						{Key: "deposit", Value: "5", Index: true},
						{Key: "unbond", Value: "10", Index: true},
					},
				},
			},
			indexSet: map[string]struct{}{},
		},
		"index some events": {
			events: events,
			expected: []abci.Event{
				{
					Type: "message",
					Attributes: []abci.EventAttribute{
						{Key: "sender", Value: "foo", Index: true},
						{Key: "recipient", Value: "bar"},
					},
				},
				{
					Type: "staking",
					Attributes: []abci.EventAttribute{
						{Key: "deposit", Value: "5", Index: true},
						{Key: "unbond", Value: "10"},
					},
				},
			},
			indexSet: map[string]struct{}{
				"message.sender":  {},
				"staking.deposit": {},
			},
		},
		"index all events": {
			events: events,
			expected: []abci.Event{
				{
					Type: "message",
					Attributes: []abci.EventAttribute{
						{Key: "sender", Value: "foo", Index: true},
						{Key: "recipient", Value: "bar", Index: true},
					},
				},
				{
					Type: "staking",
					Attributes: []abci.EventAttribute{
						{Key: "deposit", Value: "5", Index: true},
						{Key: "unbond", Value: "10", Index: true},
					},
				},
			},
			indexSet: map[string]struct{}{
				"message.sender":    {},
				"message.recipient": {},
				"staking.deposit":   {},
				"staking.unbond":    {},
			},
		},
	}

	for name, tc := range testCases {
		s.T().Run(name, func(_ *testing.T) {
			s.Require().Equal(tc.expected, sdk.MarkEventsToIndex(tc.events, tc.indexSet))
		})
	}
}
