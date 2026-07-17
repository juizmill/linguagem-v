# Aula 07 — Variáveis

## Objetivo

Aprofundar o `environment` da Aula 06, resolvendo os dois pontos deixados em
aberto: distinguir **declaração** de **reatribuição**, e trocar o `panic!`
cru de "variável não definida" por um **erro recuperável**, que o próprio
Aurora consegue capturar e explicar — sem derrubar o processo.

## Conceitos

- **Variante struct vs. variante tupla num `enum`.** `Echo(Expr)` guarda um
  valor anônimo entre parênteses (tupla); `Let { name: String, value: Expr }`
  guarda campos nomeados entre chaves (struct). Construção também muda:
  `Stmt::Echo(valor)` vs. `Stmt::Let { name: nome, value }`. *Field init
  shorthand*: quando a variável local tem o mesmo nome do campo (`value`),
  não precisa repetir (`value: value` vira só `value`).
- **`match` é exaustivo por obrigação do compilador.** Ao adicionar
  `Stmt::Let` no enum, `eval_stmt` parou de compilar até cobrir essa
  variante — diferente de um `switch` do PHP/JS, que aceita `case` faltando
  sem reclamar.
- **`Result<T, E>`** — enum genérico (`Ok(T)` / `Err(E)`) pra erro
  recuperável. Sem exceptions em Rust: erro é só um valor de retorno, e quem
  chama é obrigado pelo compilador a lidar com os dois casos.
- **Operador `?`** — açúcar sintático pra "se `Ok(x)`, desembrulha em `x`;
  se `Err(e)`, para esta função agora e devolve esse mesmo `Err(e)`". Só
  funciona dentro de uma função que também devolve `Result` — por isso
  `eval_expr`, `eval_stmt` e `run_program` viraram `Result` em cadeia.
- **`&str` vs. `String`**, e por que `Err("texto")` não compila quando a
  assinatura promete `String`: literal entre aspas é `&str` (emprestado);
  `String` é dono do próprio texto. `format!(...)` (mesma interpolação do
  `println!`/`panic!`, mas devolve `String` em vez de imprimir) resolve os
  dois problemas de uma vez — inclusive porque `Err(...)` é só um construtor
  de enum, não uma macro, então não interpola `{variavel}` sozinho.
- **Todo bloco vale sua última expressão, sem `;`.** Um `;` no final
  transforma a expressão numa *statement* (valor descartado); se sobra sem
  expressão final, o bloco vale `()`. Foi a causa real do primeiro erro em
  `eval_stmt` (`Ok(env.insert(...));` com `;` no fim não contava como
  retorno).
- **`HashMap::contains_key`** só consulta, não muta — por isso funciona
  mesmo com `env: &mut HashMap<...>` (o `mut` dá permissão de escrever, não
  obriga a escrever toda chamada).
- **`return` explícito vs. expressão final implícita.** Dentro de um `if`
  que não é a última linha do bloco, `return Err(...)` é necessário pra
  interromper a função ali mesmo; no caminho "normal" (fim do bloco), a
  expressão final sem `;` já é o retorno, sem precisar de `return`.
- **Recursão em `eval_expr`.** `Expr::Binary` guarda `left`/`right` como
  `Box<Expr>`, que pode ser outro `Binary` — por isso `eval_expr` chama a si
  mesma para resolver cada lado da árvore, com `Int`/`Float`/`Ident` como
  casos-base. Com `?`, um erro em qualquer profundidade da árvore sobe
  sozinho até o topo.
- **Testes que devolvem `Result<(), E>`.** Uma `#[test]` pode ter essa
  assinatura; o test runner do Rust já entende `Err` como falha, sem
  precisar de `.unwrap()` manual — permite usar `?` dentro do próprio teste.

## Implementação

`aurora-lexer`: novo `Token::Let`, reconhecido em `scan_identifier` do mesmo
jeito que `"echo"` virou `Token::Echo`.

`aurora-parser` (`ast.rs`): `Stmt::Let { name: String, value: Expr }`,
separado de `Stmt::Assign` (que passou a significar só reatribuição).
`parse_statement` ganhou um braço pro `Token::Let`, que consome a palavra-
chave, extrai o identificador seguinte via `match self.advance().unwrap() {
Token::Ident(nome) => nome, ... }`, exige `=` e `;`, e monta
`Stmt::Let { name: nome, value }`.

`aurora-interpreter` (`lib.rs`): `eval_expr`, `eval_stmt` e `run_program`
agora devolvem `Result<Value, String>` / `Result<(), String>`. Regras
implementadas:

```rust
Stmt::Let { name, value } => {
    // declaração: sempre grava, não checa se já existia
    let valor = eval_expr(value, env)?;
    env.insert(name.clone(), valor);
    Ok(())
}

Stmt::Assign { name, value } => {
    // reatribuição: só é válida se "name" já foi declarado com "let"
    if !env.contains_key(name) {
        return Err(format!(
            "variável '{name}' não declarada -- use 'let' antes de atribuir"
        ));
    }
    let valor = eval_expr(value, env)?;
    env.insert(name.clone(), valor);
    Ok(())
}
```

