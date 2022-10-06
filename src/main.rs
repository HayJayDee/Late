mod editor;
mod terminal;
mod document;
mod size;
mod markup;

fn main() -> Result<(), std::io::Error> {
    let terminal: crate::terminal::Terminal = crate::terminal::Terminal::default()?; // TODO: Move terminal to Late
    editor::Editor::default(crate::size::Size {width: terminal.size().width, height: terminal.size().height}).run();
    Ok(())
}
