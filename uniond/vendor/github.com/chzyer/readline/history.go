package readline

import (
	"bufio"
	"container/list"
	"fmt"
	"os"
	"strings"
	"sync"
)

type hisItem struct {
	Source  []rune
	Version int64
	Tmp     []rune
}

func (h *hisItem) Clean() {
	h.Source = nil
	h.Tmp = nil
}

type opHistory struct {
	cfg        *Config
	history    *list.List
	historyVer int64
	current    *list.Element
	fd         *os.File
	fdLock     sync.Mutex
	enable     bool
}

func newOpHistory(cfg *Config) (o *opHistory) {
	o = &opHistory{
		cfg:     cfg,
		history: list.New(),
		enable:  true,
	}
	return o
}

func (o *opHistory) Reset() {
	o.history = list.New()
	o.current = nil
}

func (o *opHistory) IsHistoryClosed() bool {
	o.fdLock.Lock()
	defer o.fdLock.Unlock()
	return o.fd.Fd() == ^(uintptr(0))
}

func (o *opHistory) Init() {
	if o.IsHistoryClosed() {
		o.initHistory()
	}
}

func (o *opHistory) initHistory() {
	if o.cfg.HistoryFile != "" {
		o.historyUpdatePath(o.cfg.HistoryFile)
	}
}

// only called by newOpHistory
func (o *opHistory) historyUpdatePath(path string) {
	o.fdLock.Lock()
	defer o.fdLock.Unlock()
	f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_RDWR, 0666)
	if err != nil {
		return
	}
	o.fd = f
	r := bufio.NewReader(o.fd)
	total := 0
	for ; ; total++ {
		line, err := r.ReadString('\n')
		if err != nil {
			break
		}
		// ignore the empty line
		line = strings.TrimSpace(line)
		if len(line) == 0 {
			continue
		}
		o.Push([]rune(line))
		o.Compact()
	}
	if total > o.cfg.HistoryLimit {
		o.rewriteLocked()
	}
	o.historyVer++
	o.Push(nil)
	return
}

func (o *opHistory) Compact() {
	for o.history.Len() > o.cfg.HistoryLimit && o.history.Len() > 0 {
		o.history.Remove(o.history.Front())
	}
}

func (o *opHistory) Rewrite() {
	o.fdLock.Lock()
	defer o.fdLock.Unlock()
	o.rewriteLocked()
}

func (o *opHistory) rewriteLocked() {
	if o.cfg.HistoryFile == "" {
		return
	}

	tmpFile := o.cfg.HistoryFile + ".tmp"
	fd, err := os.OpenFile(tmpFile, os.O_CREATE|os.O_WRONLY|os.O_TRUNC|os.O_APPEND, 0666)
	if err != nil {
		return
	}

	buf := bufio.NewWriter(fd)
	for elem := o.history.Front(); elem != nil; elem = elem.Next() {
		buf.WriteString(string(elem.Value.(*hisItem).Source) + "\n")
	}
	buf.Flush()

	// replace history file
	if err = os.Rename(tmpFile, o.cfg.HistoryFile); err != nil {
		fd.Close()
		return
	}

	if o.fd != nil {
		o.fd.Close()
	}
	// fd is write only, just satisfy what we need.
	o.fd = fd
}

func (o *opHistory) Close() {
	o.fdLock.Lock()
	defer o.fdLock.Unlock()
	if o.fd != nil {
		o.fd.Close()
	}
}

func (o *opHistory) FindBck(isNewSearch bool, rs []rune, start int) (int, *list.Element) {
	for elem := o.current; elem != nil; elem = elem.Prev() {
		item := o.showItem(elem.Value)
		if isNewSearch {
			start += len(rs)
		}
		if elem == o.current {
			if len(item) >= start {
				item = item[:start]
			}
		}
		idx := runes.IndexAllBckEx(item, rs, o.cfg.HistorySearchFold)
		if idx < 0 {
			continue
		}
		return idx, elem
	}
	return -1, nil
}

func (o *opHistory) FindFwd(isNewSearch bool, rs []rune, start int) (int, *list.Element) {
	for elem := o.current; elem != nil; elem = elem.Next() {
		item := o.showItem(elem.Value)
		if isNewSearch {
			start -= len(rs)
			if start < 0 {
				start = 0
			}
		}
		if elem == o.current {
			if len(item)-1 >= start {
				item = item[start:]
			} else {
				continue
			}
		}
		idx := runes.IndexAllEx(item, rs, o.cfg.HistorySearchFold)
		if idx < 0 {
			continue
		}
		if elem == o.current {
			idx += start
		}
		return idx, elem
	}
	return -1, nil
}

func (o *opHistory) showItem(obj interface{}) []rune {
	item := obj.(*hisItem)
	if item.Version == o.historyVer {
		return item.Tmp
	}
	return item.Source
}

func (o *opHistory) Prev() []rune {
	if o.current == nil {
		return nil
	}
	current := o.current.Prev()
	if current == nil {
		return nil
	}
	o.current = current
	return runes.Copy(o.showItem(current.Value))
}

func (o *opHistory) Next() ([]rune, bool) {
	if o.current == nil {
		return nil, false
	}
	current := o.current.Next()
	if current == nil {
		return nil, false
	}

	o.current = current
	return runes.Copy(o.showItem(current.Value)), true
}

// Disable the current history
func (o *opHistory) Disable() {
	o.enable = false
}

// Enable the current history
func (o *opHistory) Enable() {
	o.enable = true
}

func (o *opHistory) debug() {
	Debug("-------")
	for item := o.history.Front(); item != nil; item = item.Next() {
		Debug(fmt.Sprintf("%+v", item.Value))
	}
}

// save history
func (o *opHistory) New(current []rune) (err error) {

	// history deactivated
	if !o.enable {
		return nil
	}

	current = runes.Copy(current)

	// if just use last command without modify
	// just clean lastest history
	if back := o.history.Back(); back != nil {
		prev := back.Prev()
		if prev != nil {
			if runes.Equal(current, prev.Value.(*hisItem).Source) {
				o.current = o.history.Back()
				o.current.Value.(*hisItem).Clean()
				o.historyVer++
				return nil
			}
		}
	}

	if len(current) == 0 {
		o.current = o.history.Back()
		if o.current != nil {
			o.current.Value.(*hisItem).Clean()
			o.historyVer++
			return nil
		}
	}

	if o.current != o.history.Back() {
		// move history item to current command
		currentItem := o.current.Value.(*hisItem)
		// set current to last item
		o.current = o.history.Back()

		current = runes.Copy(currentItem.Tmp)
	}

	// err only can be a IO error, just report
	err = o.Update(current, true)

	// push a new one to commit current command
	o.historyVer++
	o.Push(nil)
	return
}

func (o *opHistory) Revert() {
	o.historyVer++
	o.current = o.history.Back()
}

func (o *opHistory) Update(s []rune, commit bool) (err error) {
	o.fdLock.Lock()
	defer o.fdLock.Unlock()
	s = runes.Copy(s)
	if o.current == nil {
		o.Push(s)
		o.Compact()
		return
	}
	r := o.current.Value.(*hisItem)
	r.Version = o.historyVer
	if commit {
		r.Source = s
		if o.fd != nil {
			// just report the error
			_, err = o.fd.Write([]byte(string(r.Source) + "\n"))
		}
	} else {
		r.Tmp = append(r.Tmp[:0], s...)
	}
	o.current.Value = r
	o.Compact()
	return
}

func (o *opHistory) Push(s []rune) {
	s = runes.Copy(s)
	elem := o.history.PushBack(&hisItem{Source: s})
	o.current = elem
}
