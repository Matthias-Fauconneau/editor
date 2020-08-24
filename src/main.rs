use {std::path::Path, fehler::throws, error::Error,
				ui::{text::{self, unicode_segmentation::{index, find},Attribute,Style,View,LineColumn,Span}, widget::{size, Target, EventContext, ModifiersState, Event, Widget}, edit::{Edit,Change}}};

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
			for rust::Diagnostic{range, ..} in diagnostics { view.paint_span(target, scale, from(text, *range)); }
			view.paint_span(target, scale, *selection);
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
					let Self{path, edit:Edit{view: View{data, ..}, selection, ..}, ..} = self;
					let text = AsRef::<str>::as_ref(&data);
					let EventContext{modifiers_state: ModifiersState{alt,..}, ..} = event_context;
					match event {
						Event::Key{key:'→'} if *alt => {
							let target = dbg!(rust::definition(path, index(text, text::index(text, selection.end))))?;
							if let Some(target) = target { *selection = from(text, target.range); }
							true
						},
						_ => false
					}
				}
			}
		}
	}
	ui::app::run(Editor{path, edit: ui::edit::Edit::new(&ui::text::default_font, ui::edit::Cow::Owned(buffer)), diagnostics: Vec::new()})?
}
