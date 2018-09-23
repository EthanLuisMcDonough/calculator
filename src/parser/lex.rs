use std::borrow::Cow;

simple_enum! {
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    OperatorPrecedence {
        PlusMinus,
        MultDiv,
        Exp
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operator {
    Exp,
    Mult,
    Div,
    Plus,
    Minus,
}

impl Operator {
    pub fn precedence(&self) -> OperatorPrecedence {
        use self::Operator::*;

        match self {
            Exp => OperatorPrecedence::Exp,
            Mult | Div => OperatorPrecedence::MultDiv,
            Plus | Minus => OperatorPrecedence::PlusMinus,
        }
    }

    pub fn from_char(c: char) -> Option<Operator> {
        use self::Operator::*;

        Some(match c {
            '^' => Exp,
            '*' => Mult,
            '/' => Div,
            '+' => Plus,
            '-' => Minus,
            _ => return None,
        })
    }

    pub fn get_char(&self) -> char {
        use self::Operator::*;

        match self {
            Exp => '^',
            Mult => '*',
            Div => '/',
            Plus => '+',
            Minus => '-',
        }
    }

    pub fn is_operator(c: char) -> bool {
        Self::from_char(c).is_some()
    }

    pub fn apply(&self, left: f64, right: f64) -> f64 {
        use self::Operator::*;

        match self {
            Exp => left.powf(right),
            Mult => left * right,
            Div => left / right,
            Plus => left + right,
            Minus => left - right,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Token {
    Number(f64),
    Op(Operator),
    Var(String),
    Parentheses(Vec<Token>),
    Negation,
}

impl Token {
    pub fn is_op(&self) -> bool {
        match self {
            Token::Op(_) => true,
            _ => false,
        }
    }

    pub fn is_neg(&self) -> bool {
        *self == Token::Negation
    }

    pub fn is_num(&self) -> bool {
        match self {
            Token::Number(_) => true,
            _ => false,
        }
    }

    pub fn is_var(&self) -> bool {
        match self {
            Token::Var(_) => true,
            _ => false,
        }
    }

    pub fn is_paren(&self) -> bool {
        match self {
            Token::Parentheses(_) => true,
            _ => false,
        }
    }

    pub fn get_descriptor(&self) -> Cow<'static, str> {
        use self::Token::*;

        match self {
            Number(n) => format!("number {}", n).into(),
            Parentheses(_) => Cow::Borrowed("parentheses expression"),
            Var(name) => format!("variable {}", name).into(),
            Op(op) => format!("operator {}", op.get_char()).into(),
            Negation => Cow::Borrowed("token '-'"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LexError {
    UnexpectedEOF,
    EmptyParentheses,
    UnexpectedCharacter { character: char, position: usize },
}

impl LexError {
    pub fn is_eof(&self) -> bool {
        match self {
            LexError::UnexpectedEOF => true,
            _ => false,
        }
    }
}

impl From<LexError> for Cow<'static, str> {
    fn from(e: LexError) -> Cow<'static, str> {
        use self::LexError::*;

        match e {
            UnexpectedEOF => Cow::Borrowed("Incomplete expression"),
            UnexpectedCharacter {
                character,
                position,
            } => Cow::Owned(format!(
                "Unexpected character '{}' at index {}",
                character, position
            )),
            EmptyParentheses => Cow::Borrowed("Empty parentheses"),
        }
    }
}

trait TokenBuilder: ::std::fmt::Debug {
    fn can_insert(&self, c: char) -> bool;
    fn push(&mut self, c: char) -> Result<(), ()>;
    fn into_token(self: Box<Self>) -> Result<Token, LexError>;
}

#[derive(Debug)]
struct OperatorBuilder {
    inner: Option<Operator>,
}

impl OperatorBuilder {
    fn new() -> Self {
        Self { inner: None }
    }
}

impl TokenBuilder for OperatorBuilder {
    fn can_insert(&self, c: char) -> bool {
        self.inner.is_none() && Operator::is_operator(c)
    }

    fn push(&mut self, c: char) -> Result<(), ()> {
        Operator::from_char(c)
            .filter(|_| self.inner.is_none())
            .map(|op| {
                self.inner = Some(op);
            }).ok_or(())
    }

    fn into_token(self: Box<Self>) -> Result<Token, LexError> {
        self.inner.map(Token::Op).ok_or(LexError::UnexpectedEOF)
    }
}

#[derive(Debug)]
struct NegationBuilder {
    complete: bool,
}

impl NegationBuilder {
    fn new() -> Self {
        Self { complete: false }
    }
}

impl TokenBuilder for NegationBuilder {
    fn can_insert(&self, c: char) -> bool {
        !self.complete && c == '-'
    }

    fn push(&mut self, c: char) -> Result<(), ()> {
        if self.can_insert(c) {
            self.complete = true;
            Ok(())
        } else {
            Err(())
        }
    }

    fn into_token(self: Box<Self>) -> Result<Token, LexError> {
        Some(Token::Negation)
            .filter(|_| self.complete)
            .ok_or(LexError::UnexpectedEOF)
    }
}

#[derive(Debug)]
struct NumberBuilder {
    parts: [String; 3],
    ind: usize,
}

impl NumberBuilder {
    fn new() -> Self {
        NumberBuilder {
            parts: [String::new(), String::new(), String::new()],
            ind: 0,
        }
    }
}

impl TokenBuilder for NumberBuilder {
    fn can_insert(&self, c: char) -> bool {
        c.is_digit(10)
            || c == '-' && self.ind == 2 && self.parts[self.ind].is_empty()
            || c == '+' && self.ind == 2 && self.parts[self.ind].is_empty()
            || c == '.' && self.ind == 0 && !self.parts[self.ind].is_empty()
            || c == 'E'
                && self.ind < 2
                && self.parts[self.ind]
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .count()
                    > 0
    }

    fn push(&mut self, c: char) -> Result<(), ()> {
        match c {
            '0'...'9' => self.parts[self.ind].push(c),
            '-' if self.ind == 2 && self.parts[self.ind].is_empty() => self.parts[self.ind].push(c),
            '+' if self.ind == 2 && self.parts[self.ind].is_empty() => self.parts[self.ind].push(c),
            '.' if self.ind == 0 && !self.parts[self.ind].is_empty() => self.ind += 1,
            'E' if self.ind < 2
                && self.parts[self.ind]
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .count()
                    > 0 =>
            {
                self.ind = 2
            }
            _ => return Err(()),
        }
        Ok(())
    }

    fn into_token(self: Box<Self>) -> Result<Token, LexError> {
        let inchars = [None, Some('.'), Some('E')];

        let processed_parts = (0..self.parts.len())
            .zip(inchars.iter())
            .map(|(index, optch)| {
                Some(format!(
                    "{}{}",
                    optch
                        .map(|c| c.to_string())
                        .filter(|_| !self.parts[index].is_empty())
                        .unwrap_or_default(),
                    self.parts[index]
                )).filter(|_| !self.parts[index].is_empty() || self.ind != index)
            }).collect::<Vec<Option<String>>>();

        if processed_parts.iter().any(|o| o.is_none()) {
            Err(LexError::UnexpectedEOF)
        } else {
            processed_parts
                .into_iter()
                .flatten()
                .collect::<String>()
                .parse()
                .map(Token::Number)
                .map_err(|_| LexError::UnexpectedEOF)
        }
    }
}

#[derive(Debug)]
struct ParenthesesBuilder {
    inner: String,
    level: usize,
    complete: bool,
    start: usize,
}

impl ParenthesesBuilder {
    fn new(start: usize) -> Self {
        Self {
            inner: String::new(),
            level: 0,
            complete: false,
            start,
        }
    }
}

impl TokenBuilder for ParenthesesBuilder {
    fn can_insert(&self, c: char) -> bool {
        !self.complete && (c != ')' || self.level > 0)
    }

    fn push(&mut self, c: char) -> Result<(), ()> {
        match c {
            '(' | ')' if !self.complete => {
                if c == ')' {
                    self.level = self.level.checked_sub(1).ok_or(())?;
                }

                if self.level > 0 {
                    self.inner.push(c)
                }

                if c == '(' {
                    self.level += 1;
                }

                self.complete = self.level == 0;

                Ok(())
            }
            _ if !self.complete => {
                self.inner.push(c);
                Ok(())
            }
            _ => Err(()),
        }
    }

    fn into_token(self: Box<Self>) -> Result<Token, LexError> {
        if self.inner.is_empty() {
            Err(LexError::EmptyParentheses)
        } else if self.complete {
            lex_ind(&self.inner, self.start)
                .map(Token::Parentheses)
                .map_err(|err| {
                    if let LexError::UnexpectedEOF = err {
                        LexError::UnexpectedCharacter {
                            character: ')',
                            position: self.inner.len() + self.start + 1,
                        }
                    } else {
                        err
                    }
                })
        } else {
            Err(LexError::UnexpectedEOF)
        }
    }
}

#[derive(Debug)]
struct VariableBuilder {
    inner: String,
}

impl VariableBuilder {
    fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }
}

impl TokenBuilder for VariableBuilder {
    fn can_insert(&self, c: char) -> bool {
        c.is_ascii_lowercase() || c == '_'
    }

    fn push(&mut self, c: char) -> Result<(), ()> {
        match c {
            'a'...'z' | '_' => self.inner.push(c),
            _ => return Err(()),
        }
        Ok(())
    }

    fn into_token(self: Box<Self>) -> Result<Token, LexError> {
        let not_empty = !self.inner.is_empty();
        Some(self.inner)
            .filter(|_| not_empty)
            .map(Token::Var)
            .ok_or(LexError::UnexpectedEOF)
    }
}

fn lex_ind(s: &str, mut ind: usize) -> Result<Vec<Token>, LexError> {
    use self::LexError::*;

    let mut tokens: Vec<Token> = vec![];
    let mut pending_num: Option<Box<TokenBuilder>> = None;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if pending_num.is_none() {
            let last_is_op = tokens
                .iter()
                .rev()
                .skip_while(|t| t.is_neg())
                .next()
                .filter(|t| t.is_op())
                .is_some();
            let last_is_num = tokens.last().filter(|t| t.is_num()).is_some();
            pending_num = match c {
                _ if c.is_whitespace() => None,
                '-' if (last_is_op || tokens.is_empty()) && !last_is_num => {
                    Some(Box::new(NegationBuilder::new()))
                }
                '0'...'9' if !last_is_num => Some(Box::new(NumberBuilder::new())),
                _ if Operator::is_operator(c) && !last_is_op => {
                    Some(Box::new(OperatorBuilder::new()))
                }
                '(' => Some(Box::new(ParenthesesBuilder::new(ind))),
                'a'...'z' | '_' => Some(Box::new(VariableBuilder::new())),
                _ => {
                    return Err(UnexpectedCharacter {
                        character: c,
                        position: ind,
                    })
                }
            };
        }

        if let Some(mut item) = pending_num.take() {
            item.push(c).map_err(|()| UnexpectedCharacter {
                character: c,
                position: ind,
            })?;

            let next = chars.peek();
            match next {
                Some(ch) if item.can_insert(*ch) => pending_num = Some(item),
                _ => tokens.push(item.into_token().map_err(|e| {
                    next.filter(|_| e.is_eof())
                        .map(|c| UnexpectedCharacter {
                            character: *c,
                            position: ind + 1,
                        }).unwrap_or(e)
                })?),
            }
        }

        ind += 1;
    }

    Some(tokens)
        .filter(|toks| {
            toks.last()
                .filter(|tok| tok.is_op() || tok.is_neg())
                .is_none()
        }).ok_or(UnexpectedEOF)
}

pub fn lex(s: &str) -> Result<Vec<Token>, LexError> {
    lex_ind(s, 0)
}
