// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Mathematical expression evaluator for Inspector numeric input fields.
//!
//! Evaluates standard arithmetic expressions (`+`, `-`, `*`, `/`, parentheses)
//! as well as relative offset prefixes (`+10`, `+=10`, `-=5`, `*2`, `*=2`, `/4`, `/=4`)
//! applied against a baseline value.

/// Error conditions encountered during mathematical expression evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathEvalError {
    /// Expression contains invalid characters or empty tokens.
    InvalidSyntax,
    /// Mismatched opening or closing parentheses.
    MismatchedParentheses,
    /// Division by zero or a divisor effectively approaching zero.
    DivisionByZero,
    /// Numeric token could not be parsed into a 32-bit floating point number.
    InvalidNumber,
}

/// Evaluates a user-entered text buffer into a numeric value, optionally applying
/// relative operator prefixes against a known baseline value.
/// Supports:
/// - Absolute numbers and expressions: `10`, `-5.5`, `100 / 2`, `(10 + 5) * 3`
/// - Relative addition: `+5` or `+=5` (returns `baseline + 5.0`)
/// - Relative subtraction: `-=5` (returns `baseline - 5.0`)
/// - Relative multiplication: `*2` or `*=2` (returns `baseline * 2.0`)
/// - Relative division: `/4` or `/=4` (returns `baseline / 4.0`)
pub fn evaluate_inspector_math(buffer: &str, baseline: f32) -> Result<f32, MathEvalError> {
    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        return Err(MathEvalError::InvalidSyntax);
    }

    // Relative prefix checks
    if let Some(rest) = trimmed.strip_prefix("+=") {
        let val = evaluate_expression(rest)?;
        return Ok(baseline + val);
    }
    if let Some(rest) = trimmed.strip_prefix("-=") {
        let val = evaluate_expression(rest)?;
        return Ok(baseline - val);
    }
    if let Some(rest) = trimmed.strip_prefix("*=") {
        let val = evaluate_expression(rest)?;
        return Ok(baseline * val);
    }
    if let Some(rest) = trimmed.strip_prefix("/=") {
        let val = evaluate_expression(rest)?;
        if val.abs() < 1e-6 {
            return Err(MathEvalError::DivisionByZero);
        }
        return Ok(baseline / val);
    }

    // Single-character relative prefixes:
    // `+X` -> baseline + X
    // `*X` -> baseline * X
    // `/X` -> baseline / X
    // (Note: `-X` without `=` is treated as negative number or unary minus)
    if let Some(rest) = trimmed.strip_prefix('+') {
        let val = evaluate_expression(rest)?;
        return Ok(baseline + val);
    }
    if let Some(rest) = trimmed.strip_prefix('*') {
        let val = evaluate_expression(rest)?;
        return Ok(baseline * val);
    }
    if let Some(rest) = trimmed.strip_prefix('/') {
        let val = evaluate_expression(rest)?;
        if val.abs() < 1e-6 {
            return Err(MathEvalError::DivisionByZero);
        }
        return Ok(baseline / val);
    }

    // Standard expression
    evaluate_expression(trimmed)
}

/// Evaluates a standard infix mathematical expression with operator precedence.
pub fn evaluate_expression(input: &str) -> Result<f32, MathEvalError> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err(MathEvalError::InvalidSyntax);
    }
    let mut parser = ExprParser::new(tokens);
    let result = parser.parse_expression()?;
    if parser.has_remaining() {
        return Err(MathEvalError::InvalidSyntax);
    }
    if result.is_nan() || result.is_infinite() {
        return Err(MathEvalError::DivisionByZero);
    }
    Ok(result)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f32),
    Plus,
    Minus,
    Multiply,
    Divide,
    LParen,
    RParen,
}

fn tokenize(input: &str) -> Result<Vec<Token>, MathEvalError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        match c {
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' => {
                // Determine if this is a unary minus or binary subtraction:
                // Unary minus if it's the first token or immediately follows an operator or '('.
                let is_unary = matches!(
                    tokens.last(),
                    None | Some(
                        Token::Plus
                            | Token::Minus
                            | Token::Multiply
                            | Token::Divide
                            | Token::LParen
                    )
                );

                if is_unary {
                    // Check if a number follows directly
                    let mut j = i + 1;
                    while j < chars.len() && chars[j].is_whitespace() {
                        j += 1;
                    }
                    if j < chars.len() && (chars[j].is_ascii_digit() || chars[j] == '.') {
                        let start = j;
                        while j < chars.len() && (chars[j].is_ascii_digit() || chars[j] == '.') {
                            j += 1;
                        }
                        let num_str: String = chars[start..j].iter().collect();
                        let val: f32 = num_str.parse().map_err(|_| MathEvalError::InvalidNumber)?;
                        tokens.push(Token::Number(-val));
                        i = j;
                        continue;
                    }
                }
                tokens.push(Token::Minus);
                i += 1;
            }
            '*' => {
                tokens.push(Token::Multiply);
                i += 1;
            }
            '/' => {
                tokens.push(Token::Divide);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let val: f32 = num_str.parse().map_err(|_| MathEvalError::InvalidNumber)?;
                tokens.push(Token::Number(val));
            }
            _ => return Err(MathEvalError::InvalidSyntax),
        }
    }

    Ok(tokens)
}

