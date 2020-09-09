use {std::path::{Path, PathBuf}, fehler::throws, error::Error,
				ui::{time, text::{self, unicode_segmentation::{index, find},Attribute,Style,View,LineColumn,Span,default_font, default_style},
				widget::{size, Target, EventContext, ModifiersState, Event, Widget},
				edit::{Borrowed,Cow,Scroll,Edit,Change}, app::run}};

#[throws] fn buffer(path: &Path) -> ui::edit::Owned {
	let text = String::from_utf8(std::fs::read(path)?)?;
	use {rust::{HighlightedRange, HighlightTag, HighlightModifier}, ui::text::FontStyle, ui::color::bgr};
	pub fn style<'t>(text: &'t str, highlight: impl Iterator<Item=HighlightedRange>+'t) -> impl Iterator<Item=Attribute<Style>> + 't {
		highlight.map(move |HighlightedRange{range, highlight, ..}| {
			Attribute{
				range: find(text, range.start as usize).unwrap()..find(text, range.end as usize).unwrap(),
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
	let style = style(&text, time!(rust::highlight(path))?.into_iter()).collect::<Vec::<_>>();
	ui::edit::Owned{text, style}
}

#[track_caller] fn from_index(text: &str, byte_index: usize) -> LineColumn { LineColumn::from_text_index(text, find(text, byte_index).unwrap()).unwrap() }
fn from(text: &str, range: rust::TextRange) -> Span { Span{start: from_index(text, range.start as usize), end: from_index(text, range.end as usize)} }

#[derive(derive_more::Deref)] struct Editor<'f, 't>{path: std::path::PathBuf, #[deref] scroll: Scroll<'f,'t>}
impl Editor<'_, '_> {
	fn event(&mut self, size: size, event_context: &EventContext, event: &Event) -> Change {
		let change = self.scroll.event(size, event_context, event);
		if let Change::Insert|Change::Remove|Change::Other = change {
				let Self{path, scroll: Scroll{edit: Edit{view, ..}, ..}} = self;
				std::fs::write(&path, view.text().as_bytes()).unwrap();
		}
		change
	}
}
impl Widget for Editor<'_, '_> {
	fn size(&mut self, size: size) -> size { self.scroll.size(size) }
	#[throws] fn paint(&mut self, target: &mut Target) { self.scroll.paint(target)? }
	#[throws] fn event(&mut self, size: size, event_context: &EventContext, event: &Event) -> bool { if self.event(size, event_context, event) != Change::None { true } else { false }  }
}

struct CodeEditor<'f, 't>{
	editor: Editor<'f, 't>,
	diagnostics: Vec<rust::Diagnostic>,
	message: Option<String>,
	args: Vec<String>,
	/*selection/browse_*/history: Vec<(PathBuf, Span)>,
}
impl CodeEditor<'_, '_> {
	#[throws] fn update(&mut self) {
		let Self{editor: Editor{path, scroll: Scroll{edit: Edit{view, ..}, ..}}, diagnostics, ..} = self;
		view.size = None;
		view.data = Cow::Owned(self::buffer(path)?);
		*diagnostics = time!(rust::diagnostics(path))?;
		self.message = diagnostics.first().map(|rust::Diagnostic{message, ..}| message.clone());
	}
	#[throws] fn view(&mut self, path: PathBuf) {
		self.editor.path = path;
		self.update()?
	}
}

impl Widget for CodeEditor<'_, '_> {
	fn size(&mut self, size: size) -> size { time!(self.editor.size(size)) }
	#[throws] fn paint(&mut self, target: &mut Target) {
		let Self{editor: Editor{scroll, ..}, diagnostics, message, ..} = self;
		let scale = scroll.paint_fit(target);
		let Scroll{edit: Edit{view, selection, ..}, offset} = scroll;
		for rust::Diagnostic{range, ..} in diagnostics.iter() { view.paint_span(target, scale, *offset, from(view.text(), *range), ui::color::bgr{b: false, g: false, r: true}); }
		view.paint_span(target, scale, *offset, *selection, ui::color::bgr{b: true, g: true, r: true});
		if let Some(message) = message {
			let mut view = View{font: &default_font, data: 	Borrowed{text: message, style: &default_style}, size: None};
			let size = text::fit(target.size, view.size());
			Widget::paint(&mut view, &mut target.rows_mut(target.size.y-size.y..target.size.y))?;
		}
	}
	#[throws] fn event(&mut self, size: size, event_context: &EventContext, event: &Event) -> bool {
		match self.editor.event(size, event_context, event) {
			Change::Cursor|Change::Scroll => true,
			Change::Insert|Change::Remove|Change::Other => {
				self.update()?;
				true
			}
			Change::None => {
				let EventContext{modifiers_state: ModifiersState{alt,..}, ..} = event_context;
				let Self{editor: Editor{path, scroll: Scroll{edit: Edit{view, selection, ..}, ..}}, diagnostics, args, history, ..} = self;
				let text = view.text();
				match event {
					Event::Key{key:'←'} if *alt => {
						if let Some((path, span)) = history.pop() {
							self.view(path)?;
							let scroll = &mut self.editor.scroll;
							scroll.edit.selection = span;
							scroll.keep_selection_in_view(size);
						}
						true
					},
					Event::Key{key:'→'} if *alt => {
						if let Some(target) = rust::definition(path, index(text, text::index(text, selection.end)))? {
							history.push((path.clone(), *selection));
							let rust::NavigationTarget{path, range: rust::TextRange{start,..}, ..} = target;
							self.view(path)?;
							let span = Span::new(from_index(self.editor.view.text(), start as usize));
							let scroll = &mut self.editor.scroll;
							scroll.edit.selection = span;
							scroll.keep_selection_in_view(size);
						}
						true
					},
					Event::Key{key:'⎙'} => {
						if let Some(rust::Diagnostic{range, ..}) = diagnostics.first() { *selection = from(text, *range); }
						else if let Some(cargo::Diagnostic{message, spans, ..}, ..) = cargo::build(args)? {
							let cargo::Span{file_name, line_start, column_start, line_end, column_end, ..} = spans.into_iter().next().unwrap();
							self.view(file_name.into())?;
							self.message = Some(message);
							let scroll = &mut self.editor.scroll;
							scroll.edit.selection = Span{start:LineColumn{line:line_start-1, column:column_start-1}, end:LineColumn{line:line_end-1, column:column_end-1}};
							scroll.keep_selection_in_view(size);
						} else {
							self.message = None;
							std::process::Command::new("cargo").arg("run").spawn()?; // todo: stdout → message
						}
						true
					},
					_ => false
				}
			}
		}
	}
}

#[throws] fn main() {
	#[cfg(feature="trace")] trace::sigint();
	let mut args = std::env::args().skip(1);
	let path : std::path::PathBuf = args.next().map(|a| a.into()).unwrap_or(std::env::current_dir()?);
	if let Some(project) = path.ancestors().find(|p| p.join("Cargo.toml").is_file()) {
		std::env::set_current_dir(project)?;
		let path = if path.is_file() { path } else { "src/main.rs".into() };
		let scroll = Scroll::new(Edit::new(&default_font, Cow::Owned(buffer(&path)?)));
		let diagnostics = rust::diagnostics(&path)?;
		run(CodeEditor{editor: Editor{path, scroll}, diagnostics, message: None, args: args.collect(), history: Vec::new()})?
	} else {
		let text = std::fs::read(&path)?;
		run(Editor{path, scroll: Scroll::new(Edit::new(&default_font, Cow::Borrowed(Borrowed{text: &std::str::from_utf8(&text)?, style: &default_style})))})?
	}
}
