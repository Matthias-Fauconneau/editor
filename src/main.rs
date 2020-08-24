use {std::path::Path, fehler::throws, error::Error,
				ui::{text::{self, unicode_segmentation::{index, find},Attribute,Style,View,LineColumn,Span,default_font, default_style},
				widget::{size, Target, EventContext, ModifiersState, Event, Widget},
				edit::{Cow,Edit,Change}, app::run}};

#[throws] fn buffer(path: &Path) -> ui::edit::Owned {
	let text = std::str::from_utf8(&std::fs::read(path)?)?.to_owned();
	use {rust::{HighlightedRange, HighlightTag, HighlightModifier}, ui::text::FontStyle, ui::color::bgr};
	pub fn style<'t>(text: &'t str, highlight: impl Iterator<Item=HighlightedRange>+'t) -> impl Iterator<Item=Attribute<Style>> + 't {
		highlight.map(move |HighlightedRange{range, highlight, ..}| {
			Attribute{
				range: find(text, range.start as usize)..find(text, range.end as usize),
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
	let style = style(&text, rust::highlight(path)?.into_iter()).collect::<Vec::<_>>();
	ui::edit::Owned{text, style}
}

fn from_index(text: &str, byte_index: usize) -> LineColumn { LineColumn::from_text_index(text, find(text, byte_index)).unwrap() }
fn from(text: &str, range: rust::TextRange) -> Span { Span{start: from_index(text, range.start as usize), end: from_index(text, range.end as usize)} }

#[throws] fn main() {
	let path = Path::new("src/main.rs");
	let buffer = buffer(path)?;
	struct Editor<'t>{path: &'t Path, edit: Edit<'t,'t>, diagnostics: Vec<rust::Diagnostic>};
	impl Widget for Editor<'_> {
		fn size(&mut self, size: size) -> size { self.edit.size(size) }
    #[throws] fn paint(&mut self, target: &mut Target) {
			let Self{edit: Edit{view, selection, ..}, diagnostics, ..} = self;
			let scale = view.scale(target.size);
			view.paint(target, scale);
			let text = AsRef::<str>::as_ref(&view.data);
			for rust::Diagnostic{range, ..} in diagnostics.iter() { view.paint_span(target, scale, from(text, *range), ui::color::bgr{b: false, g: false, r: true}); }
			view.paint_span(target, scale, *selection, ui::color::bgr{b: true, g: false, r: false});
			if let Some(rust::Diagnostic{message, ..}) = diagnostics.first() {
				let mut view = View{font: &default_font, data: 	ui::edit::Borrowed{text: message, style: &default_style}};
				let size = text::fit(target.size, view.size());
				Widget::paint(&mut view, &mut target.rows_mut(target.size.y-size.y..target.size.y))?;
			}
		}
    #[throws] fn event(&mut self, size: size, event_context: &EventContext, event: &Event) -> bool {
			use Change::*;
			match self.edit.event(size, event_context, event) {
				Cursor => true,
				Insert|Remove|Other => {
					let Self{path, edit: Edit{view: View{data, ..}, ..}, diagnostics} = self;
					let text = AsRef::<str>::as_ref(&data);
					std::fs::write(&path, text.as_bytes()).unwrap();
					data.get_mut().style = self::buffer(path).unwrap().style;
					*diagnostics = rust::diagnostics(path)?;
					//println!("{:?}", self.diagnostics);
					true
				}
				None => {
					let Self{path, edit:Edit{view: View{data, ..}, selection, ..}, diagnostics} = self;
					let text = AsRef::<str>::as_ref(&data);
					let EventContext{modifiers_state: ModifiersState{alt,..}, ..} = event_context;
					match event {
						Event::Key{key:'→'} if *alt => {
							if let Some(rust::NavigationTarget{range, ..}) = dbg!(rust::definition(path, index(text, text::index(text, selection.end))))? { *selection = from(text, range); }
							true
						},
						Event::Key{key:'⎙'} => {
							if let Some(rust::Diagnostic{range, ..}) = diagnostics.first() { *selection = from(text, *range); true } else { false }
						},
						_ => false
					}
				}
			}
		}
	}
	run(Editor{path, edit: Edit::new(&default_font, Cow::Owned(buffer)), diagnostics: rust::diagnostics(path)?})?
}
