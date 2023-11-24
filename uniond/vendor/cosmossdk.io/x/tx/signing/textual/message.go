package textual

import (
	"context"
	"errors"
	"fmt"
	"regexp"
	"sort"
	"strconv"
	"strings"

	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
)

var (
	headerRegex  = regexp.MustCompile(`(\d+) .+`)
	elementRegex = regexp.MustCompile(`(.+) \(\d+\/\d+\)`)
)

type messageValueRenderer struct {
	tr      *SignModeHandler
	msgDesc protoreflect.MessageDescriptor
	fds     []protoreflect.FieldDescriptor
}

func NewMessageValueRenderer(t *SignModeHandler, msgDesc protoreflect.MessageDescriptor) ValueRenderer {
	fields := msgDesc.Fields()
	fds := make([]protoreflect.FieldDescriptor, 0, fields.Len())
	for i := 0; i < fields.Len(); i++ {
		fds = append(fds, fields.Get(i))
	}
	sort.Slice(fds, func(i, j int) bool { return fds[i].Number() < fds[j].Number() })

	return &messageValueRenderer{tr: t, msgDesc: msgDesc, fds: fds}
}

func (mr *messageValueRenderer) header() string {
	return fmt.Sprintf("%s object", mr.msgDesc.Name())
}

func (mr *messageValueRenderer) Format(ctx context.Context, v protoreflect.Value) ([]Screen, error) {
	fullName := v.Message().Descriptor().FullName()
	wantFullName := mr.msgDesc.FullName()
	if fullName != wantFullName {
		return nil, fmt.Errorf(`bad message type: want "%s", got "%s"`, wantFullName, fullName)
	}

	screens := make([]Screen, 1)
	screens[0].Content = mr.header()

	for _, fd := range mr.fds {
		if !v.Message().Has(fd) {
			// Skip default values.
			continue
		}
		vr, err := mr.tr.GetFieldValueRenderer(fd)
		if err != nil {
			return nil, err
		}

		var subscreens []Screen
		if fd.IsList() {
			if r, ok := vr.(RepeatedValueRenderer); ok {
				// If the field is a list, and handles its own repeated rendering
				subscreens, err = r.FormatRepeated(ctx, v.Message().Get(fd))
			} else {
				// If the field is a list, we need to format each element of the list
				subscreens, err = mr.formatRepeated(ctx, v.Message().Get(fd), fd)
			}
		} else {
			// If the field is not list, we need to format the field
			subscreens, err = vr.Format(ctx, v.Message().Get(fd))
		}

		if err != nil {
			return nil, err
		}
		if len(subscreens) == 0 {
			return nil, fmt.Errorf("empty rendering for field %s", fd.Name())
		}

		headerScreen := Screen{
			Title:   toSentenceCase(string(fd.Name())),
			Content: subscreens[0].Content,
			Indent:  subscreens[0].Indent + 1,
			Expert:  subscreens[0].Expert,
		}
		screens = append(screens, headerScreen)

		for i := 1; i < len(subscreens); i++ {
			extraScreen := Screen{
				Title:   subscreens[i].Title,
				Content: subscreens[i].Content,
				Indent:  subscreens[i].Indent + 1,
				Expert:  subscreens[i].Expert,
			}
			screens = append(screens, extraScreen)
		}
	}

	return screens, nil
}

func (mr *messageValueRenderer) formatRepeated(ctx context.Context, v protoreflect.Value, fd protoreflect.FieldDescriptor) ([]Screen, error) {
	vr, err := mr.tr.GetFieldValueRenderer(fd)
	if err != nil {
		return nil, err
	}

	l := v.List()
	if l == nil {
		return nil, fmt.Errorf("got non-List value %T", l)
	}

	screens := make([]Screen, 1)
	// <field_name>: <int> <field_kind>
	screens[0].Content = fmt.Sprintf("%d %s", l.Len(), toSentenceCase(getKind(fd)))

	for i := 0; i < l.Len(); i++ {
		subscreens, err := vr.Format(ctx, l.Get(i))
		if err != nil {
			return nil, err
		}

		if len(subscreens) == 0 {
			return nil, errors.New("empty rendering")
		}

		headerScreen := Screen{
			// <field_name> (<int>/<int>)
			Title: fmt.Sprintf("%s (%d/%d)", toSentenceCase(string(fd.Name())), i+1, l.Len()),
			// <value rendered 1st line>
			Content: subscreens[0].Content,
			Indent:  subscreens[0].Indent + 1,
			Expert:  subscreens[0].Expert,
		}
		screens = append(screens, headerScreen)

		// <optional value rendered in the next lines>
		for i := 1; i < len(subscreens); i++ {
			extraScreen := Screen{
				Title:   subscreens[i].Title,
				Content: subscreens[i].Content,
				Indent:  subscreens[i].Indent + 1,
				Expert:  subscreens[i].Expert,
			}
			screens = append(screens, extraScreen)
		}
	}

	// End of <field_name>
	terminalScreen := Screen{
		Content: fmt.Sprintf("End of %s", toSentenceCase(string(fd.Name()))),
	}
	screens = append(screens, terminalScreen)
	return screens, nil
}

