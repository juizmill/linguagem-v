use std::collections::HashMap;

use aurora_parser::{BinOp, Expr, Stmt};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
}

pub struct Environment {
    valores: HashMap<String, Value>,
    pai: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            valores: HashMap::new(),
            pai: None,
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        match self.valores.get(name) {
            Some(valor) => Some(valor.clone()),
            None => match &self.pai {
                Some(pai) => pai.get(name),
                None => None,
            },
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.valores.insert(name, value);
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.valores.contains_key(name) {
            self.valores.insert(name.to_string(), value);
            return Ok(());
        }

        match self.pai.as_deref_mut() {
            Some(pai) => pai.assign(name, value),
            None => Err(format!(
                "variável '{name}' não declarada -- use 'let' antes de atribuir"
            )),
        }
    }
}

pub fn eval_expr(expr: &Expr, env: &Environment) -> Result<Value, String> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Float(n) => Ok(Value::Float(*n)),
        Expr::Ident(name) => match env.get(name) {
            Some(valor) => Ok(valor),
            None => Err(format!("variável não definida: {name}")),
        },
        Expr::Str(s) => Ok(Value::Str(s.clone())),
        Expr::Binary { left, op, right } => {
            let valor_esquerdo = eval_expr(left, env)?;
            let valor_direito = eval_expr(right, env)?;
            eval_binary(valor_esquerdo, op, valor_direito)
        }
    }
}

fn eval_binary(left: Value, op: &BinOp, right: Value) -> Result<Value, String> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(match op {
            BinOp::Add => a + b,
            BinOp::Subtract => a - b,
            BinOp::Multiply => a * b,
            BinOp::Divide => a / b,
        })),

        (l @ Value::Str(_), r) | (l, r @ Value::Str(_)) => match op {
            BinOp::Add => Ok(Value::Str(format!(
                "{}{}",
                format_value(&l),
                format_value(&r)
            ))),
            _ => Err(format!("operador {op:?} não é válido envolvendo string")),
        },

        (a, b) => {
            let a = to_f64(a);
            let b = to_f64(b);
            Ok(Value::Float(match op {
                BinOp::Add => a + b,
                BinOp::Subtract => a - b,
                BinOp::Multiply => a * b,
                BinOp::Divide => a / b,
            }))
        }
    }
}

fn to_f64(value: Value) -> f64 {
    match value {
        Value::Int(n) => n as f64,
        Value::Float(n) => n,
        Value::Str(_) => unreachable!("to_f64 não deveria receber Value::Str"),
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Int(n) => n.to_string(),
        Value::Float(n) => n.to_string(),
        Value::Str(s) => s.clone(),
    }
}

pub fn eval_stmt(stmt: &Stmt, env: &mut Environment) -> Result<(), String> {
    match stmt {
        Stmt::Assign { name, value } => {
            let valor = eval_expr(value, env)?;
            env.assign(name, valor)?;
            Ok(())
        }

        Stmt::Let { name, value } => {
            let valor = eval_expr(value, env)?;
            env.define(name.clone(), valor);
            Ok(())
        }

        Stmt::Echo(expr) => {
            let valor = eval_expr(expr, env)?;
            println!("{}", format_value(&valor));
            Ok(())
        }
    }
}

pub fn run_program(program: &Vec<Stmt>, env: &mut Environment) -> Result<(), String> {
    for stmt in program {
        eval_stmt(stmt, env)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_ident_busca_no_environment() -> Result<(), String> {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::Int(3));

        let resultado = eval_expr(&Expr::Ident("x".to_string()), &env)?;

        assert_eq!(resultado, Value::Int(3));

        Ok(())
    }

    #[test]
    fn eval_binary_int_com_int_continua_int() -> Result<(), String> {
        let env = Environment::new();
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
        let env = Environment::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Int(1)),
            op: BinOp::Add,
            right: Box::new(Expr::Float(2.5)),
        };

        assert_eq!(eval_expr(&expr, &env)?, Value::Float(3.5));

        Ok(())
    }

    #[test]
    fn eval_expr_str_devolve_value_str() -> Result<(), String> {
        let env = Environment::new();
        let resultado = eval_expr(&Expr::Str("oi".to_string()), &env)?;

        assert_eq!(resultado, Value::Str("oi".to_string()));
        Ok(())
    }

    #[test]
    fn eval_binary_concatena_string_com_string() -> Result<(), String> {
        let env = Environment::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Str("ola ".to_string())),
            op: BinOp::Add,
            right: Box::new(Expr::Str("mundo".to_string())),
        };

        assert_eq!(eval_expr(&expr, &env)?, Value::Str("ola mundo".to_string()));
        Ok(())
    }

    #[test]
    fn eval_binary_concatena_string_com_numero_convertendo_pra_texto() -> Result<(), String> {
        // "idade: " + 25 -> "idade: 25" (Str à ESQUERDA, número à direita)
        let env = Environment::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Str("idade: ".to_string())),
            op: BinOp::Add,
            right: Box::new(Expr::Int(25)),
        };

        assert_eq!(eval_expr(&expr, &env)?, Value::Str("idade: 25".to_string()));
        Ok(())
    }

    #[test]
    fn eval_binary_concatena_numero_com_string_mantendo_ordem() -> Result<(), String> {
        // 25 + " anos" -> "25 anos" (número à ESQUERDA, Str à direita --
        // confirma que "l"/"r" no eval_binary preservam a ordem original,
        // não "string sempre primeiro")
        let env = Environment::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Int(25)),
            op: BinOp::Add,
            right: Box::new(Expr::Str(" anos".to_string())),
        };

        assert_eq!(eval_expr(&expr, &env)?, Value::Str("25 anos".to_string()));
        Ok(())
    }

    #[test]
    fn eval_binary_concatena_string_com_float() -> Result<(), String> {
        let env = Environment::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Str("pi: ".to_string())),
            op: BinOp::Add,
            right: Box::new(Expr::Float(3.5)),
        };

        assert_eq!(eval_expr(&expr, &env)?, Value::Str("pi: 3.5".to_string()));
        Ok(())
    }

    #[test]
    fn eval_binary_subtracao_com_string_da_erro() {
        let env = Environment::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Str("a".to_string())),
            op: BinOp::Subtract,
            right: Box::new(Expr::Str("b".to_string())),
        };

        assert!(eval_expr(&expr, &env).is_err());
    }

    #[test]
    fn eval_binary_multiplicacao_com_string_da_erro() {
        let env = Environment::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Str("a".to_string())),
            op: BinOp::Multiply,
            right: Box::new(Expr::Int(3)),
        };

        assert!(eval_expr(&expr, &env).is_err());
    }

    #[test]
    fn eval_binary_divisao_com_string_da_erro() {
        let env = Environment::new();
        let expr = Expr::Binary {
            left: Box::new(Expr::Int(3)),
            op: BinOp::Divide,
            right: Box::new(Expr::Str("a".to_string())),
        };

        assert!(eval_expr(&expr, &env).is_err());
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

        let mut env = Environment::new();
        run_program(&program, &mut env)?;

        assert_eq!(env.get("x"), Some(Value::Int(3)));

        Ok(())
    }
}
