package readline

import (
	"bufio"
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"net"
	"os"
	"sync"
	"sync/atomic"
)

type MsgType int16

const (
	T_DATA = MsgType(iota)
	T_WIDTH
	T_WIDTH_REPORT
	T_ISTTY_REPORT
	T_RAW
	T_ERAW // exit raw
	T_EOF
)

type RemoteSvr struct {
	eof           int32
	closed        int32
	width         int32
	reciveChan    chan struct{}
	writeChan     chan *writeCtx
	conn          net.Conn
	isTerminal    bool
	funcWidthChan func()
	stopChan      chan struct{}

	dataBufM sync.Mutex
	dataBuf  bytes.Buffer
}

type writeReply struct {
	n   int
	err error
}

type writeCtx struct {
	msg   *Message
	reply chan *writeReply
}

func newWriteCtx(msg *Message) *writeCtx {
	return &writeCtx{
		msg:   msg,
		reply: make(chan *writeReply),
	}
}

func NewRemoteSvr(conn net.Conn) (*RemoteSvr, error) {
	rs := &RemoteSvr{
		width:      -1,
		conn:       conn,
		writeChan:  make(chan *writeCtx),
		reciveChan: make(chan struct{}),
		stopChan:   make(chan struct{}),
	}
	buf := bufio.NewReader(rs.conn)

	if err := rs.init(buf); err != nil {
		return nil, err
	}

	go rs.readLoop(buf)
	go rs.writeLoop()
	return rs, nil
}

func (r *RemoteSvr) init(buf *bufio.Reader) error {
	m, err := ReadMessage(buf)
	if err != nil {
		return err
	}
	// receive isTerminal
	if m.Type != T_ISTTY_REPORT {
		return fmt.Errorf("unexpected init message")
	}
	r.GotIsTerminal(m.Data)

	// receive width
	m, err = ReadMessage(buf)
	if err != nil {
		return err
	}
	if m.Type != T_WIDTH_REPORT {
		return fmt.Errorf("unexpected init message")
	}
	r.GotReportWidth(m.Data)

	return nil
}

func (r *RemoteSvr) HandleConfig(cfg *Config) {
	cfg.Stderr = r
	cfg.Stdout = r
	cfg.Stdin = r
	cfg.FuncExitRaw = r.ExitRawMode
	cfg.FuncIsTerminal = r.IsTerminal
	cfg.FuncMakeRaw = r.EnterRawMode
	cfg.FuncExitRaw = r.ExitRawMode
	cfg.FuncGetWidth = r.GetWidth
	cfg.FuncOnWidthChanged = func(f func()) {
		r.funcWidthChan = f
	}
}

func (r *RemoteSvr) IsTerminal() bool {
	return r.isTerminal
}

func (r *RemoteSvr) checkEOF() error {
	if atomic.LoadInt32(&r.eof) == 1 {
		return io.EOF
	}
	return nil
}

func (r *RemoteSvr) Read(b []byte) (int, error) {
	r.dataBufM.Lock()
	n, err := r.dataBuf.Read(b)
	r.dataBufM.Unlock()
	if n == 0 {
		if err := r.checkEOF(); err != nil {
			return 0, err
		}
	}

	if n == 0 && err == io.EOF {
		<-r.reciveChan
		r.dataBufM.Lock()
		n, err = r.dataBuf.Read(b)
		r.dataBufM.Unlock()
	}
	if n == 0 {
		if err := r.checkEOF(); err != nil {
			return 0, err
		}
	}

	return n, err
}

func (r *RemoteSvr) writeMsg(m *Message) error {
	ctx := newWriteCtx(m)
	r.writeChan <- ctx
	reply := <-ctx.reply
	return reply.err
}

func (r *RemoteSvr) Write(b []byte) (int, error) {
	ctx := newWriteCtx(NewMessage(T_DATA, b))
	r.writeChan <- ctx
	reply := <-ctx.reply
	return reply.n, reply.err
}

func (r *RemoteSvr) EnterRawMode() error {
	return r.writeMsg(NewMessage(T_RAW, nil))
}

func (r *RemoteSvr) ExitRawMode() error {
	return r.writeMsg(NewMessage(T_ERAW, nil))
}

func (r *RemoteSvr) writeLoop() {
	defer r.Close()

loop:
	for {
		select {
		case ctx, ok := <-r.writeChan:
			if !ok {
				break
			}
			n, err := ctx.msg.WriteTo(r.conn)
			ctx.reply <- &writeReply{n, err}
		case <-r.stopChan:
			break loop
		}
	}
}

func (r *RemoteSvr) Close() error {
	if atomic.CompareAndSwapInt32(&r.closed, 0, 1) {
		close(r.stopChan)
		r.conn.Close()
	}
	return nil
}

func (r *RemoteSvr) readLoop(buf *bufio.Reader) {
	defer r.Close()
	for {
		m, err := ReadMessage(buf)
		if err != nil {
			break
		}
		switch m.Type {
		case T_EOF:
			atomic.StoreInt32(&r.eof, 1)
			select {
			case r.reciveChan <- struct{}{}:
			default:
			}
		case T_DATA:
			r.dataBufM.Lock()
			r.dataBuf.Write(m.Data)
			r.dataBufM.Unlock()
			select {
			case r.reciveChan <- struct{}{}:
			default:
			}
		case T_WIDTH_REPORT:
			r.GotReportWidth(m.Data)
		case T_ISTTY_REPORT:
			r.GotIsTerminal(m.Data)
		}
	}
}

func (r *RemoteSvr) GotIsTerminal(data []byte) {
	if binary.BigEndian.Uint16(data) == 0 {
		r.isTerminal = false
	} else {
		r.isTerminal = true
	}
}

