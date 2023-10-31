use crate::sync::watch;

use loom::future::block_on;
use loom::thread;

#[test]
fn smoke() {
    loom::model(|| {
        let (tx, mut rx1) = watch::channel(1);
        let mut rx2 = rx1.clone();
        let mut rx3 = rx1.clone();
        let mut rx4 = rx1.clone();
        let mut rx5 = rx1.clone();

        let th = thread::spawn(move || {
            tx.send(2).unwrap();
        });

        block_on(rx1.changed()).unwrap();
        assert_eq!(*rx1.borrow(), 2);

        block_on(rx2.changed()).unwrap();
        assert_eq!(*rx2.borrow(), 2);

        block_on(rx3.changed()).unwrap();
        assert_eq!(*rx3.borrow(), 2);

        block_on(rx4.changed()).unwrap();
        assert_eq!(*rx4.borrow(), 2);

        block_on(rx5.changed()).unwrap();
        assert_eq!(*rx5.borrow(), 2);

        th.join().unwrap();
    })
}
