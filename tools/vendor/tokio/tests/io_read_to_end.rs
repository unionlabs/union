#![warn(rust_2018_idioms)]
#![cfg(feature = "full")]

use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};
use tokio_test::assert_ok;

#[tokio::test]
async fn read_to_end() {
    let mut buf = vec![];
    let mut rd: &[u8] = b"hello world";

    let n = assert_ok!(rd.read_to_end(&mut buf).await);
    assert_eq!(n, 11);
    assert_eq!(buf[..], b"hello world"[..]);
}

#[derive(Copy, Clone, Debug)]
enum State {
    Initializing,
    JustFilling,
    Done,
}

struct UninitTest {
    num_init: usize,
    state: State,
}

impl AsyncRead for UninitTest {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let me = Pin::into_inner(self);
        let real_num_init = buf.initialized().len() - buf.filled().len();
        assert_eq!(real_num_init, me.num_init, "{:?}", me.state);

        match me.state {
            State::Initializing => {
                buf.initialize_unfilled_to(me.num_init + 2);
                buf.advance(1);
                me.num_init += 1;

                if me.num_init == 24 {
                    me.state = State::JustFilling;
                }
            }
            State::JustFilling => {
                buf.advance(1);
                me.num_init -= 1;

                if me.num_init == 15 {
                    // The buffer is resized on next call.
                    me.num_init = 0;
                    me.state = State::Done;
                }
            }
            State::Done => { /* .. do nothing .. */ }
        }

        Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn read_to_end_uninit() {
    let mut buf = Vec::with_capacity(64);
    let mut test = UninitTest {
        num_init: 0,
        state: State::Initializing,
    };

    test.read_to_end(&mut buf).await.unwrap();
    assert_eq!(buf.len(), 33);
}
