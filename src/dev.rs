#![cfg(debug_assertions)]

use std::path::Path;
use std::sync::mpsc;

use notify::{RecursiveMode, Watcher};
use tower_livereload::LiveReloadLayer;
use tower_livereload::Reloader;

const STATIC_DIR: &str = "web/static";
const TEMPLATE_DIR: &str = "templates";

/// Builds the live-reload middleware and starts a watcher that triggers a
/// browser reload when the web build or a template changes. Template changes
/// are also picked up by minijinja's auto reload, so no server restart is
/// needed.
pub fn layer() -> LiveReloadLayer {
    let layer = LiveReloadLayer::new();
    watch_changes(layer.reloader());
    layer
}

fn watch_changes(reloader: Reloader) {
    std::thread::spawn(move || {
        let (events, receiver) = mpsc::channel();
        let mut watcher = match notify::recommended_watcher(move |event| {
            let _ = events.send(event);
        }) {
            Ok(watcher) => watcher,
            Err(error) => {
                tracing::warn!(%error, "live reload watcher disabled");
                return;
            }
        };

        let mut watching = 0;
        for dir in [STATIC_DIR, TEMPLATE_DIR] {
            if let Err(error) = watcher.watch(Path::new(dir), RecursiveMode::Recursive) {
                tracing::warn!(%error, path = dir, "live reload watcher disabled for path");
            } else {
                watching += 1;
                tracing::info!(path = dir, "watching for live reload");
            }
        }
        if watching == 0 {
            return;
        }

        for event in receiver {
            match event {
                Ok(event)
                    if event.kind.is_create()
                        || event.kind.is_modify()
                        || event.kind.is_remove() =>
                {
                    reloader.reload();
                }
                Ok(_) => {}
                Err(error) => tracing::warn!(%error, "live reload watcher error"),
            }
        }
    });
}
