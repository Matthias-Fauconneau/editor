#![allow(dead_code)] // vfs_glob::exclude
use {anyhow::Error, fehler::throws};
/*pub*/ use ra_db::SourceDatabaseExt; // source_root
use ra_syntax::TextUnit as TextSize;
/*impl Zero for TextSize { fn zero() -> Self { TextSize::zero() } }*/ #[allow(non_snake_case)] fn Zero_zero() -> TextSize { TextSize::/*zero()*/from_usize(0) }
pub use ra_syntax::TextRange;
mod vfs_glob;
mod load_cargo; /*pub*/ use load_cargo::load_cargo;
pub use ra_ide::{HighlightTag, HighlightedRange};

/// Complete flat HighlightedRange iterator domain coverage with a default highlight
pub struct Complete<I:Iterator> { iter: std::iter::Peekable<I>, last_end: TextSize}
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
                    range:TextRange::/*new*/from_to(self.last_end, next_start),
                    highlight: HighlightTag::None.into(),
                    binding_hash: None
                })
            } else {
                self.iter.next()
            };
        self.last_end = next.as_ref()?.range.end();
        next
    }
}
pub trait IntoComplete : Iterator<Item=HighlightedRange>+Sized { fn complete(self) -> Complete<Self> { Complete::new(self) } }
impl<T:Iterator<Item=HighlightedRange>> IntoComplete for T {}

pub struct TextHighlight { pub text: std::sync::Arc<String>, pub highlight: Vec<HighlightedRange> }
#[throws]
pub fn highlight() -> TextHighlight {
    let (host, packages) = load_cargo(&std::env::current_dir()?, false)?;
    let workspace_packages = packages.iter().filter(|(_,package)| package.is_member() ).collect::<Vec<_>>();
    let files = |(&id,_)| host.raw_database().source_root(id).walk().collect::<Vec<_>>();
    let file_id = files(workspace_packages[0])[0];
    let analysis = host.analysis();
    TextHighlight{
        text: analysis.file_text(file_id)?,
        highlight: {
            let start = std::time::Instant::now();
            eprint!("analysis.highlight(file_id)?: ");
            let value = analysis.highlight(file_id)?;
            eprintln!("{:?}\n", start.elapsed());
            value
        }
    }
}
