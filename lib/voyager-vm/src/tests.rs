use voyager_macros::model;

use crate::{
    call, conc, data, defer, noop, now, promise, seq,
    tests::utils::{BuildPrintAbc, DataA, DataB, DataC, FetchA, FetchB, PrintAbc, SimpleMessage},
    CallT, CallbackT, Op, QueueError, QueueMessage, VecDeque,
};

pub mod utils;

enum UnitMessage {}

impl QueueMessage for UnitMessage {
    type Data = ();
    type Call = ();
    type Callback = ();

    type Filter = ();

    type Context = ();
}

impl CallT<UnitMessage> for () {
    async fn process(self, (): &()) -> Result<Op<UnitMessage>, QueueError> {
        Ok(noop())
    }
}

impl CallbackT<UnitMessage> for () {
    async fn process(self, (): &(), _: VecDeque<()>) -> Result<Op<UnitMessage>, QueueError> {
        Ok(noop())
    }
}

#[model]
pub struct SimpleData {}
#[model]
pub struct SimpleCall {}
#[model]
pub struct SimpleCallback {}

#[test]
fn flatten() {
    let op = seq::<UnitMessage>([
        defer(1),
        seq([defer(2), seq([defer(3)])]),
        seq([defer(4)]),
        defer(5),
    ]);

    assert_eq!(
        op.normalize(),
        vec![seq([defer(1), defer(2), defer(3), defer(4), defer(5)])]
    );

    let op = seq::<UnitMessage>([defer(1)]);
    assert_eq!(op.normalize(), vec![defer(1)]);

    let op = conc::<UnitMessage>([defer(1)]);
    assert_eq!(op.normalize(), vec![defer(1)]);

    let op = conc::<UnitMessage>([seq([defer(1)])]);
    assert_eq!(op.normalize(), vec![defer(1)]);

    let op = seq::<UnitMessage>([noop()]);
    assert_eq!(op.normalize(), vec![]);

    let op = conc::<UnitMessage>([seq([noop()])]);
    assert_eq!(op.normalize(), vec![]);

    let op = conc::<UnitMessage>([conc([conc([noop()])])]);
    assert_eq!(op.normalize(), vec![]);
}

#[test]
fn nested_seq_conc_single() {
    // any nesting level of seq and conc should be handled in a single pass

    let op = conc::<UnitMessage>([seq([conc([noop()])])]);
    assert_eq!(op.normalize(), vec![]);

    let op = conc::<UnitMessage>([seq([conc([seq([conc([seq([conc([noop()])])])])])])]);
    assert_eq!(op.normalize(), vec![]);

    let op = conc::<UnitMessage>([seq([conc([seq([conc([seq([conc([seq([conc([
        data(()),
    ])])])])])])])])]);
    assert_eq!(op.normalize(), vec![data(())]);

    let op = seq::<UnitMessage>([conc([seq([conc([data(())])])])]);
    assert_eq!(op.normalize(), vec![data(())]);

    let op = seq::<UnitMessage>([conc([seq([conc([seq([conc([seq([conc([
        data(()),
    ])])])])])])])]);
    assert_eq!(op.normalize(), vec![data(())]);

    let op = seq::<UnitMessage>([conc([seq([conc([seq([conc([seq([conc([seq([
        conc([data(())]),
    ])])])])])])])])]);
    assert_eq!(op.normalize(), vec![data(())]);
}

#[test]
fn flatten_seq_conc_fixed_point_is_noop() {
    // this message can't be optimized any further, flattening operations should be a noop

    let op = seq::<UnitMessage>([conc([defer(1), defer(2)]), defer(3)]);
    assert_eq!(op.clone().normalize(), vec![op.clone()]);
    assert_eq!(op.clone().normalize(), vec![op]);
}

#[test]
fn conc_seq_call_call_call() {
    let op = conc::<UnitMessage>([seq([call(()), call(())]), call(())]);
    assert_eq!(
        op.clone().normalize(),
        vec![seq([call(()), call(())]), call(())]
    );
}

