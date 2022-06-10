use cursive::reexports::crossbeam_channel::unbounded;
use cursive::traits::*;
use cursive::views::{Button, Dialog, DummyView, LinearLayout, ListView, Panel, TextView};
use cursive::Cursive;
use zeditor::search::{Hit, SearchFiles};

// names of widgets
const SEARCH_RESULTS: &str = "search results";

#[tokio::main]
async fn main() {
    let (search_files_s, search_files_r) = unbounded::<zeditor::search::SearchFiles>();
    let (files_searched_s, files_searched_r) = unbounded::<Vec<zeditor::search::Hit>>();

    tokio::spawn(async move { zeditor::search::run(files_searched_s, search_files_r).await });

    let mut siv = cursive::default().into_runner();

    const NO_SEARCH: Vec<Hit> = vec![];
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
    if let Some(mut search_widget) = siv.find_name::<ListView>(SEARCH_RESULTS) {
        let _ = siv.with_user_data(|search_hits: &mut Vec<Hit>| {
            search_widget.clear();
            for (hit_pos, hit) in search_hits.clone().iter().enumerate() {
                let linear = LinearLayout::horizontal()
                    .child(TextView::new(hit.preview.clone()))
                    .child(DummyView)
                    .child(Button::new("OK", bogus))
                    .child(DummyView)
                    .child(Button::new("Skip", move |s| skip_candidate(s, hit_pos)));

                search_widget.add_child(&hit.search, linear)
            }
        });
    }
}

fn update_search_list(siv: &mut Cursive, results: Vec<Hit>) {
    siv.with_user_data(|search_hits: &mut Vec<Hit>| {
        search_hits.clear();
        for f in results {
            search_hits.push(f);
        }
    });

    refresh_search_list(siv);
}

fn skip_candidate(siv: &mut Cursive, user_data_pos: usize) {
    siv.with_user_data(|hits: &mut Vec<Hit>| {
        hits.remove(user_data_pos);
    });
    refresh_search_list(siv);
}
fn bogus(_siv: &mut Cursive) {}
