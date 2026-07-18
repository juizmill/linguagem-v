// Aula 06 — Interpretador.
use std::collections::HashMap;

use aurora_parser::{BinOp, Expr, Stmt};

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
}

// Aula 08 — Escopos.
// "pai: Option<Box<Environment>>" -- um escopo pode ou não ter um escopo
// "de fora" (o global não tem). Box porque um Environment não pode conter
// outro Environment "cru" dentro dele (tamanho infinito).
pub struct Environment {
    valores: HashMap<String, Value>,
    pai: Option<Box<Environment>>,
}

impl Environment {
    // Cria o escopo global: sem pai, sem nenhuma variável ainda.
    pub fn new() -> Self {
        Self {
            valores: HashMap::new(),
            pai: None,
        }
    }

    // Procura "name" no próprio escopo primeiro; se não achar E existir um
    // "pai", procura nele (recursivo). Se não achar em lugar nenhum, None.
    // Dica: "match &self.pai { Some(pai) => ..., None => ... }" -- parecido
    // com o "soma_todos" do No/lista ligada.
    pub fn get(&self, name: &str) -> Option<Value> {
        // self.valores.get(name) devolve Option<&Value> -- .clone() tira
        // uma cópia de dentro da referência, igual já fazia em eval_expr
        // pro Expr::Ident (Some(valor) => Ok(valor.clone())).
        match self.valores.get(name) {
            Some(valor) => Some(valor.clone()),

            // Não achou aqui. Existe um escopo "de fora" (self.pai)?
            None => match &self.pai {
                // "pai" aqui é &Box<Environment> (referência a uma caixa
                // contendo um Environment). Box "atravessa sozinho" --
                // chamar .get(name) nele funciona sem nenhuma conversão
                // manual, igual Box<Expr> atravessava em eval_expr.
                // Isso é a MESMA função se chamando de novo (recursão),
                // só que agora em cima do escopo pai -- exatamente como
                // soma_todos(prox) se chamava de novo em cima do próximo
                // nó da lista.
                Some(pai) => pai.get(name),

                // Não tem pai (chegou no escopo global) e não achou lá
                // nele também: a variável não existe em lugar nenhum.
                None => None,
            },
        }
    }

    // "let" de sempre: grava no PRÓPRIO escopo, sem checar se já existia
    // (mesma regra que Stmt::Let já tinha antes de existir Environment).
    pub fn define(&mut self, name: String, value: Value) {
        self.valores.insert(name, value);
    }

    // "=" (reatribuição). Diferente de "get": aqui precisamos ESCREVER no
    // nível da cadeia onde a variável foi declarada -- que pode não ser o
    // escopo atual. Por isso "&mut self" (precisa de permissão de escrita)
    // e Result<(), String> em vez de Option<Value> (mesma ideia do
    // Stmt::Assign de antes: se não existir em NENHUM escopo, é erro, não
    // "silenciosamente não faz nada").
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), String> {
        // contains_key só CONSULTA (não precisa de &mut pra isso, mas o
        // método inteiro já pediu &mut self porque o insert() logo abaixo
        // vai precisar). Igual o "if !env.contains_key(name)" que já
        // existia em eval_stmt, só que agora perguntando só ao ESCOPO
        // ATUAL primeiro.
        if self.valores.contains_key(name) {
            // Achou aqui mesmo: atualiza neste nível e para -- não sobe
            // pro pai, porque a variável mais "próxima" (a deste escopo)
            // é a que "= " deve afetar, não uma de mesmo nome lá em cima.
            self.valores.insert(name.to_string(), value);
            return Ok(());
        }

        // Não achou nesta valores. Existe um "pai" pra tentar?
        // self.pai é Option<Box<Environment>>. Pra RECURSÃO MUTÁVEL
        // (chamar .assign de novo, agora podendo escrever no pai),
        // precisamos de uma &mut Environment, não só uma &Environment
        // como o "match &self.pai" do get() usava.
        // ".as_deref_mut()" faz exatamente essa conversão: de
        // "&mut Option<Box<Environment>>" (o que vem de self.pai quando
        // self já é &mut) para "Option<&mut Environment>" -- ele
        // "atravessa" o Box e te dá uma referência mutável pro que tem
        // dentro, sem tirar o dono (self.pai continua sendo o dono do Box).
        match self.pai.as_deref_mut() {
            // "pai" aqui já é &mut Environment: chamar .assign(name, value)
            // nele é a MESMA função se chamando de novo, subindo um nível
            // -- só que agora com permissão de escrita, não só leitura.
            Some(pai) => pai.assign(name, value),

            // Chegou no escopo global (sem pai) e não achou em lugar
            // nenhum da cadeia: mesma mensagem de erro que Stmt::Assign
            // já usava direto no HashMap.
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
        Expr::Binary { left, op, right } => {
            let valor_esquerdo = eval_expr(left, env)?;
            let valor_direito = eval_expr(right, env)?;
            Ok(eval_binary(valor_esquerdo, op, valor_direito))
        }
    }
}

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
