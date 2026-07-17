# Aula 03 — Lexer

## Objetivo

Escrever o algoritmo que varre o código-fonte caractere por caractere e
produz uma lista de tokens (`Vec<Token>`) de verdade.

## Conceitos

- `&str` em Rust é UTF-8; não dá pra indexar por posição (`source[0]` não
  compila). Solução adotada: converter pra `Vec<char>` uma vez, no início
  (`source.chars().collect()`), e indexar essa Vec normalmente. Só funciona
  bem porque Aurora vai lidar só com ASCII por enquanto.
- O lexer é um **cursor**: uma posição (`pos: usize`) que anda por
  `Vec<char>`. Duas operações centrais: `peek` (olha sem consumir) e
  `advance` (consome e anda).
- `Option<char>` é o "talvez não tenha valor" do Rust (`Some`/`None`) —
  equivalente ao `?string` (`string|null`) do PHP.
- `&self` (método só lê) vs `&mut self` (método modifica campos) — Rust
  **obriga** declarar `&mut self` pra qualquer método que altere `self`;
  PHP deixa qualquer método mudar `$this` livremente, sem aviso.
- `match` do Rust ≈ `match` do PHP 8 (`_` no lugar de `default`), e aceita
  **match guards** (`c if condição => ...`) pra combinar padrão com uma
  condição extra.
- `while let Some(c) = self.peek() { ... }` — laço que roda enquanto a
  expressão for `Some(algo)`, já "desembrulhando" o valor pra dentro do
  laço. `loop { ... break; }` é o laço infinito (equivalente a
  `while (true)` do PHP), usado na função que junta tudo.
- Em Rust, a última linha **sem `;`** de uma função é o valor devolvido
  (ou usa `return` explícito, reservado por convenção pra saídas
  antecipadas no meio da função).
- Números: `f64` não distingue `3` de `3.0` depois de convertido — por
  isso o token virou `Int(i64)`/`Float(f64)` separados (decidido na
  Aula 02), e o lexer decide qual criar olhando se apareceu um `.`.
- Identificadores: primeiro char letra/`_`, os seguintes podem ser
  letra, dígito ou `_` — mesma regra do PHP pra nomes de variável (sem o
  `$`). Depois de acumular o texto inteiro, compara com as palavras-chave
  conhecidas (`"echo"`) antes de decidir `Token::Echo` ou
  `Token::Ident(texto)`.
- **Posição (linha/coluna) foi propositalmente deixada de fora** desta
  aula — decisão consciente (ver Resolução) de só adicionar isso quando
  houver uma necessidade real de mensagem de erro apontando "linha X",
  em vez de adicionar por antecipação.

## Implementação (final)

Em `crates/aurora-lexer/src/lib.rs`:

```rust
struct Lexer {
    source: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Lexer { source: source.chars().collect(), pos: 0 }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let atual = self.peek();
        if atual.is_some() {
            self.pos += 1;
        }
        atual
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

    fn scan_identifier(&mut self, first: char) -> Token {
        let mut text = String::new();
        text.push(first);

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                text.push(c);
                self.advance();
            } else {
                break;
            }
        }

        if text == "echo" {
            Token::Echo
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

        match c {
            '+' => Token::Plus,
            '-' => Token::Subtract,
            '/' => Token::Divide,
            '*' => Token::Multiply,
            '=' => Token::EqualsTo,
            ';' => Token::Semicolon,
            c if c.is_ascii_digit() => self.scan_number(c),
            c if c.is_alphabetic() && c.is_alphanumeric() => self.scan_identifier(c),
            _ => panic!("caractere inesperado: {c}"),
        }
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let eof = token == Token::Eof;
            tokens.push(token); // inclui o próprio Eof no vetor final, de propósito
            if eof {
                break;
            }
        }
        tokens
    }
}
```

7 testes cobrindo `peek`, `is_at_end`, `advance`, `next_token` (operadores,
espaço, Eof, `Int`/`Float`, identificador com dígito) e `tokenize` completo,
todos passando com `cargo test -p aurora-lexer`.

## Resolução (percurso do aluno)

O aluno deixou claro cedo que estava aprendendo Rust do zero ao mesmo
tempo que compiladores — pacing ajustado pra passos bem menores, com
exemplo de código antes de cada tarefa, diagramas ASCII mostrando o cursor
andando pelo `Vec<char>` (pediu explicitamente reforço visual), e em
alguns pontos precisou que eu revisasse/corrigisse o código dele
diretamente (em vez de só apontar o problema), quando travou de verdade.

Também pediu, num certo ponto, a mesma implementação em **PHP** só pra
comparar — útil especialmente pra entender por que Rust exige `&mut self`
(PHP deixa qualquer método mutar `$this` sem avisar nada) e como
`Option<char>` se relaciona com `?string` do PHP.

Erros reais que apareceram e foram corrigidos, um de cada vez:

1. `is_at_end` usando `self.indice` (nome de um exemplo diferente) em vez
   de `self.pos` — `E0609: no field 'indice'`, com o compilador listando
   os campos que realmente existem.
2. `advance` sem `&mut self` (não conseguia mover o cursor) e olhando
   `pos + 1` (o próximo char) em vez de `pos` (o atual).
3. `next_token`: usou uma variável `c` fora do escopo onde ela existia (o
   `c` do `while let` só vive dentro daquele laço) — esqueceu de consumir
   o char atual antes do `match` (`E0425: cannot find value`). Também
   esqueceu de devolver o resultado do `match` (`E0308: mismatched
   types`, com o compilador sugerindo a correção).
4. `scan_number`: usou um ternário (`? :`, não existe em Rust — vira
   `if/else` como expressão) e `is_contained_in(".")` (API instável,
   nightly-only, no lugar de simplesmente `c == '.'`). `is_float` faltava
   `mut`. E dois bugs de lógica: faltou `self.advance()` dentro do laço
   (sem isso o cursor nunca anda — **loop infinito**, o tipo de bug que o
   compilador não pega) e faltou `text.push(first)` (o primeiro dígito,
   já consumido antes de chamar a função, se perdia).
5. `scan_identifier`: a condição de continuação do laço não aceitava
   dígito (só letra ou `_`), então `x1` virava dois tokens
   (`Ident("x")` + `Int(1)`) em vez de um só. Confirmado com um teste que
   falhou (`left: Ident("x"), right: Ident("x1")`) antes de corrigir.
6. `tokenize`: a função em si (o `loop`/`push`/`break`) saiu certa de
   primeira. O bug ficou só no teste — comparou `lexer` (o objeto,
   `Vec<Token>` esperando implementar `PartialEq`/`Debug` que não tem)
   com `tokenize` (o resultado). Corrigido comparando o resultado com um
   `vec![...]` escrito à mão com a sequência esperada.

Decisão consciente de escopo: posição (linha/coluna) por token foi adiada
— o custo de adicionar agora (mexer em todos os métodos + reescrever os
testes existentes) não se paga ainda, sem um caso real de erro que precise
apontar "linha X". Fica pra quando o parser (Aula 04) ou uma mensagem de
erro específica pedir isso de verdade.

## Conclusão

Lexer completo: reconhece operadores de um caractere, `;`, números
(`Int`/`Float`), identificadores e a palavra-chave `echo`, ignora espaços,
e junta tudo em `Vec<Token>` terminado em `Eof` via `tokenize()`.

## Próximos passos

Aula 04 — Parser: consumir esse `Vec<Token>` e montar a AST (a árvore que
já apareceu no papel lá na Aula 01, com o operador na raiz e os operandos
nas folhas), começando pelas expressões aritméticas.
