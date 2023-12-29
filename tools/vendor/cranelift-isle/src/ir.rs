//! Lowered matching IR.

use crate::lexer::Pos;
use crate::log;
use crate::sema::*;

declare_id!(
    /// The id of an instruction in a `PatternSequence`.
    InstId
);

/// A value produced by a LHS or RHS instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    /// A value produced by an instruction in the Pattern (LHS).
    Pattern {
        /// The instruction that produces this value.
        inst: InstId,
        /// This value is the `output`th value produced by this pattern.
        output: usize,
    },
    /// A value produced by an instruction in the Expr (RHS).
    Expr {
        /// The instruction that produces this value.
        inst: InstId,
        /// This value is the `output`th value produced by this expression.
        output: usize,
    },
}

/// A single Pattern instruction.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PatternInst {
    /// Match a value as equal to another value. Produces no values.
    MatchEqual {
        /// The first value.
        a: Value,
        /// The second value.
        b: Value,
        /// The type of the values.
        ty: TypeId,
    },

    /// Try matching the given value as the given integer. Produces no values.
    MatchInt {
        /// The value to match on.
        input: Value,
        /// The value's type.
        ty: TypeId,
        /// The integer to match against the value.
        int_val: i128,
    },

    /// Try matching the given value as the given constant. Produces no values.
    MatchPrim {
        /// The value to match on.
        input: Value,
        /// The type of the value.
        ty: TypeId,
        /// The primitive to match against the value.
        val: Sym,
    },

    /// Try matching the given value as the given variant, producing `|arg_tys|`
    /// values as output.
    MatchVariant {
        /// The value to match on.
        input: Value,
        /// The type of the value.
        input_ty: TypeId,
        /// The types of values produced upon a successful match.
        arg_tys: Vec<TypeId>,
        /// The value type's variant that we are matching against.
        variant: VariantId,
    },

    /// Evaluate an expression and provide the given value as the result of this
    /// match instruction. The expression has access to the pattern-values up to
    /// this point in the sequence.
    Expr {
        /// The expression to evaluate.
        seq: ExprSequence,
        /// The value produced by the expression.
        output: Value,
        /// The type of the output value.
        output_ty: TypeId,
    },

    // NB: this has to come second-to-last, because it might be infallible, for
    // the same reasons that `Arg` has to be last.
    //
    /// Invoke an extractor, taking the given values as input (the first is the
    /// value to extract, the other are the `Input`-polarity extractor args) and
    /// producing an output value for each `Output`-polarity extractor arg.
    Extract {
        /// Whether this extraction is infallible or not. `false`
        /// comes before `true`, so fallible nodes come first.
        infallible: bool,
        /// The value to extract, followed by polarity extractor args.
        inputs: Vec<Value>,
        /// The types of the inputs.
        input_tys: Vec<TypeId>,
        /// The types of the output values produced upon a successful match.
        output_tys: Vec<TypeId>,
        /// This extractor's term.
        term: TermId,
        /// Is this a multi-extractor?
        multi: bool,
    },

    // NB: This has to go last, since it is infallible, so that when we sort
    // edges in the trie, we visit infallible edges after first having tried the
    // more-specific fallible options.
    //
    /// Get the Nth input argument, which corresponds to the Nth field
    /// of the root term.
    Arg {
        /// The index of the argument to get.
        index: usize,
        /// The type of the argument.
        ty: TypeId,
    },
}

