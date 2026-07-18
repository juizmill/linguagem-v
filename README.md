# Aurora

Aurora é uma linguagem de programação construída do zero, aula por aula,
com o único objetivo de aprender Ciência da Computação (compiladores,
interpretadores, sistemas de tipos, runtime, VM...) e Rust na prática.

Não é uma linguagem pronta para produção. O percurso completo — objetivo,
papel da IA, currículo e regras do curso — está em [`MASTER.md`](MASTER.md).
O material de cada aula, incluindo o que foi de fato discutido e corrigido
durante o aprendizado, está em [`aulas/`](aulas/).

## Requisitos

- [Rust](https://www.rust-lang.org/tools/install) (via `rustup`), com `cargo`
  no `PATH`.

## Estrutura do projeto

Workspace Cargo com um crate por etapa do pipeline:

```
crates/
  aurora-lexer/         texto-fonte -> tokens
  aurora-parser/         tokens -> AST (Expr/Stmt)
  aurora-interpreter/    percorre a AST e executa
  aurora-cli/            binário "aurora": lê um arquivo e roda
examples/                programas Aurora de exemplo (.aur)
aulas/                   documentação de cada aula
```

## Como compilar

Compilar o workspace inteiro (todos os crates):

```sh
cargo build --workspace
```

Para uma build otimizada (release):

```sh
cargo build --workspace --release
```

## Como rodar os testes

```sh
cargo test --workspace
```

## Como testar com um arquivo `.aur`

A extensão dos arquivos-fonte de Aurora é **`.aur`** (curta, no mesmo
padrão de `.rs`, `.go`, `.py` — ver [nota sobre a extensão](#extensão-de-arquivo) abaixo).

Rodar um arquivo direto via `cargo` (compila se necessário):

```sh
cargo run -p aurora-cli -- examples/hello.aur
```

Ou, depois de um `cargo build`, chamando o binário já compilado:

```sh
./target/debug/aurora examples/hello.aur
```

Exemplo (`examples/hello.aur`):

```
let x = 1 + 2 * 3;
echo x;
let y = x - 1;
echo y;
```

Saída:

```
7
6
```

Variável usada sem `let` antes não derruba o processo — vira um erro
legível:

```
$ echo 'y = 5; echo y;' > /tmp/teste.aur
$ cargo run -p aurora-cli -- /tmp/teste.aur
Erro: variável 'y' não declarada -- use 'let' antes de atribuir
```

### Sintaxe suportada até agora

- Declaração: `let nome = expressao;`
- Reatribuição (exige declaração prévia): `nome = expressao;`
- Impressão: `echo expressao;`
- Números inteiros e ponto flutuante, com promoção automática (`1 + 2.5`
  vira `Float`)
- Strings, entre aspas duplas: `"texto"` (aspas simples não são suportadas).
  `+` concatena — se um dos lados não for string, o outro lado é convertido
  pra texto antes de juntar (`"idade: " + 25` vira `"idade: 25"`); `-`, `*`
  e `/` envolvendo string são erro recuperável, não panic
- Operadores aritméticos `+ - * /`, com precedência (`*`/`/` antes de
  `+`/`-`)
- Escopos aninhados (`Environment` em cadeia pai-filho) — infraestrutura
  interna, ainda sem sintaxe própria que crie um escopo filho (chega com
  blocos/funções)

O que ainda falta (condicionais, laços, funções, tipos compostos...) segue
o currículo em [`MASTER.md`](MASTER.md).

## Extensão de arquivo

Os fontes Aurora usam **`.aur`**.
