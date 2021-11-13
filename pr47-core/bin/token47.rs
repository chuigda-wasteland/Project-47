use std::cell::RefCell;
use std::fs::read_to_string;

use pr47::diag::DiagContext;
use pr47::diag::diag_data::diag_message;
use pr47::diag::location::SourceCoord;
use pr47::diag::source::SourceManager;
use pr47::parse::lexer::Lexer;
use pr47::syntax::token::{Token, TokenInner};

fn main() {
    let args: Vec<String> = std::env::args()
        .into_iter()
        .map(|arg| arg.to_string())
        .collect::<_>();
    if args.len() != 2 {
        eprintln!("Program usage: token47 [filename]");
        return;
    }

    let mut source_mgr: SourceManager = SourceManager::new();
    let source: String = read_to_string(&args[1]).expect("cannot read appointed file");
    let file_id: u32 = source_mgr.add_file(&args[1], &source);

    let diag: RefCell<DiagContext> = RefCell::new(DiagContext::new());
    let mut lexer: Lexer = Lexer::new(file_id, &source, &diag);

    let mut tokens: Vec<Token> = Vec::new();
    loop {
        let token: Token = lexer.next_token();
        if token.token_inner == TokenInner::EndOfInput {
            break;
        }
        tokens.push(token);
    }

    eprintln!("tokens = {:?}", tokens);
    drop(lexer);

    for diag /*: Diagnostic<'_>*/ in diag.borrow_mut().clear_reset() {
        let location: (&str, SourceCoord) = diag.location.compute_coord(&source_mgr);
        eprintln!(
            "diag: location = ({}:{}), code = {}, message template = {}, args = {:?}",
            location.1.line,
            location.1.col,
            diag.diag_id,
            diag_message(diag.diag_id),
            diag.args
        );
        eprintln!("  |> source=\"{}\"", location.0);
        drop(diag);
    }

    drop(diag);
}