/// A single Expr instruction.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ExprInst {
    /// Produce a constant integer.
    ConstInt {
        /// This integer type.
        ty: TypeId,
        /// The integer value. Must fit within the type.
        val: i128,
    },

    /// Produce a constant extern value.
    ConstPrim {
        /// The primitive type.
        ty: TypeId,
        /// The primitive value.
        val: Sym,
    },

    /// Create a variant.
    CreateVariant {
        /// The input arguments that will make up this variant's fields.
        ///
        /// These must be in the same order as the variant's fields.
        inputs: Vec<(Value, TypeId)>,
        /// The enum type.
        ty: TypeId,
        /// The variant within the enum that we are contructing.
        variant: VariantId,
    },

    /// Invoke a constructor.
    Construct {
        /// The arguments to the constructor.
        inputs: Vec<(Value, TypeId)>,
        /// The type of the constructor.
        ty: TypeId,
        /// The constructor term.
        term: TermId,
        /// Whether this constructor is infallible or not.
        infallible: bool,
        /// Is this a multi-constructor?
        multi: bool,
    },

    /// Set the Nth return value. Produces no values.
    Return {
        /// The index of the return value to set.
        index: usize,
        /// The type of the return value.
        ty: TypeId,
        /// The value to set as the `index`th return value.
        value: Value,
    },
}

impl ExprInst {
    /// Invoke `f` for each value in this expression.
    pub fn visit_values<F: FnMut(Value)>(&self, mut f: F) {
        match self {
            &ExprInst::ConstInt { .. } => {}
            &ExprInst::ConstPrim { .. } => {}
            &ExprInst::Construct { ref inputs, .. }
            | &ExprInst::CreateVariant { ref inputs, .. } => {
                for (input, _ty) in inputs {
                    f(*input);
                }
            }
            &ExprInst::Return { value, .. } => {
                f(value);
            }
        }
    }
}

/// A linear sequence of instructions that match on and destructure an
/// argument. A pattern is fallible (may not match). If it does not fail, its
/// result consists of the values produced by the `PatternInst`s, which may be
/// used by a subsequent `Expr`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct PatternSequence {
    /// Instruction sequence for pattern.
    ///
    /// `InstId` indexes into this sequence for `Value::Pattern` values.
    pub insts: Vec<PatternInst>,
}

/// A linear sequence of instructions that produce a new value from the
/// right-hand side of a rule, given bindings that come from a `Pattern` derived
/// from the left-hand side.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct ExprSequence {
    /// Instruction sequence for expression.
    ///
    /// `InstId` indexes into this sequence for `Value::Expr` values.
    pub insts: Vec<ExprInst>,
    /// Position at which the rule producing this sequence was located.
    pub pos: Pos,
}

