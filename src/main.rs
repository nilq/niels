extern crate colored;

extern crate nanbox;

mod niels;

use niels::lexer::*;
use niels::parser::*;
use niels::source::*;

fn main() {
    let content = r#"
pub funk foo:
  hello = 10

  a = hello + fff

  b = {
    y: 100
    bax: 200
  }

  x = 10_000
  x += b.bax

  return r"hey\n\n\n"

funk bar(a, b):
  return [a, b]

foo()
bar(1, 2 + 10)
"#;

    let source = Source::from(
        "<main>",
        content.lines().map(|x| x.into()).collect::<Vec<String>>(),
    );
    let lexer = Lexer::default(content.chars().collect(), &source);

    let mut tokens = Vec::new();

    for token_result in lexer {
        if let Ok(token) = token_result {
            tokens.push(token)
        } else {
            return;
        }
    }

    let mut parser = Parser::new(tokens, &source);

    match parser.parse() {
        Ok(ref ast) => {
            println!("{:#?}", ast);
        }

        _ => return,
    }
}
