# Aula 06 — Interpretador

## Objetivo

Percorrer o `Vec<Stmt>` produzido pelo parser e **executar de verdade**:
calcular expressões, guardar variáveis num environment, imprimir no `echo`.
Fechar o pipeline completo, rodando um programa Aurora real via `aurora-cli`.

## Conceitos

- **`Value` (runtime) vs `Expr` (árvore).** `Expr` é a estrutura ainda não
  calculada (`1 + 2`); `Value` é o resultado, já pronto (`Value::Int(3)`).
  Mantido `Int(i64)`/`Float(f64)` separados, consistente com `Token`/`Expr`.
- **Decisão de linguagem:** operação entre `Int` e `Float` promove
  automaticamente pra `Float` (`1 + 2.5` → `Float(3.5)`); `Int`/`Int`
  continua `Int` (`10 / 3` → `Int(3)`, divisão inteira). Revisitar quando a
  Aula 16 (Sistema de tipos) formalizar regras mais rígidas.
- **Environment = `HashMap<String, Value>`** — o array associativo que já
  tinha sido deduzido na Aula 01, agora implementado de verdade.
- **`eval_expr` recebe `&Expr`, não `Expr`.** A árvore pertence a quem
  chamou; passar por referência só "empresta" pra leitura, sem tirar a
  posse. Consequência: `match` num `&Expr` faz os valores internos também
  virarem referências (`Expr::Int(n)` com `n: &i64`), por isso `*n`
  (desreferência) pra pegar o valor.
- **`Box<Expr>` "atravessa" sozinho** — passar um `&Box<Expr>` onde se
  espera `&Expr` funciona sem conversão manual (deref coercion).
- **`match (left, right) { ... }`** — match numa tupla compara os dois
  valores ao mesmo tempo; usado pra decidir entre "os dois são Int" vs
  "pelo menos um é Float".
- **`as f64`** — conversão explícita de tipo numérico (cast), equivalente
  ao `(float)` do PHP.
- **`&mut HashMap<...>` como parâmetro comum** — mesma ideia do `&mut self`,
  só que aqui quem "empresta pra escrever" é o `env`, passado entre funções
  (`eval_stmt`, `run_program`) em vez de morar dentro de um `self`.
- **Imprimir de verdade**: `{:?}` (debug) mostraria `Int(3)`; um
  `format_value` próprio + `println!("{}", texto)` (formatação normal, não
  debug) imprime só `3`, como um `echo` de verdade deveria.

## Implementação

Novo crate `crates/aurora-interpreter` (depende de `aurora-parser`):

```rust
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
}

pub fn eval_expr(expr: &Expr, env: &HashMap<String, Value>) -> Value {
    match expr {
        Expr::Int(n) => Value::Int(*n),
        Expr::Float(n) => Value::Float(*n),
        Expr::Ident(name) => match env.get(name) {
            Some(valor) => valor.clone(),
            None => panic!("variável não definida: {name}"),
        },
        Expr::Binary { left, op, right } => {
            let valor_esquerdo = eval_expr(left, env);
            let valor_direito = eval_expr(right, env);
            eval_binary(valor_esquerdo, op, valor_direito)
        }
    }
}

fn eval_binary(left: Value, op: &BinOp, right: Value) -> Value {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Value::Int(match op {
            BinOp::Add => a + b,
            BinOp::Subtract => a - b,
            BinOp::Multiply => a * b,
            BinOp::Divide => a / b,
        }),
        (a, b) => {
            let a = to_f64(a);
            let b = to_f64(b);
            Value::Float(match op {
                BinOp::Add => a + b,
                BinOp::Subtract => a - b,
                BinOp::Multiply => a * b,
                BinOp::Divide => a / b,
            })
        }
    }
}

fn to_f64(value: Value) -> f64 {
    match value {
        Value::Int(n) => n as f64,
        Value::Float(n) => n,
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Int(n) => n.to_string(),
        Value::Float(n) => n.to_string(),
    }
}

pub fn eval_stmt(stmt: &Stmt, env: &mut HashMap<String, Value>) {
    match stmt {
        Stmt::Assign { name, value } => {
            let valor = eval_expr(value, env);
            env.insert(name.clone(), valor);
        }
        Stmt::Echo(expr) => {
            let valor = eval_expr(expr, env);
            println!("{}", format_value(&valor));
        }
    }
}

pub fn run_program(program: &Vec<Stmt>, env: &mut HashMap<String, Value>) {
    for stmt in program {
        eval_stmt(stmt, env);
    }
}
```

`aurora-cli` ganhou dependências dos três crates (`aurora-lexer`,
`aurora-parser`, `aurora-interpreter`) e agora roda um programa de verdade:

```rust
fn main() {
    let source = "x = 1 + 2 * 3; echo x; y = x - 1; echo y;";

    let tokens = Lexer::new(source).tokenize();
    let program = Parser::new(tokens).parse_program();

    let mut env = HashMap::new();
    run_program(&program, &mut env);
}
```

`cargo run -p aurora-cli` imprime `7` e `6` — pipeline completo, ponta a
ponta, texto → tokens → AST → execução. 14 testes passando no workspace.

## Resolução (percurso do aluno)

O `eval_expr` pros casos simples (`Int`/`Float`/`Ident`) saiu quase pronto
(colado praticamente como no exemplo, por decisão consciente de foco —
entender em vez de reinventar). Um tropeço real: importou
`use crate::{BinOp, Expr, Stmt};` em vez de `use aurora_parser::{...}` —
mesma classe de erro do `Parser::new` na Aula 04, mas invertida: dessa vez
`crate::` apontava pro crate errado (o próprio `aurora-interpreter`, que
não tem esses tipos), em vez de pro crate que realmente os define
(`aurora-parser`, uma dependência).

O `eval_binary`/`to_f64` (mais denso: função auxiliar chamando outra,
match em tupla, cast `as`) foi entregue já escrito e comentado, por
decisão de ritmo.

Na etapa de `eval_stmt`/`run_program`, o aluno relatou estar "muito
perdido" — sinal reconhecido e respeitado: em vez de insistir em mais uma
tentativa independente, essa parte também foi escrita comentada linha a
linha, com foco em deixar claro o "porquê" de cada `&`/`&mut`/`.clone()`
(ver [[feedback-aurora-mentor-role]] — reservar autoria do aluno pra
partes centrais do aprendizado, mas ceder quando o acúmulo de conceitos
novos vira bloqueio real).

## Conclusão

Interpretador funcionando ponta a ponta. Aurora já executa atribuição,
aritmética com precedência e promoção Int→Float, e `echo`, via
`aurora-cli`.

## Próximos passos

Aula 07 — Variáveis: aprofundar o environment (reatribuição, erros de
variável não declarada com mensagem melhor que `panic!` cru) antes de
avançar pra Aula 08 (Escopos), onde múltiplos environments aninhados vão
importar de verdade (blocos, funções).
