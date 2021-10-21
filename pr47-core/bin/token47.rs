fn main() {
    /*
    let args: Vec<String> = std::env::args()
        .into_iter()
        .map(|arg| arg.to_string())
        .collect::<_>();
    if args.len() != 2 {
        eprintln!("Program usage: token47 [filename]");
        return;
    }

    let source: String = read_to_string(&args[1]).expect("cannot read appointed file");
    let lines: Vec<&str> = source.split('\n').collect::<Vec<_>>();

    let mut diag: DiagContext = DiagContext::new();
    let mut lexer: Lexer = Lexer::new(&args[1], &source, &mut diag);

    let mut tokens: Vec<Token> = Vec::new();
    while let Some(token /*: Token*/) = lexer.next_token() {
        tokens.push(token);
    }
    drop(lexer);

    let mut source_map: SourceMap = SourceMap::new();
    source_map.add_source(&args[1], lines);

    for diag /*: Diagnostic<'_>*/ in diag.clear_reset() {
        eprintln!("{}", prettify_diag(&diag, &source_map));
        drop(diag)
    }

    eprintln!("{:?}", tokens);

    drop(diag);
    */
    todo!()
}
