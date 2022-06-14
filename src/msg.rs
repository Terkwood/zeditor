/// Listen for `Msg::Quit` on a channel to help break
/// out of a tight loop in a spawned thread
pub enum Msg<T: Clone> {
    Event(T),
    Quit,
}
