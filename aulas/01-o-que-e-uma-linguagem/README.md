# Aula 01 — O que é uma linguagem

## Objetivo

Entender o pipeline que transforma texto (código-fonte) em comportamento, e
decidir a arquitetura de crates do Aurora antes de escrever qualquer lógica.

## Conceitos

Toda linguagem que você já usa (PHP, JS) passa por isto antes de rodar:

```
código-fonte (texto)
      │
      ▼
   LEXER          → quebra o texto em tokens (palavras com significado)
      │
      ▼
   PARSER         → organiza os tokens em uma árvore (AST)
      │
      ▼
  (análise semântica) → opcional, verifica tipos/escopos
      │
      ▼
INTERPRETADOR ou COMPILADOR
      │                        │
      ▼                        ▼
  executa a AST          gera bytecode/código de máquina
  direto                        │
                                 ▼
                              RUNTIME executa
```

O Aurora vai construir os dois caminhos: um interpretador de árvore primeiro
(aula 06), e depois bytecode + VM + compilador (aulas 18–22).

Cada estágio tem UMA responsabilidade e não sabe nada sobre os outros — o
lexer não sabe o que é uma expressão, o parser não sabe como executar nada.
É por isso que vamos usar múltiplos crates no Cargo workspace: cada crate =
um estágio do pipeline. Crates são criados sob demanda, na aula que precisar
deles (aula 02/03 vai criar `aurora-lexer`, por exemplo).

## Implementação

Criada a base do repositório:

- `Cargo.toml` na raiz — workspace, agrega tudo em `crates/*`.
- `crates/aurora-cli` — o binário `aurora`, ponto de entrada. Por enquanto só
  imprime uma mensagem; vai virar o "front door" da linguagem (rodar
  arquivos `.aurora`, REPL, etc.) conforme as aulas avançam.

```
cargo run -q
# Aurora v0.1 — a linguagem que você vai construir.
```

## Desafio

Antes da próxima aula (Tokens), sem escrever Rust ainda — só no papel/comentário:

Para este trecho PHP:

```php
$x = 1 + 2;
echo $x;
```

1. Liste, em ordem, os **tokens** que um lexer produziria (pense em cada
   "palavra" com significado: números, operadores, identificadores,
   pontuação — não pense em espaços).
2. Desenhe (em texto mesmo, com indentação) a **árvore** que representaria
   a expressão `1 + 2`. Qual nó é a raiz? O que são os filhos?
3. Esse mesmo `$x` reaparece em `echo $x`. Como o interpretador "sabe" que é
   a mesma variável? O que ele precisa guardar em algum lugar entre a
   primeira linha e a segunda?

Não existe resposta perfeita ainda — é para chegar na aula 02 já com uma
intuição do que um token é, antes de eu confirmar/ajustar.

### Resolução (o que o aluno respondeu)

Trabalhado em conversa, um pedaço de cada vez, sobre a versão simplificada
`1 + 2;` (a parte de `$x =` e `echo` ficou pra depois):

**1. Tokens**

O aluno identificou corretamente que `1`, `+`, `2` e `;` são tokens
separados, e que espaços são ignorados. Ajuste de vocabulário feito na hora:

- `+` não é "uma expressão", é um **operador**. Uma expressão é a combinação
  toda (`1 + 2` inteiro produz um valor — isso é a expressão).
- `;` não marca fim de "expressão", marca fim de **statement** (instrução
  completa). Expressão produz valor; statement executa uma ação.

Tokens finais: `NUMBER(1)`, `PLUS`, `NUMBER(2)`, `SEMICOLON`.

**2. Árvore (AST)**

Usando a analogia de chamada de função aninhada (`soma(1, 2)`: `soma` é "de
fora", `1` e `2` são argumentos/filhos), o aluno concluiu sozinho que `+`
precisa dos outros dois pra fazer sentido, logo é a **raiz**, e `1`/`2` são
**folhas**:

```
      +
     / \
    1   2
```

**3. Environment (armazenamento de variáveis)**

Pergunta: que estrutura guarda "nome da variável → valor" entre uma linha e
outra? Resposta do aluno: um array associativo (chave/valor) — correto. Em
teoria de linguagens isso se chama **environment** (ou *symbol table*); no
Aurora vai virar um `HashMap<String, Value>` de verdade lá pelas aulas
07–08 (Variáveis/Escopos).

## Conclusão

Todo código passa por um pipeline de tradução em estágios isolados. O Aurora
já tem uma casca executável (`aurora-cli`) que vai crescer, estágio por
estágio, conforme os crates forem entrando.

## Próximos passos

Aula 02 — Tokens: definir o que é um token no Aurora e criar o crate
`aurora-lexer` com o primeiro tipo `Token`.
