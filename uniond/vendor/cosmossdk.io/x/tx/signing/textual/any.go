package textual

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"github.com/cosmos/cosmos-proto/anyutil"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
	"google.golang.org/protobuf/types/dynamicpb"
	"google.golang.org/protobuf/types/known/anypb"
)

// Any messages are rendered as a one-screen header of "Object: <type_url>"
// followed by an indented rendering of the contained message.

// anyValueRenderer is a ValueRenderer for google.protobuf.Any messages.
type anyValueRenderer struct {
	tr *SignModeHandler
}

// NewAnyValueRenderer returns a ValueRenderer for google.protobuf.Any messages.
func NewAnyValueRenderer(t *SignModeHandler) ValueRenderer {
	return anyValueRenderer{tr: t}
}

// Format implements the ValueRenderer interface.
func (ar anyValueRenderer) Format(ctx context.Context, v protoreflect.Value) ([]Screen, error) {
	msg := v.Message().Interface()
	anymsg := &anypb.Any{}
	err := coerceToMessage(msg, anymsg)
	if err != nil {
		return nil, err
	}

	internalMsg, err := anyutil.Unpack(anymsg, ar.tr.fileResolver, ar.tr.typeResolver)
	if err != nil {
		return nil, err
	}

	vr, err := ar.tr.GetMessageValueRenderer(internalMsg.ProtoReflect().Descriptor())
	if err != nil {
		return nil, err
	}

	subscreens, err := vr.Format(ctx, protoreflect.ValueOfMessage(internalMsg.ProtoReflect()))
	if err != nil {
		return nil, err
	}

	// The Any value renderer suppresses emission of the object header for all
	// messages that go through the messageValueRenderer.
	omitHeader := 0
	msgValRenderer, isMsgRenderer := vr.(*messageValueRenderer)
	if isMsgRenderer {
		if subscreens[0].Content != msgValRenderer.header() {
			return nil, fmt.Errorf("any internal message expects %s, got %s", msgValRenderer.header(), subscreens[0].Content)
		}

		omitHeader = 1
	}

	screens := make([]Screen, (1-omitHeader)+len(subscreens))
	screens[0].Content = anymsg.GetTypeUrl()
	for i, subscreen := range subscreens[omitHeader:] {
		subscreen.Indent += 1 - omitHeader
		screens[i+1] = subscreen
	}

	return screens, nil
}

// Parse implements the ValueRenderer interface.
func (ar anyValueRenderer) Parse(ctx context.Context, screens []Screen) (protoreflect.Value, error) {
	if len(screens) == 0 {
		return nilValue, errors.New("expect at least one screen")
	}
	if screens[0].Indent != 0 {
		return nilValue, fmt.Errorf("bad indentation: want 0, got %d", screens[0].Indent)
	}

	typeURL := screens[0].Content
	msgType, err := ar.tr.typeResolver.FindMessageByURL(typeURL)
	if errors.Is(err, protoregistry.NotFound) {
		// If the proto v2 registry doesn't have this message, then we use
		// protoFiles (which can e.g. be initialized to gogo's MergedRegistry)
		// to retrieve the message descriptor, and then use dynamicpb on that
		// message descriptor to create a proto.Message
		typeURL := strings.TrimPrefix(typeURL, "/")

		msgDesc, err := ar.tr.fileResolver.FindDescriptorByName(protoreflect.FullName(typeURL))
		if err != nil {
			return nilValue, fmt.Errorf("textual protoFiles does not have descriptor %s: %w", typeURL, err)
		}

		msgType = dynamicpb.NewMessageType(msgDesc.(protoreflect.MessageDescriptor))
	} else if err != nil {
		return nilValue, err
	}
	vr, err := ar.tr.GetMessageValueRenderer(msgType.Descriptor())
	if err != nil {
		return nilValue, err
	}

	subscreens := make([]Screen, len(screens)-1)
	for i := 1; i < len(screens); i++ {
		if screens[i].Indent < 1 {
			return nilValue, fmt.Errorf("bad indent for subscreen %d: %d", i, screens[i].Indent)
		}
		subscreens[i-1] = screens[i]
		subscreens[i-1].Indent--
	}

	// Append with "%s object" if the message goes through the default
	// messageValueRenderer (the header() method does this for us), and
	// add a level of indentation.
	msgValRenderer, isMsgRenderer := vr.(*messageValueRenderer)
	if isMsgRenderer {
		for i := range subscreens {
			subscreens[i].Indent++
		}

		subscreens = append([]Screen{{Content: msgValRenderer.header()}}, subscreens...)
	}

	internalMsg, err := vr.Parse(ctx, subscreens)
	if err != nil {
		return nilValue, err
	}

	anyMsg, err := anyutil.New(internalMsg.Message().Interface())
	if err != nil {
		return nilValue, err
	}

	return protoreflect.ValueOfMessage(anyMsg.ProtoReflect()), nil
}