impl ExprSequence {
    /// Is this expression sequence producing a constant integer?
    ///
    /// If so, return the integer type and the constant.
    pub fn is_const_int(&self) -> Option<(TypeId, i128)> {
        if self.insts.len() == 2 && matches!(&self.insts[1], &ExprInst::Return { .. }) {
            match &self.insts[0] {
                &ExprInst::ConstInt { ty, val } => Some((ty, val)),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl PatternSequence {
    fn add_inst(&mut self, inst: PatternInst) -> InstId {
        let id = InstId(self.insts.len());
        self.insts.push(inst);
        id
    }
}

/// Used as an intermediate representation of expressions in the [RuleVisitor] implementation for
/// [PatternSequence].
pub struct ReturnExpr {
    seq: ExprSequence,
    output: Value,
    output_ty: TypeId,
}

impl RuleVisitor for PatternSequence {
    type PatternVisitor = Self;
    type ExprVisitor = ExprSequence;
    type Expr = ReturnExpr;

    fn add_arg(&mut self, index: usize, ty: TypeId) -> Value {
        let inst = self.add_inst(PatternInst::Arg { index, ty });
        Value::Pattern { inst, output: 0 }
    }

    fn add_pattern<F: FnOnce(&mut Self)>(&mut self, visitor: F) {
        visitor(self)
    }

    fn add_expr<F>(&mut self, visitor: F) -> ReturnExpr
    where
        F: FnOnce(&mut ExprSequence) -> VisitedExpr<ExprSequence>,
    {
        let mut expr = ExprSequence::default();
        let VisitedExpr { ty, value } = visitor(&mut expr);
        let index = 0;
        expr.add_inst(ExprInst::Return { index, ty, value });
        ReturnExpr {
            seq: expr,
            output: value,
            output_ty: ty,
        }
    }

    fn expr_as_pattern(&mut self, expr: ReturnExpr) -> Value {
        let inst = self.add_inst(PatternInst::Expr {
            seq: expr.seq,
            output: expr.output,
            output_ty: expr.output_ty,
        });

        // Create values for all outputs.
        Value::Pattern { inst, output: 0 }
    }

    fn pattern_as_expr(&mut self, pattern: Value) -> Value {
        pattern
    }
}

impl PatternVisitor for PatternSequence {
    type PatternId = Value;

    fn add_match_equal(&mut self, a: Value, b: Value, ty: TypeId) {
        self.add_inst(PatternInst::MatchEqual { a, b, ty });
    }

    fn add_match_int(&mut self, input: Value, ty: TypeId, int_val: i128) {
        self.add_inst(PatternInst::MatchInt { input, ty, int_val });
    }

    fn add_match_prim(&mut self, input: Value, ty: TypeId, val: Sym) {
        self.add_inst(PatternInst::MatchPrim { input, ty, val });
    }

    fn add_match_variant(
        &mut self,
        input: Value,
        input_ty: TypeId,
        arg_tys: &[TypeId],
        variant: VariantId,
    ) -> Vec<Value> {
        let outputs = arg_tys.len();
        let arg_tys = arg_tys.into();
        let inst = self.add_inst(PatternInst::MatchVariant {
            input,
            input_ty,
            arg_tys,
            variant,
        });
        (0..outputs)
            .map(|output| Value::Pattern { inst, output })
            .collect()
    }

    fn add_extract(
        &mut self,
        input: Value,
        input_ty: TypeId,
        output_tys: Vec<TypeId>,
        term: TermId,
        infallible: bool,
        multi: bool,
    ) -> Vec<Value> {
        let outputs = output_tys.len();
        let inst = self.add_inst(PatternInst::Extract {
            inputs: vec![input],
            input_tys: vec![input_ty],
            output_tys,
            term,
            infallible,
            multi,
        });
        (0..outputs)
            .map(|output| Value::Pattern { inst, output })
            .collect()
    }
}

impl ExprSequence {
    fn add_inst(&mut self, inst: ExprInst) -> InstId {
        let id = InstId(self.insts.len());
        self.insts.push(inst);
        id
    }
}

impl ExprVisitor for ExprSequence {
    type ExprId = Value;

    fn add_const_int(&mut self, ty: TypeId, val: i128) -> Value {
        let inst = self.add_inst(ExprInst::ConstInt { ty, val });
        Value::Expr { inst, output: 0 }
    }

    fn add_const_prim(&mut self, ty: TypeId, val: Sym) -> Value {
        let inst = self.add_inst(ExprInst::ConstPrim { ty, val });
        Value::Expr { inst, output: 0 }
    }

    fn add_create_variant(
        &mut self,
        inputs: Vec<(Value, TypeId)>,
        ty: TypeId,
        variant: VariantId,
    ) -> Value {
        let inst = self.add_inst(ExprInst::CreateVariant {
            inputs,
            ty,
            variant,
        });
        Value::Expr { inst, output: 0 }
    }

    fn add_construct(
        &mut self,
        inputs: Vec<(Value, TypeId)>,
        ty: TypeId,
        term: TermId,
        infallible: bool,
        multi: bool,
    ) -> Value {
        let inst = self.add_inst(ExprInst::Construct {
            inputs,
            ty,
            term,
            infallible,
            multi,
        });
        Value::Expr { inst, output: 0 }
    }
}

/// Build a sequence from a rule.
pub fn lower_rule(termenv: &TermEnv, rule: RuleId) -> (PatternSequence, ExprSequence) {
    let ruledata = &termenv.rules[rule.index()];
    log!("lower_rule: ruledata {:?}", ruledata);

    let mut pattern_seq = PatternSequence::default();
    let mut expr_seq = ruledata.visit(&mut pattern_seq, termenv).seq;
    expr_seq.pos = ruledata.pos;
    (pattern_seq, expr_seq)
}
