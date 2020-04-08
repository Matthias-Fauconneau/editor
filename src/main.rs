use {framework::*, rowan::{TextSize,TextRange}, ra_ide::{HighlightTag, HighlightedRange}};
/*impl Zero for TextSize { fn zero() -> Self { rowan::TextSize::zero() } }*/ #[allow(non_snake_case)] fn Zero_zero() -> rowan::TextSize { rowan::TextSize::zero() }
#[throws]
fn main() {
    let file = std::fs::read("src/main.rs")?;
    let text = std::str::from_utf8(&file)?;
    /// Complete flat HighlightedRange iterator domain coverage with a default highlight
    struct Complete<I:Iterator> { iter: std::iter::Peekable<I>, last_end: TextSize}
    impl<I:Iterator> Complete<I> { fn new(iter: I) -> Self { Self{iter: iter.peekable(), last_end: Zero_zero()} } }
    impl<I:Iterator<Item=HighlightedRange>> Iterator for Complete<I> {
        type Item = HighlightedRange;
        fn next(&mut self) -> Option<HighlightedRange> {
            let next_start = self.iter.peek()?.range.start(); // todo: yield any last default span
            assert!(self.last_end <= next_start);
            let next =
                if self.last_end < next_start {
                    Some(HighlightedRange{
                        range:TextRange::new(self.last_end, next_start),
                        highlight: HighlightTag::Module.into() /*fixme:None*/,
                        binding_hash: None
                    })
                } else {
                    self.iter.next()
                };
            self.last_end = next.as_ref()?.range.end();
            next
        }
    }
    Complete::new({let (analysis, file_id) = ra_ide::Analysis::from_single_file(text.to_string()); analysis.highlight(file_id)?.into_iter()}).map(|HighlightedRange{range, highlight, ..}|  {
        println!("{:?} {}", highlight, &text[range]);
         //type Color = bgra8;
        //pub struct TextStyle { pub color: Color, pub style: FontStyle }
        //pub struct Span<'a> {pub text: &'a str, pub style: TextStyle }
    }).for_each(drop);
    //window(&mut Text::new(text))?
}
