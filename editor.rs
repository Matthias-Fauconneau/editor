use {fehler::throws, error::{Error, Context, Ok}, std::path::{Path, PathBuf},
				ui::{text::{self, unicode_segmentation::{index, find},Attribute,Style,bgr,FontStyle,View,Borrowed,LineColumn,Span,default_font},
				widget::{size, Target, EventContext, ModifiersState, Event, Widget},
				edit::{Owned,Cow,Scroll,Edit,Change}, app::run}};

#[throws] fn buffer(path: &Path) -> Owned {
	let text = String::from_utf8(std::fs::read(path).context(path.to_str().unwrap().to_owned())?)?;
	use rust::{HighlightedRange, HighlightTag, SymbolKind, HighlightModifier};
	pub fn style<'t>(text: &'t str, highlight: impl Iterator<Item=HighlightedRange>+'t) -> impl Iterator<Item=Result<Attribute<Style>, Error>> + 't {
		highlight.map(move |HighlightedRange{range, highlight, ..}| Ok(
			Attribute{
				range: find(text, range.start as usize).ok()?..find(text, range.end as usize).ok()?,
				attribute: Style{
					color: {use {HighlightTag::*, SymbolKind::*}; match highlight.tag {
						Symbol(Module) => bgr{b: 1./3., r: 1./3., g: 2./3.},
						Keyword => { if !highlight.mods.iter().any(|it| it == HighlightModifier::ControlFlow) { bgr{b: 2./3.,r: 2./3.,g: 2./3.} } else { bgr{b: 0.,r: 1.,g: 1.} } },
						Symbol(Function|Macro) => bgr{b: 2./3., r: 1., g: 2./3.},
						Symbol(Struct|TypeAlias|TypeParam|Enum)|BuiltinType => bgr{b: 2./3., r: 0., g: 2./3.},
						Symbol(Field) => bgr{b: 0., r: 0.,g: 2./3.},
						Symbol(Trait) => bgr{b: 1., r: 1./2., g: 1.},
						StringLiteral|NumericLiteral|Symbol(Variant) => bgr{b: 0., r: 1., g: 1./3.},
						Symbol(LifetimeParam)|Attribute => bgr{b: 1., r: 1./3., g: 1./3.},
						Comment => bgr{b: 1./2., r: 1./2., g: 1./2.},
						_ => bgr{b: 1., r: 1., g: 1.},
					}},
					style:
						if highlight.mods.iter().any(|it| it == HighlightModifier::ControlFlow) { FontStyle::Bold }
						else {
							{use HighlightTag::*; match highlight.tag {
									Keyword => FontStyle::Bold, // fixme: Italic
									_ => FontStyle::Normal
							}}
						}
				}
			}
		))
	}
	let style = style(&text, rust::highlight(path)?.into_iter()).collect::<Result<_, _>>()?;
	//let style = style(&text, trace::timeout_(100, || rust::highlight(path), path.display().to_string())??.into_iter()).collect::<Result::<_,_>>()?;
	Owned{text, style}
}

#[track_caller] fn from_index(text: &str, byte_index: usize) -> LineColumn { LineColumn::from_text_index(text, find(text, byte_index).unwrap()).unwrap() }
fn from(text: &str, range: rust::TextRange) -> Span { Span{start: from_index(text, range.start as usize), end: from_index(text, range.end as usize)} }

#[derive(derive_more::Deref)] struct Editor<'f, 't>{
	path: std::path::PathBuf,
	#[deref] scroll: Scroll<'f,'t>
}
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
	#[throws] fn paint(&mut self, target: &mut Target) { target.fill(0.into()); self.scroll.paint(target)? }
	#[throws] fn event(&mut self, size: size, event_context: &EventContext, event: &Event) -> bool {
		if self.event(size, event_context, event) != Change::None { true } else { false }
	}
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
		*diagnostics = rust::diagnostics(path)?;
		self.message = diagnostics.first().map(|rust::Diagnostic{message, ..}| message.clone());
	}
	#[throws] fn view(&mut self, path: PathBuf) {
		self.editor.path = path;
		self.update()?
	}
}

impl Widget for CodeEditor<'_, '_> {
	fn size(&mut self, size: size) -> size { self.editor.size(size) }
	#[throws] fn paint(&mut self, target: &mut Target) {
		target.fill(0.into());
		let Self{editor: Editor{scroll, ..}, diagnostics, message, ..} = self;
		let scale = scroll.paint_fit(target);
		let Scroll{edit: Edit{view, selection, ..}, offset} = scroll;
		for rust::Diagnostic{range, ..} in diagnostics.iter() { 
			view.paint_span(target, scale, *offset, from(view.text(), *range), ui::color::bgr{b: false, g: false, r: true}); 
		}
		view.paint_span(target, scale, *offset, *selection, ui::color::bgr{b: true, g: true, r: true});
		if let Some(message) = message {
			let mut view = View::new(Borrowed::new(message));
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
				let Self{editor: Editor{path, scroll: Scroll{edit: Edit{view, selection, ..}, ..}}, diagnostics, ref args, history, ..} = self;
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
						else if let Err(cargo::Diagnostic{message, spans, ..}, ..) = cargo::build(args)? {
							let cargo::Span{file_name, line_start, column_start, line_end, column_end, ..} = spans.into_iter().next().unwrap();
							self.view(file_name.into())?;
							self.message = Some(message);
							let scroll = &mut self.editor.scroll;
							scroll.edit.selection = Span{start:LineColumn{line:line_start-1, column:column_start-1}, end:LineColumn{line:line_end-1, column:column_end-1}};
							scroll.keep_selection_in_view(size);
						} else {
							self.message = None;
							std::process::Command::new("cargo").args(args).arg("run").spawn()?; // todo: stdout → message
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
	#[cfg(feature="trace")] { trace::rstack_self().unwrap(); trace::signal_interrupt()?; }
	let mut args = std::env::args().skip(1);
	let path : std::path::PathBuf = args.next().map(|a| a.into()).unwrap_or(std::env::current_dir()?);
	if let Some(project) = path.canonicalize()?.ancestors().find(|p| p.join("Cargo.toml").is_file()) {
		std::env::set_current_dir(project)?;
		let path =
			if path.is_file() { path } else {
				["main.rs","src/main.rs"].iter().map(|path| project.join(path)).filter(|path| path.exists()).next()
				.unwrap_or_else(|| cargo::parse(&project.join("Cargo.toml")).unwrap().into())
			};
		let scroll = Scroll::new(Edit::new(default_font(), Cow::Owned(buffer(&path)?)));
		let diagnostics = rust::diagnostics(&path)?;
		run(CodeEditor{editor: Editor{path, scroll}, diagnostics, message: None, args: args.collect(), history: Vec::new()})?
	} else {
		let text = std::fs::read(&path)?;
		run(Editor{path, scroll: Scroll::new(Edit::new(default_font(), Cow::new(&std::str::from_utf8(&text)?)))})?
	}
}
