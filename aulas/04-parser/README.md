# Aula 04 — Parser

*(nesta aula, o `Expr`/AST das expressões já nasceu junto — ver nota em
"Próximos passos" sobre a Aula 05)*

## Objetivo

Transformar o `Vec<Token>` do lexer numa árvore (`Expr`), respeitando
precedência de operadores (`*`/`/` antes de `+`/`-`), com um parser de
descida recursiva (recursive descent).

## Conceitos

- **Precedência via camadas.** A gramática é organizada em níveis, um por
  precedência, de baixo pra cima. Cada nível vira uma função, e cada
  função chama a de precedência **imediatamente acima**:
  ```
  expression := term (('+' | '-') term)*      <- mais baixa
  term       := factor (('*' | '/') factor)*  <- mais alta
  factor     := NUMBER | IDENT
  ```
- **`Box<Expr>`** — um `Expr::Binary` contém dois `Expr` dentro dele
  (esquerda/direita), o que faria o tipo ter tamanho infinito (recursivo).
  `Box` guarda um ponteiro pro filho no heap, com tamanho fixo. Em PHP
  isso nunca aparece porque objetos já são sempre referência por baixo.
- **Laço, não só recursão, dentro de cada nível.** `1 + 2 + 3` (dois `+` do
  mesmo nível) vira uma árvore left-associative: `(1 + 2) + 3`. Por isso
  `parse_term`/`parse_expression` têm um `loop`: começam com o primeiro
  operando e, a cada operador do mesmo nível, "englobam" o que já tinha
  como filho esquerdo de um novo nó.
- **Módulos em arquivos separados** (`mod ast;` / `mod parser;` no
  `lib.rs`) em vez de tudo num arquivo só — cada `.rs` vira um namespace;
  precisa `pub` pra ficar visível fora do arquivo, e `use crate::modulo::Coisa`
  pra importar de um módulo irmão.
- **`self` não é mágico** — é o `$this` do PHP. Métodos do mesmo `Parser`
  (`parse_expression`, `parse_term`, `parse_primary`) compartilham
  `self.tokens`/`self.pos`; por isso nenhum precisa **receber** o estado
  como parâmetro, ele já mora em `self`.
- **Destructuring em `match`** — `Token::Int(n) => ...` extrai o valor de
  dentro da variante e nomeia ele `n`, só dentro daquele braço.
- **`.cloned()` vs `.copied()`** — `char` é `Copy` (peek do Lexer usa
  `.copied()`); `Token` carrega `String` em algumas variantes, não é
  `Copy`, por isso o Parser usa `.cloned()` (chama `.clone()` de verdade)
  e o `Token` precisou do `#[derive(Clone)]`.

## Implementação

Três arquivos em `crates/aurora-parser/src/`:

`ast.rs`:
```rust
#[derive(Debug, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Ident(String),
    Binary { left: Box<Expr>, op: BinOp, right: Box<Expr> },
}

#[derive(Debug, PartialEq)]
pub enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}
```

`parser.rs` — cursor (`is_at_end`/`peek`/`advance`, mesmo molde do `Lexer`)
mais as três funções da gramática (`parse_primary`, `parse_term`,
`parse_expression`) e `parse()` como porta de entrada pública.

`lib.rs` — só `mod ast; mod parser;` e `pub use` reexportando `Expr`,
`BinOp`, `Parser`.

Também precisou tornar `Token`/`Lexer` (e `new`/`tokenize`) `pub` no
`aurora-lexer`, que até então eram só visíveis dentro do próprio crate.

2 testes no `parser.rs` (um deles usa o `Lexer` de verdade, ponta a ponta:
texto → tokens → árvore) + 1 no `ast.rs`, todos passando.

## Resolução (percurso do aluno)

Perguntou explicitamente se valia separar em vários arquivos em vez de um
só — decisão de organização acertada, dado que o `Parser` ia crescer bem
mais que o `Lexer`. Foi a entrada pro sistema de módulos do Rust
(`mod`/`pub`/`use crate::`).

Bugs/tropeços, na ordem:

1. Escreveu `ast.rs`/`parser.rs` dentro da pasta `crates/aurora-lexer/src/`
   por engano (mesma estrutura de pastas nos dois crates) — o conteúdo em
   si estava certo, só precisou mover pro crate certo (`aurora-parser`).
   Como o `lib.rs` do `aurora-lexer` não declarava esses módulos, os
   arquivos ficaram invisíveis, sem gerar erro nenhum.
2. `Parser::new(source: &str)` — confundiu com o `Lexer::new`. O `Parser`
   não recebe texto, recebe os tokens **já prontos** (saída do
   `Lexer::tokenize()`); não tem conversão nenhuma pra fazer, só guardar.
3. Pediu ajuda explícita, sentindo a lógica de parâmetros/`self` "mágica"
   — resolvido com um trace passo a passo (tabela) de `parse("1 + 2")`
   mostrando exatamente qual função chama qual, o valor de `pos` em cada
   ponto, e de onde vem o `n` extraído em `Token::Int(n)`.
4. No `parse_expression`, copiou o molde do `parse_term` mas esqueceu de
   trocar as duas chamadas internas de `parse_primary()` para
   `parse_term()` (só trocou os operadores). Resultado: parava de
   consumir tokens cedo demais e silenciosamente ignorava o resto —
   nenhum erro de compilação, só uma árvore incompleta. Confirmado com um
   teste (`"1 + 2 * 3"` virando só `1 + 2`, perdendo o `* 3`) antes de
   corrigir.

## Conclusão

Parser de descida recursiva funcionando, com precedência correta entre
`+`/`-` e `*`/`/`, cobrindo números, floats e identificadores como átomos.

## Próximos passos

Aula 05 — AST: o `Expr`/`BinOp` já nasceram aqui (adiantado em relação à
ordem "oficial" do MASTER.md, que lista Parser antes de AST) — a Aula 05
deve revisitar e **expandir** a AST pra além de expressões: nós de
**statement** (atribuição `x = expr;`, `echo expr;`), que o parser ainda
não trata. Depois, Aula 06 — Interpretador: percorrer essa árvore e
calcular o valor de verdade.
