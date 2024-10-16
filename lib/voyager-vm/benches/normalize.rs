use criterion::{black_box, criterion_group, criterion_main, Criterion};
use unionlabs::{ibc::core::client::height::Height, id::ClientId};
use voyager_message::{
    call::FetchBlocks, callback::AggregateMsgUpdateClientsFromOrderedHeaders, core::ChainId,
    PluginMessage, VoyagerMessage,
};
use voyager_vm::{call, conc, data, noop, promise, seq, Op};

fn bench_normalize(c: &mut Criterion) {
    c.bench_function("normalize", |b| {
        b.iter_with_setup(
            || vec![mk_msg(), mk_msg(), mk_msg()],
            |op| black_box(op.into_iter().map(Op::normalize)),
        )
    });
}

fn mk_msg() -> Op<VoyagerMessage> {
    seq([
        promise(
            [
                data(PluginMessage::new("", "")),
                call(FetchBlocks {
                    chain_id: ChainId::new("chain"),
                    start_height: Height::new(1),
                }),
                conc([
                    noop(),
                    data(PluginMessage::new("", "")),
                    call(FetchBlocks {
                        chain_id: ChainId::new("chain"),
                        start_height: Height::new(1),
                    }),
                ]),
            ],
            [],
            AggregateMsgUpdateClientsFromOrderedHeaders {
                chain_id: ChainId::new("chain"),
                counterparty_client_id: ClientId::new(0),
            },
        ),
        seq([
            data(PluginMessage::new("", "")),
            call(FetchBlocks {
                chain_id: ChainId::new("chain"),
                start_height: Height::new(1),
            }),
            conc([
                noop(),
                data(PluginMessage::new("", "")),
                call(FetchBlocks {
                    chain_id: ChainId::new("chain"),
                    start_height: Height::new(1),
                }),
            ]),
        ]),
    ])
}

criterion_group!(benches, bench_normalize);

criterion_main!(benches);
