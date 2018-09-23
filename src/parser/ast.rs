use super::lex::*;
use super::{AngleMode, VarMap, VariableValue};
use std::borrow::Cow;

#[derive(Debug)]
pub enum Expression {
    Binary {
        op: Operator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    CallExpresion {
        arg: Box<Expression>,
        func: String,
    },
    Number(f64),
    Paren(Box<Expression>),
    Negation(Box<Expression>),
}

impl Expression {
    pub fn get_value(&self, mode: AngleMode, context: &VarMap) -> Result<f64, ParseError> {
        use self::Expression::*;

        match self {
            Binary { op, left, right } => Ok(op.apply(
                left.get_value(mode.clone(), context)?,
                right.get_value(mode, context)?,
            )),
            Number(value) => Ok(*value),
            Paren(exp) => exp.get_value(mode, context),
            CallExpresion { arg, func } => {
                if let Some(VariableValue::Function(f)) = context.get(&**func) {
                    arg.get_value(mode.clone(), context).map(|val| f(val, mode))
                } else {
                    Err(ParseError::NonFunction(func.clone()))
                }
            }
            Negation(exp) => exp.get_value(mode, context).map(|v| -v),
        }
    }

    pub fn negate(mut self, neg: usize) -> Expression {
        for _ in 0..neg {
            self = Expression::Negation(self.into());
        }
        self
    }

    pub fn non_neg(self) -> (Expression, usize) {
        let mut expr = self;
        let mut level = 0;
        while let Expression::Negation(inner) = expr {
            expr = *inner;
            level += 1;
        }
        (expr, level)
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEOF,
    UndefinedIdent(String),
    UnexpectedToken(Token),
    NonFunction(String),
}

impl From<ParseError> for Cow<'static, str> {
    fn from(e: ParseError) -> Self {
        use self::ParseError::*;

        match e {
            UnexpectedEOF => Cow::Borrowed("Unexpected end of file"),
            UnexpectedToken(token) => Cow::Owned(format!("Unexpected {}", token.get_descriptor())),
            UndefinedIdent(ident) => Cow::Owned(format!("Undefined variable \"{}\"", ident)),
            NonFunction(ident) => Cow::Owned(format!("\"{}\" is not a function", ident)),
        }
    }
}

#[derive(Debug)]
struct ContextualizedTokens {
    expressions: Vec<Expression>,
    operators: Vec<Operator>,
}

impl ContextualizedTokens {
    fn from(variables: &VarMap, arr: Vec<Token>) -> Result<ContextualizedTokens, ParseError> {
        let mut expressions = vec![];
        let mut operators = vec![];

        let mut negation_stack = 0;
        let mut func: Option<String> = None;
        let mut last_paren = false;
        let mut last_op = false;

        for token in arr.into_iter() {
            match token {
                Token::Number(num)
                    if func.is_none() && (last_op || last_paren || expressions.is_empty()) =>
                {
                    if expressions.len() != operators.len() {
                        operators.push(Operator::Mult);
                    }
                    expressions.push(Expression::Number(num).negate(negation_stack));
                    last_op = false;
                    last_paren = false;
                    negation_stack = 0;
                }
                Token::Negation if func.is_none() => negation_stack += 1,
                Token::Op(ref op) if func.is_none() && !last_op => {
                    operators.push(op.clone());
                    last_op = true;
                    last_paren = false;
                }
                Token::Parentheses(paren) => {
                    if expressions.len() != operators.len() {
                        operators.push(Operator::Mult);
                    }
                    if let Some(func) = func.take() {
                        expressions.push(
                            Expression::CallExpresion {
                                func,
                                arg: ast_gen(paren, variables)?.into(),
                            }.negate(negation_stack),
                        )
                    } else {
                        expressions.push(Expression::Paren(
                            ast_gen(paren, variables)?.negate(negation_stack).into(),
                        ));
                    }
                    negation_stack = 0;
                    last_op = false;
                    last_paren = true;
                }
                Token::Var(ref ident) if func.is_none() => match variables
                    .get(&ident[..])
                    .ok_or(ParseError::UndefinedIdent(ident.clone()))?
                {
                    VariableValue::Constant(num) => {
                        if expressions.len() != operators.len() {
                            operators.push(Operator::Mult);
                        }
                        expressions.push(Expression::Number(*num));
                        last_op = false;
                        last_paren = true;
                    }
                    VariableValue::Function(_) => {
                        func = Some(ident.clone());
                    }
                },
                _ => return Err(ParseError::UnexpectedToken(token)),
            }
        }

        Some(Self {
            expressions,
            operators,
        }).filter(|tokens| {
            tokens
                .expressions
                .len()
                .checked_sub(tokens.operators.len())
                .filter(|diff| *diff == 1)
                .is_some()
        }).ok_or(ParseError::UnexpectedEOF)
    }

    fn reduce_at(&mut self, ind: usize, is_exp: bool) {
        if ind + 1 < self.expressions.len() && ind < self.operators.len() {
            let op = self.operators.remove(ind);
            let left = self.expressions.remove(ind);
            let right = self.expressions.remove(ind).into();
            let expr = if is_exp {
                let (left, level) = left.non_neg();
                Expression::Binary {
                    op,
                    left: left.into(),
                    right,
                }.negate(level)
            } else {
                Expression::Binary {
                    op,
                    left: left.into(),
                    right,
                }
            };
            self.expressions.insert(ind, expr);
        }
    }

    pub fn into_ast(mut self) -> Result<Expression, ParseError> {
        if self.expressions.len() != self.operators.len() + 1 {
            return Err(ParseError::UnexpectedEOF);
        }

        for prec in OperatorPrecedence::VALUES.iter().rev() {
            let instances = (0..self.operators.len())
                .filter(|i| {
                    self.operators
                        .get(*i)
                        .filter(|op| op.precedence() == *prec)
                        .is_some()
                }).enumerate()
                .flat_map(|(index, ind)| ind.checked_sub(index))
                .collect::<Vec<usize>>();
            for index in instances {
                self.reduce_at(index, *prec == OperatorPrecedence::Exp);
            }
        }

        self.expressions
            .pop()
            .filter(|_| self.expressions.len() == 0 && self.operators.len() == 0)
            .ok_or(ParseError::UnexpectedEOF)
    }
}

pub fn ast_gen(tokens: Vec<Token>, variables: &VarMap) -> Result<Expression, ParseError> {
    ContextualizedTokens::from(variables, tokens)?.into_ast()
}
