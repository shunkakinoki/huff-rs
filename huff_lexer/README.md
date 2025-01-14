## Huff Lexer

Lexical analyzer for the Huff Language.

The Huff Lexer is instantiable with a `FullFileSource` containing the source code, a `FileSource`, and "spans" (a `Vec<(FileSource, Span)>`).

Once instantiated, the lexer can be used to iterate over the tokens in the source code.
It also exposes a number of practical methods for accessing information about the source code
throughout lexing.

#### Usage

The following example steps through the lexing of a simple, single-line source code macro
definition.

```rust
use huff_utils::prelude::*;
use huff_lexer::{Lexer};
use std::ops::Deref;

// Instantiate a new lexer
let source = "#define macro HELLO_WORLD()";
let flattened_source = FullFileSource { source, file: None, spans: vec![] };
let mut lexer = Lexer::new(flattened_source);

// This token should be a Define identifier
let tok = lexer.next().unwrap().unwrap();
assert_eq!(tok, Token::new(TokenKind::Define, Span::new(0..7, None)));
assert_eq!(lexer.current_span().deref(), &Span::new(0..7, None));

// The next token should be the whitespace
let tok = lexer.next().unwrap().unwrap();
assert_eq!(tok, Token::new(TokenKind::Whitespace, Span::new(7..8, None)));
assert_eq!(lexer.current_span().deref(), &Span::new(7..8, None));

// Then we should parse the macro keyword
let tok = lexer.next().unwrap().unwrap();
assert_eq!(tok, Token::new(TokenKind::Macro, Span::new(8..13, None)));
assert_eq!(lexer.current_span().deref(), &Span::new(8..13, None));

// The next token should be another whitespace
let tok = lexer.next().unwrap().unwrap();
assert_eq!(tok, Token::new(TokenKind::Whitespace, Span::new(13..14, None)));
assert_eq!(lexer.current_span().deref(), &Span::new(13..14, None));

// Then we should get the function name
let tok = lexer.next().unwrap().unwrap();
assert_eq!(tok, Token::new(TokenKind::Ident("HELLO_WORLD".to_string()), Span::new(14..25, None)));
assert_eq!(lexer.current_span().deref(), &Span::new(14..25, None));

// Then we should have an open paren
let tok = lexer.next().unwrap().unwrap();
assert_eq!(tok, Token::new(TokenKind::OpenParen, Span::new(25..26, None)));
assert_eq!(lexer.current_span().deref(), &Span::new(25..26, None));

// Lastly, we should have a closing parenthesis
let tok = lexer.next().unwrap().unwrap();
assert_eq!(tok, Token::new(TokenKind::CloseParen, Span::new(26..27, None)));
assert_eq!(lexer.current_span().deref(), &Span::new(26..27, None));

// We covered the whole source
assert_eq!(lexer.current_span().end, source.len());
assert!(lexer.eof);
```
