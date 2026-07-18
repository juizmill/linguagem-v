// Aula 02 — Tokens.
// TODO: definir o enum `Token` aqui.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Float(f64),    // Literais para salvar valor ponto flutuante
    Int(i64),      // Literal para salvar valores inteiros
    Ident(String), // Identificadores: nomes de variável
    Str(String),   // Literais de string
    Plus,          // operador +
    Subtract,      // operador -
    Divide,        // operador /
    Multiply,      // operador *
    EqualsTo,      // operador =
    Let,           // Define uma variavel let x=1;
    Semicolon,     // delimitador do final da instrução por exemplo ;
    Echo,          // palavra-chave
    Eof,           // fim do token
}

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
        }
    }

    fn is_at_end(&self) -> bool {
        // self.pos é o campo que existe na struct (declarado lá em cima).
        // Comparamos a posição atual do cursor com o tamanho do vetor.
        self.pos >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        // .get(indice) nunca quebra o programa: se o índice não existir,
        // devolve None em vez de dar panic (diferente de source[pos]).
        // .copied() tira o char de dentro da referência (char é barato de copiar).
        self.source.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        // &mut self (com "mut"): esse método MODIFICA self.pos, por isso
        // precisa de acesso mutável — diferente de peek/is_at_end, que só leem.

        let atual = self.peek(); // 1. olha o char na posição ATUAL (reaproveita o peek)

        if atual.is_some() {
            self.pos += 1; // 2. só move o cursor se realmente havia um char ali
        }

        atual // 3. devolve o char que estava na posição antes de mover
    }

    fn scan_number(&mut self, first: char) -> Token {
        let mut is_float = false;
        let mut text = String::new();
        text.push(first);

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                text.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                is_float = true;
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            Token::Float(text.parse().unwrap())
        } else {
            Token::Int(text.parse().unwrap())
        }
    }

    fn scan_string(&mut self) -> Token {
        let mut text = String::new();

        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance();
                return Token::Str(text);
            }

            text.push(c);
            self.advance();
        }

        panic!("string não fechada: \"{text}");
    }

    fn scan_identifier(&mut self, first: char) -> Token {
        let mut text = String::new();
        text.push(first);

        while let Some(c) = self.peek() {
            if c.is_alphabetic() || c == '_' || c.is_alphanumeric() {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if text == "echo" {
            Token::Echo
        } else if text == "let" {
            Token::Let
        } else {
            Token::Ident(text)
        }
    }

    fn next_token(&mut self) -> Token {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }

        if self.is_at_end() {
            return Token::Eof;
        }

        let c = self.advance().unwrap();

        let token = match c {
            '+' => Token::Plus,
            '-' => Token::Subtract,
            '/' => Token::Divide,
            '*' => Token::Multiply,
            '=' => Token::EqualsTo,
            ';' => Token::Semicolon,
            '"' => self.scan_string(),
            c if c.is_ascii_digit() => self.scan_number(c),
            c if c.is_alphabetic() && c.is_alphanumeric() => self.scan_identifier(c),
            _ => panic!("caractere inesperado: {c}"),
        };

        token
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let eof = token == Token::Eof;
            tokens.push(token);
            if eof {
                break;
            }
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_sao_comparaveis() {
        // seu teste aqui, ex: assert_eq!(Token::Plus, Token::Plus);
        assert_eq!(Token::Float(3.0), Token::Float(3.0));
        assert_ne!(Token::Float(3.0), Token::Float(2.0));

        assert_eq!(Token::Int(4), Token::Int(4));
        assert_ne!(Token::Int(3), Token::Int(2));
    }

    #[test]
    fn peek_e_is_at_end_funcionam() {
        let lexer = Lexer::new("+;");

        // pos começa em 0 -> aponta pro '+' (índice 0)
        assert_eq!(lexer.peek(), Some('+'));
        assert_eq!(lexer.is_at_end(), false);
    }

    #[test]
    fn advance_consome_e_anda_o_cursor() {
        // "mut": aqui lexer precisa ser mutável, porque advance() vai mudar pos.
        let mut lexer = Lexer::new("+;");

        assert_eq!(lexer.advance(), Some('+')); // consome o '+' (índice 0), pos vira 1
        assert_eq!(lexer.peek(), Some(';')); // agora o atual é o ';' (índice 1)

        assert_eq!(lexer.advance(), Some(';')); // consome o ';' (índice 1), pos vira 2
        assert_eq!(lexer.is_at_end(), true); // pos (2) == len (2), acabou

        assert_eq!(lexer.advance(), None); // não há mais nada pra consumir
    }

    #[test]
    fn next_token_reconhece_operadores_e_para_no_eof() {
        let mut lexer = Lexer::new("+ - ;");

        // mesma sequência do dry-run que fizemos no papel:
        // pula espaços, reconhece cada operador, e termina em Eof.
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Subtract);
        assert_eq!(lexer.next_token(), Token::Semicolon);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn next_token_reconhece_int_e_float() {
        let mut lexer = Lexer::new("123 12.5");

        assert_eq!(lexer.next_token(), Token::Int(123));
        assert_eq!(lexer.next_token(), Token::Float(12.5));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn next_token_reconhece_string() {
        let mut lexer = Lexer::new("\"ola\"");

        assert_eq!(lexer.next_token(), Token::Str("ola".to_string()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn next_token_reconhece_string_vazia() {
        // "" -- string sem nenhum caractere entre as aspas. O laço do
        // scan_string precisa achar o "de fechamento" já na primeira
        // olhada (peek), sem nunca entrar no braço que dá push.
        let mut lexer = Lexer::new("\"\"");

        assert_eq!(lexer.next_token(), Token::Str("".to_string()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn next_token_reconhece_string_com_espaco_e_numero() {
        // scan_string não filtra por TIPO de caractere (diferente de
        // scan_number/scan_identifier) -- espaço, dígito, tudo deve entrar
        // no texto igual, até achar a aspa de fechamento.
        let mut lexer = Lexer::new("\"idade: 25\"");

        assert_eq!(lexer.next_token(), Token::Str("idade: 25".to_string()));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    #[should_panic(expected = "string não fechada")]
    fn next_token_string_sem_fechar_da_panic() {
        // "ola sem a aspa de fechamento -- scan_string deve chegar no fim
        // do arquivo (peek() vira None) e reclamar, em vez de devolver um
        // Token::Str silenciosamente incompleto.
        let mut lexer = Lexer::new("\"ola");
        lexer.next_token();
    }

    #[test]
    fn next_token_reconhece_identificador_com_numero() {
        let mut lexer = Lexer::new("x1 echo");

        // "x1" inteiro deveria virar UM token Ident("x1")
        assert_eq!(lexer.next_token(), Token::Ident("x1".to_string()));
        assert_eq!(lexer.next_token(), Token::Echo);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn tokenize_eof() {
        let mut lexer = Lexer::new("x = 1 + 2;");
        let tokens = lexer.tokenize();

        // vec![...] cria um Vec direto, tipo um array literal do PHP: [a, b, c]
        assert_eq!(
            tokens,
            vec![
                Token::Ident("x".to_string()),
                Token::EqualsTo,
                Token::Int(1),
                Token::Plus,
                Token::Int(2),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }
}
