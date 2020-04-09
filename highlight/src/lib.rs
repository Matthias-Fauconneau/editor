use {framework::{*, color::bgr}, rowan::{TextSize,TextRange}, ra_ide::{Analysis, HighlightTag, HighlightedRange}};
/*impl Zero for TextSize { fn zero() -> Self { rowan::TextSize::zero() } }*/ #[allow(non_snake_case)] fn Zero_zero() -> rowan::TextSize { rowan::TextSize::zero() }

/// Complete flat HighlightedRange iterator domain coverage with a default highlight
struct Complete<I:Iterator> { iter: std::iter::Peekable<I>, last_end: TextSize}
impl<I:Iterator> Complete<I> { fn new(iter: I) -> Self { Self{iter: iter.peekable(), last_end: Zero_zero()} } }
impl<I:Iterator<Item=HighlightedRange>> Iterator for Complete<I> {
    type Item = HighlightedRange;
    fn next(&mut self) -> Option<HighlightedRange> {
        let next_start = self.iter.peek()?.range.start(); // todo: yield any last default span
#[macro_export] macro_rules! prefer { ($cond:expr, $($val:expr),* ) => { if !$cond { println!("{}. {:?}", stringify!($cond), ( $( format!("{} = {:?}", stringify!($val), $val), )* ) ); } } }
        prefer!(self.last_end <= next_start, self.last_end, next_start);
        let next =
            if self.last_end < next_start {
                Some(HighlightedRange{
                    range:TextRange::new(self.last_end, next_start),
                    highlight: HighlightTag::None.into() /*fixme:None*/,
                    binding_hash: None
                })
            } else {
                self.iter.next()
            };
        self.last_end = next.as_ref()?.range.end();
        next
    }
}

pub type Color = bgr;
pub enum FontStyle { Normal, Bold, /*Italic, BoldItalic*/ }
pub struct TextStyle { pub color: Color, pub style: FontStyle }
pub struct Span<'a> {pub text: &'a str, pub style: TextStyle }

#[throws]
pub fn highlight(text : &str) -> Vec<Span<'_>> {
    let highlight = {let (analysis, file_id) = Analysis::from_single_file(text.to_string()); analysis.highlight(file_id)?};
    Complete::new(highlight.into_iter()).map(|HighlightedRange{range, highlight, ..}| {
        //println!("{:?} {}", highlight, &text[range]);
        use HighlightTag::*;
        Span{text: &text[range], style: TextStyle{
            color: match highlight.tag {
                Function|Macro => bgr{b:0.,r:1.,g:1.},
                Struct|TypeAlias|BuiltinType|TypeParam => bgr{b:1.,r:0.,g:1.},
                Field => bgr{b:0.,r:0.,g:1.},
                StringLiteral|NumericLiteral|Enum => bgr{b:0.,r:1.,g:1./3.},
                Lifetime|Attribute => bgr{b:1.,r:1./3.,g:1./3.},
                Comment => bgr{b:1./2.,r:1./2.,g:1./2.},
                _ => bgr{b:1.,r:1.,g:1.},
            },
            style: match highlight.tag {
                Keyword => FontStyle::Bold, // todo: Italic fn
                _ => FontStyle::Normal
            }}
        }
    }).collect()
}
