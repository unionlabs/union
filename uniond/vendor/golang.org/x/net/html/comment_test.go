// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package html

import (
	"bytes"
	"strings"
	"testing"
)

// TestComments exhaustively tests every 'interesting' N-byte string is
// correctly parsed as a comment. N ranges from 4+1 to 4+maxSuffixLen
// inclusive. 4 is the length of the "<!--" prefix that starts an HTML comment.
//
// 'Interesting' means that the N-4 byte suffix consists entirely of bytes
// sampled from the interestingCommentBytes const string, below. These cover
// all of the possible state transitions from comment-related parser states, as
// listed in the HTML spec (https://html.spec.whatwg.org/#comment-start-state
// and subsequent sections).
//
// The spec is written as an explicit state machine that, as a side effect,
// accumulates "the comment token's data" to a separate buffer.
// Tokenizer.readComment in this package does not have an explicit state
// machine and usually returns the comment text as a sub-slice of the input,
// between the opening '<' and closing '>' or EOF. This test confirms that the
// two algorithms match.
func TestComments(t *testing.T) {
	const prefix = "<!--"
	const maxSuffixLen = 6
	buffer := make([]byte, 0, len(prefix)+maxSuffixLen)
	testAllComments(t, append(buffer, prefix...))
}

// NUL isn't in this list, even though the HTML spec sections 13.2.5.43 -
// 13.2.5.52 mentions it. It's not interesting in terms of state transitions.
// It's equivalent to any other non-interesting byte (other than being replaced
// by U+FFFD REPLACEMENT CHARACTER).
//
// EOF isn't in this list. The HTML spec treats EOF as "an input character" but
// testOneComment below breaks the loop instead.
//
// 'x' represents all other "non-interesting" comment bytes.
var interestingCommentBytes = [...]byte{
	'!', '-', '<', '>', 'x',
}

// testAllComments recursively fills in buffer[len(buffer):cap(buffer)] with
// interesting bytes and then tests that this package's tokenization matches
// the HTML spec.
//
// Precondition: len(buffer) < cap(buffer)
// Precondition: string(buffer[:4]) == "<!--"
func testAllComments(t *testing.T, buffer []byte) {
	for _, interesting := range interestingCommentBytes {
		b := append(buffer, interesting)
		testOneComment(t, b)
		if len(b) < cap(b) {
			testAllComments(t, b)
		}
	}
}

func testOneComment(t *testing.T, b []byte) {
	z := NewTokenizer(bytes.NewReader(b))
	if next := z.Next(); next != CommentToken {
		t.Fatalf("Next(%q): got %v, want %v", b, next, CommentToken)
	}
	gotRemainder := string(b[len(z.Raw()):])
	gotComment := string(z.Text())

	i := len("<!--")
	wantBuffer := []byte(nil)
loop:
	for state := 43; ; {
		// Consume the next input character, handling EOF.
		if i >= len(b) {
			break
		}
		nextInputCharacter := b[i]
		i++

		switch state {
		case 43: // 13.2.5.43 Comment start state.
			switch nextInputCharacter {
			case '-':
				state = 44
			case '>':
				break loop
			default:
				i-- // Reconsume.
				state = 45
			}

		case 44: // 13.2.5.44 Comment start dash state.
			switch nextInputCharacter {
			case '-':
				state = 51
			case '>':
				break loop
			default:
				wantBuffer = append(wantBuffer, '-')
				i-- // Reconsume.
				state = 45
			}

		case 45: // 13.2.5.45 Comment state.
			switch nextInputCharacter {
			case '-':
				state = 50
			case '<':
				wantBuffer = append(wantBuffer, '<')
				state = 46
			default:
				wantBuffer = append(wantBuffer, nextInputCharacter)
			}

		case 46: // 13.2.5.46 Comment less-than sign state.
			switch nextInputCharacter {
			case '!':
				wantBuffer = append(wantBuffer, '!')
				state = 47
			case '<':
				wantBuffer = append(wantBuffer, '<')
				state = 46
			default:
				i-- // Reconsume.
				state = 45
			}

		case 47: // 13.2.5.47 Comment less-than sign bang state.
			switch nextInputCharacter {
			case '-':
				state = 48
			default:
				i-- // Reconsume.
				state = 45
			}

		case 48: // 13.2.5.48 Comment less-than sign bang dash state.
			switch nextInputCharacter {
			case '-':
				state = 49
			default:
				i-- // Reconsume.
				state = 50
			}

		case 49: // 13.2.5.49 Comment less-than sign bang dash dash state.
			switch nextInputCharacter {
			case '>':
				break loop
			default:
				i-- // Reconsume.
				state = 51
			}

		case 50: // 13.2.5.50 Comment end dash state.
			switch nextInputCharacter {
			case '-':
				state = 51
			default:
				wantBuffer = append(wantBuffer, '-')
				i-- // Reconsume.
				state = 45
			}

		case 51: // 13.2.5.51 Comment end state.
			switch nextInputCharacter {
			case '!':
				state = 52
			case '-':
				wantBuffer = append(wantBuffer, '-')
			case '>':
				break loop
			default:
				wantBuffer = append(wantBuffer, "--"...)
				i-- // Reconsume.
				state = 45
			}

		case 52: // 13.2.5.52 Comment end bang state.
			switch nextInputCharacter {
			case '-':
				wantBuffer = append(wantBuffer, "--!"...)
				state = 50
			case '>':
				break loop
			default:
				wantBuffer = append(wantBuffer, "--!"...)
				i-- // Reconsume.
				state = 45
			}

		default:
			t.Fatalf("input=%q: unexpected state %d", b, state)
		}
	}

	wantRemainder := ""
	if i < len(b) {
		wantRemainder = string(b[i:])
	}
	wantComment := string(wantBuffer)
	if (gotComment != wantComment) || (gotRemainder != wantRemainder) {
		t.Errorf("input=%q\ngot:  %q + %q\nwant: %q + %q",
			b, gotComment, gotRemainder, wantComment, wantRemainder)
		return
	}

	// suffix is the "N-4 byte suffix" per the TestComments comment.
	suffix := string(b[4:])

	// Test that a round trip, rendering (escaped) and re-parsing, of a comment
	// token (with that suffix as the Token.Data) preserves that string.
	tok := Token{
		Type: CommentToken,
		Data: suffix,
	}
	z2 := NewTokenizer(strings.NewReader(tok.String()))
	if next := z2.Next(); next != CommentToken {
		t.Fatalf("round-trip Next(%q): got %v, want %v", suffix, next, CommentToken)
	}
	gotComment2 := string(z2.Text())
	if gotComment2 != suffix {
		t.Errorf("round-trip\ngot:  %q\nwant: %q", gotComment2, suffix)
		return
	}
}

