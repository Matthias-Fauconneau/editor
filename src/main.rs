use {std::{str::from_utf8, fs::read, env::args}, framework::*, rowan::{TextSize,TextRange}, ra_ide::{Analysis, HighlightTag, HighlightedRange}};
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

#[throws]
fn main() {
    let file = read(args().nth(1).ok()?)?;
    let text = from_utf8(&file)?;
    let highlight = {let (analysis, file_id) = Analysis::from_single_file(text.to_string()); analysis.highlight(file_id)?};
    Complete::new(highlight.into_iter()).map(|HighlightedRange{range, highlight, ..}|  {
        println!("{:?} {}", highlight, &text[range]);
         //type Color = bgra8;
        //pub struct TextStyle { pub color: Color, pub style: FontStyle }
        //pub struct Span<'a> {pub text: &'a str, pub style: TextStyle }
    }).for_each(drop);
    //window(&mut Text::new(text))?
}
