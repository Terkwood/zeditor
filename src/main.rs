use cursive::reexports::crossbeam_channel::unbounded;
use cursive::traits::*;
use cursive::views::{Button, Dialog, DummyView, LinearLayout, ListView, Panel, TextView};
use cursive::Cursive;
use std::thread;
use zeditor::search::{FileSearched, SearchFiles};

#[derive(Clone)]
struct ReplacementCandidate {
    search: String,
    preview_blurb: String,
}

// names of widgets
const SEARCH_RESULTS: &str = "search results";

fn main() {
    let (search_files_s, search_files_r) = unbounded::<zeditor::search::SearchFiles>();
    let (files_searched_s, files_searched_r) = unbounded::<Vec<zeditor::search::FileSearched>>();

    thread::spawn(move || zeditor::search::run(files_searched_s, search_files_r));

    let mut siv = cursive::default().into_runner();

    const NO_SEARCH: Vec<ReplacementCandidate> = vec![];
    siv.set_user_data(NO_SEARCH);

    let search_results = ListView::new().with_name(SEARCH_RESULTS);

    let perm_buttons = Panel::new(
        LinearLayout::vertical()
            .child(Button::new("Replace All", |s| bogus(s)))
            .child(Button::new("Search", move |_| {
                search_files_s.send(SearchFiles).unwrap()
            }))
            .child(DummyView)
            .child(Button::new("Quit", Cursive::quit)),
    );

    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(search_results)
                .child(DummyView)
                .child(perm_buttons),
        )
        .title("zeditor"),
    );

    refresh_search_list(&mut siv);

    // manipulate the cursive event loop so that we can receive messages
    siv.refresh();
    while siv.is_running() {
        siv.step();
        for files_searched in files_searched_r.try_iter() {
            update_search_list(&mut siv, files_searched);
            // force refresh of UI
            siv.cb_sink().send(Box::new(Cursive::noop)).unwrap();
        }
    }
}

fn refresh_search_list(siv: &mut Cursive) {
    if let Some(mut fake_stuff) = siv.find_name::<ListView>(SEARCH_RESULTS) {
        let _ = siv.with_user_data(|blurbs: &mut Vec<ReplacementCandidate>| {
            fake_stuff.clear();
            for (pos, b) in blurbs.iter().enumerate() {
                let linear = LinearLayout::horizontal()
                    .child(TextView::new(b.preview_blurb.clone()))
                    .child(DummyView)
                    .child(Button::new("OK", bogus))
                    .child(DummyView)
                    .child(Button::new("Skip", move |s| skip_candidate(s, pos)));

                fake_stuff.add_child(&b.search, linear)
            }
        });
    }
}

fn update_search_list(siv: &mut Cursive, results: Vec<FileSearched>) {
    let candidates = ReplacementCandidate::convert(results);
    siv.with_user_data(|blurbs: &mut Vec<ReplacementCandidate>| blurbs.clear());
    for c in candidates {
        siv.with_user_data(|blurbs: &mut Vec<ReplacementCandidate>| blurbs.push(c.clone()));
    }
    refresh_search_list(siv);
}

fn skip_candidate(siv: &mut Cursive, user_data_pos: usize) {
    siv.with_user_data(|blurbs: &mut Vec<ReplacementCandidate>| {
        blurbs.remove(user_data_pos);
    });
    refresh_search_list(siv);
}
fn bogus(_siv: &mut Cursive) {}

impl ReplacementCandidate {
    pub fn convert(searched: Vec<FileSearched>) -> Vec<Self> {
        let mut out = vec![];
        for fs in searched {
            for hit in fs.hits {
                out.push(ReplacementCandidate {
                    search: hit.search,
                    preview_blurb: hit.preview,
                })
            }
        }

        out
    }
}
