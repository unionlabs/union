package promptui

import "testing"

func TestDefinedCursors(t *testing.T) {
	t.Run("pipeCursor", func(t *testing.T) {
		p := string(pipeCursor([]rune{}))
		if p != "|" {
			t.Fatalf("%x!=%x", "|", p)
		}
	})
}

func TestCursor(t *testing.T) {
	t.Run("empty", func(t *testing.T) {
		cursor := Cursor{Cursor: pipeCursor}
		cursor.End()
		f := cursor.Format()
		if f != "|" {
			t.Errorf("% x!=% x", "|", cursor.Format())
		}

		cursor.Update("sup")
		if cursor.Format() != "sup|" {
			t.Errorf("% x!=% x", "sup|", cursor.Format())
		}
	})

	t.Run("Cursor at end, append additional", func(t *testing.T) {
		cursor := Cursor{input: []rune("a"), Cursor: pipeCursor}
		cursor.End()
		f := cursor.Format()
		if f != "a|" {
			t.Errorf("% x!=% x", "a|", cursor.Format())
		}

		cursor.Update(" hi")
		if cursor.Format() != "a hi|" {
			t.Errorf("% x!=% x", "a hi!", cursor.Format())
		}
	})

	t.Run("Cursor at at end, backspace", func(t *testing.T) {
		cursor := Cursor{input: []rune("default"), Cursor: pipeCursor}
		cursor.Place(len(cursor.input))
		cursor.Backspace()

		if cursor.Format() != "defaul|" {
			t.Errorf("expected defaul|; found %s", cursor.Format())
		}

		cursor.Update(" hi")
		if cursor.Format() != "defaul hi|" {
			t.Errorf("expected 'defaul hi|'; found '%s'", cursor.Format())
		}
	})

	t.Run("Cursor at beginning, append additional", func(t *testing.T) {
		cursor := Cursor{input: []rune("default"), Cursor: pipeCursor}
		t.Log("init", cursor.String())
		cursor.Backspace()
		if cursor.Format() != "|default" {
			t.Errorf("expected |default; found %s", cursor.Format())
		}

		cursor.Update("hi ")
		t.Log("after add", cursor.String())
		if cursor.Format() != "hi |default" {
			t.Errorf("expected 'hi |default'; found '%s'", cursor.Format())
		}
		cursor.Backspace()
		t.Log("after backspace", cursor.String())
		if cursor.Format() != "hi|default" {
			t.Errorf("expected 'hi|default'; found '%s'", cursor.Format())
		}

		cursor.Backspace()
		t.Log("after backspace", cursor.String())
		if cursor.Format() != "h|default" {
			t.Errorf("expected 'h|default'; found '%s'", cursor.Format())
		}
	})

	t.Run("Move", func(t *testing.T) {
		cursor := Cursor{input: []rune("default"), Cursor: pipeCursor}
		if cursor.Format() != "|default" {
			t.Errorf("expected |default; found %s", cursor.Format())
		}
		cursor.Move(-1)
		if cursor.Format() != "|default" {
			t.Errorf("moved backwards from beginning |default; found %s", cursor.Format())
		}

		cursor.Move(1)
		if cursor.Format() != "d|efault" {
			t.Errorf("expected 'd|efault'; found '%s'", cursor.Format())
		}
		cursor.Move(10)
		if cursor.Format() != "default|" {
			t.Errorf("expected 'default|'; found '%s'", cursor.Format())
		}
	})
}
