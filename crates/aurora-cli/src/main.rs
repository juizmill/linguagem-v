use std::collections::HashMap;
use std::env;
use std::fs;

use aurora_interpreter::run_program;
use aurora_lexer::Lexer;
use aurora_parser::Parser;

fn main() {
    // env::args() devolve um iterador com os argumentos da linha de comando;
    // o item [0] é sempre o caminho do próprio binário, por isso o arquivo
    // que a gente quer é o [1]. .collect() junta tudo num Vec<String>.
    let args: Vec<String> = env::args().collect();

    // args.get(1): Option<&String> -- None se o usuário não passou nenhum
    // argumento. match decide o que fazer nos dois casos.
    let caminho = match args.get(1) {
        Some(caminho) => caminho,
        None => {
            eprintln!("uso: aurora <arquivo.aurora>"); // eprintln!: imprime no stderr, não no stdout
            return; // sai do main mais cedo, sem tentar continuar
        }
    };

    // fs::read_to_string devolve Result<String, io::Error>: Ok(texto) se deu
    // certo, Err(erro) se o arquivo não existir, não tiver permissão, etc.
    let source = match fs::read_to_string(caminho) {
        Ok(texto) => texto,
        Err(erro) => {
            eprintln!("erro ao ler {caminho}: {erro}");
            return;
        }
    };

    let tokens = Lexer::new(&source).tokenize();
    let program = Parser::new(tokens).parse_program();

    let mut ambiente = HashMap::new();
    match run_program(&program, &mut ambiente) {
        Ok(texto) => texto,
        Err(erro) => {
            eprint!("Erro: {erro}");
            return;
        }
    }
}
