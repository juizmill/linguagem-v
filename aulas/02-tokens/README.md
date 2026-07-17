# Aula 02 — Tokens

## Objetivo

Modelar o tipo `Token` em Rust para o Aurora (identificadores sem sigilo,
decidido na Aula 01: `x`, não `$x`).

## Conceitos

Rust tem enums algébricos: cada variante pode carregar um dado diferente
(diferente de um enum comum tipo os do PHP 8.1, onde as variantes normalmente
compartilham um único valor escalar de fundo). Isso é o formato ideal pra um
token: `Number` carrega o valor numérico, `Plus` não carrega nada.

Categorias de token levantadas: literais (número), identificador (nome de
variável), operadores (`+ - * / =`), pontuação (`;`), palavra-chave (`echo`)
e um marcador de fim de arquivo (`Eof`) — pro parser (aula 04) saber quando
os tokens acabaram sem checar índice fora do array toda hora.

Posição (linha/coluna) do token fica pra Aula 03, junto do algoritmo de
scanning.

## Implementação

- Criado o crate `crates/aurora-lexer` (lib), adicionado automaticamente ao
  workspace via `members = ["crates/*"]`.
- Enum `Token` definido em `crates/aurora-lexer/src/lib.rs`, com
  `#[derive(Debug, PartialEq)]` pra permitir comparação (`==`) e formatação
  de debug — necessário pros testes.
- Teste unitário validando igualdade/desigualdade entre tokens que carregam
  dado (`Token::Number`).

```
cargo test -p aurora-lexer
```

## Desafio

Escrever o enum `Token` cobrindo as categorias acima, mais um teste
`#[test]` comparando tokens com `assert_eq!`/`assert_ne!`.

## Resolução (percurso do aluno)

O enum passou por três versões até fechar, cada uma corrigida em conversa:

**v1** — só uma variante (`Plus`), sem cobrir as outras categorias. Corrigido
completando as variantes que faltavam.

**v2** — variante `String(String)` pra guardar nome de identificador (nome
confuso — dava a entender "literal de string", conteúdo que só entra na
Aula 09). Também apareceu `Delemiter` (typo) representando só `;`, sem
espaço pra outros delimitadores futuros (`(`, `)`, `,`). Nessa versão o
teste também tentava `assert_eq!(Token::Number(3.0), 3.0)` — comparação
entre um `Token` e um `f64` cru, que não compila (tipos diferentes); o
próprio compilador sugeriu a correção (`E0308: mismatched types`, com
`help` embutido).

**v3 (intermediária)** — na tentativa de resolver o problema do
identificador/delimitador, o aluno generalizou pra `Operator(char)` e
`Delimiter(char)`. Funciona, mas troca segurança de compilação por menos
variantes: com variantes explícitas (`Plus`, `Semicolon`...) o Rust
**obriga** tratar cada caso num `match` (exhaustiveness checking) — esquecer
um caso vira erro de compilação. Com `Operator(char)`, `Operator('%')`
compila mesmo se `%` não for um operador válido; o erro só apareceria em
runtime, num `match` com caso "catch-all". Essa é uma escolha real de
arquitetura de linguagens (o próprio `rustc` usa variantes explícitas).

**v4** — voltou pra variantes explícitas em tudo, inclusive `Semicolon`
(nome em inglês pra "ponto e vírgula", só convenção — a comunidade Rust
nomeia em inglês). Não adicionou `Comma`: Aurora ainda não tem funções
(Aula 13), então não precisa ainda — decisão consciente de não antecipar o
que a linguagem não usa.

**v5 (final)** — pergunta do aluno sobre `Number(f64)`: um `f64` representa
`3` e `3.14` sem perda de valor, mas depois de convertido pra `f64` perde-se
a informação de que o literal era um inteiro no código-fonte (`3` e `3.0`
viram idênticos). Isso importa para o sistema de tipos futuro (Aula 16):
JS trata todo número como um único tipo por baixo (`typeof 3 === typeof
3.5`); PHP/Rust/Python distinguem `int` de `float` como tipos diferentes,
com regras de coerção. Decisão: Aurora distingue os dois — `Number(f64)`
virou `Int(i64)` e `Float(f64)`. A regra de qual criar fica pra Aula 03: se
o literal tem `.`, é `Float`; senão, `Int`.

```rust
#[derive(Debug, PartialEq)]
enum Token {
    Float(f64),      // Literais para salvar valor ponto flutuante
    Int(i64),        // Literal para salvar valores inteiros
    Ident(String),   // Identificadores: nomes de variável
    Plus,            // operador +
    Subtract,        // operador -
    Divide,          // operador /
    Multiply,        // operador *
    EqualsTo,        // operador =
    Semicolon,       // delimitador do final da instrução, por exemplo ;
    Echo,            // palavra-chave
    Eof,             // fim do token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_sao_comparaveis() {
        assert_eq!(Token::Float(3.0), Token::Float(3.0));
        assert_ne!(Token::Float(3.0), Token::Float(2.0));

        assert_eq!(Token::Int(4), Token::Int(4));
        assert_ne!(Token::Int(3), Token::Int(2));
    }
}
```

## Conclusão

`Token` modelado com variantes explícitas, cobrindo literais (`Int`/`Float`
separados), identificador, operadores, pontuação, palavra-chave e EOF.
Decisão consciente de manter exhaustiveness checking do Rust como rede de
segurança pro parser/interpretador futuros, em vez de generalizar com dado
solto (`char`), e de distinguir inteiros de floats desde o token — decisão
que vai ecoar no sistema de tipos da Aula 16.

## Próximos passos

Aula 03 — Lexer: escrever o algoritmo que varre uma `&str` caractere por
caractere e produz um `Vec<Token>` de verdade, incluindo rastreamento de
posição (linha/coluna) pra mensagens de erro.
