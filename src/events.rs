pub enum InputEvent<I> {
    Input(I),
}

pub enum TerminalEvent {
    View(String),
    Quit,
}

pub enum AppAction {
    Quit,
    None,
    PlayPause,
}
