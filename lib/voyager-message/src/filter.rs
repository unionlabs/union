use std::rc::Rc;

use anyhow::anyhow;
use jaq_interpret::{Ctx, Filter, FilterT, ParseCtx, RcIter, Val};
use tracing::{error, instrument, trace};
use unionlabs::ErrorReporter;
use voyager_vm::{
    filter::{FilterResult, Interest, InterestFilter},
    Op,
};

use crate::{module::PluginInfo, VoyagerMessage};

#[derive(Debug, Clone)]
pub struct JaqInterestFilter {
    pub filters: Vec<(Filter, String)>,
}

impl JaqInterestFilter {
    pub fn new(filters: Vec<PluginInfo>) -> anyhow::Result<Self> {
        Ok(Self {
            filters: filters
                .into_iter()
                .map(make_filter)
                .collect::<anyhow::Result<_>>()?,
        })
    }
}

pub fn make_filter(
    PluginInfo {
        name,
        interest_filter,
    }: PluginInfo,
) -> anyhow::Result<(Filter, String)> {
    let mut ctx = ParseCtx::new(["PLUGIN_NAME".to_owned()].into());
    ctx.insert_natives(jaq_core::core());
    ctx.insert_defs(jaq_std::std());

    // parse the filter
    let lexed = jaq_syn::Lexer::new(&interest_filter).lex().map_err(|es| {
        anyhow!(es
            .iter()
            .map(|(expect, s)| format!("({}: {s})", expect.as_str()))
            .collect::<Vec<_>>()
            .join(","))
    })?;

    let f = jaq_syn::Parser::new(&lexed)
        .parse(|p| p.module(|p| p.term()))
        .map_err(|es| {
            anyhow!(es
                .iter()
                .map(|(expect, maybe_token)| match maybe_token {
                    Some(token) => {
                        format!("({}, {})", expect.as_str(), token.as_str())
                    }
                    None => format!("({})", expect.as_str()),
                })
                .collect::<Vec<_>>()
                .join(","))
        })?;

    // compile the filter in the context of the given definitions
    let filter = ctx.compile(f.conv(&interest_filter));

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

        let mut tags = vec![];

        for (filter, plugin_name) in &self.filters {
            match run_filter(filter, plugin_name, msg_json.clone()) {
                Ok(JaqFilterResult::Copy(tag)) => tags.push(tag),
                Ok(JaqFilterResult::Take(tag)) => {
                    tags.push(tag);
                    return FilterResult::Interest(Interest { tags, remove: true });
                }
                Ok(JaqFilterResult::NoInterest) => {}
                Err(_) => {}
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
    level = "info",
    skip_all,
    fields(%plugin_name)
)]
pub fn run_filter<'a>(
    filter: &Filter,
    plugin_name: &'a str,
    msg_json: Val,
) -> Result<JaqFilterResult<'a>, ()> {
    let inputs = RcIter::new(core::iter::empty());
    let mut out = filter
        .run((
            Ctx::new([Val::Str(Rc::new(plugin_name.to_owned()))], &inputs),
            msg_json.clone(),
        ))
        .peekable();

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

/// For simple filters that either take the item they're interested in or express no interest (i.e. they never just copy an item). This wraps the provided filter (which is expected to return a bool) in an expression maps that maps false to null.
pub fn simple_take_filter(inner_filter: String) -> String {
    format!(r#"if {inner_filter} then true else null end"#)
}
