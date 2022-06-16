use std::sync::{Arc, Mutex};

use crate::{db::Db, screens::ZeditorScreens};
use cursive::{
    traits::*,
    views::{Button, Dialog, DummyView, LinearLayout, ListView, ScrollView, TextArea, TextView},
    Cursive,
};

const INPUTS: &str = "configurable search and replace inputs";

pub fn render(siv: &mut Cursive, screens: ZeditorScreens, db: Arc<Mutex<Db>>) {
    siv.set_screen(screens.config);

    use cursive::utils::markup::StyledString;

    let mut existing_search_inputs = ListView::new();
    let mut replace_inputs = ListView::new();

    if let Ok(sr) = db.lock().unwrap().get_search_replace() {
        for (search, replace) in sr {
            existing_search_inputs.add_child("", TextView::new(search));

            replace_inputs.add_child("", {
                let mut rta = TextArea::new();
                rta.set_content(replace);
                rta
            });
        }
    }

    let inputs_with_header = ScrollView::new(
        LinearLayout::horizontal()
            .child(
                LinearLayout::vertical()
                    .child(TextView::new(StyledString::styled(
                        "SEARCH",
                        cursive::theme::Effect::Bold,
                    )))
                    .child(existing_search_inputs),
            )
            .child(
                LinearLayout::vertical()
                    .child(TextView::new(StyledString::styled(
                        "REPLACE",
                        cursive::theme::Effect::Bold,
                    )))
                    .child(replace_inputs),
            ),
    );

    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(inputs_with_header)
                .child(DummyView)
                .child(Button::new("Home", move |s| {
                    s.set_screen(screens.home);
                })),
        )
        .title("zeditor"),
    );
}