struct ExprParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl ExprParser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    fn has_remaining(&self) -> bool {
        self.pos < self.tokens.len()
    }

    // expr = term (('+' | '-') term)*
    fn parse_expression(&mut self) -> Result<f32, MathEvalError> {
        let mut left = self.parse_term()?;

        while let Some(tok) = self.peek() {
            match tok {
                Token::Plus => {
                    self.next();
                    let right = self.parse_term()?;
                    left += right;
                }
                Token::Minus => {
                    self.next();
                    let right = self.parse_term()?;
                    left -= right;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // term = factor (('*' | '/') factor)*
    fn parse_term(&mut self) -> Result<f32, MathEvalError> {
        let mut left = self.parse_factor()?;

        while let Some(tok) = self.peek() {
            match tok {
                Token::Multiply => {
                    self.next();
                    let right = self.parse_factor()?;
                    left *= right;
                }
                Token::Divide => {
                    self.next();
                    let right = self.parse_factor()?;
                    if right.abs() < 1e-6 {
                        return Err(MathEvalError::DivisionByZero);
                    }
                    left /= right;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // factor = primary | '+' factor | '-' factor
    fn parse_factor(&mut self) -> Result<f32, MathEvalError> {
        match self.peek() {
            Some(Token::Plus) => {
                self.next();
                self.parse_factor()
            }
            Some(Token::Minus) => {
                self.next();
                let val = self.parse_factor()?;
                Ok(-val)
            }
            _ => self.parse_primary(),
        }
    }

    // primary = Number | '(' expr ')'
    fn parse_primary(&mut self) -> Result<f32, MathEvalError> {
        match self.next() {
            Some(Token::Number(val)) => Ok(val),
            Some(Token::LParen) => {
                let val = self.parse_expression()?;
                match self.next() {
                    Some(Token::RParen) => Ok(val),
                    _ => Err(MathEvalError::MismatchedParentheses),
                }
            }
            _ => Err(MathEvalError::InvalidSyntax),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_number() {
        assert_eq!(evaluate_inspector_math("42", 0.0), Ok(42.0));
        assert_eq!(evaluate_inspector_math("-3.5", 0.0), Ok(-3.5));
    }

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(evaluate_inspector_math("10 + 20", 0.0), Ok(30.0));
        assert_eq!(evaluate_inspector_math("50 - 15", 0.0), Ok(35.0));
        assert_eq!(evaluate_inspector_math("6 * 7", 0.0), Ok(42.0));
        assert_eq!(evaluate_inspector_math("100 / 4", 0.0), Ok(25.0));
    }

    #[test]
    fn test_precedence_and_parentheses() {
        assert_eq!(evaluate_inspector_math("2 + 3 * 4", 0.0), Ok(14.0));
        assert_eq!(evaluate_inspector_math("(2 + 3) * 4", 0.0), Ok(20.0));
        assert_eq!(evaluate_inspector_math("100 / (2 + 3)", 0.0), Ok(20.0));
    }

    #[test]
    fn test_relative_prefixes() {
        let baseline = 100.0;
        assert_eq!(evaluate_inspector_math("+25", baseline), Ok(125.0));
        assert_eq!(evaluate_inspector_math("+=50", baseline), Ok(150.0));
        assert_eq!(evaluate_inspector_math("-=30", baseline), Ok(70.0));
        assert_eq!(evaluate_inspector_math("*2", baseline), Ok(200.0));
        assert_eq!(evaluate_inspector_math("*=3", baseline), Ok(300.0));
        assert_eq!(evaluate_inspector_math("/4", baseline), Ok(25.0));
        assert_eq!(evaluate_inspector_math("/=2", baseline), Ok(50.0));
    }

    #[test]
    fn test_division_by_zero() {
        assert_eq!(
            evaluate_inspector_math("10 / 0", 0.0),
            Err(MathEvalError::DivisionByZero)
        );
        assert_eq!(
            evaluate_inspector_math("/0", 50.0),
            Err(MathEvalError::DivisionByZero)
        );
    }

    #[test]
    fn test_syntax_errors() {
        assert_eq!(
            evaluate_inspector_math("", 0.0),
            Err(MathEvalError::InvalidSyntax)
        );
        assert_eq!(
            evaluate_inspector_math("hello", 0.0),
            Err(MathEvalError::InvalidSyntax)
        );
        assert_eq!(
            evaluate_inspector_math("(1 + 2", 0.0),
            Err(MathEvalError::MismatchedParentheses)
        );
    }
}