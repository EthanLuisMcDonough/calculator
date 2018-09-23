use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Debug};

#[macro_use]
mod macros;
mod ast;
pub mod lex;

pub type VarMap = HashMap<&'static str, VariableValue>;

lazy_static! {
    pub static ref DEFAULT_VARS: VarMap = var_map! {
        pi => { ::std::f64::consts::PI },
        e => { ::std::f64::consts::E },
        sin => {
            fn(rad! x) {
                x.sin()
            }
        },
        cos => {
            fn(rad! x) {
                x.cos()
            }
        },
        tan => {
            fn(rad! x) {
                x.tan()
            }
        },
        asin => {
            fn(x, mode) {
                let v = x.asin();
                if mode.is_deg() {
                    v.to_degrees()
                } else { v }
            }
        },
        acos => {
            fn(x, mode) {
                let v = x.acos();
                if mode.is_deg() {
                    v.to_degrees()
                } else { v }
            }
        },
        atan => {
            fn(x, mode) {
                let v = x.atan();
                if mode.is_deg() {
                    v.to_degrees()
                } else { v }
            }
        },
        ceil => {
            fn(x) {
                x.ceil()
            }
        },
        floor => {
            fn(x) {
                x.floor()
            }
        },
        round => {
            fn(x) {
                x.round()
            }
        },
        ln => {
            fn(x) {
                x.ln()
            }
        },
        log => {
            fn(x) {
                x.log10()
            }
        },
        abs => {
            fn(x) {
                x.abs()
            }
        },
        sqrt => {
            fn(x) {
                x.sqrt()
            }
        }
    };
}

pub enum VariableValue {
    Constant(f64),
    Function(Box<Fn(f64, AngleMode) -> f64 + Send + Sync>),
}

impl Debug for VariableValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VariableValue::Constant(num) => write!(f, "VariableValue::Constant({})", num),
            VariableValue::Function(_) => write!(f, "VariableValue::Function"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AngleMode {
    Deg,
    Rad,
}

impl AngleMode {
    pub fn is_deg(&self) -> bool {
        *self == AngleMode::Deg
    }
}

impl ToString for AngleMode {
    fn to_string(&self) -> String {
        match self {
            AngleMode::Deg => "Deg",
            AngleMode::Rad => "Rad",
        }.to_string()
    }
}

impl ::std::ops::Not for AngleMode {
    type Output = AngleMode;

    fn not(self) -> Self {
        match self {
            AngleMode::Deg => AngleMode::Rad,
            AngleMode::Rad => AngleMode::Deg,
        }
    }
}

pub fn eval_math(s: &str, mode: AngleMode) -> Result<f64, Cow<'static, str>> {
    ast::ast_gen(lex::lex(s)?, &DEFAULT_VARS)?
        .get_value(mode, &DEFAULT_VARS)
        .map_err(|e| e.into())
}

pub fn to_fixed(f: f64, place: u32) -> f64 {
    let pow_place = 10f64.powi(place as i32);
    (f * pow_place).round() / pow_place
}

#[cfg(test)]
mod tests;