// This table below summarizes the HTML-comment-related state machine from
// 13.2.5.43 "Comment start state" and subsequent sections.
// https://html.spec.whatwg.org/#comment-start-state
//
// Get to state 13.2.5.43 after seeing "<!--". Specifically, starting from the
// initial 13.2.5.1 "Data state":
//   - "<"  moves to 13.2.5.6  "Tag open state",
//   - "!"  moves to 13.2.5.42 "Markup declaration open state",
//   - "--" moves to 13.2.5.43 "Comment start state".
// Each of these transitions are the only way to get to the 6/42/43 states.
//
// State   !         -         <         >         NUL       EOF       default   HTML spec section
// 43      ...       s44       ...       s01.T.E0  ...       ...       r45       13.2.5.43 Comment start state
// 44      ...       s51       ...       s01.T.E0  ...       T.Z.E1    r45.A-    13.2.5.44 Comment start dash state
// 45      ...       s50       s46.A<    ...       t45.A?.E2 T.Z.E1    t45.Ax    13.2.5.45 Comment state
// 46      s47.A!    ...       t46.A<    ...       ...       ...       r45       13.2.5.46 Comment less-than sign state
// 47      ...       s48       ...       ...       ...       ...       r45       13.2.5.47 Comment less-than sign bang state
// 48      ...       s49       ...       ...       ...       ...       r50       13.2.5.48 Comment less-than sign bang dash state
// 49      ...       ...       ...       s01.T     ...       T.Z.E1    r51.E3    13.2.5.49 Comment less-than sign bang dash dash state
// 50      ...       s51       ...       ...       ...       T.Z.E1    r45.A-    13.2.5.50 Comment end dash state
// 51      s52       t51.A-    ...       s01.T     ...       T.Z.E1    r45.A--   13.2.5.51 Comment end state
// 52      ...       s50.A--!  ...       s01.T.E4  ...       T.Z.E1    r45.A--!  13.2.5.52 Comment end bang state
//
// State 43 is the "Comment start state" meaning that we've only seen "<!--"
// and nothing else. Similarly, state 44 means that we've only seen "<!---",
// with three dashes, and nothing else. For the other states, we deduce
// (working backwards) that the immediate prior input must be:
//   - 45  something that's not '-'
//   - 46  "<"
//   - 47  "<!"
//   - 48  "<!-"
//   - 49  "<!--"  not including the opening "<!--"
//   - 50  "-"     not including the opening "<!--" and also not "--"
//   - 51  "--"    not including the opening "<!--"
//   - 52  "--!"
//
// The table cell actions:
//   - ...   do the default action
//   - A!    append "!"      to the comment token's data.
//   - A-    append "-"      to the comment token's data.
//   - A--   append "--"     to the comment token's data.
//   - A--!  append "--!"    to the comment token's data.
//   - A<    append "<"      to the comment token's data.
//   - A?    append "\uFFFD" to the comment token's data.
//   - Ax    append the current input character to the comment token's data.
//   - E0    parse error (abrupt-closing-of-empty-comment).
//   - E1    parse error (eof-in-comment).
//   - E2    parse error (unexpected-null-character).
//   - E3    parse error (nested-comment).
//   - E4    parse error (incorrectly-closed-comment).
//   - T     emit the current comment token.
//   - Z     emit an end-of-file token.
//   - rNN   reconsume in the 13.2.5.NN     state (after any A* or E* operations).
//   - s01   switch to the    13.2.5.1 Data state (after any A* or E* operations).
//   - sNN   switch to the    13.2.5.NN     state (after any A* or E* operations).
//   - tNN   stay in the      13.2.5.NN     state (after any A* or E* operations).
//
// The E* actions are called errors in the HTML spec but they are not fatal
// (https://html.spec.whatwg.org/#parse-errors says "may [but not must] abort
// the parser"). They are warnings that, in practice, browsers simply ignore.
