# Aula 09 — Strings

## Objetivo

Adicionar o primeiro tipo de dado novo desde `Int`/`Float`: strings entre
aspas duplas, atravessando as quatro etapas do pipeline (lexer → AST/parser
→ interpreter), e decidir o que os operadores aritméticos existentes (`+ -
* /`) devem fazer quando um dos operandos é uma string.

## Conceitos

- **Um tipo novo toca o pipeline inteiro.** Mesma sequência já reconhecida
  desde a Aula 06: `Token` (lexer) → `Expr`/`Value` (parser/interpreter). O
  compilador guia o processo via exaustividade de `match`: assim que
  `Value` ganhou a variante `Str`, `to_f64` parou de compilar até decidir o
  que fazer com ela — o mesmo mecanismo do `Stmt::Let` na Aula 07, agora
  em `Value` em vez de `Stmt`.
- **Consumir sem guardar (`scan_string`).** Diferente de `scan_number`/
  `scan_identifier` (onde o primeiro caractere É parte do valor), a aspa de
  abertura de uma string só serve de sinal — não deve entrar no `text`. A
  aspa de **fechamento** também não entra, mas precisa ser **consumida**
  (`advance()`) antes de devolver o token, senão sobra pendurada pra
  próxima chamada de `next_token` tropeçar nela. Por isso o `return`
  explícito dentro do laço (não dá pra só `break` como as outras `scan_*`).
- **`@` (binding) vs. desestruturação simples.** `Value::Str(s)` extrai só
  o `String` de dentro, perdendo a informação "isso era um `Value`
  inteiro". `l @ Value::Str(_)` confirma o formato (`Str`) E guarda o
  `Value` inteiro em `l` — necessário aqui porque `format_value` espera um
  `&Value` (funciona pra qualquer variante), não um `&String`.
- **Padrões alternativos (`|`) com bindings.** `(l @ Value::Str(_), r) | (l,
  r @ Value::Str(_))` é um braço só, com duas formas de bater (esquerda é
  `Str`, OU direita é `Str`). Regra do Rust: cada lado do `|` precisa
  declarar exatamente os mesmos nomes, com o mesmo tipo — por isso os dois
  lados usam `l`/`r`, cada um alternando qual dos dois recebe o `@
  Value::Str(_)`. Dentro do braço, `l` é sempre o operando da esquerda e
  `r` o da direita, na ordem original, não importa qual dos dois disparou o
  match.
- **`unreachable!()` vs. `Err` — reforçando a distinção da Aula 07.**
  `to_f64` mantém um braço pra `Value::Str`, mas usa `unreachable!()`, não
  `0.0` nem `Err`: nesse ponto do código, `eval_binary` já garantiu (no
  braço anterior) que nenhum dos dois lados é `Str` — se `to_f64` algum dia
  receber uma, é bug interno (garantia quebrada), não erro do programador
  Aurora. Erro do programador Aurora (`"a" - "b"`) continua sendo `Err`
  recuperável.

## Implementação

`aurora-lexer`: `Token::Str(String)`; `scan_string(&mut self) -> Token`
consome qualquer caractere até achar `"` (sem filtrar por tipo, diferente
das outras `scan_*`), devolve `Token::Str(texto)`; string não fechada até o
fim do arquivo dá `panic!("string não fechada: ...")`.

`aurora-parser`: `Expr::Str(String)` na AST; braço `Token::Str(s) =>
Expr::Str(s)` em `parse_primary`, mesmo padrão de `Token::Int`/`Token::Ident`
— nenhum código extra precisou ser escrito em `parse_statement`, porque
`let nome = "texto";` já desce naturalmente por `parse_expression →
parse_term → parse_primary`.

`aurora-interpreter`:

```rust
pub enum Value { Int(i64), Float(f64), Str(String) }

fn eval_binary(left: Value, op: &BinOp, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(...)),

        (l @ Value::Str(_), r) | (l, r @ Value::Str(_)) => match op {
            BinOp::Add => Ok(Value::Str(format!("{}{}", format_value(&l), format_value(&r)))),
            _ => Err(format!("operador {op:?} não é válido envolvendo string")),
        },

        (a, b) => { /* promoção pra Float, igual antes */ }
    }
}
```

`eval_binary` passou de `-> Value` para `-> Result<Value, String>` (agora
existe um caminho de erro real). `format_value` ganhou o braço `Value::Str`
(usado tanto pelo `echo` quanto pela concatenação, pra converter o lado
numérico em texto). `to_f64` ganhou `Value::Str(_) =>
unreachable!(...)` — nunca deveria ser alcançado, já que o braço `Str` de
`eval_binary` intercepta esse caso antes.

Testado manualmente via `aurora-cli` e depois coberto por teste
automatizado (ver `Regra de testes` abaixo):

```
echo "idade: " + 25;          → idade: 25
echo "ola" + " " + "mundo";   → ola mundo
echo 1 + 2.5;                 → 3.5   (sem regressão)
echo "a" - "b";                → Erro: operador Subtract não é válido envolvendo string
```

28 testes passam no workspace (12 `aurora-interpreter`, 11 `aurora-lexer`,
5 `aurora-parser`).

## Desafio

Três decisões de design, tomadas pelo aluno antes de qualquer código:

1. Delimitador: só aspas duplas (`"..."`); aspas simples não têm suporte
   nenhum (caem naturalmente no `panic!` genérico de caractere inesperado
   que o lexer já tinha desde a Aula 02 — nenhum código novo precisou ser
   escrito pra isso).
