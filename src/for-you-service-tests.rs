use super::*;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

/// A loopback HTTP server answering fixed paths.
fn serve(routes: HashMap<String, Vec<u8>>) -> String {
    serve_recording(routes).0
}

/// Also records each request as "METHOD path body".
fn serve_recording(
    routes: HashMap<String, Vec<u8>>,
) -> (String, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let requests = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let log = requests.clone();
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let method = line.split_whitespace().next().unwrap_or("").to_owned();
            let path = line.split_whitespace().nth(1).unwrap_or("").to_owned();
            let mut length = 0;
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header).unwrap_or(0) <= 2 {
                    break;
                }
                if let Some((name, value)) = header.split_once(':')
                    && name.eq_ignore_ascii_case("content-length")
                {
                    length = value.trim().parse().unwrap_or(0);
                }
            }
            let mut body = vec![0; length];
            let _ = std::io::Read::read_exact(&mut reader, &mut body);
            log.lock().unwrap().push(format!(
                "{method} {path} {}",
                String::from_utf8_lossy(&body)
            ));
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
    (origin, requests)
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
fn landscape_images_from_the_first_version_are_never_shown_as_posters() {
    let dir = temp_dir();
    let preferences = dir.join("preferences.json");
    let old_images = preferences.with_extension("for-you");
    fs::create_dir_all(&old_images).unwrap();
    fs::write(old_images.join("tt1.jpg"), b"landscape").unwrap();
    fs::write(preferences.with_extension("for-you.json"), b"{}").unwrap();
    assert_ne!(image_dir(&preferences), old_images);
    forget_legacy(&preferences);
    assert!(!old_images.exists());
    assert!(!preferences.with_extension("for-you.json").exists());
    fs::remove_dir_all(dir).unwrap();
}

fn library(entries: &[(&str, u64)]) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"result": entries.iter().map(|(id, offset)| serde_json::json!({
        "_id": id, "type": "movie", "name": format!("Title {id}"), "removed": false, "temp": false,
        "state": {"timeOffset": offset, "duration": 6_000_000, "lastWatched": "2026-11-08T00:00:00.000Z"}
    })).collect::<Vec<_>>()}))
    .unwrap()
}

