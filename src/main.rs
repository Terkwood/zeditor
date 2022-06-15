use cursive::reexports::crossbeam_channel::{unbounded, Sender};
use cursive::theme::{Color, PaletteColor, Theme};
use cursive::traits::*;
use cursive::views::{
    Button, Dialog, DummyView, LastSizeView, LinearLayout, ListChild, ListView, NamedView, Panel,
    TextView,
};
use cursive::{Cursive, CursiveRunnable, CursiveRunner};
use std::sync::{Arc, Mutex};
use zeditor::db::Db;
use zeditor::msg::Msg;
use zeditor::replace::{HitsReplaced, ReplaceHits};
use zeditor::search::{Hit, SearchFiles};
use zeditor::skip::SkipRepo;

// ListView containing many LinearLayouts , each with a TextView in first position
const FOUND: &str = "search results list view";

// this is the LastSizeView computing displayed lines
const FOUND_LASTSIZE: &str = "lastsize search results";

// Display some text with these count
const FOUND_LINES_REPORT: &str = "computed search lines report";

const FILENAME_LABEL_LENGTH: usize = 15;

const PERM_BUTTONS_SIZE: (usize, usize) = (30, 11);

struct STATE(pub Vec<Hit>);

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(Db::new().expect("open db conn")));
    let db2 = db.clone();

    let skip_repo = Arc::new(Mutex::new(SkipRepo::new(db.clone())));

    let (search_files_s, search_files_r) = unbounded::<zeditor::search::SearchFiles>();
    let (files_searched_s, files_searched_r) = unbounded::<Vec<zeditor::search::Hit>>();

    tokio::spawn(async move { zeditor::search::run(db, files_searched_s, search_files_r).await });

    let (replace_hits_s, replace_hits_r) = unbounded::<Msg<ReplaceHits>>();
    let (hits_replaced_s, hits_replaced_r) = unbounded::<HitsReplaced>();

    tokio::spawn(async move { zeditor::replace::run(db2, hits_replaced_s, replace_hits_r).await });

    let mut siv = cursive::default().into_runner();

    let theme = terminal_default_theme(&siv);
    siv.set_theme(theme);

    const NO_SEARCH: STATE = STATE(vec![]);
    siv.set_user_data(NO_SEARCH);

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
                .child(Button::new("Quit", move |s| {
                    replace_s2.send(Msg::Quit).expect("send");

                    Cursive::quit(s)
                })),
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

    refresh_found_widget(&mut siv, &replace_hits_s, skip_repo.clone());

    // manipulate the cursive event loop so that we can receive messages
    siv.refresh();
    while siv.is_running() {
        // update hacky counts & display size widgets
        update_report_widgets(&mut siv);

        siv.step();

        for files_searched in files_searched_r.try_iter() {
            update_found_user_data(&mut siv, files_searched, &replace_hits_s, skip_repo.clone());
            // force refresh of UI
            siv.cb_sink().send(Box::new(Cursive::noop)).expect("send");
        }

        for _ in hits_replaced_r.try_iter() {
            // just clear the entire list and re-search
            update_found_user_data(&mut siv, vec![], &replace_hits_s, skip_repo.clone());

            search_files_s.send(SearchFiles).expect("send");
        }
    }
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
                    let linear = LinearLayout::horizontal()
                        .child(TextView::new(hit.preview.clone()))
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

                    let label: String = hit
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

                    search_widget.add_child(&label, linear);
                }
            }
        });
    }
}

fn update_found_user_data(
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
            count_lines += hit.preview.lines().count();
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

fn count_visible_lines(siv: &mut Cursive) -> Option<usize> {
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

fn update_report_widgets(siv: &mut CursiveRunner<CursiveRunnable>) {
    let found_count = count_found(siv);

    // update hacky count widget
    if let Some(mut found_count_report) = siv.find_name::<TextView>(FOUND_LINES_REPORT) {
        found_count_report.set_content(format!("Found: {}", found_count));
    }

    // without this you'll lag behind by a step
    siv.refresh();
}

fn terminal_default_theme(siv: &Cursive) -> Theme {
    let mut theme = siv.current_theme().clone();

    theme.palette[PaletteColor::Background] = Color::TerminalDefault;

    theme
}
