use cursive::reexports::crossbeam_channel::{unbounded, Sender};
use cursive::traits::*;
use cursive::views::{Button, Dialog, DummyView, LinearLayout, ListView, Panel, TextView};
use cursive::Cursive;
use zeditor::search::{Hit, SearchFiles};

const SEARCH_RESULTS_WIDGET: &str = "search results";

const FILENAME_LABEL_LENGTH: usize = 15;

#[tokio::main]
async fn main() {
    let (search_files_s, search_files_r) = unbounded::<zeditor::search::SearchFiles>();
    let (files_searched_s, files_searched_r) = unbounded::<Vec<zeditor::search::Hit>>();

    tokio::spawn(async move { zeditor::search::run(files_searched_s, search_files_r).await });

    let (replace_hits_s, replace_hits_r) = unbounded::<zeditor::replace::ReplaceHits>();
    let (hits_replaced_s, hits_replace_r) = unbounded::<zeditor::replace::HitsReplaced>();

    tokio::spawn(async move { zeditor::replace::run(hits_replaced_s, replace_hits_r).await });

    let mut siv = cursive::default().into_runner();

    const NO_SEARCH: Vec<Hit> = vec![];
    siv.set_user_data(NO_SEARCH);

    let search_results = ListView::new().with_name(SEARCH_RESULTS_WIDGET);

    let perm_buttons = {
        let msg = search_files_s.clone();
        Panel::new(
            LinearLayout::vertical()
                .child(Button::new("Replace All", |s| bogus(s)))
                .child(Button::new("Search", move |_| {
                    msg.send(SearchFiles).unwrap()
                }))
                .child(DummyView)
                .child(Button::new("Quit", Cursive::quit)),
        )
    };

    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(search_results)
                .child(DummyView)
                .child(perm_buttons),
        )
        .title("zeditor"),
    );

    refresh_search_list(&mut siv, &replace_hits_s);

    // manipulate the cursive event loop so that we can receive messages
    siv.refresh();
    while siv.is_running() {
        siv.step();
        for files_searched in files_searched_r.try_iter() {
            update_search_list(&mut siv, files_searched, &replace_hits_s);
            // force refresh of UI
            siv.cb_sink().send(Box::new(Cursive::noop)).unwrap();
        }

        for _ in hits_replace_r.try_iter() {
            // just clear the entire list and re-search

            update_search_list(&mut siv, vec![], &replace_hits_s);

            search_files_s.send(SearchFiles).expect("send");
        }
    }
}

fn refresh_search_list(siv: &mut Cursive, replace_hits_s: &Sender<zeditor::replace::ReplaceHits>) {
    if let Some(mut search_widget) = siv.find_name::<ListView>(SEARCH_RESULTS_WIDGET) {
        let _ = siv.with_user_data(|search_hits: &mut Vec<Hit>| {
            search_widget.clear();
            for (hit_pos, hit) in search_hits.clone().iter().enumerate() {
                let replace_hits_chan = replace_hits_s.clone();
                let replace_hits_chan2 = replace_hits_s.clone();
                let hitc = hit.clone();
                let linear = LinearLayout::horizontal()
                    .child(TextView::new(hit.preview.clone()))
                    .child(DummyView)
                    .child(Button::new("OK", move |_| {
                        replace_hits_chan
                            .send(zeditor::replace::ReplaceHits(vec![hitc.clone()]))
                            .expect("send")
                    }))
                    .child(DummyView)
                    .child(Button::new("Skip", move |s| {
                        skip_candidate(s, hit_pos, &replace_hits_chan2)
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

                search_widget.add_child(&label, linear)
            }
        });
    }
}

fn update_search_list(
    siv: &mut Cursive,
    results: Vec<Hit>,
    replace_hits_s: &Sender<zeditor::replace::ReplaceHits>,
) {
    siv.with_user_data(|search_hits: &mut Vec<Hit>| {
        search_hits.clear();
        for f in results {
            search_hits.push(f);
        }
    });

    refresh_search_list(siv, replace_hits_s);
}

fn skip_candidate(
    siv: &mut Cursive,
    user_data_pos: usize,
    replace_hits_s: &Sender<zeditor::replace::ReplaceHits>,
) {
    siv.with_user_data(|hits: &mut Vec<Hit>| {
        hits.remove(user_data_pos);
    });
    refresh_search_list(siv, replace_hits_s);
}

fn bogus(_siv: &mut Cursive) {}
