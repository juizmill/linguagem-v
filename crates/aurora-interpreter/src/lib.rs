// Aula 06 — Interpretador.
use std::collections::HashMap;

use aurora_parser::{BinOp, Expr, Stmt};

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
            // left/right são Box<Expr>; Rust "enxerga através" do Box
            // automaticamente aqui, então dá pra passar direto pra eval_expr
            // (que espera &Expr) sem nenhuma conversão manual.
            let valor_esquerdo = eval_expr(left, env);
            let valor_direito = eval_expr(right, env);
            eval_binary(valor_esquerdo, op, valor_direito)
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

pub fn eval_stmt(stmt: &Stmt, env: &mut HashMap<String, Value>) {
    match stmt {
        Stmt::Assign { name, value } => {
            // "value" aqui é um &Expr (a árvore ainda não calculada, ex: 1 + 2).
            // eval_expr calcula ela e devolve o Value pronto (ex: Value::Int(3)).
            // Passamos "env" (não "&mut env"): eval_expr só PRECISA LER o
            // environment (pra achar variáveis dentro da expressão, se houver),
            // por isso ele pede &HashMap, não &mut -- e como já estamos com uma
            // referência (env: &mut HashMap), o Rust deixa "emprestar de volta"
            // como leitura sem problema.
            let valor = eval_expr(value, env);

            // env.insert(chave, valor): grava no HashMap, igual $env['x'] = valor;
            // "name" é &String (emprestado da árvore) -- o HashMap precisa ser
            // DONO da própria chave, então .clone() faz uma cópia da string
            // só pra ele guardar (a árvore continua com a sua própria cópia).
            env.insert(name.clone(), valor);
        }

        Stmt::Echo(expr) => {
            let valor = eval_expr(expr, env);

            // format_value transforma o Value num texto "limpo" (sem o nome
            // da variante); println!("{}", ...) imprime esse texto no
            // terminal, com quebra de linha no final -- igual um echo do PHP.
            println!("{}", format_value(&valor));
        }
    }
}

pub fn run_program(program: &Vec<Stmt>, env: &mut HashMap<String, Value>) {
    // "for stmt in program": percorre cada Stmt da lista, na ordem, chamando
    // eval_stmt pra cada um. Como cada eval_stmt pode ALTERAR o env (no caso
    // do Assign), passamos env pra cada chamada -- é o mesmo HashMap sendo
    // atualizado statement após statement (por isso "x = 1;" antes de
    // "echo x;" funciona: quando o echo roda, o x já está no env).
    for stmt in program {
        eval_stmt(stmt, env);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_ident_busca_no_environment() {
        let mut env = HashMap::new();
        env.insert("x".to_string(), Value::Int(3));

        let resultado = eval_expr(&Expr::Ident("x".to_string()), &env);

        assert_eq!(resultado, Value::Int(3));
    }

    #[test]
    fn eval_binary_int_com_int_continua_int() {
        let env = HashMap::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Int(1)),
            op: BinOp::Add,
            right: Box::new(Expr::Int(2)),
        };

        assert_eq!(eval_expr(&expr, &env), Value::Int(3));
    }

    #[test]
    fn eval_binary_promove_pra_float_quando_mistura() {
        let env = HashMap::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Int(1)),
            op: BinOp::Add,
            right: Box::new(Expr::Float(2.5)),
        };

        assert_eq!(eval_expr(&expr, &env), Value::Float(3.5));
    }

    #[test]
    fn run_program_executa_atribuicao_e_echo() {
        // x = 1 + 2; echo x;  -- não dá pra testar o println! diretamente,
        // então conferimos o efeito observável: o valor final de "x" no env.
        let program = vec![
            Stmt::Assign {
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
        run_program(&program, &mut env);

        assert_eq!(env.get("x"), Some(&Value::Int(3)));
    }
}