`Expr::Ident` não encontrado no `env` também virou `Err(format!(...))`, em
vez de `panic!`. `aurora-cli/src/main.rs` trata o `Result` de `run_program`
com `match` (mesmo padrão já usado ali pra `fs::read_to_string`), imprimindo
`Erro: {mensagem}` em `stderr` sem derrubar o processo.

Testado ponta a ponta com `aurora-cli`:

```
let x = 1 + 2; echo x; x = 10; echo x;
```
→ imprime `3`, `10`.

```
y = 5; echo y;
```
(sem `let`) → `Erro: variável 'y' não declarada -- use 'let' antes de atribuir`,
sem panic, processo termina normalmente.

18 testes passando no workspace (4 no `aurora-interpreter`, incluindo os
atualizados desta aula).

## Desafio

Duas decisões de design, tomadas pelo aluno antes de qualquer código:

1. Aurora exige declaração explícita com `let` (diferente do PHP, onde o
   primeiro `=` já cria a variável).
2. Diferenciar **erro recuperável** (programador Aurora esqueceu o `let`,
   ou usou uma variável inexistente — deve virar `Result`/mensagem legível)
   de **bug do interpretador** (uma garantia interna quebrada — continua
   sendo `panic!`/`.unwrap()`). O aluno chegou sozinho a essa distinção
   antes de ela ser nomeada.

Implementação em duas etapas: lexer/AST/parser pro `let` primeiro, depois
`Result` + `?` encanados por `eval_expr` → `eval_stmt` → `run_program` →
`main`.

## Resolução (percurso do aluno)

**Etapa A (sintaxe do `let`).** Lexer e AST saíram corretos de primeira. No
parser, dois erros reais: `Stmt::Let(value)` tentando construir uma
variante struct com sintaxe de tupla (erro do compilador, E0533, usado como
gancho pra explicar a diferença entre as duas formas de variante); e a
lógica ainda não lia o identificador entre `let` e `=` — a primeira versão
chamava `parse_expression()` direto depois do `let`, sem consumir o nome.
O aluno travou aqui ("fiquei confuso, não estou entendendo como devemos
montar isso") — trecho escrito comentado linha a linha, seguindo o padrão
já reconhecido no projeto (ver decisão registrada em memória sobre ceder
quando o acúmulo de conceitos novos vira bloqueio real).

**Etapa B (`Result` + `?`).** No `eval_expr`, o aluno acertou a estrutura
geral sozinho, com dois erros de tipo pontuais: `Err("...{name}")` sem
`format!` (string literal não interpola, e `&str` ≠ `String`); e esquecer o
`?` nas duas chamadas recursivas do braço `Binary`, faltando também o `Ok(...)`
em volta da chamada final a `eval_binary`. Os dois foram corrigidos pelo
aluno depois de ver a mensagem do compilador e um exemplo isolado
(`analisa`/`dobro`, depois `ler_positivo`/`soma_dois`).

No `eval_stmt` o aluno relatou bloqueio de novo ("o conceito ainda não está
claro na minha mente, não estou conseguindo imaginar o passo a passo") —
sinal reconhecido, função inteira escrita comentada (a checagem
`contains_key` antes do `Assign`, o `return Err(...)` explícito dentro do
`if`, o `?` nas duas chamadas de `eval_expr`, e o motivo de cada `Ok(...)`
não poder terminar em `;`).

Depois disso, o aluno resolveu sozinho e sem ajuda adicional: `main.rs`
(usando o padrão de `match` que já existia ali pro `fs::read_to_string`);
três dos quatro testes (usando uma alternativa válida que eu nem tinha
sugerido — fazer a própria função de teste devolver `Result<(), String>` e
usar `?` dentro dela, em vez de embrulhar cada valor esperado em `Ok(...)`);
e o quarto teste (`run_program_executa_atribuicao_e_echo`), percebendo
sozinho que o `Stmt::Assign` inicial precisava virar `Stmt::Let`, já que a
variável ainda não existia no `env` daquele teste.

**Observação registrada, não implementada:** o aluno levantou, como
exemplo, se declarar uma variável `Int` e depois reatribuir como `String`
deveria ser erro de tipo. Deixado de propósito para a Aula 16 (Sistema de
tipos) — hoje Aurora continua dinamicamente tipada, tipo pode mudar numa
reatribuição sem erro.

## Conclusão

Aurora agora distingue declaração (`let`) de reatribuição (`=`), e trata
variável não declarada como erro recuperável (`Result`), reportado de forma
legível via `aurora-cli`, sem `panic!`. `?` está encanado por toda a cadeia
de execução (`eval_expr` → `eval_stmt` → `run_program` → `main`).

## Próximos passos

Aula 08 — Escopos: múltiplos `environment`s aninhados vão importar de
verdade (blocos, funções) — hoje só existe um único `HashMap` global.
