use std::io::{self, Write as IoWrite};
use std::fmt;
use std::str::Chars;
use std::iter::Peekable;
use MathError::*;
use Token::*;



fn main() {
    println!("Shunting Yard algorithm calculator, enter an expression to be evaluated.");
    println!("Type `exit` to exit");
    let mut input = String::new();
    // let mut _ans: f64;
    loop {
        print!(">>> ");
        io::stdout().flush().expect("Cannot flush stdout.");
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read stdin.");

        match &input.trim()[..] {
            "exit" => {
                println!("Goodbye.");
                return;
            },
            _ => match Tokens::eval(&input) {
                Ok(float) => {
                    println!("{:.3}", float);
                    // _ans = float;
                }
                Err(error) => println!("{}", error),
            }
        }
        input.clear();
    }
}

#[derive(Debug)]
struct Tokens(Vec<Token>);
impl Tokens {
    fn parse_num(input: &mut Peekable<Chars>) -> Result<f64, MathError> {
        let mut buf = String::new();

        while matches!(input.peek(), Some('0'..='9' | '.')) {
            buf.push(input.next().unwrap());
        }
        return match buf.parse::<f64>() {
            Ok(float) => Ok(float),
            Err(_) => Err(ParseNum(buf))
        }
    }

    fn parse(input: &str) -> Result<Self, MathError> {
        let mut chars = input.chars().peekable();
        let mut tokens = Vec::<Token>::new();
        loop {
            match chars.peek() {
                Some('0'..='9' | '.') => tokens.push(Num(Tokens::parse_num(&mut chars)?)),
                Some('+' | '-' | '*' | '/' | '(' | ')') => tokens.push(Token::from_char(chars.next().unwrap())),
                Some(chr @ '=') | Some(chr) if chr.is_whitespace() => {
                    chars.next().unwrap();
                },
                Some(&badchar) => return Err(BadChar(badchar)),
                None => return Ok(Tokens(tokens))
            }
        }
    }

    fn shunting(self) -> Result<Self, MathError> {
        let mut op_stack = Vec::<Token>::new();
        let mut queue = Vec::<Token>::new();

        for token in &self.0 {
            match token {
                Num(_) => queue.push(*token),
                ParenOpen => op_stack.push(*token),
                ParenClose => {
                    while let Some(top) = op_stack.last()
                            .filter(|top| !matches!(top, ParenOpen)) {
                        queue.push(op_stack.pop().unwrap());
                    }
                    if let None = op_stack.pop() {
                        return Err(UnmatchedParens(self));
                    }
                },
                Oper(_) => {
                    while let Some(_) = op_stack.last()
                            .filter(|top| token.is_lower(top)) {
                        queue.push(op_stack.pop().unwrap());
                    }
                    op_stack.push(*token);
                }
            }
        }
        while let Some(elem) = op_stack.pop() {
            match elem {
                ParenOpen => return Err(UnclosedParens(self)),
                Oper(_) => queue.push(elem),
                _ => unreachable!(),
            }
        }
        Ok(Tokens(queue))
    }

    fn solve(self) -> Result<f64, MathError> {
        let mut stack = Vec::<f64>::new();
        for token in &self.0 {
            match token {
                Num(float) => stack.push(*float),
                Oper(oper) => {
                    if stack.len() < 2 {
                        return Err(NotEnoughTokens(self));
                    }
                    let (y, x) = (stack.pop().unwrap(), stack.pop().unwrap());
                    stack.push(oper.call(x, y));
                },
                _ => unreachable!()
            }
        }
        match stack.len() {
            1 => return Ok(stack.pop().unwrap()),
            _ => return Err(NotEnoughTokens(self)),
        }
    }

    fn eval(input: &str) -> Result<f64, MathError> {
        Self::parse(input.trim())?
            .shunting()?
            .solve()
    }

}
impl fmt::Display for Tokens {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut tokens = self.0.iter();

        write!(f, "{{")?;
        if let Some(token) = tokens.next() {
            write!(f, "{}", token)?;
        }
        for token in tokens {
            write!(f, ", {}", token)?;
        }
        write!(f, "}}")
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
enum Token {
    Num(f64),
    Oper(Operator),
    ParenOpen,
    ParenClose,
}
impl Token {
    fn from_char(chr: char) -> Self {
        match chr {
            '+' | '-' | '*' | '/' => Oper(Operator::from_char(chr)),
            '(' => ParenOpen,
            ')' => ParenClose,
            _ => unreachable!()
        }
    }

    fn is_lower(&self, token: &Token) -> bool {
        if let Oper(oper) = self {
            if let Oper(other) = token {
                return oper.precedence() < other.precedence();
            }
            return false;
        }
        unreachable!()
    }

    fn is_num(&self) -> bool {
        matches!(self, Num(_))
    }

    fn is_oper(&self) -> bool {
        matches!(self, Oper(_))
    }

    fn is_paren(&self) -> bool {
        matches!(self, ParenOpen | ParenClose)
    }

}
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Num(float) => write!(f, "Num({:.3})", float),
            Token::Oper(oper) => write!(f, "Oper({})", oper),
            Token::ParenOpen => write!(f, "ParenOpen"),
            Token::ParenClose => write!(f, "ParenClose"),
            _ => write!(f, "Token `fmt`: not implemented!")
        }
    }
}


#[derive(Copy, Clone, PartialEq, Debug)]
enum Operator {
    Add, Sub, Mul, Div
}
impl Operator {
    fn from_char(chr: char) -> Self {
        match chr {
            '+' => Operator::Add,
            '-' => Operator::Sub,
            '*' => Operator::Mul,
            '/' => Operator::Div,
            _ => unreachable!("Invalid char: `{}`")
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            Operator::Add | Operator::Sub => 1,
            Operator::Mul | Operator::Div => 2,
            _ => panic!("`{}.precendence()`: Not implemented!", self)
        }
    }

    fn call(&self, x: f64, y: f64) -> f64 {
        match self {
            Operator::Add => x + y,
            Operator::Sub => x - y,
            Operator::Mul => x * y,
            Operator::Div => x / y,
            _ => panic!("`{}.call(x, y)`: Not implemented!", self)
        }
    }

}
impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Operator::Add => '+',
            Operator::Sub => '-',
            Operator::Mul => '*',
            Operator::Div => '/',
            _ => '?'
        })
    }
}


enum MathError {
    Generic(String),
    ParseNum(String),
    BadChar(char),
    UnclosedParens(Tokens),
    UnmatchedParens(Tokens),
    NotEnoughTokens(Tokens),
}
impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Generic(string) => write!(f, "Error: `{}`", string),
            ParseNum(string) => write!(f, "Cannot parse literal: `{}`", string),
            BadChar(chr) => write!(f, "Character not supported: `{}`", chr),
            UnclosedParens(tokens) => write!(f, "Opened parentheses were not closed: {}", tokens),
            UnmatchedParens(tokens) => write!(f, "Unmatched closed parentheses: {}", tokens),
            NotEnoughTokens(tokens) => write!(f, "Unmatched numbers and operators: {}", tokens),
            _ => write!(f, "An error occured, but there is no error message implemented for this error"),
        }
    }
}
