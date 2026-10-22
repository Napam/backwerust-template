#![cfg(debug_assertions)]

use std::path::Path;
use std::sync::mpsc;

use notify::{RecursiveMode, Watcher};
use tower_livereload::LiveReloadLayer;
use tower_livereload::Reloader;

const STATIC_DIR: &str = "web/static";

/// Builds the live-reload middleware and starts a watcher that triggers a
/// browser reload when the web build changes. Rust changes are handled by
/// bacon, which rebuilds and restarts the server; the browser reconnects.
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

        if let Err(error) = watcher.watch(Path::new(STATIC_DIR), RecursiveMode::Recursive) {
            tracing::warn!(%error, path = STATIC_DIR, "live reload watcher disabled");
            return;
        }
        tracing::info!(path = STATIC_DIR, "watching for live reload");

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
