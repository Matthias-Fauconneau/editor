use {fehler::throws, error::Error, ui::{text::{Attribute,Style,FontStyle}, color::bgr}, highlight::{HighlightedRange, HighlightTag, HighlightModifier, TextHighlight}};

pub fn style(highlight: impl Iterator<Item=HighlightedRange>) -> impl Iterator<Item=Attribute<Style>> {
	highlight.map(|HighlightedRange{range, highlight, ..}| {
		Attribute{
			range: range.start().into()..range.end().into(),
			attribute: Style{
				color: {use HighlightTag::*; match highlight.tag {
					Module => bgr{b: 0., r: 1., g: 1./3.},
					Keyword if !highlight.modifiers.iter().any(|it| it == HighlightModifier::ControlFlow) => bgr{b: 2./3.,r: 2./3.,g: 2./3.},
					Function|Macro => bgr{b: 2./3., r: 1., g: 2./3.},
					Struct|TypeAlias|BuiltinType|TypeParam|Enum => bgr{b: 2./3., r: 0., g: 2./3.},
					Field => bgr{b: 0., r: 0.,g: 2./3.},
					Trait => bgr{b: 1., r: 1./2., g: 1.},
					StringLiteral|NumericLiteral|EnumVariant => bgr{b: 0., r: 1., g: 1./3.},
					Lifetime|Attribute => bgr{b: 1., r: 1./3., g: 1./3.},
					Comment => bgr{b: 1./2., r: 1./2., g: 1./2.},
					_ => bgr{b: 1., r: 1., g: 1.},
				}},
				style:
					if highlight.modifiers.iter().any(|it| it == HighlightModifier::ControlFlow) { FontStyle::Bold }
					else {
						{use HighlightTag::*; match highlight.tag {
								Keyword => FontStyle::Bold, // fixme: Italic
								_ => FontStyle::Normal
						}}
					}
			}
		}
	})
}

#[throws] fn main() {
	let TextHighlight{text, highlight} = highlight::highlight()?;
	/*fn print(span: &str, Style{color, style}: Style) {
		let code = match style {
			FontStyle::Normal => 31,
			FontStyle::Bold => 1,
		};
		let ui::color::bgra8{b,g,r,..} = color.into();
		print!("\x1b[{}m\x1b[38;2;{};{};{}m{}\x1b(B\x1b[m",code, r,g,b, span)
	}
	let (mut current, mut attributes) = (None, style.iter().peekable());
	for (index, span) in text.chars().enumerate() {
		use iter::{PeekableExt, Single};
		current = current.filter(|a:&&Attribute<Style>| a.contains(&(index as u32))).or_else(|| attributes.peeking_take_while(|a| a.contains(&(index as u32))).single());
		print(&span.to_string(), current.map(|c| c.attribute).unwrap_or_default());
	}
	println!();*/
	ui::app::run(ui::edit::Edit::new(&ui::text::default_font, ui::edit::Cow::Borrowed(ui::edit::Buffer{text: &text, style: &style(highlight.into_iter()).collect::<Vec::<_>>()})))?
}
