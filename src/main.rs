use {std::sync::Arc, framework::{Error,throws, text::{Style,Attribute,FontStyle}}};

pub struct StyledText { pub text: Arc<String>, pub style: Vec<Attribute<Style>> }
#[cfg(feature="rust-analyzer")] mod highlight {
    use {super::*, rust_analyzer::*};
    #[throws]
    pub fn style(highlight: impl Iterator<Item=HighlightedRange>) -> impl Iterator<Item=StyledTextRange> {
        highlight.complete().map(|HighlightedRange{range, highlight, ..}| {
            use HighlightTag::*;
            StyledTextRange{
                range,
                style: TextStyle{
                    color: match highlight.tag {
                        Module => bgr{b:0.,r:1.,g:1./3.},
                        Keyword if !highlight.modifiers.iter().any(|it| it == HighlightModifier::ControlFlow) => bgr{b:2./3.,r:2./3.,g:2./3.},
                        Function|Macro => bgr{b:2./3.,r:1.,g:2./3.},
                        Struct|TypeAlias|BuiltinType|TypeParam|Enum => bgr{b:2./3,r:0.,g:2./3.},
                        Field => bgr{b:0.,r:0.,g:2./3},
                        Trait => bgr{b:1.,r:1./2.,g:1.},
                        StringLiteral|NumericLiteral|EnumVariant => bgr{b:0.,r:1.,g:1./3.},
                        Lifetime|Attribute => bgr{b:1.,r:1./3.,g:1./3.},
                        Comment => bgr{b:1./2.,r:1./2.,g:1./2.},
                        _ => bgr{b:1.,r:1.,g:1.},
                    },
                    style:
                        if highlight.modifiers.iter().any(|it| it == HighlightModifier::ControlFlow) { FontStyle::Bold } //else { FontStyle::Normal }
                        else {
                            match highlight.tag {
                                Keyword => FontStyle::Bold, // fixme: Italic
                                _ => FontStyle::Normal
                            }
                        }
                }
            }
        })
    }
    pub fn highlight() -> StyledText { let TextHighlight{text, highlight} = highlight()?; StyledText{text, style: style(highlight().into_iter()).collect()} }
}
#[cfg(not(feature="rust-analyzer"))] mod highlight { // Stub highlight to develop text editor #3720
    use super::*;
    #[throws]
    pub fn highlight() -> StyledText {
        let file = std::fs::read("src/main.rs")?;
        let source = std::str::from_utf8(&file)?;
        let mut depth = 0;
        let mut line_last_root_bracket = None;
        let mut target = String::with_capacity(source.len());
        for (offset, char) in source.char_indices() { // Root item summary
            if char == '{' {
                if depth == 0 { line_last_root_bracket = Some(offset); }
                depth += 1;
            }
            if depth == 0 { target.push(char) }
            if char == '\n' { line_last_root_bracket = None; }
            if char == '}' {
                depth -= 1;
                if depth == 0 { if let Some(backtrack) = line_last_root_bracket { target.push_str(&source[backtrack..=offset]) } }
            }
        }
        let text = Arc::new(target);
        StyledText{text, style: default_style.to_vec()}
    }
}

use framework::*;
#[throws] fn main() {
	rstack_self()?;
    let highlight = highlight::highlight()?;
    if false {
		for &Attribute{range, attribute} in highlight.style.iter() {
			fn print(text: &str, Style{color, style}: Style) {
				let code = match style {
					FontStyle::Normal => 31,
					FontStyle::Bold => 1,
				};
				let bgra8{b,g,r,..} = color.into();
				print!("\x1b[{}m\x1b[38;2;{};{};{}m{}\x1b(B\x1b[m",code, r,g,b, text)
			}
			print(&highlight.text[range], attribute);
		}
	}
    window::run(&mut TextEdit::new(Text::new(&default_font, &highlight.text, &highlight.style)))?
}
