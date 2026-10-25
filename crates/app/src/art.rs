//! Artwork for `image()`. nd's pipeline fetches, decodes at display size and keeps the LRU; this
//! picks the URL a box of a given size should ask for, and has the store's [`ArtCache`] paint the
//! demo set's covers and fetch Kopuz's artwork, which the daemon serves as bytes, not URLs.

use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

use formalmusic_api::Thumbnail;
use formalmusic_core::art::{ArtCache, DEMO_SCHEME, KOPUZ_SCHEME, url_for};
use nd::prelude::*;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

thread_local! {
    /// Demo and Kopuz artwork asked for this run, landed or not, so a failure is not asked for again
    /// on every frame.
    static ASKED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

struct Daemon {
    cache: Arc<ArtCache>,
    runtime: tokio::runtime::Handle,
    landed: UnboundedSender<()>,
}

static DAEMON: OnceLock<Daemon> = OnceLock::new();

/// Lets [`source`] fetch Kopuz artwork through `cache`, repainting when a picture lands.
pub fn install(_cx: &mut App, cache: Arc<ArtCache>, runtime: tokio::runtime::Handle) {
    let (landed, rx) = unbounded_channel();
    if DAEMON.set(Daemon { cache, runtime, landed }).is_ok() {
        crate::bridge::on_main(rx, |(), cx| cx.refresh_windows());
    }
}

/// The image for `thumbnails` in a box `points` wide, or `None` when there is none yet.
pub fn source(thumbnails: &[Thumbnail], points: f32) -> Option<ImageSource> {
    // Asked for at twice the box, so it is sharp on a Retina or 2x display.
    let url = url_for(thumbnails, (points * 2.) as u32)?;
    // Demo covers are painted off the main thread, as Kopuz's are fetched: Home asks for dozens
    // on its first frame, and painting them there held the window empty for over a second.
    if url.starts_with(DEMO_SCHEME) || url.starts_with(KOPUZ_SCHEME) {
        return cached(url);
    }
    Some(ImageSource::url(url))
}

fn cached(url: String) -> Option<ImageSource> {
    let daemon = DAEMON.get()?;
    let path = daemon.cache.path_for(&url);
    if path.is_file() {
        return Some(ImageSource::path(path));
    }
    if ASKED.with(|asked| asked.borrow_mut().insert(url.clone())) {
        let cache = daemon.cache.clone();
        let landed = daemon.landed.clone();
        daemon.runtime.spawn(async move {
            if cache.fetch(&url).await.is_some() {
                let _ = landed.send(());
            }
        });
    }
    None
}

/// A cover `size` points square: the artwork, or the pipeline's placeholder while there is none.
pub fn cover(
    id: impl Into<SharedString>,
    thumbnails: &[Thumbnail],
    size: f32,
    alt: impl Into<SharedString>,
) -> Image {
    let el = match source(thumbnails, size) {
        Some(source) => image(source),
        None => image_placeholder(),
    };
    el.id(id).alt(alt).size(px(size)).flex_none()
}
