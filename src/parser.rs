use std::collections::HashMap;
use std::process::exit;
use std::{fs, vec};

use crate::ast::{BlockStmt, ExprType, Ident, Literal, Precedence, Program, Stmt};
use crate::lexer::Lexing;
use crate::token::{self, Token};

pub struct Parser<'a> {
    lex: Lexing<'a>,
    current: Token,
    next: Token,
    prev: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lex: Lexing<'a>) -> Parser {
        let prev = Token::Eof;
        let current = lex.next();
        let next = lex.next();
        Parser {
            lex,
            current,
            next,
            prev,
        }
    }

    pub fn parse(&mut self) -> Program {
        let mut program: Program = vec![];
        while self.current != Token::Eof {
            match self.parse_stmt() {
                Some(stmt) => {
                    program.push(stmt);
                }
                None => {
                    break;
                }
            };
        }
        program
    }

    pub fn get_imports(&mut self, program: Program) -> Option<HashMap<String, String>> {
        let imports = program.iter().filter(|stmt| match stmt {
            Stmt::Import(_) => true,
            _ => false,
        });
        if imports.clone().count() == 0 {
            return None;
        }
        let mut progs = HashMap::new();
        for import in imports.clone() {
            match import {
                Stmt::Import(s) => {
                    let current_dir = std::env::current_dir().unwrap();
                    let filename = format!("{}.lox", s);
                    println!("current_dir: {:?}", current_dir.join(filename.clone()));
                    let file_contents = fs::read_to_string(current_dir.join(filename)).unwrap();
                    if !file_contents.is_empty() {
                        progs.insert(s.to_string(), file_contents);
                    }
                }
                _ => {}
            }
        }
        Some(progs)
    }

    fn parse_ident(&mut self) -> Option<Ident> {
        match self.current.clone() {
            Token::Identifier(s) => Some(Ident(s)),
            _ => None,
        }
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.current.clone() {
            Token::Eof => {
                return None;
            }
            Token::Fun => {
                self.next();
                let indent = self.parse_ident().unwrap();
                if let Some(ExprType::Function { params, body }) = self.parse_function() {
                    return Some(Stmt::Function(indent, params, body));
                }
                return None;
            }
            Token::Import => self.parse_import_stmt(),
            Token::Return => self.parse_return(),
            Token::Var => self.parse_var_stmt(),
            Token::Switch => self.parse_switch(),
            Token::While => self.parse_while(),
            Token::Class => self.parse_class(),
            Token::For => self.parse_for_loop(),
            Token::Assert => self.parse_assert_expr(),
            Token::LeftBrace => {
                self.next();
                let mut stmts: Program = vec![];
                while self.current != Token::RightBrace {
                    match self.parse_stmt() {
                        Some(stmt) => {
                            stmts.push(stmt);
                        }
                        None => {
                            break;
                        }
                    }
                }
                if self.current != Token::RightBrace {
                    exit(65);
                }
                self.next();
                return Some(Stmt::Block(stmts));
            }
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_expr_stmt(&mut self) -> Option<Stmt> {
        match self.parse_expr(Precedence::Lowest) {
            Some(expr) => {
                if self.current == Token::Semicolon {
                    self.next();
                }
                return Some(Stmt::Expr(expr));
            }
            None => {
                return None;
            }
        }
    }

    fn parse_var_stmt(&mut self) -> Option<Stmt> {
        self.next();
        let ident = match self.current.clone() {
            Token::Identifier(s) => {
                self.next();
                Ident(s)
            }
            _ => {
                return None;
            }
        };
        if self.current == Token::Semicolon {
            self.next();
            return Some(Stmt::Var(ident, ExprType::Literal(Literal::Nil)));
        } else if self.current == Token::In {
            return Some(Stmt::Var(ident, ExprType::Literal(Literal::Nil)));
        }
        self.next();
        let expr = self.parse_expr(Precedence::Lowest).unwrap();
        if self.current != Token::Semicolon {
            eprintln!(
                "self.current != Token::Semicolon Unexpected token: {:?}",
                self.current
            );
            exit(0);
        }
        self.next();
        Some(Stmt::Var(ident, expr))
    }

    fn parse_for_loop(&mut self) -> Option<Stmt> {
        self.next();
        if self.current != Token::LeftParen {
            self.lex
                .log_error(self.current.clone(), "Expect '(' after for");
            return None;
        }
        self.next();
        let init = match self.current {
            Token::Var => self.parse_var_stmt().unwrap(),
            _ => {
                self.lex
                    .log_error(self.current.clone(), "Expect var after for");
                return None;
            }
        };
        if self.current == Token::In {
            self.next();
            let expr = self.parse_expr(Precedence::Lowest).unwrap();
            if self.current != Token::RightParen {
                self.lex
                    .log_error(self.current.clone(), "Expect ')' after for condition");
                return None;
            }
            self.next();
            let block = self.parse_block().unwrap();
            return Some(Stmt::ForIn {
                var: Box::new(init),
                iter: Box::new(expr),
                block: block,
            });
        }
        let condition = self.parse_expr(Precedence::Lowest).unwrap();
        if self.current != Token::Semicolon {
            self.lex
                .log_error(self.current.clone(), "Expect ';' after for condition");
            return None;
        }
        self.next();

        let step = self.parse_expr(Precedence::Lowest).unwrap();
        if self.current != Token::RightParen {
            self.lex
                .log_error(self.current.clone(), "Expect ')' after for condition");
            return None;
        }
        self.next();
        let block = self.parse_block().unwrap();
        Some(Stmt::For {
            init: Box::new(init),
            conditions: Box::new(condition),
            step: Box::new(Stmt::Expr(step)),
            block: block,
        })
    }

    fn parse_class(&mut self) -> Option<Stmt> {
        self.next();
        let ident = self.parse_ident().unwrap();
        self.next();
        if self.current != Token::LeftBrace {
            self.lex
                .log_error(self.current.clone(), "Expect '{' after class name");
            return None;
        }
        let properties = self.parse_block().unwrap();

        Some(Stmt::ClassStmt {
            name: ident,
            properties: properties,
        })
    }

    fn parse_while(&mut self) -> Option<Stmt> {
        self.next();
        if self.current != Token::LeftParen {
            self.lex
                .log_error(self.current.clone(), "Expect '(' after while");
            return None;
        }
        self.next();
        let expr = self.parse_expr(Precedence::Lowest).unwrap();
        if self.current != Token::RightParen {
            self.lex
                .log_error(self.current.clone(), "Expect ')' after while condition");
            return None;
        }
        self.next();
        if self.current != Token::LeftBrace {
            self.lex
                .log_error(self.current.clone(), "Expect '{' after while condition");
            return None;
        }
        let body = self.parse_block().unwrap();
        Some(Stmt::While(expr, body))
    }

    fn parse_case_block(&mut self) -> Option<BlockStmt> {
        let mut stmts = vec![];
        while self.current != Token::RightBrace
            && self.current != Token::Case
            && self.current != Token::Default
        {
            match self.parse_stmt() {
                Some(stmt) => {
                    stmts.push(stmt);
                }
                None => {
                    break;
                }
            }
        }
        Some(stmts)
    }

    fn parse_block(&mut self) -> Option<BlockStmt> {
        self.next();
        let mut stmts: Program = vec![];
        while self.current != Token::RightBrace {
            match self.parse_stmt() {
                Some(stmt) => {
                    stmts.push(stmt);
                }
                None => {
                    break;
                }
            }
        }

        if self.current != Token::RightBrace {
            eprintln!("self.current != Token::RightBrace {:?}", self.current);
            exit(0);
        }
        self.next();
        Some(stmts)
    }

    fn parse_if(&mut self) -> Option<ExprType> {
        self.next();
        if self.current != Token::LeftParen {
            self.lex
                .log_error(self.current.clone(), "Expect '(' after if");
            return None;
        }
        self.next();
        let condition = self.parse_expr(Precedence::Lowest);
        if self.current != Token::RightParen {
            self.lex
                .log_error(self.current.clone(), "Expect ')' after if condition");
            return None;
        }
        self.next();
        let then_branch: Program = self.parse_block().unwrap();
        let mut elseif: Vec<(Box<ExprType>, Program)> = vec![];
        let mut else_branch: Program = vec![];

        while self.current == Token::Else && self.next == Token::If {
            self.next();
            self.next();
            if self.current != Token::LeftParen {
                self.lex
                    .log_error(self.current.clone(), "Expect '(' after if");
                return None;
            }
            self.next();
            let condition = self.parse_expr(Precedence::Lowest);
            if self.current != Token::RightParen {
                self.lex
                    .log_error(self.current.clone(), "Expect ')' after if condition");
                return None;
            }
            self.next();
            let block = self.parse_block().unwrap();
            elseif.push((Box::new(condition.unwrap()), block));
        }

        if self.current == Token::Else {
            self.next();
            else_branch = self.parse_block().unwrap();
        }

        Some(ExprType::If {
            condition: Box::new(condition.unwrap()),
            elseif: elseif,
            then_branch: then_branch,
            else_branch: else_branch,
        })
    }

    fn parse_function(&mut self) -> Option<ExprType> {
        self.next();
        if self.current != Token::LeftParen {
            self.lex
                .log_error(self.current.clone(), "Expect '(' after function");
            return None;
        }
        self.next();
        let mut params: Vec<Ident> = vec![];
        while self.current != Token::RightParen {
            let ident = match self.current.clone() {
                Token::Identifier(s) => {
                    self.next();
                    Ident(s)
                }
                _ => {
                    self.lex
                        .log_error(self.current.clone(), "Expect identifier");
                    return None;
                }
            };
            params.push(ident);
            if self.current == Token::Comma {
                self.next();
            }
        }
        self.next();
        let body = self.parse_block().unwrap();
        Some(ExprType::Function { params, body })
    }

    fn parse_call(&mut self, left: ExprType) -> Option<ExprType> {
        self.next();
        let mut args = vec![];
        while self.current != Token::RightParen {
            let arg = self.parse_expr(Precedence::Lowest);
            if let Some(arg) = arg {
                args.push(arg);
            }
            if self.current == Token::Comma {
                self.next();
            }
        }
        self.next();
        Some(ExprType::Call {
            callee: Box::new(left),
            args: args,
        })
    }

    fn current_token_precedence(&self) -> Precedence {
        self.token_precedence(self.current.clone())
    }

    // fn next_token_precedence(&self) -> Precedence {
    //     self.token_precedence(self.next.clone())
    // }

    fn token_precedence(&self, token: Token) -> Precedence {
        match token {
            Token::And | Token::Or => Precedence::And,
            Token::EqualEqual | Token::BangEqual | Token::Equal => Precedence::Equals,
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => {
                Precedence::LessGreater
            }
            Token::SlashSelf | Token::StarSelf | Token::ModSelf => Precedence::OpSelfMul,
            Token::MinusSelf | Token::PlusSelf => Precedence::OpSelfSum,
            Token::PlusPlus | Token::MinusMinus => Precedence::PlusPlus,
            Token::Plus | Token::Minus => Precedence::Plus,
            Token::Star | Token::Mod | Token::Slash => Precedence::Star,
            Token::LeftParen => Precedence::Call,
            Token::LeftBracket => Precedence::Index,
            Token::Dot => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }

    fn parse_return(&mut self) -> Option<Stmt> {
        self.next();
        let value = self.parse_expr(Precedence::Lowest);
        if self.current != Token::Semicolon {
            self.lex.log_error(
                self.current.clone(),
                format!("Expect ';' after return, {:?}", self.current).as_str(),
            );
            return None;
        }
        self.next();
        Some(Stmt::Return(value.unwrap()))
    }

    fn parse_switch(&mut self) -> Option<Stmt> {
        self.next();
        let expr = self.parse_expr(Precedence::Lowest).unwrap();
        let mut cases = vec![];
        if self.current != Token::LeftBrace {
            self.lex
                .log_error(self.current.clone(), "Expect '{' after switch");
            return None;
        }
        self.next();

        while self.current == Token::Case {
            let case = self.parse_case().unwrap();
            cases.push(case);
        }

        if self.current == Token::Default {
            let default = self.parse_default().unwrap();
            cases.push(default);
        }

        if self.current != Token::RightBrace {
            self.lex
                .log_error(self.current.clone(), "Expect '}' after switch");
            return None;
        }
        self.next();

        Some(Stmt::Switch(expr, cases))
    }

    fn parse_case(&mut self) -> Option<Stmt> {
        self.next();
        let left = self.parse_expr(Precedence::Lowest);
        if self.current != Token::Colon {
            self.lex.log_error(self.current.clone(), "Expect Colon");
            return None;
        }
        self.next();
        let body = self.parse_case_block().unwrap();
        Some(Stmt::Case(left.unwrap(), body))
    }

    fn parse_default(&mut self) -> Option<Stmt> {
        self.next();
        if self.current != Token::Colon {
            self.lex.log_error(self.current.clone(), "Expect Colon");
            return None;
        }
        self.next();
        let body = self.parse_case_block().unwrap();
        Some(Stmt::Default(body))
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Option<ExprType> {
        // println!("parse_expr: {:?} {:?}", self.current, precedence);
        // prefix
        let mut left = match self.current.clone() {
            Token::Bang | Token::Plus | Token::Minus => self.parse_prefix_expr(),
            Token::LeftParen => self.parse_grouped_expr(),
            Token::Number(_) => self.parse_number_literal(),
            Token::Identifier(ident) => {
                self.next();
                Some(ExprType::Ident(Ident(ident)))
            }
            Token::True => {
                self.next();
                Some(ExprType::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.next();
                Some(ExprType::Literal(Literal::Bool(false)))
            }
            Token::String(s) => {
                self.next();
                Some(ExprType::Literal(Literal::String(s)))
            }
            Token::LeftBracket => {
                self.next();
                let mut elements = vec![];
                while self.current != Token::RightBracket {
                    let element = self.parse_expr(Precedence::Lowest);
                    if let Some(element) = element {
                        elements.push(element);
                    }
                    if self.current == Token::Comma {
                        self.next();
                    }
                }
                self.next();
                Some(ExprType::Literal(Literal::Array(elements)))
            }
            Token::LeftBrace => self.parse_hash_literal(),
            Token::Nil => {
                self.next();
                Some(ExprType::Literal(Literal::Nil))
            }
            Token::New => self.parse_new_class(),
            Token::This => {
                self.next();
                if self.current != Token::Dot {
                    self.lex
                        .log_error(self.current.clone(), "Expect '.' after this");
                    return None;
                }
                self.next();
                let ident = self.parse_ident().unwrap();
                self.next();
                if self.current == Token::LeftParen {
                    self.next();
                    let args = self.parse_args_list();
                    return Some(ExprType::ThisCall {
                        method: ident,
                        args: args.unwrap(),
                    });
                }
                Some(ExprType::ThisExpr(ident))
            }
            Token::If => self.parse_if(),
            Token::Fun => self.parse_function(),
            Token::Print => {
                self.next();
                let mut exprs = vec![];
                while self.current != Token::Semicolon {
                    let expr = self.parse_expr(Precedence::Lowest);
                    if let Some(expr) = expr {
                        exprs.push(expr);
                    }
                    if self.current == Token::Comma {
                        self.next();
                    }
                }
                return Some(ExprType::PrintExpr(Box::new(exprs)));
            }
            _ => {
                println!("Unexpected token in parse_expr: {:?}", self.current);
                self.lex
                    .log_error(self.current.clone(), "Expect expression");
                return None;
            }
        };
        // println!("left: {:?} {:?}", left, self.current);
        // println!(
        //     "left: {:?} precedence:{:?} current_precedence:{:?}, token: {:?}, < {:?}",
        //     left,
        //     precedence,
        //     self.current_token_precedence(),
        //     self.current,
        //     self.current != Token::Semicolon && precedence < self.current_token_precedence()
        // );
        // infix
        while self.current != Token::Semicolon && precedence < self.current_token_precedence() {
            match self.current.clone() {
                Token::Star
                | Token::Slash
                | Token::Plus
                | Token::Minus
                | Token::EqualEqual
                | Token::Equal
                | Token::BangEqual
                | Token::Less
                | Token::LessEqual
                | Token::Greater
                | Token::And
                | Token::Or
                | Token::MinusSelf
                | Token::PlusSelf
                | Token::StarSelf
                | Token::SlashSelf
                | Token::ModSelf
                | Token::Mod
                | Token::GreaterEqual => {
                    left = self.parse_infix_expr(left.unwrap());
                }
                Token::LeftParen => {
                    left = self.parse_call(left.unwrap());
                }
                Token::LeftBracket => {
                    left = self.parse_index_expr(left.unwrap());
                }
                Token::Dot => {
                    let class_name = match left {
                        Some(ExprType::Ident(ident)) => ident,
                        _ => {
                            self.lex
                                .log_error(self.current.clone(), "Expect identifier before '.'");
                            return None;
                        }
                    };

                    self.next();
                    let ident = self.parse_ident().unwrap();
                    self.next();
                    if self.current != Token::LeftParen {
                        left = Some(ExprType::ClassGet {
                            callee: class_name,
                            prop: ident,
                        });
                    } else {
                        self.next();
                        let args = self.parse_args_list();

                        left = Some(ExprType::ClassCall {
                            callee: class_name,
                            method: ident,
                            args: args.unwrap(),
                        });
                    }
                }
                _ => return left,
            }
        }

        // println!("left: {:?}", left);
        left
    }

    fn parse_hash_literal(&mut self) -> Option<ExprType> {
        self.next();
        let mut hash = vec![];
        while self.current != Token::RightBrace {
            let key = self.parse_expr(Precedence::Lowest).unwrap();
            if self.current != Token::Colon {
                self.lex
                    .log_error(self.current.clone(), "Expect ':' after hash key");
                return None;
            }
            self.next();
            let value = self.parse_expr(Precedence::Lowest).unwrap();
            if self.current == Token::Comma {
                self.next();
            }
            hash.push((key, value));
        }
        self.next();
        Some(ExprType::Literal(Literal::Hash(hash)))
    }

    fn parse_import_stmt(&mut self) -> Option<Stmt> {
        self.next();
        if let Token::String(s) = self.current.clone() {
            self.next();
            if self.current != Token::Semicolon {
                self.lex
                    .log_error(self.current.clone(), "Expect ';' after import");
                return None;
            }
            self.next();
            return Some(Stmt::Import(s));
        }
        self.lex
            .log_error(self.current.clone(), "Expect string after import");
        return None;
    }

    fn parse_index_expr(&mut self, left: ExprType) -> Option<ExprType> {
        self.next();

        let index = self.parse_expr(Precedence::Lowest).unwrap();

        if self.current == Token::RightBracket {
            self.next();
        } else {
            self.lex
                .log_error(self.current.clone(), "Expect ']' after index");
            return None;
        }

        if let ExprType::Literal(Literal::Number(v)) = index {
            return Some(ExprType::IndexExpr(
                Box::new(left),
                Box::new(ExprType::Literal(Literal::Index(v as usize))),
            ));
        }

        Some(ExprType::IndexExpr(Box::new(left), Box::new(index)))
    }

    fn parse_args_list(&mut self) -> Option<Vec<ExprType>> {
        let mut args = vec![];
        while self.current != Token::RightParen {
            let arg = self.parse_expr(Precedence::Lowest);
            if let Some(arg) = arg {
                args.push(arg);
            }
            if self.current == Token::Comma {
                self.next();
            }
        }
        self.next();
        Some(args)
    }

    fn parse_new_class(&mut self) -> Option<ExprType> {
        self.next();
        let ident = self.parse_ident().unwrap();
        self.next();
        if self.current != Token::LeftParen {
            self.lex
                .log_error(self.current.clone(), "Expect '(' after new");
            return None;
        }
        self.next();
        let mut args = vec![];
        while self.current != Token::RightParen {
            let arg = self.parse_expr(Precedence::Lowest);
            if let Some(arg) = arg {
                args.push(arg);
            }
            if self.current == Token::Comma {
                self.next();
            }
        }
        self.next();
        Some(ExprType::ClassInit {
            name: ident,
            args: args,
        })
    }

    fn parse_grouped_expr(&mut self) -> Option<ExprType> {
        self.next();

        let expr = self.parse_expr(Precedence::Lowest);
        if self.current == Token::RightParen {
            self.next();
        } else {
            println!("Unexpected token: {:?}", self.current);
            self.lex
                .log_error(self.current.clone(), "Expect expression");
            return None;
        }
        match expr {
            Some(expr) => Some(ExprType::GroupingExpr(Box::new(expr))),
            None => {
                println!("Unexpected token: {:?}", self.current);
                self.lex
                    .log_error(self.current.clone(), "Expect expression");
                return None;
            }
        }
    }

    fn parse_prefix_expr(&mut self) -> Option<ExprType> {
        let op = self.current.clone();
        self.next();
        let right = self.parse_expr(Precedence::Prefix);
        if let Some(right) = right {
            return Some(ExprType::PrefixExpr(op, Box::new(right)));
        }
        println!("Unexpected token: {:?}", self.current);
        self.lex
            .log_error(self.current.clone(), "Expect expression");
        return None;
    }

    fn parse_number_literal(&mut self) -> Option<ExprType> {
        return match self.current.clone() {
            Token::Number(n) => {
                let num = n.parse::<f64>().unwrap();
                self.next();
                Some(ExprType::Literal(Literal::Number(num)))
            }
            _ => {
                panic!("Unexpected");
            }
        };
    }

    fn parse_infix_expr(&mut self, left: ExprType) -> Option<ExprType> {
        let op = self.current.clone();
        let mut precedence = self.current_token_precedence();
        self.next();
        if op == token::Token::Equal {
            precedence = self.current_token_precedence();
        }

        // println!("parse_infix_expr: {:?} {:?} {:?} {:?}", left, op, self.current, precedence);

        if let Some(right) = self.parse_expr(precedence) {
            return Some(ExprType::InfixExpr(Box::new(left), op, Box::new(right)));
        }
        println!("Unexpected token: {:?}", self.current);
        self.lex
            .log_error(self.current.clone(), "Expect expression");
        return None;
    }

    fn parse_assert_expr(&mut self) -> Option<Stmt> {
        self.next();
        let expr = self.parse_expr(Precedence::Lowest);
        if self.current == Token::Comma {
            self.next();
            if let Token::String(s) = self.current.clone() {
                self.next();

                if self.current != Token::Semicolon {
                    self.lex
                        .log_error(self.current.clone(), "Expect ';' after assert message");
                    return None;
                }
                self.next();
                return Some(Stmt::Assert {
                    condition: Box::new(expr.unwrap()),
                    message: Box::new(ExprType::Literal(Literal::String(s))),
                });
            }
        } else if self.current != Token::Semicolon {
            self.lex
                .log_error(self.current.clone(), "Expect ';' after assert");
            return None;
        }

        self.next();
        Some(Stmt::Assert {
            condition: Box::new(expr.unwrap()),
            message: Box::new(ExprType::Literal(Literal::String("".to_string()))),
        })
    }

    fn next(&mut self) -> Token {
        let next = self.lex.next();
        self.prev = self.current.clone();
        self.current = self.next.clone();
        self.next = next;
        let tok = self.current.clone();
        match tok {
            Token::Comment(_) => self.next(),
            _ => tok,
        }
    }

    pub fn has_errors(&self) -> bool {
        self.lex.has_errors()
    }

    // fn peek(&self) -> Token {
    //     self.next.clone()
    // }
}

#[cfg(test)]
mod test {

    use crate::ast::{ExprType, Ident, Literal, Stmt};
    use crate::lexer::Lexing;
    use crate::parser::Parser;
    use crate::token::Token;

    #[test]
    fn test_var() {
        let input = "var a = 10;".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Var(
                Ident(String::from("a")),
                ExprType::Literal(Literal::Number(10.0))
            ),]
        );
    }

    #[test]
    fn test_nagetive_number() {
        let input = "var a = -10;".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Var(
                Ident(String::from("a")),
                ExprType::PrefixExpr(
                    Token::Minus,
                    Box::new(ExprType::Literal(Literal::Number(10.0)))
                )
            ),]
        );
    }

    #[test]
    fn test_parser() {
        let input = "1 + 2 * 3;".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        println!("lex: {:?}", lex.errors);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Expr(ExprType::InfixExpr(
                Box::new(ExprType::Literal(Literal::Number(1.0))),
                Token::Plus,
                Box::new(ExprType::InfixExpr(
                    Box::new(ExprType::Literal(Literal::Number(2.0))),
                    Token::Star,
                    Box::new(ExprType::Literal(Literal::Number(3.0)))
                ))
            ))]
        )
    }

    #[test]
    fn test_grouped() {
        let input = "(24 * -74 / (61 * 77))".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        println!("{:?}", program[0]);
    }

    #[test]
    fn test_arithmetic_operators_1() {
        let input = "16 * 38 / 58".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Expr(ExprType::InfixExpr(
                Box::new(ExprType::InfixExpr(
                    Box::new(ExprType::Literal(Literal::Number(16.0))),
                    Token::Star,
                    Box::new(ExprType::Literal(Literal::Number(38.0))),
                )),
                Token::Slash,
                Box::new(ExprType::Literal(Literal::Number(58.0)))
            ))]
        );
    }

    #[test]
    fn test_arithmetic_operators_2() {
        let input = "(11 * -77 / (98 * 67))".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        // assert_eq!(
        //     program,
        //     vec![Stmt::Expr(ExprType::InfixExpr(
        //         Box::new(ExprType::InfixExpr(
        //             Box::new(ExprType::Literal(Literal::Number(11.0))),
        //             Token::Star,
        //             Box::new(ExprType::Literal(Literal::Number(38.0))),
        //         )),
        //         Token::Slash,
        //         Box::new(ExprType::Literal(Literal::Number(58.0)))
        //     ))]
        // );
    }

    #[test]
    fn test_double_minutes() {
        let input = "var a = 10; var b = 20; var c = a - -b;".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 3);
        assert_eq!(
            program,
            vec![
                Stmt::Var(
                    Ident(String::from("a")),
                    ExprType::Literal(Literal::Number(10.0))
                ),
                Stmt::Var(
                    Ident(String::from("b")),
                    ExprType::Literal(Literal::Number(20.0))
                ),
                Stmt::Var(
                    Ident(String::from("c")),
                    ExprType::InfixExpr(
                        Box::new(ExprType::Ident(Ident(String::from("a")))),
                        Token::Minus,
                        Box::new(ExprType::PrefixExpr(
                            Token::Minus,
                            Box::new(ExprType::Ident(Ident(String::from("b"))))
                        ))
                    )
                )
            ]
        );
    }

    #[test]
    fn test_bang_true() {
        let input = "!true";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Expr(ExprType::PrefixExpr(
                Token::Bang,
                Box::new(ExprType::Literal(Literal::Bool(true)))
            ))]
        );
    }

    #[test]
    fn test_grouped_string() {
        let input = "(\"foo\")";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Expr(ExprType::GroupingExpr(Box::new(
                ExprType::Literal(Literal::String(String::from("foo")))
            )))]
        );
    }

    #[test]
    fn test_grouped_nil() {
        let input = "(nil)";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Expr(ExprType::GroupingExpr(Box::new(
                ExprType::Literal(Literal::Nil)
            )))]
        );
    }

    #[test]
    fn test_arithmetic_operators_3() {
        let input = "52 + 80 - 94";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Expr(ExprType::InfixExpr(
                Box::new(ExprType::InfixExpr(
                    Box::new(ExprType::Literal(Literal::Number(52.0))),
                    Token::Plus,
                    Box::new(ExprType::Literal(Literal::Number(80.0))),
                )),
                Token::Minus,
                Box::new(ExprType::Literal(Literal::Number(94.0)))
            ))]
        );
    }

    #[test]
    fn test_arithmetic_issue_2() {
        let input = "(-43 + 95) * (68 * 80) / (55 + 75)";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Expr(ExprType::InfixExpr(
                Box::new(ExprType::InfixExpr(
                    Box::new(ExprType::InfixExpr(
                        Box::new(ExprType::PrefixExpr(
                            Token::Minus,
                            Box::new(ExprType::Literal(Literal::Number(43.0)))
                        )),
                        Token::Plus,
                        Box::new(ExprType::Literal(Literal::Number(95.0))),
                    )),
                    Token::Star,
                    Box::new(ExprType::InfixExpr(
                        Box::new(ExprType::Literal(Literal::Number(68.0))),
                        Token::Star,
                        Box::new(ExprType::Literal(Literal::Number(80.0))),
                    )),
                )),
                Token::Slash,
                Box::new(ExprType::InfixExpr(
                    Box::new(ExprType::InfixExpr(
                        Box::new(ExprType::Literal(Literal::Number(55.0))),
                        Token::Plus,
                        Box::new(ExprType::Literal(Literal::Number(75.0))),
                    )),
                    Token::Plus,
                    Box::new(ExprType::Literal(Literal::Number(75.0))),
                )),
            ))]
        );
    }

    #[test]
    fn test_comparison_operator() {
        let input = "83 < 99 < 115";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Expr(ExprType::InfixExpr(
                Box::new(ExprType::InfixExpr(
                    Box::new(ExprType::Literal(Literal::Number(83.0))),
                    Token::Less,
                    Box::new(ExprType::Literal(Literal::Number(99.0))),
                )),
                Token::Less,
                Box::new(ExprType::Literal(Literal::Number(115.0)))
            ))]
        );
    }

    #[test]
    fn test_output_error() {
        let input = "(foo";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        for error in parse.lex.errors {
            eprintln!("{}", error);
        }
        assert_eq!(program.len(), 0);
    }
}
