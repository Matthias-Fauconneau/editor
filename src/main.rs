use framework::{color::bgr, *};

pub type Color = bgr;
pub enum FontStyle { Normal, Bold, /*Italic, BoldItalic*/ }
pub struct TextStyle { pub color: Color, pub style: FontStyle }
use text_size::{TextSize, TextRange}; // ~Range<u32> with impl SliceIndex for String
pub struct StyledTextRange { pub range: TextRange, pub style: TextStyle }
pub struct StyledText { pub text: std::sync::Arc<String>, pub style: Vec<StyledTextRange> }

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
#[cfg(not(feature="rust-analyzer"))] mod highlight { // Develop text editor without styling while initial analysis is too slow, blocked on parallel items: rust-analyzer#3485,3720
    use super::*;
    #[throws]
    pub fn highlight() -> StyledText {
        let text = std::sync::Arc::new(std::str::from_utf8(&std::fs::read("src/main.rs")?)?.to_string());
        let style = vec![StyledTextRange{range: TextRange::new(TextSize::zero(), TextSize::of(&text)), style: TextStyle{ color: bgr{b:1.,r:1.,g:1.}, style: FontStyle::Normal }}];
        StyledText{text, style}
    }
}

#[throws]
fn main() {
    let text = highlight::highlight()?;
    for StyledTextRange{range, style} in text.style {
        fn print(text: &str, TextStyle{color, style}: TextStyle) {
            let code = match style {
                FontStyle::Normal => 31,
                FontStyle::Bold => 1,
                //_ => 31
            };
            let bgra8{b,g,r,..} = color.into();
            print!("\x1b[{}m\x1b[38;2;{};{};{}m{}\x1b(B\x1b[m",code, r,g,b, text)
        }
        print(&text.text[range], style);
    }
    //window(&mut Text::new(text))?
}
