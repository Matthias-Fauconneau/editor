use {rust_analyzer::*, framework::{color::bgr, *}};

pub type Color = bgr;
pub enum FontStyle { Normal, Bold, /*Italic, BoldItalic*/ }
pub struct TextStyle { pub color: Color, pub style: FontStyle }
pub struct Span {pub range: TextRange, pub style: TextStyle }

pub fn style(highlight: impl Iterator<Item=HighlightedRange>) -> impl Iterator<Item=Span> {
    highlight.complete().map(|HighlightedRange{range, highlight, ..}| {
        //println!("{:?} {}", highlight, &text[range]);
        use HighlightTag::*;
        Span{range, style: TextStyle{
            color: match highlight.tag {
                Function|Macro => bgr{b:1./2.,r:1.,g:1.},
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
    })
}

#[throws]
fn main() {
    let start = std::time::Instant::now();
    let (host, packages) = load_cargo(&std::env::current_dir()?, false)?;
    let workspace_packages = packages.iter().filter(|(_,package)| package.is_member() ).collect::<Vec<_>>();
    let files = |(&id,_)| host.raw_database().source_root(id).walk().collect::<Vec<_>>();
    let file_id = files(workspace_packages[0])[0];
    let analysis = host.analysis();
    let text = analysis.file_text(file_id)?;
    for Span{range, style} in style(analysis.highlight(file_id)?.into_iter()) {
        fn print(text: &str, TextStyle{color, style}: TextStyle) {
            let code = match style {
                FontStyle::Normal => 31,
                FontStyle::Bold => 1,
                //_ => 31
            };
            let bgra8{b,g,r,..} = color.into();
            print!("\x1b[{}m\x1b[38;2;{};{};{}m{}\x1b(B\x1b[m",code, r,g,b, text)
        }
        print(&text[range], style);
    }
    eprintln!("{:?}\n", start.elapsed());
    //window(&mut Text::new(text))?
}
