use std::fs::read_to_string;
use pr47::util::diag::DiagContext;
use pr47::parse::lexer::Lexer;

fn main() {
    let args: Vec<String> = std::env::args()
        .into_iter()
        .map(|arg| arg.to_string())
        .collect::<_>();
    if args.len() != 2 {
        eprintln!("Program usage: token47 [filename]");
        return;
    }

    let source: String = read_to_string(&args[1]).expect("cannot read appointed file");

    let mut diag: DiagContext = DiagContext::new();
    let mut lexer: Lexer = Lexer::new(&args[1], &source, &mut diag);

    eprint!("[");
    while let Some(token /*: Token*/) = lexer.next_token() {
        eprint!("{}, ", token);
    }
    eprintln!("]");
    drop(lexer);

    if diag.has_error() {
        eprintln!("*** There's error in source code");
    }

    drop(diag);
}
