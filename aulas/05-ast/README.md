# Aula 05 — AST (statements)

## Objetivo

Expandir a AST pra além de expressões: adicionar `Stmt` (statement),
cobrindo atribuição (`x = 1 + 2;`) e `echo x;`, e fazer o parser produzir
um programa inteiro (`Vec<Stmt>`) a partir do exemplo original da Aula 01.

## Conceitos

- **`Expr` produz valor, `Stmt` executa uma ação** — retomando a distinção
  da Aula 01. `Stmt::Assign`/`Stmt::Echo` carregam um `Expr` dentro (a
  parte que precisa ser calculada), mas o statement em si não "vale" nada,
  só faz algo acontecer.
- **`expect`** — um helper que consome o token atual só se ele bater com o
  esperado; senão, `panic!`. Usado pra exigir `=` e `;` nos lugares certos
  da gramática (ex: depois do nome da variável, tem que vir um `=`).
- **`peek` antes de decidir, `advance` depois de decidir** — `parse_statement`
  olha o token atual com `peek` (sem consumir) só pra escolher qual
  caminho seguir (`Echo` ou `Assign`); o consumo de verdade (`advance`)
  só acontece depois, dentro de cada braço do `match`.
- **Por que `parse_program` não podia copiar o molde do `tokenize()`**: no
  lexer, `Token::Eof` é um token **válido**, que entra na lista de
  resultado — por isso dava pra "fazer o trabalho, depois checar se era
  Eof, decidir se para". Aqui não existe um `Stmt` equivalente ao Eof —
  se `parse_statement` encontrasse um `Token::Eof`, cairia no
  `panic!` de "statement inesperado". Por isso a checagem de fim
  (`self.peek() != Some(Token::Eof)`) precisa vir **antes** de cada
  chamada a `parse_statement`, não depois.

## Implementação

`ast.rs` ganhou:
```rust
#[derive(Debug, PartialEq)]
pub enum Stmt {
    Assign { name: String, value: Expr },
    Echo(Expr),
}
```

`parser.rs` ganhou três métodos:

```rust
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
```

Teste cobrindo o pipeline completo, ponta a ponta, com o exemplo original
da Aula 01:

```rust
let tokens = Lexer::new("x = 1 + 2; echo x;").tokenize();
let programa = Parser::new(tokens).parse_program();
// programa == vec![
//     Stmt::Assign { name: "x", value: Binary(Int(1), Add, Int(2)) },
//     Stmt::Echo(Expr::Ident("x")),
// ]
```

10 testes passando no workspace inteiro (`cargo test`).

## Resolução (percurso do aluno)

`Stmt`, `expect` e `parse_statement` saíram certos de primeira, seguindo
os exemplos comentados linha a linha.

`parse_program` foi copiado do molde do `tokenize()` do lexer sem adaptar
a diferença estrutural: tentou usar `self.next_token()` (método que só
existe no `Lexer`, não no `Parser`), comparar um `Stmt` com `Token::Eof`
(tipos incompatíveis) e dar `.push()` num `Stmt` solto (só `Vec` tem
`.push()`, e a variável nem tinha virado `Vec` — ficou com o tipo do
primeiro `Stmt` retornado). O compilador pegou os três problemas
(`E0599` duas vezes, `E0308` uma vez). Corrigido invertendo a ordem do
laço: checar `peek() != Some(Token::Eof)` **antes** de chamar
`parse_statement`, em vez de tentar decidir depois — porque, diferente do
lexer, não existe um "`Stmt` versão do Eof" pra detectar depois do fato.

## Conclusão

Pipeline completo, ponta a ponta: texto → tokens → `Vec<Stmt>`, cobrindo
atribuição e `echo`, com precedência de operadores respeitada dentro de
cada expressão.

## Próximos passos

Aula 06 — Interpretador: percorrer o `Vec<Stmt>` e executar de verdade —
calcular o valor de cada `Expr`, guardar valores num `environment`
(`HashMap<String, valor>`, a estrutura que você já tinha deduzido lá na
Aula 01) e imprimir de verdade no `echo`.
