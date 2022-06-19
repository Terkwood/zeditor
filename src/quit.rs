use cursive::{reexports::crossbeam_channel::Sender, views::Button, Cursive};

use crate::{msg::Msg, replace::ReplaceCommand, search::SearchCommand};

pub fn quit_button(
    replace: Sender<Msg<ReplaceCommand>>,
    search: Sender<Msg<SearchCommand>>,
) -> Button {
    Button::new("Quit", move |s| {
        replace.send(Msg::Quit).expect("send");
        search.send(Msg::Quit).expect("send");

        Cursive::quit(s)
    })
}
