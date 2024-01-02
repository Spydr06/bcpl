use crate::{ast::{pattern::{Pattern, PatternTerm}, expr::Expr}, source_file::{WithLocation, Located}, token::TokenKind};

use super::{Parser, ParseResult, ParseError, stmt::StmtContext};

#[derive(PartialEq, PartialOrd)]
enum PatternPrecedence {
    And = 2,
    Or = 1,
    Lowest = 0 
}

impl<'a> TryFrom<&TokenKind<'a>> for PatternPrecedence {
    type Error = ();

    fn try_from(value: &TokenKind<'a>) -> Result<Self, Self::Error> {
        match value {
            TokenKind::LogAnd => Ok(PatternPrecedence::And),
            TokenKind::LogOr => Ok(PatternPrecedence::Or),
            _ => Err(())
        }
    }
}

impl<'a> Parser<'a> {
    pub(super) fn parse_pattern(&mut self) -> ParseResult<'a, Located<Pattern>> {
        self.parse_pattern_with_precedence(PatternPrecedence::Lowest)
    }

    fn parse_pattern_with_precedence(&mut self, precedence: PatternPrecedence) -> ParseResult<'a, Located<Pattern>> {
        let mut pattern = self.parse_prefix_pattern()?;
        
        while let Ok(op_prec) = self.current().kind().try_into() && precedence < op_prec {
            pattern = self.parse_infix_pattern(pattern)?;
        }

        Ok(pattern)
    }

    pub(super) fn parse_pattern_list(&mut self) -> ParseResult<'a, Vec<Located<Pattern>>> {
        let mut patterns = vec![self.parse_pattern()?];
        while self.advance_if(&[TokenKind::Comma])?.is_some() {
            patterns.push(self.parse_pattern()?);
        }
        Ok(patterns)
    }

    fn parse_prefix_pattern(&mut self) -> ParseResult<'a, Located<Pattern>> {
        let loc = self.current().location().clone();

        match self.current().kind().clone() {
            TokenKind::QuestionMark => self.advance().map(|_| Pattern::Any),
            TokenKind::Ident(ident) => {
                let ident = ident.to_string();
                self.advance()?;

                if self.current().kind() == &TokenKind::LParen {
                    let inner = self.parse_pattern_list()?;
                    self.expect(&[TokenKind::RParen])?;
                    Ok(Pattern::Variant(ident, inner))
                }
                else {
                    Ok(Pattern::Query(ident))
                }
            }
            TokenKind::LBracket => {
                self.advance()?;
                if self.advance_if(&[TokenKind::RBracket])?.is_some() {
                    Ok(Pattern::List(vec![]))
                }
                else {
                    let inner = self.parse_pattern_list()?;
                    self.expect(&[TokenKind::RBracket])?;
                    Ok(Pattern::List(inner))
                }
            }
            TokenKind::LParen => {
                self.advance()?;
                let pattern = self.parse_pattern()?;
                self.expect(&[TokenKind::RParen])?;
                Ok(pattern.unwrap())
            }
            TokenKind::Eq => self.parse_prefix_pattern_term(PatternTerm::Eq),
            TokenKind::Ne => self.parse_prefix_pattern_term(PatternTerm::Ne),
            TokenKind::Gt => self.parse_prefix_pattern_term(PatternTerm::Gt),
            TokenKind::Ge => self.parse_prefix_pattern_term(PatternTerm::Ge),
            TokenKind::Lt => self.parse_prefix_pattern_term(PatternTerm::Lt),
            TokenKind::Le => self.parse_prefix_pattern_term(PatternTerm::Le),
            TokenKind::Range => self.advance().map(|_| Pattern::Remaining),
            _ => {
                let expr = self.parse_expr(&StmtContext::Empty)?;
                if self.current().kind() == &TokenKind::Range {
                    self.advance()?;
                    Ok(Pattern::Term(PatternTerm::Range(expr, self.parse_expr(&StmtContext::Empty)?)))
                }
                else {
                    Ok(Pattern::Term(PatternTerm::Basic(expr)))
                }
            }
        }.map(|pattern| pattern.with_location(loc))
    }

    fn parse_prefix_pattern_term(&mut self, init: fn(Expr) -> PatternTerm) -> ParseResult<'a, Pattern> {
        self.advance()?;
        let expr = self.parse_expr(&StmtContext::Empty)?;
        Ok(Pattern::Term(init(expr)))
    }

    fn parse_infix_pattern(&mut self, left: Located<Pattern>) -> ParseResult<'a, Located<Pattern>> {
        let loc = self.current().location().clone();
        match self.current().kind() {
            TokenKind::LogAnd => {
                self.advance()?;
                Ok(
                    Pattern::And(
                        Box::new(left),
                        Box::new(self.parse_pattern_with_precedence(PatternPrecedence::And)?)
                    ).with_location(loc)
                )
            }
            TokenKind::LogOr => {
                self.advance()?;
                Ok(
                    Pattern::Or(
                        Box::new(left),
                        Box::new(self.parse_pattern_with_precedence(PatternPrecedence::Or)?)
                    ).with_location(loc)
                )
            }
            _ => unreachable!()
        }
    } 
}
