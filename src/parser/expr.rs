use std::cell::RefCell;

use crate::{
    ast::{expr::{Expr, ExprKind},
    types::TypeKind, stmt::StmtKind, pattern::Pattern},
    token::TokenKind, source_file::{WithLocation, Located}
};

use super::{Parser, ParseResult, stmt::StmtContext, ParseError};

#[derive(PartialEq, PartialOrd)]
enum OperatorPrecedence {
    Product = 3,
    Sum = 2,
    Call = 1,
    Lowest = 0 
}

impl<'a> TryFrom<&TokenKind<'a>> for OperatorPrecedence {
    type Error = ();

    fn try_from(value: &TokenKind<'a>) -> Result<Self, Self::Error> {
        match value {
            TokenKind::LParen => Ok(Self::Call),
            TokenKind::Plus | TokenKind::Minus => Ok(Self::Sum),
            TokenKind::Star | TokenKind::Slash | TokenKind::Mod => Ok(Self::Product),
            _ => Err(())
        }
    }
}

impl<'a> Parser<'a> {
    pub(super) fn parse_expr(&mut self, context: &StmtContext) -> ParseResult<'a, Expr> {
        self.parse_expr_with_precedence(context, OperatorPrecedence::Lowest)
    }

    fn parse_expr_with_precedence(&mut self, context: &StmtContext, precedence: OperatorPrecedence) -> ParseResult<'a, Expr> {
        let mut expr = self.parse_prefix_expr(context)?;

        while let Ok(op_prec) = self.current().kind().try_into() && precedence < op_prec {
            expr = self.parse_infix_expr(context, expr)?; 
        }

        Ok(expr)
    }

    fn parse_infix_expr(&mut self, context: &StmtContext, left: Expr) -> ParseResult<'a, Expr> {
        match self.current().kind() {
            TokenKind::LParen => self.parse_function_call(context, left),
            TokenKind::Plus | TokenKind::Minus => self.parse_sum(context, left),
            TokenKind::Star | TokenKind::Slash => self.parse_product(context, left),
            _ => self.unexpected(&[TokenKind::Ident("operator".into())])
        }
    }

    fn parse_prefix_expr(&mut self, context: &StmtContext) -> ParseResult<'a, Expr> {
        match self.current().kind() {
            TokenKind::Ident(ident) => self.parse_ident(ident.to_string()),
            TokenKind::Atom(atom) => self.parse_atom(atom.to_string()),
            TokenKind::True | TokenKind::False => self.parse_bool_lit(),
            TokenKind::IntegerLit(int) => self.parse_integer_lit(*int),
            TokenKind::StringLit(str) => self.parse_string_lit(str.to_string()),
            TokenKind::ValOf => self.parse_valof(context),
            TokenKind::LParen => self.parse_parens(context),
            TokenKind::Match => self.parse_match_expr(context, ExprKind::Match),
            TokenKind::Every => self.parse_match_expr(context, ExprKind::Every),
            _ => self.unexpected(&[TokenKind::Ident("expression".into())])
        }
    }

    fn parse_ident(&mut self, ident: String) -> ParseResult<'a, Expr> {
        let loc = self.advance()?.location().clone();

        Ok(Expr::new(loc, None, ExprKind::Ident(ident)))
    }
    
    fn parse_atom(&mut self, atom: String) -> ParseResult<'a, Expr> {
        let loc = self.advance()?.location().clone();

        Ok(Expr::new(
            loc,
            Some(self.get_type(TypeKind::Atom)),
            ExprKind::Atom(self.ast.lock().unwrap().add_atom(atom))
        ))
    }

    fn parse_bool_lit(&mut self) -> ParseResult<'a, Expr> {
        let loc = self.current().location().clone();
        let t = matches!(self.expect(&[TokenKind::True, TokenKind::False])?.kind(), TokenKind::True);

        Ok(Expr::new(loc, Some(self.get_type(TypeKind::Bool)), if t { ExprKind::True } else { ExprKind::False} ))
    }

    fn parse_integer_lit(&mut self, value: u64) -> ParseResult<'a, Expr> {
        let loc = self.advance()?.location().clone();

        let typ = match value {
            _ if value > std::i64::MAX as u64 => TypeKind::UInt64,
            _ if value > std::u32::MAX as u64 => TypeKind::Int64,
            _ if value > std::i32::MAX as u64 => TypeKind::UInt32,
            _ => TypeKind::Int32
        };

        Ok(Expr::new(loc, Some(self.get_type(typ)), ExprKind::IntLit(value)))
    }

    fn parse_string_lit(&mut self, value: String) -> ParseResult<'a, Expr> {
        let loc = self.advance()?.location().clone();
        Ok(Expr::new(loc, Some(self.get_string_type()), ExprKind::StringLit(value)))
    }

    fn parse_valof(&mut self, context: &StmtContext) -> ParseResult<'a, Expr> {
        let loc = self.current().location().clone();
        self.expect(&[TokenKind::ValOf])?;

        let typ = RefCell::new(None);
        let stmt = self.parse_stmt(&StmtContext::ValOf(&typ, context))?;
            
        let typ = typ.take()
            .ok_or_else(|| ParseError::NoResultValue.with_location(loc.clone()))?;
        Ok(Expr::new(loc, typ, ExprKind::ValOf(Box::new(stmt))))
    }

    fn parse_parens(&mut self, context: &StmtContext) -> ParseResult<'a, Expr> {
        self.expect(&[TokenKind::LParen])?;
        let expr = self.parse_expr(context)?;
        self.expect(&[TokenKind::RParen])?;
        Ok(expr)
    }

    fn parse_function_call(&mut self, context: &StmtContext, callee: Expr) -> ParseResult<'a, Expr> {
        let loc = self.expect(&[TokenKind::LParen])?.location().clone();

        let args = self.parse_list(TokenKind::RParen, TokenKind::Comma, Self::parse_expr, context)?;

        Ok(Expr::new(loc, None, ExprKind::FuncCall(Box::new(callee), args)))
    }

    fn parse_sum(&mut self, context: &StmtContext, left: Expr) -> ParseResult<'a, Expr> {
        let tok = self.advance()?;
        let mut right = self.parse_expr_with_precedence(context, OperatorPrecedence::Sum)?;
        
        let typ = left.typ().clone();
        if let Some(typ) = &typ && &Some(*typ) != right.typ() {
            right = right.implicit_cast(*typ);
        }

        let kind = if tok.kind() == &TokenKind::Plus { ExprKind::Add } else { ExprKind::Sub }
            (Box::new(left), Box::new(right));

        Ok(Expr::new(tok.location().clone(), typ, kind))
    }

    fn parse_product(&mut self, context: &StmtContext, left: Expr) -> ParseResult<'a, Expr> {
        let tok = self.advance()?;
        let mut right = self.parse_expr_with_precedence(context, OperatorPrecedence::Product)?;

        let typ = left.typ().clone();
        if let Some(typ) = &typ && &Some(*typ) != right.typ() {
            right = right.implicit_cast(*typ);
        }

        let kind = if tok.kind() == &TokenKind::Star { ExprKind::Mul } else { ExprKind::Div }
            (Box::new(left), Box::new(right));

        Ok(Expr::new(tok.location().clone(), typ, kind))
    }

    fn parse_match_expr(&mut self, context: &StmtContext, init: fn(Vec<Expr>, Vec<(Vec<Located<Pattern>>, Box<Expr>)>) -> ExprKind) -> ParseResult<'a, Expr> {
        let loc = self.advance()?.location().clone();

        let args = if self.current().kind() != &TokenKind::LParen {
            vec![self.parse_expr(context)?]
        }
        else {
            self.parse_optional_list(TokenKind::LParen, TokenKind::RParen, TokenKind::Comma, Self::parse_expr, context)?
        };

        let mut branches = vec![];
        let mut typ = None;
        while self.advance_if(&[TokenKind::Colon])?.is_some() {
            let patterns = self.parse_pattern_list()?;
            if patterns.len() != args.len() {
                return Err(ParseError::WrongNumOfPatterns(args.len()).with_location(loc))
            }

            self.expect(&[TokenKind::Condition])?;
            let mut expr = self.parse_expr(&StmtContext::Match(context))?;
            
            if let Some(typ) = typ {
                if let Some(_) = typ && expr.typ() != &typ {
                    expr = expr.implicit_cast(typ.unwrap());
                }
            }
            else {
                typ = Some(expr.typ().clone());
            }

            branches.push((patterns, Box::new(expr)))
        }

        if branches.is_empty() {
            return Err(ParseError::MissingBranch("match".into()).with_location(loc))
        }

        Ok(Expr::new(loc, typ.unwrap(), init(args, branches)))
    }
}
