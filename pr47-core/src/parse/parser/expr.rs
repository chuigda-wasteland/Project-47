use xjbutil::defer;
use super::Parser;

use crate::syntax::expr::{ConcreteBinaryExpr, ConcreteExpr, ConcreteUnaryExpr};
use crate::syntax::token::{Token, TokenInner};

pub fn operator_prec(token_inner: TokenInner) -> Option<u8> {
    use TokenInner::*;

    match token_inner {
        SymAster | SymSlash | SymPercent => Some(120),
        SymPlus | SymMinus => Some(110),
        SymDLt | SymDGt => Some(100),
        SymAmp => Some(90),
        SymCaret => Some(80),
        SymPipe => Some(70),
        SymDEq | SymNe | SymLt | SymLe | SymGt | SymGe => Some(60),
        SymDAmp | SymDCaret | SymDPipe => Some(50),
        SymEq | SymPlusEq | SymMinusEq | SymAsterEq | SymSlashEq | SymPercentEq => Some(40),
        _ => None
    }
}

impl<'s, 'd> Parser<'s, 'd> {
    pub fn parse_expression(
        &mut self,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteExpr<'s>> {
        let this: &mut Self = self;
        defer!(|this: &mut Self| this.pop_lexer_mode(), this);

        this.parse_binary_expression(
            operator_prec(TokenInner::SymEq).unwrap(),
            skip_set
        )
    }

    pub fn parse_binary_expression(
        &mut self,
        prev_prec: u8,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteExpr<'s>> {
        let mut expr: ConcreteExpr<'s> = self.parse_unary_expression(skip_set)?;

        while let Some(prec) = operator_prec(self.current_token().token_inner) {
            if prec < prev_prec {
                return Some(expr);
            }

            let op_token: Token<'s> = self.consume_token();
            let rhs_expr: ConcreteExpr<'s> = self.parse_binary_expression(prec + 10, skip_set)?;
            expr = ConcreteExpr::BinaryExpr(ConcreteBinaryExpr {
                op: op_token,
                lhs: Box::new(expr),
                rhs: Box::new(rhs_expr)
            })
        }

        unreachable!()
    }

    pub fn parse_unary_expression(
        &mut self,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteExpr<'s>> {
        match self.current_token().token_inner {
            TokenInner::SymPlus | TokenInner::SymMinus |
            TokenInner::SymExclaim | TokenInner::SymTilde => {
                let op_token: Token<'s> = self.consume_token();
                let expr: ConcreteExpr<'s> = self.parse_unary_expression(skip_set)?;
                Some(ConcreteExpr::UnaryExpr(ConcreteUnaryExpr {
                    op: op_token,
                    operand: Box::new(expr)
                }))
            },
            _ => self.parse_postfix_expr(skip_set)
        }
    }

    pub fn parse_postfix_expr(
        &mut self,
        _skip_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteExpr<'s>> {
        todo!()
    }
}