func (r *RemoteSvr) GotReportWidth(data []byte) {
	atomic.StoreInt32(&r.width, int32(binary.BigEndian.Uint16(data)))
	if r.funcWidthChan != nil {
		r.funcWidthChan()
	}
}

func (r *RemoteSvr) GetWidth() int {
	return int(atomic.LoadInt32(&r.width))
}

// -----------------------------------------------------------------------------

type Message struct {
	Type MsgType
	Data []byte
}

func ReadMessage(r io.Reader) (*Message, error) {
	m := new(Message)
	var length int32
	if err := binary.Read(r, binary.BigEndian, &length); err != nil {
		return nil, err
	}
	if err := binary.Read(r, binary.BigEndian, &m.Type); err != nil {
		return nil, err
	}
	m.Data = make([]byte, int(length)-2)
	if _, err := io.ReadFull(r, m.Data); err != nil {
		return nil, err
	}
	return m, nil
}

func NewMessage(t MsgType, data []byte) *Message {
	return &Message{t, data}
}

func (m *Message) WriteTo(w io.Writer) (int, error) {
	buf := bytes.NewBuffer(make([]byte, 0, len(m.Data)+2+4))
	binary.Write(buf, binary.BigEndian, int32(len(m.Data)+2))
	binary.Write(buf, binary.BigEndian, m.Type)
	buf.Write(m.Data)
	n, err := buf.WriteTo(w)
	return int(n), err
}

// -----------------------------------------------------------------------------

type RemoteCli struct {
	conn        net.Conn
	raw         RawMode
	receiveChan chan struct{}
	inited      int32
	isTerminal  *bool

	data  bytes.Buffer
	dataM sync.Mutex
}

func NewRemoteCli(conn net.Conn) (*RemoteCli, error) {
	r := &RemoteCli{
		conn:        conn,
		receiveChan: make(chan struct{}),
	}
	return r, nil
}

func (r *RemoteCli) MarkIsTerminal(is bool) {
	r.isTerminal = &is
}

func (r *RemoteCli) init() error {
	if !atomic.CompareAndSwapInt32(&r.inited, 0, 1) {
		return nil
	}

	if err := r.reportIsTerminal(); err != nil {
		return err
	}

	if err := r.reportWidth(); err != nil {
		return err
	}

	// register sig for width changed
	DefaultOnWidthChanged(func() {
		r.reportWidth()
	})
	return nil
}

func (r *RemoteCli) writeMsg(m *Message) error {
	r.dataM.Lock()
	_, err := m.WriteTo(r.conn)
	r.dataM.Unlock()
	return err
}

func (r *RemoteCli) Write(b []byte) (int, error) {
	m := NewMessage(T_DATA, b)
	r.dataM.Lock()
	_, err := m.WriteTo(r.conn)
	r.dataM.Unlock()
	return len(b), err
}

func (r *RemoteCli) reportWidth() error {
	screenWidth := GetScreenWidth()
	data := make([]byte, 2)
	binary.BigEndian.PutUint16(data, uint16(screenWidth))
	msg := NewMessage(T_WIDTH_REPORT, data)

	if err := r.writeMsg(msg); err != nil {
		return err
	}
	return nil
}

func (r *RemoteCli) reportIsTerminal() error {
	var isTerminal bool
	if r.isTerminal != nil {
		isTerminal = *r.isTerminal
	} else {
		isTerminal = DefaultIsTerminal()
	}
	data := make([]byte, 2)
	if isTerminal {
		binary.BigEndian.PutUint16(data, 1)
	} else {
		binary.BigEndian.PutUint16(data, 0)
	}
	msg := NewMessage(T_ISTTY_REPORT, data)
	if err := r.writeMsg(msg); err != nil {
		return err
	}
	return nil
}

func (r *RemoteCli) readLoop() {
	buf := bufio.NewReader(r.conn)
	for {
		msg, err := ReadMessage(buf)
		if err != nil {
			break
		}
		switch msg.Type {
		case T_ERAW:
			r.raw.Exit()
		case T_RAW:
			r.raw.Enter()
		case T_DATA:
			os.Stdout.Write(msg.Data)
		}
	}
}

func (r *RemoteCli) ServeBy(source io.Reader) error {
	if err := r.init(); err != nil {
		return err
	}

	go func() {
		defer r.Close()
		for {
			n, _ := io.Copy(r, source)
			if n == 0 {
				break
			}
		}
	}()
	defer r.raw.Exit()
	r.readLoop()
	return nil
}

func (r *RemoteCli) Close() {
	r.writeMsg(NewMessage(T_EOF, nil))
}

func (r *RemoteCli) Serve() error {
	return r.ServeBy(os.Stdin)
}

func ListenRemote(n, addr string, cfg *Config, h func(*Instance), onListen ...func(net.Listener) error) error {
	ln, err := net.Listen(n, addr)
	if err != nil {
		return err
	}
	if len(onListen) > 0 {
		if err := onListen[0](ln); err != nil {
			return err
		}
	}
	for {
		conn, err := ln.Accept()
		if err != nil {
			break
		}
		go func() {
			defer conn.Close()
			rl, err := HandleConn(*cfg, conn)
			if err != nil {
				return
			}
			h(rl)
		}()
	}
	return nil
}

func HandleConn(cfg Config, conn net.Conn) (*Instance, error) {
	r, err := NewRemoteSvr(conn)
	if err != nil {
		return nil, err
	}
	r.HandleConfig(&cfg)

	rl, err := NewEx(&cfg)
	if err != nil {
		return nil, err
	}
	return rl, nil
}

func DialRemote(n, addr string) error {
	conn, err := net.Dial(n, addr)
	if err != nil {
		return err
	}
	defer conn.Close()

	cli, err := NewRemoteCli(conn)
	if err != nil {
		return err
	}
	return cli.Serve()
}
