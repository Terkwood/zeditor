pub enum Msg<T: Clone> {
    Event(T),
    Quit,
}