#[test]
fn continue_watching_reads_the_library_with_the_saved_key_and_saves_its_posters() {
    let mut routes = HashMap::new();
    routes.insert(
        "/api/datastoreGet".into(),
        library(&[("tt5", 600_000), ("tt6", 0)]),
    );
    routes.insert("/poster/small/tt5/img".into(), jpeg());
    let (origin, requests) = serve_recording(routes);
    let client = Client::new(Some(&origin)).unwrap();
    let dir = temp_dir();
    let auth = dir.join("stremio-auth.json");
    fs::write(&auth, br#"{"authKey":"secret-key"}"#).unwrap();
    let images = dir.join("images");
    let (sender, receiver) = mpsc::channel();

    refresh_resume(&client, &auth, &images, &sender).unwrap();

    let responses: Vec<_> = receiver.try_iter().collect();
    let Response::Resume(items) = &responses[0] else {
        panic!("items first")
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, "tt5");
    assert!(matches!(&responses[1], Response::Image(id) if id == "tt5"));
    assert!(images.join("tt5.jpg").is_file());
    let requests = requests.lock().unwrap();
    let call = requests
        .iter()
        .find(|r| r.starts_with("POST /api/datastoreGet "))
        .expect("library request");
    let body: serde_json::Value =
        serde_json::from_str(call.trim_start_matches("POST /api/datastoreGet ")).unwrap();
    assert_eq!(body["authKey"], "secret-key");
    assert_eq!(body["collection"], "libraryItem");
    assert_eq!(body["all"], true);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn without_a_saved_key_continue_watching_is_empty_and_nothing_is_requested() {
    let (origin, requests) = serve_recording(HashMap::new());
    let client = Client::new(Some(&origin)).unwrap();
    let dir = temp_dir();
    let (sender, receiver) = mpsc::channel();
    refresh_resume(&client, &dir.join("missing.json"), &dir, &sender).unwrap();
    assert!(matches!(
        receiver.try_iter().collect::<Vec<_>>().as_slice(),
        [Response::Resume(items)] if items.is_empty()
    ));
    assert!(requests.lock().unwrap().is_empty());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_rejected_key_keeps_the_row_as_it_was() {
    let mut routes = HashMap::new();
    routes.insert(
        "/api/datastoreGet".into(),
        br#"{"error":{"message":"Session not found","code":1}}"#.to_vec(),
    );
    let client = Client::new(Some(&serve(routes))).unwrap();
    let dir = temp_dir();
    let auth = dir.join("stremio-auth.json");
    fs::write(&auth, br#"{"authKey":"expired"}"#).unwrap();
    let (sender, receiver) = mpsc::channel();
    assert!(refresh_resume(&client, &auth, &dir, &sender).is_err());
    assert!(receiver.try_iter().next().is_none());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn the_key_file_sits_next_to_the_preferences() {
    let preferences = Path::new("/data/preferences.json");
    assert_eq!(
        auth_path(preferences),
        Path::new("/data/preferences.stremio-auth.json")
    );
}

#[test]
fn a_poster_already_on_disk_is_announced_without_downloading_it_again() {
    // After a restart only catalog posters are marked ready; a title in progress whose poster
    // was saved earlier must still show it.
    let mut routes = HashMap::new();
    routes.insert("/api/datastoreGet".into(), library(&[("tt5", 600_000)]));
    let (origin, requests) = serve_recording(routes);
    let client = Client::new(Some(&origin)).unwrap();
    let dir = temp_dir();
    let auth = dir.join("stremio-auth.json");
    fs::write(&auth, br#"{"authKey":"k"}"#).unwrap();
    let images = dir.join("images");
    fs::create_dir_all(&images).unwrap();
    fs::write(images.join("tt5.jpg"), jpeg()).unwrap();
    let (sender, receiver) = mpsc::channel();
    refresh_resume(&client, &auth, &images, &sender).unwrap();
    let responses: Vec<_> = receiver.try_iter().collect();
    assert!(
        responses
            .iter()
            .any(|r| matches!(r, Response::Image(id) if id == "tt5")),
        "the saved poster is announced"
    );
    assert!(
        !requests
            .lock()
            .unwrap()
            .iter()
            .any(|r| r.contains("/poster/")),
        "and not downloaded again"
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn only_titles_in_progress_keep_their_posters() {
    let mut routes = HashMap::new();
    routes.insert("/api/datastoreGet".into(), library(&[("tt5", 600_000)]));
    routes.insert("/poster/small/tt5/img".into(), jpeg());
    let client = Client::new(Some(&serve(routes))).unwrap();
    let dir = temp_dir();
    let auth = dir.join("stremio-auth.json");
    fs::write(&auth, br#"{"authKey":"k"}"#).unwrap();
    let images = dir.join("images");
    fs::create_dir_all(&images).unwrap();
    fs::write(images.join("tt7.jpg"), b"finished").unwrap();
    let (sender, _receiver) = mpsc::channel();
    refresh_resume(&client, &auth, &images, &sender).unwrap();
    assert!(images.join("tt5.jpg").is_file());
    assert!(
        !images.join("tt7.jpg").exists(),
        "a finished title's poster is pruned"
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn the_retired_catalog_cache_is_removed_but_posters_stay() {
    let dir = temp_dir();
    let preferences = dir.join("preferences.json");
    let catalog = preferences.with_extension("for-you-posters.json");
    fs::write(&catalog, br#"{"fetched":1,"items":[]}"#).unwrap();
    fs::create_dir_all(image_dir(&preferences)).unwrap();
    forget_legacy(&preferences);
    assert!(!catalog.exists());
    assert!(image_dir(&preferences).is_dir());
    fs::remove_dir_all(dir).unwrap();
}
