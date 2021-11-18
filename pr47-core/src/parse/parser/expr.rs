use super::Parser;

use xjbutil::defer;

use crate::awa;
use crate::diag::diag_data;
use crate::diag::location::SourceRange;
use crate::parse::lexer::LexerMode;
use crate::syntax::expr::{
    ConcreteAsExpr,
    ConcreteAwaitExpr,
    ConcreteBinaryExpr,
    ConcreteExpr,
    ConcreteFieldRefExpr,
    ConcreteFuncCallExpr,
    ConcreteLiteralExpr,
    ConcreteParenthesizedExpr,
    ConcreteSubscriptExpr,
    ConcreteUnaryExpr
};
use crate::syntax::id::Identifier;
use crate::syntax::token::{Token, TokenInner};
use crate::syntax::ty::ConcreteType;

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
        defer!(|this: &mut Self| this.lexer.pop_lexer_mode(), this);
        this.push_lexer_mode(LexerMode::LexExpr);

        this.parse_binary_expression(
            operator_prec(TokenInner::SymEq).unwrap(),
            skip_set
        )
    }

    pub fn parse_expression_no_assign(
        &mut self,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteExpr<'s>> {
        self.parse_binary_expression(
            operator_prec(TokenInner::SymDAmp).unwrap(),
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

        Some(expr)
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
            _ => self.parse_postfix_expression(skip_set)
        }
    }

    pub fn parse_postfix_expression(
        &mut self,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteExpr<'s>> {
        let mut expr: ConcreteExpr<'s> = self.parse_atomic_expression(skip_set)?;

        loop {
            match self.current_token().token_inner {
                TokenInner::SymLBracket => {
                    let lbracket_range: SourceRange = self.consume_token().range;
                    let idx_expr: ConcreteExpr<'s> = self.parse_expression_no_assign(skip_set)?;
                    let rbracket_range: SourceRange =
                        self.expect_n_consume(TokenInner::SymRBracket, skip_set)?.range;
                    expr = ConcreteExpr::SubscriptExpr(ConcreteSubscriptExpr {
                        base: Box::new(expr),
                        idx: Box::new(idx_expr),
                        lbracket_loc: lbracket_range.left(),
                        rbracket_loc: rbracket_range.right()
                    })
                },
                TokenInner::SymLParen => {
                    let lparen_range: SourceRange = self.consume_token().range;
                    let (expr_list, rparen_range): (Vec<ConcreteExpr<'s>>, SourceRange) =
                        self.parse_expression_list(
                            TokenInner::SymComma,
                            TokenInner::SymRParen,
                            skip_set
                        )?;
                    expr = ConcreteExpr::FuncCallExpr(ConcreteFuncCallExpr {
                        func: Box::new(expr),
                        args: expr_list,
                        lparen_loc: lparen_range.left(),
                        rparen_loc: rparen_range.left()
                    })
                },
                TokenInner::SymDot => {
                    let dot_range: SourceRange = self.consume_token().range;
                    match self.current_token().token_inner {
                        TokenInner::Ident(_) => {
                            let ident: Identifier<'s> = self.parse_unqual_ident().or_else(|| {
                                self.skip_to_any_of(skip_set); None
                            })?;
                            expr = ConcreteExpr::FieldRefExpr(ConcreteFieldRefExpr {
                                base: Box::new(expr),
                                id: ident,
                                dot_loc: dot_range.left()
                            })
                        },
                        TokenInner::KwdAwait => {
                            let await_range: SourceRange = self.consume_token().range;
                            expr = ConcreteExpr::AwaitExpr(ConcreteAwaitExpr {
                                base: Box::new(expr),
                                dot_loc: dot_range.left(),
                                await_range
                            });
                        },
                        _ => {
                            self.diag.borrow_mut()
                                .diag(self.current_token().range.left(),
                                      diag_data::err_expected_any_of_0_got_1)
                                .add_arg2(awa![TokenInner::Ident(""), TokenInner::KwdAwait])
                                .add_arg2(self.current_token().token_inner)
                                .add_mark(self.current_token().range.into())
                                .emit();
                            self.skip_to_any_of(skip_set);
                            return None;
                        }
                    }
                },
                TokenInner::KwdAs => {
                    let as_range: SourceRange = self.consume_token().range;
                    let ty: ConcreteType<'s> = self.parse_type(skip_set)?;
                    expr = ConcreteExpr::AsExpr(ConcreteAsExpr {
                        operand: Box::new(expr),
                        dest_type: ty,
                        as_range
                    })
                },
                _ => return Some(expr)
            }
        }
    }

    pub fn parse_atomic_expression(
        &mut self,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<ConcreteExpr<'s>> {
        match self.current_token().token_inner {
            TokenInner::Ident(_) => {
                let ident: Identifier<'s> = self.parse_ident().or_else(|| {
                    self.skip_to_any_of(skip_set);
                    None
                })?;
                Some(ConcreteExpr::IdRefExpr(ident))
            },
            TokenInner::LitInt(lit) => {
                Some(ConcreteExpr::LiteralExpr(ConcreteLiteralExpr::new_lit_int(
                    lit, self.consume_token().range
                )))
            },
            TokenInner::LitFloat(lit) => {
                Some(ConcreteExpr::LiteralExpr(ConcreteLiteralExpr::new_lit_float(
                    lit, self.consume_token().range
                )))
            },
            TokenInner::LitStr(lit) => {
                Some(ConcreteExpr::LiteralExpr(ConcreteLiteralExpr::new_lit_str(
                    lit, self.consume_token().range
                )))
            },
            TokenInner::LitChar(lit) => {
                Some(ConcreteExpr::LiteralExpr(ConcreteLiteralExpr::new_lit_char(
                    lit, self.consume_token().range
                )))
            },
            TokenInner::KwdTrue => {
                Some(ConcreteExpr::LiteralExpr(ConcreteLiteralExpr::new_lit_bool(
                    true, self.consume_token().range
                )))
            },
            TokenInner::KwdFalse => {
                Some(ConcreteExpr::LiteralExpr(ConcreteLiteralExpr::new_lit_bool(
                    false, self.consume_token().range
                )))
            },
            TokenInner::SymLParen => {
                let lparen_range: SourceRange = self.consume_token().range;
                let expr: ConcreteExpr<'s> = self.parse_expression_no_assign(skip_set)?;
                let rparen_range: SourceRange =
                    self.expect_n_consume(TokenInner::SymRParen, skip_set)?.range;
                Some(ConcreteExpr::ParenthesizedExpr(ConcreteParenthesizedExpr {
                    inner: Box::new(expr),
                    lparen_loc: lparen_range.left(),
                    rparen_loc: rparen_range.right()
                }))
            },
            _ => {
                self.diag.borrow_mut()
                    .diag(self.current_token().range.left(),
                          diag_data::err_expected_any_of_0_got_1)
                    .add_arg2(awa![
                        TokenInner::Ident(""),
                        TokenInner::LitInt(0),
                        TokenInner::LitFloat(0.0),
                        TokenInner::LitChar(' '),
                        TokenInner::LitStr(""),
                        TokenInner::KwdTrue,
                        TokenInner::KwdFalse,
                        TokenInner::SymLParen
                    ])
                    .add_arg2(self.current_token().token_inner)
                    .add_mark(self.current_token().range.into())
                    .emit();
                self.skip_to_any_of(skip_set);
                None
            }
        }
    }

    pub fn parse_expression_list(
        &mut self,
        separation: TokenInner,
        termination: TokenInner,
        skip_set: &[&[TokenInner<'_>]]
    ) -> Option<(Vec<ConcreteExpr<'s>>, SourceRange)> {
        self.parse_list_alike(
            Self::parse_expression_no_assign,
            skip_set,
            separation,
            termination,
            skip_set
        )
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;

    use crate::diag::DiagContext;
    use crate::parse::parser::Parser;

    #[test]
    fn test_expr_parsing() {
        let source: &str = "(a.await + b) / c as float";
        let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
        let mut parser: Parser = Parser::new(0, source, &diag);

        dbg!(parser.parse_expression(&[]).unwrap());
    }
}
