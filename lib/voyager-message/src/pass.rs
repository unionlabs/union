use std::rc::Rc;

use jaq_interpret::{Ctx, Filter, FilterT, ParseCtx, RcIter, Val};
use queue_msg::{
    optimize::{OptimizationResult, PurePass},
    BoxDynError, Op,
};
use tracing::{error, trace, trace_span};
use unionlabs::ErrorReporter;

use crate::VoyagerMessage;

#[derive(Debug, Clone)]
pub struct JaqInterestFilter {
    pub filters: Vec<(Filter, String)>,
}

impl JaqInterestFilter {
    /// `(plugin_name, filter)[]`
    pub fn new(filters: Vec<(String, String)>) -> Result<Self, BoxDynError> {
        Ok(Self {
            filters: filters
                .into_iter()
                .map(|(plugin_name, filter)| {
                    let mut ctx = ParseCtx::new(["PLUGIN_NAME".to_owned()].into());

                    ctx.insert_natives(jaq_core::core());
                    ctx.insert_defs(jaq_std::std());

                    // parse the filter
                    let lexed = jaq_syn::Lexer::new(&filter).lex().map_err(|es| {
                        es.iter()
                            .map(|(expect, s)| format!("({}: {s})", expect.as_str()))
                            .collect::<Vec<_>>()
                            .join(",")
                    })?;
                    let f = jaq_syn::Parser::new(&lexed)
                        .parse(|p| p.module(|p| p.term()))
                        .map_err(|es| {
                            es.iter()
                                .map(|(expect, maybe_token)| match maybe_token {
                                    Some(token) => {
                                        format!("({}, {})", expect.as_str(), token.as_str())
                                    }
                                    None => format!("({})", expect.as_str()),
                                })
                                .collect::<Vec<_>>()
                                .join(",")
                        });

                    // compile the filter in the context of the given definitions
                    let filter = ctx.compile(f?.conv(&filter));

                    assert!(
                        ctx.errs.is_empty(),
                        "{:?}",
                        ctx.errs
                            .into_iter()
                            .map(|x| x.0.to_string())
                            .collect::<Vec<_>>()
                    );

                    Ok((filter, plugin_name))
                })
                .collect::<Result<_, BoxDynError>>()?,
        })
    }
}

impl PurePass<VoyagerMessage> for JaqInterestFilter {
    fn run_pass_pure(&self, msgs: Vec<Op<VoyagerMessage>>) -> OptimizationResult<VoyagerMessage> {
        let mut opt_res = OptimizationResult::default();

        let inputs = RcIter::new(core::iter::empty());

        'outer: for (idx, msg) in msgs.into_iter().enumerate() {
            let msg_json = Val::from(serde_json::to_value(msg.clone()).unwrap());

            for (filter, plugin_name) in &self.filters {
                let _span = trace_span!("checking interest", %plugin_name).entered();

                let mut out = filter.run((
                    Ctx::new([Val::Str(Rc::new(plugin_name.clone()))], &inputs),
                    msg_json.clone(),
                ));

                let Some(result) = out.next() else {
                    error!("filter didn't return any values");

                    continue;
                };

                let result = match result {
                    Ok(ok) => ok,
                    Err(err) => {
                        error!(%msg_json, err = %ErrorReporter(err), "filter failed");

                        continue;
                    }
                };

                if out.next().is_some() {
                    error!("filter returned multiple values, only a single boolean value is valid");
                } else {
                    match result {
                        Val::Bool(true) => {
                            trace!(%msg_json, "interest");

                            opt_res
                                .optimize_further
                                .push((vec![idx], msg, plugin_name.clone()));

                            continue 'outer;
                        }
                        Val::Bool(false) => {
                            trace!(%msg_json, "no interest");
                        }
                        _ => error!("filter returned a non-boolean value: {result:?}"),
                    }
                }
            }

            opt_res.ready.push((vec![idx], msg));
        }

        opt_res
    }
}
