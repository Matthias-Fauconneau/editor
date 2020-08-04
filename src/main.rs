use {std::sync::Arc, core::error::{throws,Error}, ui::text::{Style,Attribute,FontStyle}};

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
        fn items(text: &str) -> impl Iterator<Item=&str> {
			let mut it = text.char_indices().scan(0, |depth, (i, c)| {
				if c == '{' { *depth += 1; }
				let c_depth = *depth;
				if c == '}' { *depth -= 1; }
				Some((i, c_depth, c))
			}).peekable();
			std::iter::from_fn(move || loop {
				for _ in it.peeking_take_while(|&(_, _, c)| c == '\n') {}
				if it.peek().is_none() { return None; }
				let mut it = it.peeking_take_while(|&(_, _, c)| c != '\n').peekable();
				for _ in it.peeking_take_while(|&(_, depth, c)| depth > 0 || c==' ') {}
				if let Some((start,_,_)) = it.next() {
					let mut end = start+1;
					loop {
						for (i,_,_) in it.peeking_take_while(|&(_, depth, _)| depth == 0) { end = i; }
						if it.peek().is_none() { break; }
						for _ in it.peeking_take_while(|&(_, depth, c)| depth > 0 || c==' ') {}
					}
					return Some(text[start..end].trim_end())
				}
			})
		}
		use itertools::Itertools;
        StyledText{text: Arc::new(items(std::str::from_utf8(&std::fs::read("src/main.rs")?)?).join("\n")), style: ui::text::default_style.to_vec()}
    }
}

#[throws] fn main() {
	//core::rstack_self()?;
	let highlight = highlight::highlight()?;
	if false {
		for &Attribute{range: _range, attribute} in highlight.style.iter() {
			fn print(text: &str, Style{color, style}: Style) {
				let code = match style {
					FontStyle::Normal => 31,
					FontStyle::Bold => 1,
				};
				let image::bgra8{b,g,r,..} = color.into();
				print!("\x1b[{}m\x1b[38;2;{};{};{}m{}\x1b(B\x1b[m",code, r,g,b, text)
			}
			//print(&highlight.text[range], attribute);
			print(&highlight.text, attribute);
		}
		println!();
	}
	ui::app::run(ui::edit::Edit::new(&ui::text::default_font, ui::edit::Cow::Borrowed(ui::edit::Buffer{text: &highlight.text, style: &highlight.style})))?
}
