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
	ui::app::run(ui::edit::Edit::new(&ui::text::default_font, ui::edit::Cow::Borrowed(ui::edit::Buffer{text: &text, style: &style(highlight.into_iter()).collect::<Vec::<_>>()})))?
}