2. `+` concatena; se um dos lados não for string, o outro é convertido pra
   texto antes de juntar (`"1" + "2"` vira `"12"`, `"idade: " + 25` vira
   `"idade: 25"`).
3. `-`, `*`, `/` envolvendo qualquer string: erro recuperável (`Result`),
   não `panic!`.

## Resolução (percurso do aluno)

**Lexer (`scan_string`).** Primeira tentativa foi um copy-paste quase
literal de `scan_number` (checando `is_ascii_digit`/`.` em vez do
caractere de fechamento, e referenciando `is_float`, uma variável que só
existia em `scan_number` — erro de compilação real). O aluno pediu ajuda
pra montar o teste primeiro ("talvez com o caso de teste fique mais claro
o que eu tenho que fazer") — abordagem validada e reforçada depois como
regra oficial do projeto (ver abaixo). Ainda travou em "não estou sabendo
criar o teste" — teste escrito comentado linha a linha (o motivo do `\"`
dentro de uma string Rust, por que o valor não deve incluir as aspas).
Nova tentativa de implementação usou uma variável `total_quotes`
contabilizando aspas vistas, mas empurrava a aspa de abertura (`first`)
pro `text` antes mesmo de entrar no laço, quebrando a condição
`total_quotes == 1` logo na primeira iteração. Pedido explícito de ajuda
("não dei conta de montar") — implementação final escrita comentada,
removendo o parâmetro `first` (a aspa de abertura não é conteúdo,
diferente de `scan_number`/`scan_identifier`) e usando `return` dentro do
laço em vez de `break`.

**Parser.** `ast.rs` (`Expr::Str(String)`) e o braço `Token::Str(s) =>
Expr::Str(s)` em `parse_primary` saíram corretos de primeira. Um desvio:
o aluno também adicionou um braço `Some(Token::Str(s)) => Stmt::Assign {
name: s, value: Expr::Str(s) }` dentro de `parse_statement` — erro de
compilação (`use of moved value: s`, já que `String` não é `Copy`) que
escondia um problema conceitual maior (uma string sozinha não inicia
nenhum statement válido; ela só existe como expressão, do lado direito de
um `=`, e isso já funcionava sem esse braço extra, via
`parse_expression → parse_primary`). Explicado com a distinção
"`parse_statement` decide QUAL statement está começando; strings só
aparecem dentro de expressões" — removido pelo aluno, resolveu o build.

**Interpreter (`eval_binary` com `Str`).** Depois do erro esperado de
`match` não-exaustivo em `eval_expr` (mesmo mecanismo da Aula 07, agora em
`Expr::Str`), a primeira tentativa resolveu a exaustividade de `to_f64`
adicionando `Value::Str(_) => 0.0` — compilava, mas *não implementava*
nenhuma das três regras combinadas: testado manualmente
(`echo "idade: " + 25;`), o resultado foi `25` (a string virou `0.0`
silenciosamente, sem nenhum erro). Segunda tentativa, em `eval_binary`,
adicionou um braço `(Value::Str(_), _) | (_, Value::Str(_)) =>
Value::Str("".to_string())` — instinto certo de posição (antes do
catch-all), mas devolvia sempre string vazia, sem checar o operador nem
concatenar. Pedido direto ("mostre como deve ser") — implementação final
escrita comentada, incluindo a primeira introdução do aluno ao `@`
(binding) e a padrões alternativos (`|`) com bindings compartilhados,
marcada por ele como "extremamente nova, nunca tinha visto algo parecido"
— explicada em detalhe com tabela comparando desestruturação simples vs.
`@`, e o motivo de `l`/`r` precisarem existir nos dois lados do `|`.

**Nova regra de processo (teste primeiro).** Depois de ver o bug do `to_f64`
passar despercebido até um teste manual pela CLI, o aluno pediu
explicitamente: a partir de agora, a IA escreve os testes definindo o
comportamento esperado, e ele implementa a solução com ajuda da IA até
eles passarem — formalizado no `CLAUDE.md` ("Regra de testes"). Aplicado
ainda nesta mesma aula: escritos 13 testes novos cobrindo o que faltava
(lexer: string vazia, string com espaço/número, string não fechada dando
panic; parser: `Expr::Str` via `parse_primary`, `let` com string;
interpreter: `eval_expr` de `Expr::Str`, concatenação `Str+Str`,
`Str+Int`, `Int+Str` — confirmando que a ordem original esquerda/direita é
preservada —, `Str+Float`, e erro em `-`/`*`/`/` envolvendo string). Como
a implementação já estava correta nesse ponto, todos passaram de primeira
— o ganho foi a garantia contra regressão futura, não uma correção.

## Conclusão

Aurora agora tem um terceiro tipo de valor (`Value::Str`), com concatenação
via `+` (convertendo números pra texto quando necessário) e erro
recuperável em `-`/`*`/`/` envolvendo string — sem panics, seguindo o
mesmo padrão de `Result` da Aula 07. O projeto ganhou também uma mudança
de processo permanente: testes escritos pela IA antes da implementação,
formalizada no `CLAUDE.md`.

## Próximos passos

Aula 10 — Operadores: os operadores atuais (`+ - * /`) foram tratados numa
mini-decisão de tipos nesta aula (`Str` com aritmética); a Aula 10 deve
expandir o conjunto de operadores (comparação, lógicos) — que vai reabrir a
mesma pergunta de "o que cada `Value` faz com cada operador", já com o
precedente de erro recuperável estabelecido aqui.
