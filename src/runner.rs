use crate::Expr;

pub fn interpret(statement: Expr) {
    match statement {
        Expr::Print(e) => match *e {
            Expr::String(s) => {
                println!("{}", s);
            }
            Expr::Number(n) => {
                println!("{}", n);
            }
            Expr::Bool(b) => {
                println!("{}", b);
            }
            Expr::Nil => {
                println!("nil");
            }
            _ => {
                print!("Invalid expression");
            }
        },
        _ => {}
    }
}
