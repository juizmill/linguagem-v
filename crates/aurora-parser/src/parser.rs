use aurora_lexer::Token;

use crate::{BinOp, Expr, Stmt};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn advance(&mut self) -> Option<Token> {
        let current = self.peek();

        if current.is_some() {
            self.pos += 1;
        }

        current
    }

    fn parse_primary(&mut self) -> Expr {
        let token = self.advance();
        let token = token.unwrap();

        match token {
            Token::Int(n) => Expr::Int(n),
            Token::Float(n) => Expr::Float(n),
            Token::Ident(nome) => Expr::Ident(nome),
            Token::Str(s) => Expr::Str(s),
            outro => panic!("token inesperado em parse_primary: {outro:?}"),
        }
    }

    fn parse_term(&mut self) -> Expr {
        let mut expr = self.parse_primary();

        loop {
            let op = match self.peek() {
                Some(Token::Multiply) => BinOp::Multiply,
                Some(Token::Divide) => BinOp::Divide,
                _ => break,
            };

            self.advance();
            let right = self.parse_primary();

            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_expression(&mut self) -> Expr {
        let mut expr = self.parse_term();

        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Subtract) => BinOp::Subtract,
                _ => break,
            };

            self.advance();
            let right = self.parse_term();

            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        expr
    }

    fn expect(&mut self, esperado: Token) -> Token {
        let token = self.advance().unwrap();

        if token != esperado {
            panic!("esperava {esperado:?}, encontrei {token:?}");
        }

        token
    }

    fn parse_statement(&mut self) -> Stmt {
        match self.peek() {
            Some(Token::Echo) => {
                self.advance();
                let value = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::Echo(value)
            }

            Some(Token::Let) => {
                self.advance();

                let nome = match self.advance().unwrap() {
                    Token::Ident(nome) => nome,
                    outro => panic!("esperava identificador depois de 'let', encontrei {outro:?}"),
                };

                self.expect(Token::EqualsTo);
                let value = self.parse_expression();
                self.expect(Token::Semicolon);

                Stmt::Let { name: nome, value }
            }

            Some(Token::Ident(nome)) => {
                self.advance();
                self.expect(Token::EqualsTo);
                let value = self.parse_expression();
                self.expect(Token::Semicolon);
                Stmt::Assign { name: nome, value }
            }

            outro => panic!("statement inesperado: {outro:?}"),
        }
    }

    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();

        while self.peek() != Some(Token::Eof) {
            stmts.push(self.parse_statement());
        }

        stmts
    }

    pub fn parse(&mut self) -> Expr {
        self.parse_expression()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_lexer::Lexer;

    #[test]
    fn respeita_precedencia_multiplicacao_antes_de_soma() {
        // "1 + 2 * 3" tem que virar 1 + (2 * 3), não (1 + 2) * 3
        let tokens = Lexer::new("1 + 2 * 3").tokenize();
        let arvore = Parser::new(tokens).parse();

        assert_eq!(
            arvore,
            Expr::Binary {
                left: Box::new(Expr::Int(1)),
                op: BinOp::Add,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Int(2)),
                    op: BinOp::Multiply,
                    right: Box::new(Expr::Int(3)),
                }),
            }
        );
    }

    #[test]
    fn parse_primary_reconhece_string() {
        // Mesmo padrão de Token::Int(n) => Expr::Int(n): confirma que
        // Token::Str vira Expr::Str, sem nenhuma transformação no conteúdo.
        let tokens = Lexer::new("\"oi\"").tokenize();
        let arvore = Parser::new(tokens).parse();

        assert_eq!(arvore, Expr::Str("oi".to_string()));
    }

    #[test]
    fn parseia_let_com_string() {
        // "let nome = "oi";" precisa funcionar pela MESMA cadeia
        // parse_statement -> parse_expression -> parse_term -> parse_primary
        // que já parseia números -- nenhum código dedicado a "let de string".
        let tokens = Lexer::new("let nome = \"oi\";").tokenize();
        let programa = Parser::new(tokens).parse_program();

        assert_eq!(
            programa,
            vec![Stmt::Let {
                name: "nome".to_string(),
                value: Expr::Str("oi".to_string()),
            }]
        );
    }

    #[test]
    fn parseia_programa_com_atribuicao_e_echo() {
        // o exemplo original da Aula 01
        let tokens = Lexer::new("x = 1 + 2; echo x;").tokenize();
        let programa = Parser::new(tokens).parse_program();

        assert_eq!(
            programa,
            vec![
                Stmt::Assign {
                    name: "x".to_string(),
                    value: Expr::Binary {
                        left: Box::new(Expr::Int(1)),
                        op: BinOp::Add,
                        right: Box::new(Expr::Int(2)),
                    },
                },
                Stmt::Echo(Expr::Ident("x".to_string())),
            ]
        );
    }
}
