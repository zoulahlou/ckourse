use crate::db::{self, DbState};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use tauri::Manager;

// Change only the root: relative filenames and stable lesson IDs stay intact.
fn remap(path: &str, old: &Path, new: &Path) -> Option<String> {
    let (base, suffix) = match path.rsplit_once("#subtitle:") {
        Some((base, index)) if index.parse::<u32>().is_ok() => (base, format!("#subtitle:{index}")),
        _ => (path, String::new()),
    };
    let relative = Path::new(base).strip_prefix(old).ok()?;
    Some(format!("{}{suffix}", new.join(relative).to_string_lossy()))
}

fn relocate(conn: &mut Connection, course_id: i64, folder: &Path) -> Result<String, String> {
    let course = db::get_course_by_id(conn, course_id).map_err(|e| e.to_string())?.ok_or("Course not found")?;
    let old = Path::new(&course.folder_path);
    if !old.is_absolute() { return Err("Changing folders is available for local courses only".into()); }
    let new = folder.canonicalize().map_err(|_| "The selected folder is not accessible")?;
    if !new.is_dir() { return Err("Select a course folder".into()); }
    let duplicate = db::get_all_courses(conn).map_err(|e| e.to_string())?.iter().any(|c| c.id != course_id &&
        (Path::new(&c.folder_path) == new || Path::new(&c.folder_path).canonicalize().is_ok_and(|p| p == new)));
    if duplicate { return Err("That folder already belongs to another course".into()); }
    let detail = db::get_course_detail(conn, course_id).map_err(|e| e.to_string())?.ok_or("Course not found")?;
    let mut videos = Vec::new();
    for lesson in detail.sections.iter().flat_map(|s| &s.lessons) {
        let path = remap(&lesson.video_path, old, &new).ok_or("A lesson is outside the original course folder")?;
        if !std::fs::metadata(&path).is_ok_and(|m| m.is_file() && m.len() > 0) || std::fs::File::open(&path).is_err() {
            return Err(format!("Video not found: {}. Choose the course root with the same filenames and subfolders. No paths were changed.", Path::new(&lesson.video_path).strip_prefix(old).unwrap_or(Path::new(&lesson.video_path)).display()));
        }
        videos.push((lesson.id, path));
    }
    if videos.is_empty() { return Err("This course has no lessons to reconnect".into()); }
    // Read all replacements before writing; a missing file must leave the DB intact.
    let mut assets: Vec<(&str, i64, String)> = Vec::new();
    for (table, query) in [
        ("subtitles", "SELECT t.id,t.path FROM subtitles t JOIN lessons l ON l.id=t.lesson_id JOIN sections s ON s.id=l.section_id WHERE s.course_id=?1"),
        ("resources", "SELECT id,path FROM resources WHERE course_id=?1"),
    ] {
        let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([course_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))).map_err(|e| e.to_string())?;
        for row in rows {
            let (id, path) = row.map_err(|e| e.to_string())?;
            if let Some(mapped) = remap(&path, old, &new) {
                let base = mapped.rsplit_once("#subtitle:").filter(|(_, n)| n.parse::<u32>().is_ok()).map_or(mapped.as_str(), |(base, _)| base);
                if !Path::new(base).exists() { return Err(format!("Missing subtitle or resource: {base}. No paths were changed.")); }
                assets.push((table, id, mapped));
            }
        }
    }
    let cover = course.thumbnail_path.as_ref().map(|p| remap(p, old, &new).unwrap_or_else(|| p.clone()));
    let folder = new.to_string_lossy().into_owned();
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    for (id, path) in videos { tx.execute("UPDATE lessons SET video_path=?1 WHERE id=?2", params![path, id]).map_err(|e| e.to_string())?; }
    for (table, id, path) in assets {
        tx.execute(&format!("UPDATE {table} SET path=?1 WHERE id=?2"), params![path, id]).map_err(|e| e.to_string())?;
    }
    tx.execute("UPDATE courses SET folder_path=?1,thumbnail_path=?2 WHERE id=?3", params![folder, cover, course_id]).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(folder)
}

#[tauri::command]
pub async fn relocate_course(app: tauri::AppHandle, course_id: i64, folder_path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DbState>();
        let mut conn = state.conn.lock().map_err(|e| e.to_string())?;
        relocate(&mut conn, course_id, &PathBuf::from(folder_path))
    }).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reconnects_missing_root_atomically_and_preserves_learning_data() {
        let dir = std::env::temp_dir().join(format!("relink-{}", uuid::Uuid::new_v4()));
        let old = dir.join("missing course");
        let new = dir.join("new #course");
        std::fs::create_dir_all(new.join("Section")).unwrap();
        let mut conn = db::init_db(&dir.join("db")).unwrap();
        conn.execute("INSERT INTO courses(id,title,folder_path,created_at,updated_at) VALUES(1,'Training',?1,'now','now')", [old.to_str().unwrap()]).unwrap();
        conn.execute("INSERT INTO sections(id,course_id,title) VALUES(1,1,'Section')", []).unwrap();
        conn.execute("INSERT INTO lessons(id,section_id,title,video_path,last_position,completed,is_last_watched) VALUES(1,1,'Lesson',?1,42.5,1,1)", [old.join("Section/a.ts").to_str().unwrap()]).unwrap();
        conn.execute("INSERT INTO subtitles(lesson_id,path) VALUES(1,?1)", [format!("{}#subtitle:2", old.join("Section/a.ts").display())]).unwrap();
        conn.execute("INSERT INTO resources(course_id,path,title) VALUES(1,?1,'PDF')", [old.join("guide.pdf").to_str().unwrap()]).unwrap();
        let note = db::add_note(&conn, 1, 1, "Lesson", "Timestamp 42").unwrap();
        db::toggle_bookmark(&conn, 1).unwrap();
        db::toggle_favorite(&conn, 1).unwrap();
        assert!(relocate(&mut conn, 1, &new).is_err());
        assert_eq!(db::get_course_by_id(&conn, 1).unwrap().unwrap().folder_path, old.to_string_lossy());
        std::fs::write(new.join("Section/a.ts"), b"video").unwrap();
        // A missing resource also aborts without partially updating the video.
        assert!(relocate(&mut conn, 1, &new).is_err());
        assert_eq!(db::get_course_detail(&conn, 1).unwrap().unwrap().sections[0].lessons[0].video_path, old.join("Section/a.ts").to_string_lossy());
        std::fs::write(new.join("guide.pdf"), b"pdf").unwrap();
        let folder = relocate(&mut conn, 1, &new).unwrap();
        let detail = db::get_course_detail(&conn, 1).unwrap().unwrap();
        let lesson = &detail.sections[0].lessons[0];
        assert_eq!(lesson.id, 1); assert_eq!(lesson.last_position, 42.5);
        assert!(lesson.completed && lesson.is_last_watched && lesson.favorited);
        assert!(lesson.video_path.starts_with(&folder));
        assert_eq!(db::get_course_notes(&conn, 1).unwrap()[0].id, note.id);
        assert!(db::get_course_by_id(&conn, 1).unwrap().unwrap().bookmarked);
        assert_eq!(db::get_lesson_subtitles(&conn, 1).unwrap()[0].path, format!("{folder}/Section/a.ts#subtitle:2"));
        assert_eq!(remap("/course-other/file.ts", Path::new("/course"), &new), None);
        drop(conn); std::fs::remove_dir_all(dir).unwrap();
    }
}