// getKind returns the field kind: if the field is a protobuf
// message, then we return the message's name. Or else, we
// return the protobuf kind.
func getKind(fd protoreflect.FieldDescriptor) string {
	if fd.Kind() == protoreflect.MessageKind {
		return string(fd.Message().Name())
	} else if fd.Kind() == protoreflect.EnumKind {
		return string(fd.Enum().Name())
	}

	return fd.Kind().String()
}

// toSentenceCase formats a field name in sentence case, as specified in:
// https://github.com/cosmos/cosmos-sdk/blob/b6f867d0b674d62e56b27aa4d00f5b6042ebac9e/docs/architecture/adr-050-sign-mode-textual-annex1.md?plain=1#L110
func toSentenceCase(name string) string {
	if len(name) == 0 {
		return name
	}
	return strings.ToTitle(name[0:1]) + strings.ReplaceAll(name[1:], "_", " ")
}

var nilValue = protoreflect.Value{}

func (mr *messageValueRenderer) Parse(ctx context.Context, screens []Screen) (protoreflect.Value, error) {
	if len(screens) == 0 {
		return nilValue, errors.New("expect at least one screen")
	}

	if screens[0].Content != mr.header() {
		return nilValue, fmt.Errorf(`bad header: want "%s", got "%s"`, mr.header(), screens[0].Title)
	}
	if screens[0].Indent != 0 {
		return nilValue, fmt.Errorf("bad message indentation: want 0, got %d", screens[0].Indent)
	}

	msgType, err := protoregistry.GlobalTypes.FindMessageByName(mr.msgDesc.FullName())
	if err != nil {
		return nilValue, err
	}
	msg := msgType.New()
	idx := 1

	for _, fd := range mr.fds {
		if idx >= len(screens) {
			// remaining fields are default
			break
		}

		vr, err := mr.tr.GetFieldValueRenderer(fd)
		if err != nil {
			return nilValue, err
		}

		if screens[idx].Indent != 1 {
			return nilValue, fmt.Errorf("bad message indentation: want 1, got %d", screens[idx].Indent)
		}

		expectedTitle := toSentenceCase(string(fd.Name()))
		if screens[idx].Title != expectedTitle {
			// we must have skipped this fd because of a default value
			continue
		}

		// Make a new screen without the prefix
		subscreens := make([]Screen, 1)
		subscreens[0] = screens[idx]
		subscreens[0].Title = screens[idx].Title
		subscreens[0].Content = screens[idx].Content
		subscreens[0].Indent--
		idx++

		// Gather nested screens
		for idx < len(screens) && screens[idx].Indent > 1 {
			scr := screens[idx]
			scr.Indent--
			subscreens = append(subscreens, scr)
			idx++
		}

		var val protoreflect.Value
		// We have a repeated field...
		if fd.IsList() {
			nf := msg.NewField(fd)
			if r, ok := vr.(RepeatedValueRenderer); ok {
				err = r.ParseRepeated(ctx, subscreens, nf.List())
			} else {
				err = mr.parseRepeated(ctx, subscreens, nf.List(), vr)

				// Skip List Terminator
				idx++
			}
			if err != nil {
				return nilValue, err
			}
			msg.Set(fd, nf)
		} else {
			val, err = vr.Parse(ctx, subscreens)
			if err != nil {
				return nilValue, err
			}
			msg.Set(fd, val)
		}
	}

	return protoreflect.ValueOfMessage(msg), nil
}

func (mr *messageValueRenderer) parseRepeated(ctx context.Context, screens []Screen, l protoreflect.List, vr ValueRenderer) error {
	res := headerRegex.FindAllStringSubmatch(screens[0].Content, -1)
	if res == nil {
		return errors.New("failed to match <int> <field_kind>")
	}

	lengthStr := res[0][1]
	length, err := strconv.Atoi(lengthStr)
	if err != nil {
		return fmt.Errorf("malformed length: %q with error: %w", lengthStr, err)
	}

	idx := 1
	elementIndex := 1

	// <field_name> (<int>/<int>): <value rendered 1st line>
	elementRes := elementRegex.FindAllStringSubmatch(screens[idx].Title, -1)
	if elementRes == nil {
		return errors.New("element malformed")
	}
	fieldName := elementRes[0][1]

	for idx < len(screens) && elementIndex <= length {
		prefix := fmt.Sprintf("%s (%d/%d): ", fieldName, elementIndex, length)
		// Make a new screen without the prefix
		subscreens := make([]Screen, 1)
		subscreens[0] = screens[idx]
		subscreens[0].Content = strings.TrimPrefix(screens[idx].Content, prefix)
		subscreens[0].Indent--
		idx++

		// Gather nested screens
		for idx < len(screens) && screens[idx].Indent > 1 {
			scr := screens[idx]
			scr.Indent--
			subscreens = append(subscreens, scr)
			idx++
		}

		val, err := vr.Parse(ctx, subscreens)
		if err != nil {
			return err
		}

		elementIndex++
		l.Append(val)
	}
	return nil
}
