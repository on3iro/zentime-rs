pub enum InputEvent<I> {
    Input(I),
}

pub struct ViewState {
    pub is_break: bool,
    pub round: u64,
    pub time: String,
}

pub enum TerminalEvent {
    View(ViewState),
    Quit,
}

pub enum AppAction {
    Quit,
    None,
    PlayPause,
}
