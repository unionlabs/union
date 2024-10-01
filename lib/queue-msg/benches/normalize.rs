use criterion::{black_box, criterion_group, criterion_main, Criterion};
use queue_msg::{call, conc, data, noop, normalize::normalize, promise, seq, Op};
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{
    call::FetchBlocks, callback::AggregateMsgUpdateClientsFromOrderedHeaders, core::ChainId,
    data::LatestHeight, VoyagerMessage,
};

fn bench_normalize(c: &mut Criterion) {
    c.bench_function("normalize", |b| {
        b.iter_with_setup(
            || vec![mk_msg(), mk_msg(), mk_msg()],
            |msgs| black_box(normalize(msgs)),
        )
    });
}

fn mk_msg() -> Op<VoyagerMessage> {
    seq([
        promise(
            [
                data(LatestHeight {
                    chain_id: ChainId::new("chain"),
                    height: Height {
                        revision_number: 1,
                        revision_height: 1,
                    },
                }),
                call(FetchBlocks {
                    chain_id: ChainId::new("chain"),
                    start_height: Height {
                        revision_number: 1,
                        revision_height: 1,
                    },
                }),
                conc([
                    noop(),
                    data(LatestHeight {
                        chain_id: ChainId::new("chain"),
                        height: Height {
                            revision_number: 1,
                            revision_height: 1,
                        },
                    }),
                    call(FetchBlocks {
                        chain_id: ChainId::new("chain"),
                        start_height: Height {
                            revision_number: 1,
                            revision_height: 1,
                        },
                    }),
                ]),
            ],
            [],
            AggregateMsgUpdateClientsFromOrderedHeaders {
                chain_id: ChainId::new("chain"),
                counterparty_client_id: "counterparty_chain".parse().unwrap(),
            },
        ),
        seq([
            data(LatestHeight {
                chain_id: ChainId::new("chain"),
                height: Height {
                    revision_number: 1,
                    revision_height: 1,
                },
            }),
            call(FetchBlocks {
                chain_id: ChainId::new("chain"),
                start_height: Height {
                    revision_number: 1,
                    revision_height: 1,
                },
            }),
            conc([
                noop(),
                data(LatestHeight {
                    chain_id: ChainId::new("chain"),
                    height: Height {
                        revision_number: 1,
                        revision_height: 1,
                    },
                }),
                call(FetchBlocks {
                    chain_id: ChainId::new("chain"),
                    start_height: Height {
                        revision_number: 1,
                        revision_height: 1,
                    },
                }),
            ]),
        ]),
    ])
}

criterion_group!(benches, bench_normalize);

criterion_main!(benches);
