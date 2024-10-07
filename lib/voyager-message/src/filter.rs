use std::rc::Rc;

use jaq_interpret::{Ctx, Filter, FilterT, ParseCtx, RcIter, Val};
use tracing::{error, instrument, trace};
use unionlabs::ErrorReporter;
use voyager_vm::{
    filter::{FilterResult, InterestFilter},
    BoxDynError, Op,
};

use crate::{module::PluginInfo, VoyagerMessage};

#[derive(Debug, Clone)]
pub struct JaqInterestFilter {
    pub filters: Vec<(Filter, String)>,
}

impl JaqInterestFilter {
    pub fn new(filters: Vec<PluginInfo>) -> Result<Self, BoxDynError> {
        Ok(Self {
            filters: filters
                .into_iter()
                .map(make_filter)
                .collect::<Result<_, BoxDynError>>()?,
        })
    }
}

pub fn make_filter(
    PluginInfo {
        name,
        interest_filter,
    }: PluginInfo,
) -> Result<(Filter, String), BoxDynError> {
    let mut ctx = ParseCtx::new(["PLUGIN_NAME".to_owned()].into());
    ctx.insert_natives(jaq_core::core());
    ctx.insert_defs(jaq_std::std());

    // parse the filter
    let lexed = jaq_syn::Lexer::new(&interest_filter).lex().map_err(|es| {
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
    let filter = ctx.compile(f?.conv(&interest_filter));

    assert!(
        ctx.errs.is_empty(),
        "{:?}",
        ctx.errs
            .into_iter()
            .map(|x| x.0.to_string())
            .collect::<Vec<_>>()
    );

    Ok((filter, name))
}

impl InterestFilter<VoyagerMessage> for JaqInterestFilter {
    fn check_interest<'a>(&'a self, op: &Op<VoyagerMessage>) -> FilterResult<'a> {
        let msg_json = Val::from(serde_json::to_value(op.clone()).unwrap());

        for (filter, plugin_name) in &self.filters {
            match run_filter(filter, plugin_name, msg_json.clone()) {
                Ok(interest @ FilterResult::Interest(_)) => return interest,
                Ok(FilterResult::NoInterest) => {}
                Err(_) => {}
            }
        }

        FilterResult::NoInterest
    }
}

#[instrument(
    name = "checking interest",
    level = "trace",
    skip_all,
    fields(%plugin_name)
)]
pub fn run_filter<'a>(
    filter: &Filter,
    plugin_name: &'a str,
    msg_json: Val,
) -> Result<FilterResult<'a>, ()> {
    let inputs = RcIter::new(core::iter::empty());
    let mut out = filter.run((
        Ctx::new([Val::Str(Rc::new(plugin_name.to_owned()))], &inputs),
        msg_json.clone(),
    ));

    let Some(result) = out.next() else {
        error!("filter didn't return any values");

        return Err(());
    };

    let result = match result {
        Ok(ok) => ok,
        Err(err) => {
            error!(err = %ErrorReporter(err), "filter failed");

            return Err(());
        }
    };

    if out.next().is_some() {
        error!("filter returned multiple values, only a single boolean value is valid");
        Err(())
    } else {
        match result {
            Val::Bool(true) => {
                trace!("interest");

                Ok(FilterResult::Interest(plugin_name))
            }
            Val::Bool(false) => {
                trace!("no interest");

                Ok(FilterResult::NoInterest)
            }
            _ => {
                error!("filter returned a non-boolean value: {result:?}");

                Err(())
            }
        }
    }
}
