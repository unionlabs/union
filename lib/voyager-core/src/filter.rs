use std::fmt::Debug;

use anyhow::anyhow;
use jaq_core::{
    Ctx, Filter, Native, RcIter,
    load::{Arena, File, Loader},
};
use jaq_json::Val;
use opentelemetry::{KeyValue, global, metrics::Histogram};
use tracing::{error, instrument, trace};
use voyager_rpc::types::PluginInfo;
use voyager_vm::{
    Op,
    filter::{FilterResult, Interest, InterestFilter},
};

use crate::VoyagerMessage;

#[derive(Clone)]
pub struct InterestFilters {
    pub filters: Vec<(Filter<Native<Val>>, &'static str)>,
    filter_run_time_histogram: Histogram<f64>,
}

impl InterestFilters {
    pub fn new(filters: Vec<PluginInfo>) -> anyhow::Result<Self> {
        Ok(Self {
            filters: filters
                .into_iter()
                .map(make_filter)
                .collect::<anyhow::Result<_>>()?,
            filter_run_time_histogram: global::meter("voyager")
                .f64_histogram("plugin.filter.run_time")
                .with_unit("s")
                .with_boundaries(vec![
                    0.0,       // 0s
                    0.000_001, // 1µs
                    0.000_01,  // 10µs
                    0.000_1,   // 100μs
                    0.001,     // 1ms
                    0.005,     // 5ms
                    0.01,      // 10ms
                    0.05,      // 50ms
                    0.1,       // 100ms
                    0.5,       // 500ms
                    1.0,       // 1s+
                ])
                .build(),
        })
    }
}

pub fn make_filter(
    PluginInfo {
        name,
        interest_filter,
    }: PluginInfo,
) -> anyhow::Result<(Filter<Native<Val>>, &'static str)> {
    fn map_jq_errs(es: Vec<(File<&str, &str>, impl Debug)>) -> anyhow::Error {
        anyhow!(
            es.iter()
                .map(|(file, error)| format!("{}: {:?}", file.path, error))
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    let program = File {
        code: &*interest_filter,
        path: &*name,
    };

    let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
    let arena = Arena::default();

    let modules = loader.load(&arena, program).map_err(map_jq_errs)?;

    let filter = jaq_core::Compiler::default()
        .with_funs(jaq_std::funs().chain(jaq_json::funs()))
        .compile(modules)
        .map_err(map_jq_errs)?;

    // let mut ctx = ParseCtx::new(["PLUGIN_NAME".to_owned()].into());
    // ctx.insert_natives(jaq_core::core());
    // ctx.insert_defs(jaq_std::std());

    // // parse the filter
    // let lexed = jaq_syn::Lexer::new(&interest_filter).lex().map_err(|es| {
    //     anyhow!(es
    //         .iter()
    //         .map(|(expect, s)| format!("({}: {s})", expect.as_str()))
    //         .collect::<Vec<_>>()
    //         .join(","))
    // })?;

    // let f = jaq_syn::Parser::new(&lexed)
    //     .parse(|p| p.module(|p| p.term()))
    //     .map_err(|es| {
    //         anyhow!(es
    //             .iter()
    //             .map(|(expect, maybe_token)| match maybe_token {
    //                 Some(token) => {
    //                     format!("({}, {})", expect.as_str(), token.as_str())
    //                 }
    //                 None => format!("({})", expect.as_str()),
    //             })
    //             .collect::<Vec<_>>()
    //             .join(","))
    //     })?;

    // // compile the filter in the context of the given definitions
    // let filter = ctx.compile(f.conv(&interest_filter));

    // assert!(
    //     ctx.errs.is_empty(),
    //     "{:?}",
    //     ctx.errs
    //         .into_iter()
    //         .map(|x| x.0.to_string())
    //         .collect::<Vec<_>>()
    // );

    // name lives for the runtime of the program, leak it so we don't need to clone a full string everywhere
    Ok((filter, String::leak(name)))
}

impl InterestFilter<VoyagerMessage> for InterestFilters {
    #[instrument(skip_all)]
    fn check_interest<'a>(&'a self, op: &Op<VoyagerMessage>) -> FilterResult<'a> {
        let msg_json = Val::from(serde_json::to_value(op.clone()).unwrap());

        let mut tags = vec![];
        let mut take = false;

        for (filter, plugin_name) in &self.filters {
            let now = std::time::SystemTime::now();

            let result = match run_filter(filter, plugin_name, msg_json.clone()) {
                Ok(JaqFilterResult::Copy(tag)) => {
                    tags.push(tag);
                    "copy"
                }
                Ok(JaqFilterResult::Take(tag)) => {
                    tags.push(tag);
                    take = true;
                    "take"
                }
                Ok(JaqFilterResult::NoInterest) => "no_interest",
                Err(_) => "error",
            };

            let elapsed = now.elapsed().unwrap_or_default();

            self.filter_run_time_histogram.record(
                elapsed.as_secs_f64(),
                &[
                    KeyValue::new("plugin", *plugin_name),
                    KeyValue::new("result", result),
                ],
            );

            if take {
                return FilterResult::Interest(Interest { tags, remove: true });
            }
        }

        if tags.is_empty() {
            FilterResult::NoInterest
        } else {
            FilterResult::Interest(Interest {
                tags,
                remove: false,
            })
        }
    }
}

#[instrument(
    name = "checking interest",
    level = "debug",
    skip_all,
    fields(%plugin_name)
)]
#[allow(clippy::result_unit_err)]
pub fn run_filter<'a>(
    filter: &Filter<Native<Val>>,
    plugin_name: &'a str,
    msg_json: Val,
) -> Result<JaqFilterResult<'a>, ()> {
    let inputs = RcIter::new(core::iter::empty());
    let mut out = filter
        .run((Ctx::new([], &inputs), msg_json.clone()))
        .peekable();

    let Some(result) = out.next() else {
        error!("filter didn't return any values");

        return Err(());
    };

    let result = match result {
        Ok(ok) => ok,
        Err(err) => {
            error!(%err, "filter failed");

            return Err(());
        }
    };

    if out.peek().is_some() {
        let tail = out
            .map(|r| match r {
                Ok(ok) => ok.to_string(),
                Err(err) => err.to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ");

        error!(
            additional_items = %tail,
            "filter returned multiple values, only a single boolean value is valid"
        );
        Err(())
    } else {
        match result {
            Val::Bool(true) => {
                trace!("take");

                Ok(JaqFilterResult::Take(plugin_name))
            }
            Val::Bool(false) => {
                trace!("copy");

                Ok(JaqFilterResult::Copy(plugin_name))
            }
            Val::Null => {
                trace!("no interest");

                Ok(JaqFilterResult::NoInterest)
            }
            _ => {
                error!("filter returned a non-boolean value: {result:?}");

                Err(())
            }
        }
    }
}

pub enum JaqFilterResult<'a> {
    NoInterest,
    Copy(&'a str),
    Take(&'a str),
}
