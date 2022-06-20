use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    db::Db, msg::Msg, quit::quit_button, replace::ReplaceCommand, screens::ZeditorScreens,
    search::SearchCommand,
};
use cursive::{
    event::Event,
    reexports::crossbeam_channel::Sender,
    traits::*,
    views::{
        Button, Dialog, DummyView, LinearLayout, ListView, OnEventView, ScrollView, TextArea,
        TextView,
    },
    Cursive,
};

const EXISTING_SEARCH_INPUTS: &str = "existing search inputs";
const EXISTING_REPLACE_INPUTS: &str = "existing replace inputs";
const NEW_SEARCH_INPUT: &str = "new search input";
const NEW_REPLACE_INPUT: &str = "new replace input";

pub fn render(
    siv: &mut Cursive,
    screens: ZeditorScreens,
    db: Arc<Mutex<Db>>,
    replace_s: Sender<Msg<ReplaceCommand>>,
    search_s: Sender<Msg<SearchCommand>>,
) {
    siv.set_screen(screens.config);

    use cursive::utils::markup::StyledString;

    let existing_search_inputs = ListView::new().with_name(EXISTING_SEARCH_INPUTS);
    let existing_replace_inputs = ListView::new().with_name(EXISTING_REPLACE_INPUTS);
    let new_search_input = TextArea::new().with_name(NEW_SEARCH_INPUT);
    let db2 = db.clone();
    let scs2 = search_s.clone();
    let rs2 = replace_s.clone();
    let new_replace_input = OnEventView::new(TextArea::new().with_name(NEW_REPLACE_INPUT))
        .on_event(Event::FocusLost, move |s| {
            let mut nri = s.find_name::<TextArea>(NEW_SEARCH_INPUT).unwrap();
            let mut nsi = s.find_name::<TextArea>(NEW_REPLACE_INPUT).unwrap();

            // avoid borrow checker error
            let nri_text = nri.get_content();
            let search_2 = nri_text.to_string();

            match nri_text {
                search => match nsi.get_content() {
                    replace => {
                        let db = db2.lock().unwrap();

                        // never write empty replace term
                        if !replace.trim().is_empty() {
                            db.upsert_search_replace(&search, replace)
                                .expect("upsert search replace");

                            if let Ok(sr) = db.get_search_replace() {
                                update_search_inputs(s, &sr);
                                update_replace_inputs(s, &sr, db2.clone());
                            } else {
                                eprintln!("failed db get search and replace in entry")
                            }

                            nri.set_content("");
                            nsi.set_content("");
                            search_s
                                .send(SearchCommand::RecompileSearch(search_2).into())
                                .expect("send search command");
                            replace_s
                                .send(ReplaceCommand::RefreshSearchReplace.into())
                                .expect("send replace command");
                        }
                    }
                },
            }
        });

    let inputs_with_header = ScrollView::new(
        LinearLayout::horizontal()
            .child(
                LinearLayout::vertical()
                    .child(TextView::new(StyledString::styled(
                        "SEARCH",
                        cursive::theme::Effect::Bold,
                    )))
                    .child(new_search_input)
                    .child(existing_search_inputs),
            )
            .child(
                LinearLayout::vertical()
                    .child(TextView::new(StyledString::styled(
                        "REPLACE",
                        cursive::theme::Effect::Bold,
                    )))
                    .child(new_replace_input)
                    .child(existing_replace_inputs),
            ),
    );

    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(inputs_with_header)
                .child(DummyView)
                .child(
                    LinearLayout::vertical()
                        .child(Button::new("Home", move |s| {
                            s.set_screen(screens.home);
                        }))
                        .child(DummyView)
                        .child(quit_button(rs2, scs2)),
                ),
        )
        .title("zeditor"),
    );

    if let Ok(sr) = db.lock().unwrap().get_search_replace() {
        update_search_inputs(siv, &sr);
        update_replace_inputs(siv, &sr, db.clone());
    }
}

pub fn update_search_inputs(siv: &mut Cursive, sr: &HashMap<String, String>) {
    let mut inputs = siv.find_name::<ListView>(EXISTING_SEARCH_INPUTS).unwrap();
    inputs.clear();

    for (key, _) in sr {
        inputs.add_child("", TextView::new(key));
    }
}

pub fn update_replace_inputs(
    siv: &mut Cursive,
    search_replace: &HashMap<String, String>,
    db: Arc<Mutex<Db>>,
) {
    let mut replace_inputs = siv.find_name::<ListView>(EXISTING_REPLACE_INPUTS).unwrap();
    replace_inputs.clear();
    for (search, replace) in search_replace.clone() {
        let cdb = db.clone();

        let replace2 = replace.clone();
        let search2 = search.clone();
        replace_inputs.add_child(
            "",
            OnEventView::new({
                let mut rta = TextArea::new();
                rta.set_content(replace);
                rta
            })
            .on_event(Event::FocusLost, move |_| {
                let db = cdb.lock().unwrap();

                // never write empty replace term
                if !replace2.trim().is_empty() {
                    db.upsert_search_replace(&search2, &replace2)
                        .expect("upsert search replace");

                    todo!();
                }

                todo!()
            }),
        );
    }
}
