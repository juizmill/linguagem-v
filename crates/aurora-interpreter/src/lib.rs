// Aula 06 — Interpretador.
use std::collections::HashMap;

use aurora_parser::{BinOp, Expr, Stmt};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
}

pub fn eval_expr(expr: &Expr, env: &HashMap<String, Value>) -> Result<Value, String> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Float(n) => Ok(Value::Float(*n)),
        Expr::Ident(name) => match env.get(name) {
            Some(valor) => Ok(valor.clone()),
            None => Err(format!("variável não definida: {name}")),
        },
        Expr::Binary { left, op, right } => {
            // left/right são Box<Expr>; Rust "enxerga através" do Box
            // automaticamente aqui, então dá pra passar direto pra eval_expr
            // (que espera &Expr) sem nenhuma conversão manual.
            let valor_esquerdo = eval_expr(left, env)?;
            let valor_direito = eval_expr(right, env)?;
            Ok(eval_binary(valor_esquerdo, op, valor_direito))
        }
    }
}

// Recebe os dois valores JÁ calculados (não a árvore) e decide o resultado.
fn eval_binary(left: Value, op: &BinOp, right: Value) -> Value {
    // match numa tupla (left, right): compara os dois valores AO MESMO TEMPO.
    // Só cai no primeiro braço se os dois forem Int; qualquer outra combinação
    // (Float+Float, Int+Float, Float+Int) cai no braço de baixo.
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Value::Int(match op {
            BinOp::Add => a + b,
            BinOp::Subtract => a - b,
            BinOp::Multiply => a * b,
            BinOp::Divide => a / b,
        }),

        // pelo menos um dos dois é Float: promove os dois pra f64 e o
        // resultado vira Float, do jeito que você decidiu.
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

// "as f64": conversão explícita de tipo numérico (equivalente ao cast (float)
// do PHP). Int(3) as f64 vira 3.0; Float(n) já é f64, só devolve.
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

// Result<(), String>: "()" é o tipo "nada" (o void do PHP) -- eval_stmt não
// tem valor útil pra devolver quando dá certo, só precisa avisar se deu Err.
pub fn eval_stmt(stmt: &Stmt, env: &mut HashMap<String, Value>) -> Result<(), String> {
    match stmt {
        Stmt::Assign { name, value } => {
            // "=" é REATRIBUIÇÃO: só faz sentido se "name" já foi declarado
            // com "let" antes. contains_key só CONSULTA o HashMap (não altera
            // nada) -- por isso pode ser chamado mesmo com env sendo &mut:
            // "&mut" dá PERMISSÃO de escrever, não obriga a escrever toda vez.
            // "name" aqui já é &String (veio do destructuring de &Stmt), então
            // passa direto pro contains_key, sem precisar de .clone().
            if !env.contains_key(name) {
                // "return" explícito: sai da função AGORA, de dentro de um
                // "if" (não é a última linha do bloco, por isso não dá pra
                // só "deixar a expressão pendurada" como no Ok(()) lá embaixo
                // -- precisa do "return" pra interromper o resto do braço).
                return Err(format!(
                    "variável '{name}' não declarada -- use 'let' antes de atribuir"
                ));
            }

            // "value" aqui é um &Expr (a árvore ainda não calculada, ex: 1 + 2).
            // eval_expr calcula ela e devolve Result<Value, String>; o "?"
            // desembrulha o Ok(valor) OU já sai desta função com o mesmo Err,
            // sem precisar de match manual (mesma ideia do "dobro"/"analisa").
            let valor = eval_expr(value, env)?;

            // env.insert(chave, valor): grava no HashMap, igual $env['x'] = valor;
            // "name" é &String (emprestado da árvore) -- o HashMap precisa ser
            // DONO da própria chave, então .clone() faz uma cópia da string
            // só pra ele guardar (a árvore continua com a sua própria cópia).
            env.insert(name.clone(), valor);

            // Ok(()): deu tudo certo, e não tem valor nenhum pra devolver além
            // disso -- daí o "()" vazio dentro do Ok. SEM ";" no final: essa
            // linha precisa ser a ÚLTIMA EXPRESSÃO do braço, senão o bloco
            // "cai pro final" sem valor (o erro que você teve antes).
            Ok(())
        }

        Stmt::Let { name, value } => {
            // "let" é DECLARAÇÃO: sempre grava, sem checar se "name" já
            // existia -- diferente do Assign. (Se já existia, isso troca o
            // valor antigo; formalizar "erro ao redeclarar" fica pra depois,
            // se algum dia você decidir que faz sentido pra Aurora.)
            let valor = eval_expr(value, env)?;
            env.insert(name.clone(), valor);
            Ok(())
        }

        Stmt::Echo(expr) => {
            let valor = eval_expr(expr, env)?;

            // format_value transforma o Value num texto "limpo" (sem o nome
            // da variante); println!("{}", ...) imprime esse texto no
            // terminal, com quebra de linha no final -- igual um echo do PHP.
            println!("{}", format_value(&valor));
            Ok(())
        }
    }
}

pub fn run_program(program: &Vec<Stmt>, env: &mut HashMap<String, Value>) -> Result<(), String> {
    // "for stmt in program": percorre cada Stmt da lista, na ordem, chamando
    // eval_stmt pra cada um. Como cada eval_stmt pode ALTERAR o env (no caso
    // do Assign/Let), passamos env pra cada chamada -- é o mesmo HashMap
    // sendo atualizado statement após statement (por isso "x = 1;" antes de
    // "echo x;" funciona: quando o echo roda, o x já está no env).
    for stmt in program {
        // "?" aqui também: se QUALQUER statement do programa der erro, o
        // "for" para nesse ponto (não roda o resto do programa) e
        // run_program já devolve esse mesmo Err pra quem chamou (o main).
        eval_stmt(stmt, env)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_ident_busca_no_environment() -> Result<(), String> {
        let mut env = HashMap::new();
        env.insert("x".to_string(), Value::Int(3));

        let resultado = eval_expr(&Expr::Ident("x".to_string()), &env)?;

        assert_eq!(resultado, Value::Int(3));

        Ok(())
    }

    #[test]
    fn eval_binary_int_com_int_continua_int() -> Result<(), String> {
        let env = HashMap::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Int(1)),
            op: BinOp::Add,
            right: Box::new(Expr::Int(2)),
        };

        assert_eq!(eval_expr(&expr, &env)?, Value::Int(3));
        Ok(())
    }

    #[test]
    fn eval_binary_promove_pra_float_quando_mistura() -> Result<(), String> {
        let env = HashMap::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Int(1)),
            op: BinOp::Add,
            right: Box::new(Expr::Float(2.5)),
        };

        assert_eq!(eval_expr(&expr, &env)?, Value::Float(3.5));

        Ok(())
    }

    #[test]
    fn run_program_executa_atribuicao_e_echo() -> Result<(), String> {
        // x = 1 + 2; echo x;  -- não dá pra testar o println! diretamente,
        // então conferimos o efeito observável: o valor final de "x" no env.
        let program = vec![
            Stmt::Let {
                name: "x".to_string(),
                value: Expr::Binary {
                    left: Box::new(Expr::Int(1)),
                    op: BinOp::Add,
                    right: Box::new(Expr::Int(2)),
                },
            },
            Stmt::Echo(Expr::Ident("x".to_string())),
        ];

        let mut env = HashMap::new();
        run_program(&program, &mut env)?;

        assert_eq!(env.get("x"), Some(&Value::Int(3)));

        Ok(())
    }
}
