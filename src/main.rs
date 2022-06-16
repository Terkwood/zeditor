use cursive::reexports::crossbeam_channel::unbounded;
use cursive::traits::*;
use cursive::Cursive;
use std::sync::{Arc, Mutex};
use zeditor::db::Db;
use zeditor::msg::Msg;
use zeditor::replace::{HitsReplaced, ReplaceHits};
use zeditor::search::SearchFiles;
use zeditor::skip::SkipRepo;
use zeditor::STATE;
use zeditor::{config_screen, home_screen};

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

    siv.load_toml(include_str!("theme.toml")).unwrap();

    const NO_SEARCH: STATE = STATE(vec![]);
    siv.set_user_data(NO_SEARCH);

    let home_id = siv.active_screen();
    let config_id = siv.add_screen();

    // TODO hacked
    siv.set_screen(config_id);

    match siv.active_screen() {
        id if id == config_id => {
            config_screen::render(&mut siv, home_id);
        }
        _ => {
            home_screen::render(
                &mut siv,
                replace_hits_s.clone(),
                search_files_s.clone(),
                skip_repo.clone(),
            );
        }
    }

    // manipulate the cursive event loop so that we can receive messages
    siv.refresh();
    while siv.is_running() {
        // update hacky counts & display size widgets
        home_screen::update_report_widgets(&mut siv);

        siv.step();

        for files_searched in files_searched_r.try_iter() {
            home_screen::update_found_user_data(
                &mut siv,
                files_searched,
                &replace_hits_s,
                skip_repo.clone(),
            );
            // force refresh of UI
            siv.cb_sink().send(Box::new(Cursive::noop)).expect("send");
        }

        for _ in hits_replaced_r.try_iter() {
            // just clear the entire list and re-search
            home_screen::update_found_user_data(
                &mut siv,
                vec![],
                &replace_hits_s,
                skip_repo.clone(),
            );

            search_files_s.send(SearchFiles).expect("send");
        }
    }
}
