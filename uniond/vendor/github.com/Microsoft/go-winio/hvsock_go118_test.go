//go:build windows && go1.18

package winio

import (
	"errors"
	"fmt"
	"testing"
	"time"
)

func FuzzHvSockRxTx(f *testing.F) {
	for _, b := range [][]byte{
		[]byte("hello?"),
		[]byte("This is a really long string that should be a good example of the really long " +
			"payloads that may be sent over hvsockets when really long inputs are being used, tautologically. " +
			"That means that we will have to test with really long input sequences, which means that " +
			"we need to include really long byte sequences or strings in our testing so that we know that " +
			"the sockets can deal with really long inputs. Look at this key mashing: " +
			"sdflhsdfgkjdhskljjsad;kljfasd;lfkjsadl ;fasdjfopiwej09q34iur092\"i4o[piwajfliasdkf-012ior]-" +
			"01oi3;'lSD<Fplkasdjgoisaefjoiasdlj\"hgfoaisdkf';laksdjdf[poaiseefk-0923i4roi3qwjrf9" +
			"08sEJKEFOLIsaejf[09saEJFLKSADjf;lkasdjf;kljaslddhgaskghk"),
		{0x5c, 0xbd, 0xb5, 0xe7, 0x6b, 0xcb, 0xe7, 0x23, 0xff, 0x7a, 0x19, 0x77, 0x2c, 0xca, 0xab, 0x3b},
	} {
		f.Add(b)
	}

	f.Fuzz(func(t *testing.T, a []byte) {
		if string(a) == "" {
			t.Skip("skipping empty string")
		}
		t.Logf("testing %q (%d)", a, len(a))
		u := newUtil(t)
		cl, sv, _ := clientServer(u)

		svCh := u.Go(func() error {
			n, err := cl.Write(a)
			if err != nil {
				return fmt.Errorf("client write: %w", err)
			}
			if n != len(a) {
				return errors.New("client did not send full message")
			}

			b := make([]byte, len(a)+5) // a little extra to make sure nothing else is sent
			n, err = cl.Read(b)
			if err != nil {
				return fmt.Errorf("client read: %w", err)
			}
			if n != len(a) {
				return errors.New("client did not read full message")
			}
			bn := b[:n]
			if string(a) != string(bn) {
				return fmt.Errorf("client payload mismatch %q != %q", a, bn)
			}
			t.Log("client received")
			return nil
		})

		clCh := u.Go(func() error {
			b := make([]byte, len(a)+5) // a little extra to make sure nothing else is sent
			n, err := sv.Read(b)
			if err != nil {
				return fmt.Errorf("server read: %w", err)
			}
			if n != len(a) {
				return errors.New("server did not read full message")
			}
			bn := b[:n]
			if string(a) != string(bn) {
				return fmt.Errorf("server payload mismatch %q != %q", a, bn)
			}

			n, err = sv.Write(bn)
			if err != nil {
				return fmt.Errorf("server write: %w", err)
			}
			if n != len(a) {
				return errors.New("server did not send full message")
			}
			t.Log("server sent")
			return nil
		})
		u.WaitErr(svCh, 250*time.Millisecond)
		u.WaitErr(clCh, 250*time.Millisecond)
	})
}
