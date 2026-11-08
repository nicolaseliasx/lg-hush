use super::*;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

/// A loopback HTTP server answering fixed paths, like the weather smoke's.
fn serve(routes: HashMap<String, Vec<u8>>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let path = line.split_whitespace().nth(1).unwrap_or("").to_owned();
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header).unwrap_or(0) <= 2 {
                    break;
                }
            }
            let mut stream = stream;
            match routes.get(&path) {
                Some(body) if body.starts_with(b"redirect:") => {
                    let location = String::from_utf8_lossy(&body[9..]).into_owned();
                    let _ = write!(
                        stream,
                        "HTTP/1.1 307 Temporary Redirect\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    );
                }
                Some(body) => {
                    let _ = write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(body);
                }
                None => {
                    let _ = write!(
                        stream,
                        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    );
                }
            }
        }
    });
    origin
}

fn jpeg() -> Vec<u8> {
    let mut bytes = vec![];
    image::RgbImage::from_pixel(48, 27, image::Rgb([20, 30, 40]))
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Jpeg,
        )
        .unwrap();
    bytes
}

fn catalog(ids: &[&str]) -> Vec<u8> {
    serde_json::to_vec(
        &serde_json::json!({"metas": ids.iter().map(|id| serde_json::json!({
        "id": id, "name": format!("Title {id}"), "genres": ["Drama"], "runtime": "90 min"
    })).collect::<Vec<_>>()}),
    )
    .unwrap()
}

fn temp_dir() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "luma-for-you-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_origins_are_loopback_http_only() {
    assert!(Client::new(Some("http://127.0.0.1:1234")).is_ok());
    for invalid in [
        "http://example.com:1234",
        "https://127.0.0.1:1234",
        "http://127.0.0.1:1234/x",
    ] {
        assert!(Client::new(Some(invalid)).is_err(), "{invalid}");
    }
}

#[test]
fn a_refresh_caches_interleaved_items_then_announces_each_saved_image() {
    let mut routes = HashMap::new();
    routes.insert("/catalog/movie/top.json".into(), catalog(&["tt1", "tt2"]));
    routes.insert("/catalog/series/top.json".into(), catalog(&["tt9", "tt8"]));
    // New titles: metahub redirects to the host that renders the image on demand.
    routes.insert(
        "/poster/small/tt8/img".into(),
        b"redirect:/live/tt8".to_vec(),
    );
    routes.insert("/live/tt8".into(), jpeg());
    routes.insert("/poster/small/tt1/img".into(), jpeg());
    let mut webp = vec![];
    image::RgbImage::from_pixel(8, 8, image::Rgb([1, 2, 3]))
        .write_to(
            &mut std::io::Cursor::new(&mut webp),
            image::ImageFormat::WebP,
        )
        .unwrap();
    // Posters come as WebP or JPEG; cards load by the .jpg extension, so WebP is converted.
    routes.insert("/poster/small/tt9/img".into(), webp);
    routes.insert("/poster/small/tt2/img".into(), b"not an image".to_vec());
    // tt2 has no image: its card keeps the placeholder.
    let client = Client::new(Some(&serve(routes))).unwrap();
    let dir = temp_dir();
    let cache_path = dir.join("cache.json");
    let images = dir.join("images");
    fs::create_dir_all(&images).unwrap();
    fs::write(images.join("tt7.jpg"), b"stale").unwrap();
    let (sender, receiver) = mpsc::channel();

    refresh(&client, &cache_path, &images, 1_000, &sender).unwrap();

    let responses: Vec<_> = receiver.try_iter().collect();
    let Response::Items(cache) = &responses[0] else {
        panic!("items first")
    };
    let ids: Vec<_> = cache.items.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(ids, ["tt1", "tt9", "tt2", "tt8"]);
    assert_eq!(cache.fetched, 1_000);
    let saved: Vec<_> = responses[1..]
        .iter()
        .map(|r| match r {
            Response::Image(id) => id.as_str(),
            Response::Items(_) => "items",
        })
        .collect();
    assert_eq!(saved, ["tt1", "tt9", "tt8"]);
    assert!(images.join("tt1.jpg").is_file());
    assert_eq!(
        image::ImageReader::open(images.join("tt9.jpg"))
            .unwrap()
            .with_guessed_format()
            .unwrap()
            .format(),
        Some(image::ImageFormat::Jpeg),
        "WebP posters are stored as JPEG"
    );
    assert!(
        !images.join("tt2.jpg").exists(),
        "undecodable images are never written"
    );
    assert!(
        !images.join("tt7.jpg").exists(),
        "images of dropped titles are pruned"
    );
    let (loaded, ready) = load(&cache_path, &images);
    assert_eq!(loaded.unwrap().items.len(), 4);
    assert_eq!(
        ready,
        HashSet::from(["tt1".to_string(), "tt9".to_string(), "tt8".to_string()])
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn an_unavailable_catalog_keeps_the_previous_cache() {
    let client = Client::new(Some(&serve(HashMap::new()))).unwrap();
    let dir = temp_dir();
    let cache_path = dir.join("cache.json");
    fs::write(&cache_path, br#"{"fetched":5,"items":[]}"#).unwrap();
    let (sender, _receiver) = mpsc::channel();
    assert!(refresh(&client, &cache_path, &dir, 1_000, &sender).is_err());
    assert_eq!(load(&cache_path, &dir).0.unwrap().fetched, 5);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn landscape_images_from_the_first_version_are_never_shown_as_posters() {
    let dir = temp_dir();
    let preferences = dir.join("preferences.json");
    let old_images = preferences.with_extension("for-you");
    fs::create_dir_all(&old_images).unwrap();
    fs::write(old_images.join("tt1.jpg"), b"landscape").unwrap();
    fs::write(preferences.with_extension("for-you.json"), b"{}").unwrap();
    assert_ne!(image_dir(&preferences), old_images);
    assert_ne!(
        cache_path(&preferences),
        preferences.with_extension("for-you.json")
    );
    forget_legacy(&preferences);
    assert!(!old_images.exists());
    assert!(!preferences.with_extension("for-you.json").exists());
    fs::remove_dir_all(dir).unwrap();
}
