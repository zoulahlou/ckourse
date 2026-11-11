use crate::{db, parser};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, process::Command, sync::Mutex};
use tauri::Manager;

static EXTRACTION: Mutex<()> = Mutex::new(());

#[tauri::command]
pub async fn get_course_cover(app: tauri::AppHandle, state: tauri::State<'_, db::DbState>, course_id: i64) -> Result<Option<String>, String> {
    let (cover, video) = {
        let conn = state.conn.lock().map_err(|e| e.to_string())?;
        let course = db::get_course_by_id(&conn, course_id).map_err(|e| e.to_string())?.ok_or("Course not found")?;
        let video = db::get_course_detail(&conn, course_id).map_err(|e| e.to_string())?
            .and_then(|d| d.sections.into_iter().flat_map(|s| s.lessons).next()).map(|l| l.video_path);
        (course.thumbnail_path, video)
    };
    if let Some(path) = cover.filter(|p| PathBuf::from(p).is_file()) { return Ok(Some(path)); }
    let Some(video) = video.filter(|p| PathBuf::from(p).is_file()) else { return Ok(None); };
    let cache = app.path().app_cache_dir().map_err(|e| e.to_string())?.join("covers");
    let ffmpeg = parser::find_bundled_bin("ffmpeg").ok_or("Bundled ffmpeg missing")?;
    tauri::async_runtime::spawn_blocking(move || extract_cover(&PathBuf::from(video), &cache, &ffmpeg))
        .await.map_err(|e| e.to_string())?
}

fn extract_cover(video: &std::path::Path, cache: &std::path::Path, ffmpeg: &std::path::Path) -> Result<Option<String>, String> {
    let metadata = fs::metadata(video).map_err(|e| e.to_string())?;
    let key = format!("{:x}", Sha256::digest(format!("{}:{}:{:?}", video.display(), metadata.len(), metadata.modified())));
    fs::create_dir_all(cache).map_err(|e| e.to_string())?;
    let target = cache.join(format!("{key}.jpg"));
    let _guard = EXTRACTION.lock().map_err(|e| e.to_string())?;
    if target.is_file() { return Ok(Some(target.to_string_lossy().into())); }
    let temporary = cache.join(format!("{}.jpg", uuid::Uuid::new_v4()));
    for position in ["5", "0"] {
        let result = Command::new(ffmpeg).args(["-nostdin", "-v", "error", "-y", "-ss", position, "-i"])
            .arg(video).args(["-frames:v", "1", "-vf", "scale=640:-2", "-q:v", "3"]).arg(&temporary).output();
        if result.is_ok_and(|r| r.status.success()) && fs::metadata(&temporary).is_ok_and(|m| m.len() > 0) {
            fs::rename(&temporary, &target).map_err(|e| e.to_string())?;
            return Ok(Some(target.to_string_lossy().into()));
        }
    }
    let _ = fs::remove_file(temporary);
    Ok(None)
}

#[tauri::command]
pub async fn set_course_cover(app: tauri::AppHandle, state: tauri::State<'_, db::DbState>, course_id: i64, path: String) -> Result<(), String> {
    let source = PathBuf::from(path).canonicalize().map_err(|e| e.to_string())?;
    let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if !["jpg", "jpeg", "png", "webp"].contains(&ext.as_str()) || !source.is_file() || fs::metadata(&source).map_err(|e| e.to_string())?.len() > 50 * 1024 * 1024 {
        return Err("Choose a JPG, PNG or WebP image smaller than 50 MB".into());
    }
    use std::io::Read;
    let mut signature = [0u8; 12];
    fs::File::open(&source).and_then(|mut file| file.read_exact(&mut signature)).map_err(|_| "Invalid image file")?;
    if !signature.starts_with(b"\x89PNG\r\n\x1a\n") && !signature.starts_with(b"\xff\xd8\xff")
        && !(signature.starts_with(b"RIFF") && &signature[8..12] == b"WEBP") {
        return Err("Invalid image file".into());
    }
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    if db::get_course_by_id(&conn, course_id).map_err(|e| e.to_string())?.is_none() { return Err("Course not found".into()); }
    let folder = app.path().app_data_dir().map_err(|e| e.to_string())?.join("covers");
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let destination = folder.join(format!("{}.{}", uuid::Uuid::new_v4(), ext));
    fs::copy(&source, &destination).map_err(|e| e.to_string())?;
    if let Err(error) = conn.execute("UPDATE courses SET thumbnail_path = ?1 WHERE id = ?2", rusqlite::params![destination.to_string_lossy(), course_id]) {
        let _ = fs::remove_file(destination);
        return Err(error.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires bundled ARM ffmpeg"]
    fn creates_small_cached_preview_without_touching_readonly_video() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("cover-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let video = dir.join("lesson.ts");
        let ffmpeg = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries/ffmpeg-aarch64-apple-darwin");
        assert!(Command::new(&ffmpeg).args(["-v", "error", "-f", "lavfi", "-i", "testsrc2=size=1280x720:rate=1", "-t", "1", "-c:v", "libx264"]).arg(&video).status().unwrap().success());
        fs::set_permissions(&video, fs::Permissions::from_mode(0o444)).unwrap();
        let original = fs::read(&video).unwrap();
        let path = extract_cover(&video, &dir.join("cache"), &ffmpeg).unwrap().unwrap();
        let metadata = fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0 && metadata.len() < 200_000);
        assert_eq!(Some(path.clone()), extract_cover(&video, &dir.join("cache"), &ffmpeg).unwrap());
        assert_eq!(metadata.modified().unwrap(), fs::metadata(&path).unwrap().modified().unwrap());
        assert_eq!(original, fs::read(&video).unwrap());
        fs::remove_dir_all(dir).unwrap();
    }
}
