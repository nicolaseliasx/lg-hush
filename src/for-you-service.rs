//! Fetches "Para Você" catalogs and images off the SDL thread. The renderer loads card
//! images from files and keeps a failed load for the session, so an image is announced only
//! after it was validated, written atomically and renamed into place.
use crate::for_you::{self, Cache, Item, Kind};
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

/// The series catalog is ~560 KB.
const CATALOG_LIMIT: usize = 1024 * 1024;
/// `poster/small` is 300x450, ~5-30 KB.
const IMAGE_LIMIT: usize = 512 * 1024;

#[derive(Clone)]
pub struct Client {
    agent: ureq::Agent,
    /// Images may redirect: metahub sends new titles to the host that renders them.
    image_agent: ureq::Agent,
    catalogs: String,
    images: String,
}
impl Client {
    pub fn new(test_server: Option<&str>) -> Result<Self, String> {
        let (catalogs, images) = if let Some(server) = test_server {
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
            (format!("http://{address}"), format!("http://{address}"))
        } else {
            // The catalog's own origin: v3-cinemeta.strem.io answers with a redirect here.
            (
                "https://cinemeta-catalogs.strem.io/top".into(),
                "https://images.metahub.space".into(),
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
            catalogs,
            images,
        })
    }
    fn get(&self, agent: &ureq::Agent, url: &str, limit: usize) -> Result<Vec<u8>, String> {
        let mut response = agent
            .get(url)
            .header("User-Agent", "LumaNative")
            .call()
            .map_err(|e| format!("{url}: {e}"))?;
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
    fn catalog(&self, kind: Kind) -> Result<Vec<Item>, String> {
        let url = format!("{}/catalog/{}/top.json", self.catalogs, kind.as_str());
        let value = serde_json::from_slice(&self.get(&self.agent, &url, CATALOG_LIMIT)?)
            .map_err(|_| format!("{url}: unreadable catalog"))?;
        Ok(for_you::parse_catalog(&value, kind, for_you::COUNT))
    }
    /// The 300x450 poster. Metahub answers with WebP or JPEG; the renderer picks its decoder
    /// from the `.jpg` name, so WebP is converted here, off the SDL thread.
    fn image(&self, id: &str) -> Result<Vec<u8>, String> {
        let bytes = self.get(
            &self.image_agent,
            &format!("{}/poster/small/{id}/img", self.images),
            IMAGE_LIMIT,
        )?;
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
    Items(Cache),
    /// The image of this id is now on disk.
    Image(String),
}

// Posters (2:3) use new paths: the first version cached 16:9 images under the same ids.
pub fn cache_path(preferences: &Path) -> PathBuf {
    preferences.with_extension("for-you-posters.json")
}
pub fn image_dir(preferences: &Path) -> PathBuf {
    preferences.with_extension("for-you-posters")
}
/// Removes the first version's 16:9 cache (best effort).
pub fn forget_legacy(preferences: &Path) {
    let _ = fs::remove_file(preferences.with_extension("for-you.json"));
    let _ = fs::remove_dir_all(preferences.with_extension("for-you"));
}

/// Cached items and the ids whose image file is present.
pub fn load(cache_path: &Path, images: &Path) -> (Option<Cache>, HashSet<String>) {
    let mut bytes = Vec::new();
    let cache = fs::File::open(cache_path)
        .ok()
        .and_then(|file| file.take(256 * 1024).read_to_end(&mut bytes).ok())
        .and_then(|_| serde_json::from_slice::<Cache>(&bytes).ok())
        .map(|mut cache| {
            cache.items.retain(|item| for_you::valid_id(&item.id));
            cache.items.truncate(for_you::COUNT);
            cache
        });
    let ready = cache
        .iter()
        .flat_map(|cache| &cache.items)
        .filter(|item| {
            fs::metadata(for_you::image_path(images, &item.id)).is_ok_and(|m| m.len() > 0)
        })
        .map(|item| item.id.clone())
        .collect();
    (cache, ready)
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

/// Fetches both catalogs, caches the cards, then saves missing images one at a time.
pub fn refresh(
    client: &Client,
    cache_path: &Path,
    images: &Path,
    now: i64,
    sender: &mpsc::Sender<Response>,
) -> Result<(), String> {
    let items = for_you::interleave(
        client.catalog(Kind::Movie)?,
        client.catalog(Kind::Series)?,
        for_you::COUNT,
    );
    if items.is_empty() {
        return Err("The catalogs had no usable titles".into());
    }
    let cache = Cache {
        fetched: now,
        items,
    };
    write_atomic(
        cache_path,
        &serde_json::to_vec(&cache).map_err(|e| e.to_string())?,
    )?;
    fs::create_dir_all(images).map_err(|e| e.to_string())?;
    let ids: HashSet<_> = cache
        .items
        .iter()
        .map(|i| format!("{}.jpg", i.id))
        .collect();
    for entry in fs::read_dir(images).map_err(|e| e.to_string())?.flatten() {
        if !ids.contains(&*entry.file_name().to_string_lossy()) {
            let _ = fs::remove_file(entry.path());
        }
    }
    let items = cache.items.clone();
    if sender.send(Response::Items(cache)).is_err() {
        return Ok(());
    }
    for item in items {
        let path = for_you::image_path(images, &item.id);
        if path.is_file() {
            continue;
        }
        match client
            .image(&item.id)
            .and_then(|bytes| write_atomic(&path, &bytes))
        {
            Ok(()) => {
                if sender.send(Response::Image(item.id)).is_err() {
                    break;
                }
            }
            Err(error) => eprintln!("Luma For You: {error}"),
        }
    }
    Ok(())
}

pub struct Service {
    requests: mpsc::Sender<i64>,
    pub receiver: mpsc::Receiver<Response>,
}
impl Service {
    pub fn new(client: Client, cache_path: PathBuf, images: PathBuf) -> Self {
        let (requests, jobs) = mpsc::channel::<i64>();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            for now in jobs {
                if let Err(error) = refresh(&client, &cache_path, &images, now, &sender) {
                    eprintln!("Luma For You: {error}");
                }
            }
        });
        Self { requests, receiver }
    }
    /// Starts one refresh; `now` is the Unix time recorded with the result.
    pub fn refresh(&self, now: i64) {
        let _ = self.requests.send(now);
    }
}

#[cfg(test)]
#[path = "for-you-service-tests.rs"]
mod tests;
