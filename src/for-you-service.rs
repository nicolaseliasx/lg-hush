//! Reads "Continue Watching" from the owner's Stremio library and saves the posters off the
//! SDL thread. The renderer loads card images from files and keeps a failed load for the
//! session, so an image is announced only after it was validated, written atomically and
//! renamed into place.
use crate::for_you::{self, Item};
use std::{
    collections::HashSet,
    fs,
    io::Read,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

/// `poster/small` is 300x450, ~5-30 KB.
const IMAGE_LIMIT: usize = 512 * 1024;
/// A library of ~250 titles is ~150 KB.
const LIBRARY_LIMIT: usize = 4 * 1024 * 1024;

#[derive(Clone)]
pub struct Client {
    agent: ureq::Agent,
    /// Images may redirect: metahub sends new titles to the host that renders them.
    image_agent: ureq::Agent,
    images: String,
    /// Stremio's account API, for the owner's library.
    api: String,
}
impl Client {
    pub fn new(test_server: Option<&str>) -> Result<Self, String> {
        let (images, api) = if let Some(server) = test_server {
            let uri: ureq::http::Uri = server
                .parse()
                .map_err(|e| format!("Invalid For You test server: {e}"))?;
            let address: SocketAddr = uri
                .authority()
                .ok_or("For You test server needs a loopback address and port")?
                .as_str()
                .parse()
                .map_err(|_| "For You tests require an IP-literal loopback address and port")?;
            if uri.scheme_str() != Some("http")
                || !address.ip().is_loopback()
                || address.port() == 0
                || !matches!(uri.path(), "" | "/")
                || uri.query().is_some()
            {
                return Err("For You test server must be an HTTP loopback origin".into());
            }
            (format!("http://{address}"), format!("http://{address}/api"))
        } else {
            (
                "https://images.metahub.space".into(),
                "https://api.strem.io/api".into(),
            )
        };
        let agent = |redirects| {
            ureq::Agent::config_builder()
                .timeout_global(Some(Duration::from_secs(15)))
                .max_redirects(redirects)
                .proxy(None)
                .build()
                .new_agent()
        };
        Ok(Self {
            agent: agent(0),
            image_agent: agent(3),
            images,
            api,
        })
    }
    fn read(
        url: &str,
        mut response: ureq::http::Response<ureq::Body>,
        limit: usize,
    ) -> Result<Vec<u8>, String> {
        if !response.status().is_success() {
            return Err(format!("{url}: HTTP {}", response.status()));
        }
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(limit as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("{url}: {e}"))?;
        if bytes.len() > limit {
            return Err(format!("{url}: response too large"));
        }
        Ok(bytes)
    }
    /// Titles in progress in the owner's library. The key is never logged.
    fn library(&self, auth_key: &str) -> Result<Vec<Item>, String> {
        let url = format!("{}/datastoreGet", self.api);
        let body = serde_json::json!({
            "authKey": auth_key, "collection": "libraryItem", "ids": [], "all": true
        });
        let response = self
            .agent
            .post(&url)
            .header("User-Agent", "LumaNative")
            .header("Content-Type", "application/json")
            .send(body.to_string().as_bytes())
            .map_err(|e| format!("{url}: {e}"))?;
        let value: serde_json::Value =
            serde_json::from_slice(&Self::read(&url, response, LIBRARY_LIMIT)?)
                .map_err(|_| format!("{url}: unreadable library"))?;
        if !value["result"].is_array() {
            // An expired or revoked session answers with an error object.
            return Err(format!("{url}: the Stremio session was not accepted"));
        }
        Ok(for_you::parse_library(&value, for_you::COUNT))
    }
    /// The 300x450 poster. Metahub answers with WebP or JPEG; the renderer picks its decoder
    /// from the `.jpg` name, so WebP is converted here, off the SDL thread.
    fn image(&self, id: &str) -> Result<Vec<u8>, String> {
        let url = format!("{}/poster/small/{id}/img", self.images);
        let response = self
            .image_agent
            .get(&url)
            .header("User-Agent", "LumaNative")
            .call()
            .map_err(|e| format!("{url}: {e}"))?;
        let bytes = Self::read(&url, response, IMAGE_LIMIT)?;
        let format = image::guess_format(&bytes).map_err(|e| format!("{id}: {e}"))?;
        let image = image::load_from_memory_with_format(&bytes, format)
            .map_err(|e| format!("{id}: {e}"))?;
        if image.width() > 1920 || image.height() > 1080 {
            return Err(format!("{id}: image too large"));
        }
        if format == image::ImageFormat::Jpeg {
            return Ok(bytes);
        }
        let mut jpeg = vec![];
        image
            .into_rgb8()
            .write_to(
                &mut std::io::Cursor::new(&mut jpeg),
                image::ImageFormat::Jpeg,
            )
            .map_err(|e| format!("{id}: {e}"))?;
        Ok(jpeg)
    }
}

pub enum Response {
    /// Titles in progress (empty without a saved key).
    Resume(Vec<Item>),
    /// The image of this id is now on disk.
    Image(String),
}

pub fn image_dir(preferences: &Path) -> PathBuf {
    preferences.with_extension("for-you-posters")
}
/// The Stremio session key, written by `scripts/stremio-login.sh`; absent until then.
pub fn auth_path(preferences: &Path) -> PathBuf {
    preferences.with_extension("stremio-auth.json")
}
/// Removes the caches of earlier versions (best effort): the first version's 16:9 images and
/// the retired catalog's card list. Posters of titles in progress stay.
pub fn forget_legacy(preferences: &Path) {
    let _ = fs::remove_file(preferences.with_extension("for-you.json"));
    let _ = fs::remove_dir_all(preferences.with_extension("for-you"));
    let _ = fs::remove_file(preferences.with_extension("for-you-posters.json"));
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let tmp = path.with_extension(format!(
        "tmp-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&tmp, bytes)
        .and_then(|()| fs::rename(&tmp, path))
        .map_err(|e| {
            let _ = fs::remove_file(&tmp);
            e.to_string()
        })
}

/// Saves the images of `items` that are not on disk yet, one at a time, and announces each
/// image on disk, so a poster saved in an earlier session shows without downloading it again.
fn save_images(client: &Client, images: &Path, items: &[Item], sender: &mpsc::Sender<Response>) {
    for item in items {
        let path = for_you::image_path(images, &item.id);
        if path.is_file() {
            if sender.send(Response::Image(item.id.clone())).is_err() {
                break;
            }
            continue;
        }
        match client
            .image(&item.id)
            .and_then(|bytes| write_atomic(&path, &bytes))
        {
            Ok(()) => {
                if sender.send(Response::Image(item.id.clone())).is_err() {
                    break;
                }
            }
            Err(error) => eprintln!("Luma For You: {error}"),
        }
    }
}

/// Reads the titles in progress with the saved key, prunes the posters of titles that left the
/// list and saves the missing ones. Without a key the row is empty; a failed request leaves
/// the row as it was.
pub fn refresh_resume(
    client: &Client,
    auth_path: &Path,
    images: &Path,
    sender: &mpsc::Sender<Response>,
) -> Result<(), String> {
    let mut bytes = Vec::new();
    let key = fs::File::open(auth_path)
        .ok()
        .and_then(|file| file.take(4096).read_to_end(&mut bytes).ok())
        .and_then(|_| for_you::auth_key(&bytes));
    let items = match key {
        Some(key) => client.library(&key)?,
        None => vec![],
    };
    fs::create_dir_all(images).map_err(|e| e.to_string())?;
    let kept: HashSet<_> = items.iter().map(|i| format!("{}.jpg", i.id)).collect();
    for entry in fs::read_dir(images).map_err(|e| e.to_string())?.flatten() {
        if !kept.contains(&*entry.file_name().to_string_lossy()) {
            let _ = fs::remove_file(entry.path());
        }
    }
    if sender.send(Response::Resume(items.clone())).is_ok() {
        save_images(client, images, &items, sender);
    }
    Ok(())
}

pub struct Service {
    requests: mpsc::Sender<()>,
    pub receiver: mpsc::Receiver<Response>,
}
impl Service {
    pub fn new(client: Client, images: PathBuf, auth_path: PathBuf) -> Self {
        let (requests, jobs) = mpsc::channel::<()>();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            for () in jobs {
                if let Err(error) = refresh_resume(&client, &auth_path, &images, &sender) {
                    eprintln!("Luma For You: {error}");
                }
            }
        });
        Self { requests, receiver }
    }
    /// Reads the titles in progress again.
    pub fn resume(&self) {
        let _ = self.requests.send(());
    }
}

#[cfg(test)]
#[path = "for-you-service-tests.rs"]
mod tests;
