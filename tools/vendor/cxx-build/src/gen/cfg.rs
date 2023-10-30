use crate::gen::{CfgEvaluator, CfgResult};
use crate::syntax::cfg::CfgExpr;
use crate::syntax::report::Errors;
use crate::syntax::Api;
use quote::quote;
use std::collections::BTreeSet as Set;
use syn::Error;

pub(super) struct UnsupportedCfgEvaluator;

impl CfgEvaluator for UnsupportedCfgEvaluator {
    fn eval(&self, name: &str, value: Option<&str>) -> CfgResult {
        let _ = name;
        let _ = value;
        let msg = "cfg attribute is not supported".to_owned();
        CfgResult::Undetermined { msg }
    }
}

pub(super) fn strip(
    cx: &mut Errors,
    cfg_errors: &mut Set<String>,
    cfg_evaluator: &dyn CfgEvaluator,
    apis: &mut Vec<Api>,
) {
    apis.retain(|api| eval(cx, cfg_errors, cfg_evaluator, api.cfg()));
    for api in apis {
        match api {
            Api::Struct(strct) => strct
                .fields
                .retain(|field| eval(cx, cfg_errors, cfg_evaluator, &field.cfg)),
            Api::Enum(enm) => enm
                .variants
                .retain(|variant| eval(cx, cfg_errors, cfg_evaluator, &variant.cfg)),
            _ => {}
        }
    }
}

pub(super) fn eval(
    cx: &mut Errors,
    cfg_errors: &mut Set<String>,
    cfg_evaluator: &dyn CfgEvaluator,
    expr: &CfgExpr,
) -> bool {
    match try_eval(cfg_evaluator, expr) {
        Ok(value) => value,
        Err(errors) => {
            for error in errors {
                if cfg_errors.insert(error.to_string()) {
                    cx.push(error);
                }
            }
            false
        }
    }
}

fn try_eval(cfg_evaluator: &dyn CfgEvaluator, expr: &CfgExpr) -> Result<bool, Vec<Error>> {
    match expr {
        CfgExpr::Unconditional => Ok(true),
        CfgExpr::Eq(ident, string) => {
            let key = ident.to_string();
            let value = string.as_ref().map(|string| string.value());
            match cfg_evaluator.eval(&key, value.as_deref()) {
                CfgResult::True => Ok(true),
                CfgResult::False => Ok(false),
                CfgResult::Undetermined { msg } => {
                    let span = quote!(#ident #string);
                    Err(vec![Error::new_spanned(span, msg)])
                }
            }
        }
        CfgExpr::All(list) => {
            let mut all_errors = Vec::new();
            for subexpr in list {
                match try_eval(cfg_evaluator, subexpr) {
                    Ok(true) => {}
                    Ok(false) => return Ok(false),
                    Err(errors) => all_errors.extend(errors),
                }
            }
            if all_errors.is_empty() {
                Ok(true)
            } else {
                Err(all_errors)
            }
        }
        CfgExpr::Any(list) => {
            let mut all_errors = Vec::new();
            for subexpr in list {
                match try_eval(cfg_evaluator, subexpr) {
                    Ok(true) => return Ok(true),
                    Ok(false) => {}
                    Err(errors) => all_errors.extend(errors),
                }
            }
            if all_errors.is_empty() {
                Ok(false)
            } else {
                Err(all_errors)
            }
        }
        CfgExpr::Not(subexpr) => match try_eval(cfg_evaluator, subexpr) {
            Ok(value) => Ok(!value),
            Err(errors) => Err(errors),
        },
    }
}

impl Api {
    fn cfg(&self) -> &CfgExpr {
        match self {
            Api::Include(include) => &include.cfg,
            Api::Struct(strct) => &strct.cfg,
            Api::Enum(enm) => &enm.cfg,
            Api::CxxType(ety) | Api::RustType(ety) => &ety.cfg,
            Api::CxxFunction(efn) | Api::RustFunction(efn) => &efn.cfg,
            Api::TypeAlias(alias) => &alias.cfg,
            Api::Impl(imp) => &imp.cfg,
        }
    }
}

impl From<bool> for CfgResult {
    fn from(value: bool) -> Self {
        if value {
            CfgResult::True
        } else {
            CfgResult::False
        }
    }
}
