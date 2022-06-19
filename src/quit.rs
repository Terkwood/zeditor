use cursive::{reexports::crossbeam_channel::Sender, views::Button, Cursive};

use crate::{msg::Msg, replace::ReplaceHits};

pub fn quit_button(replace: Sender<Msg<ReplaceHits>>) -> Button {
    Button::new("Quit", move |s| {
        replace.send(Msg::Quit).expect("send");

        Cursive::quit(s)
    })
}
