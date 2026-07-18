#[derive(Debug, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Ident(String),
    Str(String),
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, PartialEq)]
pub enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Assign { name: String, value: Expr },
    Let { name: String, value: Expr },
    Echo(Expr),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monta_arvore_manualmente() {
        let arvore = Expr::Binary {
            left: Box::new(Expr::Int(1)),
            op: BinOp::Add,
            right: Box::new(Expr::Int(2)),
        };

        assert_eq!(
            arvore,
            Expr::Binary {
                left: Box::new(Expr::Int(1)),
                op: BinOp::Add,
                right: Box::new(Expr::Int(2)),
            }
        );
    }
}
