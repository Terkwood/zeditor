use crate::msg::Msg;
use crate::quit::quit_button;
use crate::replace::ReplaceHits;
use crate::screens::ZeditorScreens;
use crate::search::{Hit, SearchFiles};
use crate::skip::SkipRepo;
use crate::STATE;
use cursive::reexports::crossbeam_channel::Sender;
use cursive::theme::{BaseColor, Color};
use cursive::traits::*;
use cursive::utils::markup::StyledString;
use cursive::views::{
    Button, Dialog, DummyView, LastSizeView, LinearLayout, ListChild, ListView, NamedView, Panel,
    TextView,
};
use cursive::{Cursive, CursiveRunnable, CursiveRunner};
use std::sync::{Arc, Mutex};

// ListView containing many LinearLayouts , each with a TextView in first position
const FOUND: &str = "search results list view";

// this is the LastSizeView computing displayed lines
const FOUND_LASTSIZE: &str = "lastsize search results";

// Display some text with these count
const FOUND_LINES_REPORT: &str = "computed search lines report";

const FILENAME_LABEL_LENGTH: usize = 15;

const PERM_BUTTONS_SIZE: (usize, usize) = (30, 11);

pub fn render(
    siv: &mut Cursive,
    replace_hits_s: Sender<Msg<ReplaceHits>>,
    search_files_s: Sender<SearchFiles>,
    skip_repo: Arc<Mutex<SkipRepo>>,
    screens: ZeditorScreens,
) {
    siv.set_screen(screens.home);

    let found = ListView::new().with_name(FOUND);

    let found_lines = TextView::new("").with_name(FOUND_LINES_REPORT);

    let found_lastsize = LastSizeView::new(found).with_name(FOUND_LASTSIZE);

    let perm_buttons = {
        let search_s = search_files_s.clone();
        let replace_s = replace_hits_s.clone();
        let replace_s2 = replace_hits_s.clone();

        Panel::new(
            LinearLayout::vertical()
                .child(found_lines)
                .child(DummyView)
                .child(Button::new("Replace All", move |s| {
                    let visible_lines = count_visible_lines(s).unwrap_or_default();
                    let visible_hits = take_found_user_data(s, visible_lines);

                    let msg = Msg::Event(ReplaceHits(visible_hits.clone()));

                    replace_s.send(msg).expect("send")
                }))
                .child(Button::new("Search", move |_| {
                    search_s.send(SearchFiles).expect("send")
                }))
                .child(DummyView)
                .child(Button::new("Config", move |s| {
                    s.set_screen(screens.config);
                }))
                .child(DummyView)
                .child(quit_button(replace_s2)),
        )
    };

    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(found_lastsize)
                .child(DummyView)
                .child(perm_buttons.fixed_size(PERM_BUTTONS_SIZE)),
        )
        .title("zeditor"),
    );

    refresh_found_widget(siv, &replace_hits_s, skip_repo.clone());
}

fn refresh_found_widget(
    siv: &mut Cursive,
    replace_hits_s: &Sender<Msg<ReplaceHits>>,
    skip_repo: Arc<Mutex<SkipRepo>>,
) {
    if let Some(mut search_widget) = siv.find_name::<ListView>(FOUND) {
        let _ = siv.with_user_data(|state: &mut STATE| {
            search_widget.clear();
            for (hit_pos, hit) in state.0.clone().iter().enumerate() {
                let psm = skip_repo.clone();

                if !psm
                    .lock()
                    .expect("skip repo check search")
                    .contains(&hit.clone().into())
                {
                    let replace_hits_chan = replace_hits_s.clone();
                    let replace_hits_chan2 = replace_hits_s.clone();
                    let hitc = hit.clone();

                    let filename: String = hit
                        .path
                        .file_name()
                        .and_then(|o| o.to_str())
                        .unwrap_or("")
                        .trim()
                        .to_string()
                        .chars()
                        .into_iter()
                        .take(FILENAME_LABEL_LENGTH)
                        .collect();

                    let mut preview_text = StyledString::plain(&hit.preview.before);
                    preview_text.append(StyledString::styled(
                        &hit.search,
                        Color::Light(BaseColor::Magenta),
                    ));
                    preview_text.append(StyledString::plain(&hit.preview.after));

                    let linear = LinearLayout::horizontal()
                        .child(TextView::new(StyledString::styled(
                            &filename,
                            Color::Light(BaseColor::Cyan),
                        )))
                        .child(DummyView)
                        .child(TextView::new(preview_text))
                        .child(DummyView)
                        .child(Button::new("OK", move |_| {
                            replace_hits_chan
                                .send(Msg::Event(ReplaceHits(vec![hitc.clone()])))
                                .expect("send")
                        }))
                        .child(DummyView)
                        .child(Button::new("Skip", move |s| {
                            skip_candidate(s, hit_pos, &replace_hits_chan2, psm.clone())
                        }));

                    search_widget.add_child("", linear);
                }
            }
        });
    }
}

pub fn update_found_user_data(
    siv: &mut Cursive,
    results: Vec<Hit>,
    replace_hits_s: &Sender<Msg<ReplaceHits>>,
    skip_repo: Arc<Mutex<SkipRepo>>,
) {
    siv.with_user_data(|state: &mut STATE| {
        state.0.clear();
        for f in results {
            if !skip_repo
                .lock()
                .expect("skip repo mutex")
                .contains(&f.clone().into())
            {
                state.0.push(f);
            }
        }
    });

    refresh_found_widget(siv, replace_hits_s, skip_repo);
}

fn take_found_user_data(siv: &mut Cursive, until_lines: usize) -> Vec<Hit> {
    let mut out = vec![];
    siv.with_user_data(|state: &mut STATE| {
        let mut count_lines = 0;

        for hit in &state.0 {
            count_lines += hit.preview.as_text(&hit.search).lines().count();
            if count_lines >= until_lines {
                break;
            }

            out.push(hit.clone());
        }
    });

    out
}

fn skip_candidate(
    siv: &mut Cursive,
    user_data_pos: usize,
    replace_hits_s: &Sender<Msg<ReplaceHits>>,
    skip_repo: Arc<Mutex<SkipRepo>>,
) {
    siv.with_user_data(|state: &mut STATE| {
        let hit = state.0.remove(user_data_pos);

        skip_repo.lock().expect("skip repo lock").add(hit.into())
    });
    refresh_found_widget(siv, replace_hits_s, skip_repo);
}

pub fn count_visible_lines(siv: &mut Cursive) -> Option<usize> {
    if let Some(fl) = siv.find_name::<LastSizeView<NamedView<ListView>>>(FOUND_LASTSIZE) {
        Some(fl.size.y)
    } else {
        None
    }
}

/// count the number of entries found in app state
fn count_found(siv: &mut Cursive) -> usize {
    if let Some(found) = siv.find_name::<ListView>(FOUND) {
        let mut count = 0;
        for c in found.children() {
            count += match c {
                ListChild::Delimiter => 0,
                ListChild::Row(_, _) => 1,
            };
        }

        count
    } else {
        0
    }
}

pub fn update_report_widgets(siv: &mut CursiveRunner<CursiveRunnable>) {
    let found_count = count_found(siv);

    // update hacky count widget
    if let Some(mut found_count_report) = siv.find_name::<TextView>(FOUND_LINES_REPORT) {
        found_count_report.set_content(format!("Found: {}", found_count));
    }

    // without this you'll lag behind by a step
    siv.refresh();
}
