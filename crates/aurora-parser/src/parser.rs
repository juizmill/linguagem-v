// Aula 04 — Parser.
// TODO: definir o struct Parser aqui.
use aurora_lexer::Token;

use crate::{BinOp, Expr, Stmt};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    // Diferente do Lexer::new: aqui não tem texto pra converter.
    // O Parser recebe os tokens já prontos (produzidos pelo Lexer::tokenize())
    // e só guarda essa lista — por isso "tokens" (sem ponto, é o próprio parâmetro).
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
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
        // "primary" é o nível mais baixo da gramática: um número ou um nome
        // sozinho, não uma combinação de vários — por isso não tem laço aqui,
        // só consome UM token.
        let token = self.advance();

        // advance() devolve Option<Token> (pode ser None se não sobrasse nada).
        // .unwrap() "desembrulha" o Option assumindo que existe valor — seguro
        // aqui porque, pra essa função ter sido chamada, ainda tem pelo menos
        // o token Eof esperando pra ser consumido.
        let token = token.unwrap();

        // match compara "token" com cada padrão até achar um que bate.
        // Token::Int(n): o valor de dentro da variante Int é extraído e vira
        // a variável "n" -- ela só existe dentro desse braço do match.
        match token {
            Token::Int(n) => Expr::Int(n),
            Token::Float(n) => Expr::Float(n),
            Token::Ident(nome) => Expr::Ident(nome),
            outro => panic!("token inesperado em parse_primary: {outro:?}"),
        }
    }

    fn parse_term(&mut self) -> Expr {
        // primeiro operando (maior precedência: * e /, então chama parse_primary)
        let mut expr = self.parse_primary();

        loop {
            // olha o token atual: é * ou /? Se não for nenhum dos dois, para o laço.
            let op = match self.peek() {
                Some(Token::Multiply) => BinOp::Multiply,
                Some(Token::Divide) => BinOp::Divide,
                _ => break,
            };

            self.advance(); // consome o operador (* ou /)
            let right = self.parse_primary(); // o operando da direita

            // "engloba" o que já tinha, virando o filho esquerdo do novo nó
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
        // olha o token atual SEM consumir ainda, pra decidir qual caminho seguir
        match self.peek() {
            Some(Token::Echo) => {
                self.advance(); // agora sim consome o "echo"
                let value = self.parse_expression(); // a expressão depois do echo
                self.expect(Token::Semicolon); // exige o ";" no final
                Stmt::Echo(value)
            }

            // Token::Ident(nome): desempacota o nome de dentro do token, igual
            // fizemos no parse_primary — "nome" só existe dentro desse braço.
            Some(Token::Ident(nome)) => {
                self.advance(); // consome o identificador (ex: "x")
                self.expect(Token::EqualsTo); // exige o "="
                let value = self.parse_expression(); // a expressão do lado direito
                self.expect(Token::Semicolon); // exige o ";"
                Stmt::Assign { name: nome, value }
            }

            outro => panic!("statement inesperado: {outro:?}"),
        }
    }

    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();

        // confere ANTES de cada statement se ainda não chegamos no fim —
        // parse_statement não sabe lidar com Token::Eof (daria panic).
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
