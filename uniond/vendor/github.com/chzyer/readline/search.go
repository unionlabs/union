package readline

import (
	"bytes"
	"container/list"
	"fmt"
	"io"
)

const (
	S_STATE_FOUND = iota
	S_STATE_FAILING
)

const (
	S_DIR_BCK = iota
	S_DIR_FWD
)

type opSearch struct {
	inMode    bool
	state     int
	dir       int
	source    *list.Element
	w         io.Writer
	buf       *RuneBuffer
	data      []rune
	history   *opHistory
	cfg       *Config
	markStart int
	markEnd   int
	width     int
}

func newOpSearch(w io.Writer, buf *RuneBuffer, history *opHistory, cfg *Config, width int) *opSearch {
	return &opSearch{
		w:       w,
		buf:     buf,
		cfg:     cfg,
		history: history,
		width:   width,
	}
}

func (o *opSearch) OnWidthChange(newWidth int) {
	o.width = newWidth
}

func (o *opSearch) IsSearchMode() bool {
	return o.inMode
}

func (o *opSearch) SearchBackspace() {
	if len(o.data) > 0 {
		o.data = o.data[:len(o.data)-1]
		o.search(true)
	}
}

func (o *opSearch) findHistoryBy(isNewSearch bool) (int, *list.Element) {
	if o.dir == S_DIR_BCK {
		return o.history.FindBck(isNewSearch, o.data, o.buf.idx)
	}
	return o.history.FindFwd(isNewSearch, o.data, o.buf.idx)
}

func (o *opSearch) search(isChange bool) bool {
	if len(o.data) == 0 {
		o.state = S_STATE_FOUND
		o.SearchRefresh(-1)
		return true
	}
	idx, elem := o.findHistoryBy(isChange)
	if elem == nil {
		o.SearchRefresh(-2)
		return false
	}
	o.history.current = elem

	item := o.history.showItem(o.history.current.Value)
	start, end := 0, 0
	if o.dir == S_DIR_BCK {
		start, end = idx, idx+len(o.data)
	} else {
		start, end = idx, idx+len(o.data)
		idx += len(o.data)
	}
	o.buf.SetWithIdx(idx, item)
	o.markStart, o.markEnd = start, end
	o.SearchRefresh(idx)
	return true
}

func (o *opSearch) SearchChar(r rune) {
	o.data = append(o.data, r)
	o.search(true)
}

func (o *opSearch) SearchMode(dir int) bool {
	if o.width == 0 {
		return false
	}
	alreadyInMode := o.inMode
	o.inMode = true
	o.dir = dir
	o.source = o.history.current
	if alreadyInMode {
		o.search(false)
	} else {
		o.SearchRefresh(-1)
	}
	return true
}

func (o *opSearch) ExitSearchMode(revert bool) {
	if revert {
		o.history.current = o.source
		o.buf.Set(o.history.showItem(o.history.current.Value))
	}
	o.markStart, o.markEnd = 0, 0
	o.state = S_STATE_FOUND
	o.inMode = false
	o.source = nil
	o.data = nil
}

func (o *opSearch) SearchRefresh(x int) {
	if x == -2 {
		o.state = S_STATE_FAILING
	} else if x >= 0 {
		o.state = S_STATE_FOUND
	}
	if x < 0 {
		x = o.buf.idx
	}
	x = o.buf.CurrentWidth(x)
	x += o.buf.PromptLen()
	x = x % o.width

	if o.markStart > 0 {
		o.buf.SetStyle(o.markStart, o.markEnd, "4")
	}

	lineCnt := o.buf.CursorLineCount()
	buf := bytes.NewBuffer(nil)
	buf.Write(bytes.Repeat([]byte("\n"), lineCnt))
	buf.WriteString("\033[J")
	if o.state == S_STATE_FAILING {
		buf.WriteString("failing ")
	}
	if o.dir == S_DIR_BCK {
		buf.WriteString("bck")
	} else if o.dir == S_DIR_FWD {
		buf.WriteString("fwd")
	}
	buf.WriteString("-i-search: ")
	buf.WriteString(string(o.data))         // keyword
	buf.WriteString("\033[4m \033[0m")      // _
	fmt.Fprintf(buf, "\r\033[%dA", lineCnt) // move prev
	if x > 0 {
		fmt.Fprintf(buf, "\033[%dC", x) // move forward
	}
	o.w.Write(buf.Bytes())
}