#[test]
fn extract_data_simple() {
    let op = seq::<UnitMessage>([
        data(()),
        seq([data(()), seq([data(())])]),
        seq([data(())]),
        data(()),
    ]);
    assert_eq!(
        op.normalize(),
        vec![data(()), data(()), data(()), data(()), data(()),],
    );
}

#[test]
fn extract_data_seq_in_promise_queue() {
    let op = promise::<UnitMessage>([seq([call(()), data(())])], [], ());
    assert_eq!(op.clone().normalize(), vec![op]);
}

#[test]
fn seq_defer_call_data() {
    let op = seq([seq::<UnitMessage>([defer(1), call(())]), data(())]);
    assert_eq!(
        op.clone().normalize(),
        vec![seq([defer(1), call(()), data(())])]
    );
}

#[test]
fn extract_data_complex() {
    let op = seq::<UnitMessage>([
        data(()),
        call(()),
        seq([call(()), data(()), seq([data(())])]),
        call(()),
        seq([data(()), call(())]),
        data(()),
    ]);
    assert_eq!(
        op.normalize(),
        vec![
            data(()),
            seq([
                call(()),
                call(()),
                data(()),
                data(()),
                call(()),
                data(()),
                call(()),
                data(()),
            ])
        ],
    );
}

#[test]
fn normalize_works_in_single_pass() {
    let op = seq::<SimpleMessage>([
        call(FetchA {}),
        seq([
            data(DataA {}),
            noop(),
            call(FetchA {}),
            conc([
                call(PrintAbc {
                    a: DataA {},
                    b: DataB {},
                    c: DataC {},
                }),
                data(DataC {}),
            ]),
            call(FetchA {}),
        ]),
    ]);

    let expected_output = vec![seq([
        call(FetchA {}),
        data(DataA {}),
        call(FetchA {}),
        data(DataC {}),
        call(PrintAbc {
            a: DataA {},
            b: DataB {},
            c: DataC {},
        }),
        call(FetchA {}),
    ])];

    assert_eq!(op.clone().normalize(), expected_output);

    assert_eq!(op.normalize(), expected_output);
}

#[test]
fn seq_call_data() {
    let op = seq::<SimpleMessage>([call(FetchA {}), data(DataA {})]);

    // should be the same
    let expected_output = vec![op.clone()];

    assert_eq!(op.normalize(), expected_output);
}

#[test]
fn seq_conc_conc() {
    let op = seq::<SimpleMessage>([
        conc([
            promise([], [], BuildPrintAbc {}),
            promise([], [], BuildPrintAbc {}),
        ]),
        conc([
            promise([], [], BuildPrintAbc {}),
            promise([], [], BuildPrintAbc {}),
        ]),
        conc([
            seq([call(FetchA {}), defer(now() + 10)]),
            seq([call(FetchB {}), defer(now() + 10)]),
            // this seq is the only message that should be flattened
            seq([
                call(PrintAbc {
                    a: DataA {},
                    b: DataB {},
                    c: DataC {},
                }),
                seq([
                    promise([], [], BuildPrintAbc {}),
                    promise([], [], BuildPrintAbc {}),
                    promise([], [], BuildPrintAbc {}),
                ]),
            ]),
        ]),
    ]);

    let expected_output = vec![seq::<SimpleMessage>([
        conc([
            promise([], [], BuildPrintAbc {}),
            promise([], [], BuildPrintAbc {}),
        ]),
        conc([
            promise([], [], BuildPrintAbc {}),
            promise([], [], BuildPrintAbc {}),
        ]),
        conc([
            seq([call(FetchA {}), defer(now() + 10)]),
            seq([call(FetchB {}), defer(now() + 10)]),
            seq([
                call(PrintAbc {
                    a: DataA {},
                    b: DataB {},
                    c: DataC {},
                }),
                promise([], [], BuildPrintAbc {}),
                promise([], [], BuildPrintAbc {}),
                promise([], [], BuildPrintAbc {}),
            ]),
        ]),
    ])];

    assert_eq!(op.clone().normalize(), expected_output);

    assert_eq!(op.normalize(), expected_output);
}
