use std::path::Path;

use crate::db::{self, DbState, SaveCourseInput};
use crate::parser;

#[tauri::command]
pub async fn parse_course_folder(folder_path: String) -> Result<parser::ParsedCourse, String> {
    tauri::async_runtime::spawn_blocking(move || parser::parse_folder(Path::new(&folder_path)))
        .await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn import_course(
    state: tauri::State<'_, DbState>,
    mut parsed: parser::ParsedCourse,
    config: SaveCourseInput,
) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    ensure_new_folder(&conn, &mut parsed)?;
    db::save_parsed_course(&conn, &parsed, &config).map_err(|e| e.to_string())
}

fn ensure_new_folder(conn: &rusqlite::Connection, parsed: &mut parser::ParsedCourse) -> Result<(), String> {
    if let Ok(path) = Path::new(&parsed.folder_path).canonicalize() {
        parsed.folder_path = path.to_string_lossy().into_owned();
    }
    let duplicate = db::get_all_courses(&conn).map_err(|e| e.to_string())?.iter().any(|c| {
        c.folder_path == parsed.folder_path || Path::new(&c.folder_path).canonicalize().is_ok_and(|p| p.to_string_lossy() == parsed.folder_path)
    });
    if duplicate { return Err("This folder is already in your library. Your existing notes and progress are preserved.".into()); }
    Ok(())
}

#[tauri::command]
pub fn get_courses(state: tauri::State<'_, DbState>) -> Result<Vec<db::Course>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_all_courses(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_course(state: tauri::State<'_, DbState>, course_id: i64) -> Result<Option<db::Course>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_course_by_id(&conn, course_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_course_detail(
    state: tauri::State<'_, DbState>,
    course_id: i64,
) -> Result<Option<db::CourseDetail>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_course_detail(&conn, course_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_course(
    state: tauri::State<'_, DbState>,
    course_id: i64,
    title: String,
    author: String,
    accent_color: String,
    category: String,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::update_course(&conn, course_id, &title, &author, &accent_color, &category)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_course_progress(state: tauri::State<'_, DbState>, course_id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::reset_course_progress(&conn, course_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_course(state: tauri::State<'_, DbState>, course_id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_course(&conn, course_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_bookmarked_courses(
    state: tauri::State<'_, DbState>,
) -> Result<Vec<db::Course>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_bookmarked_courses(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_bookmark(
    state: tauri::State<'_, DbState>,
    course_id: i64,
) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::toggle_bookmark(&conn, course_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_dashboard_stats(
    state: tauri::State<'_, DbState>,
) -> Result<db::DashboardStats, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_dashboard_stats(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_progress_data(
    state: tauri::State<'_, DbState>,
) -> Result<db::ProgressData, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_progress_data(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_all_data(state: tauri::State<'_, DbState>) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_all_data(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_custom_categories(state: tauri::State<'_, DbState>) -> Result<Vec<String>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::get_custom_categories(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_custom_category(state: tauri::State<'_, DbState>, name: String) -> Result<(), String> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err("Category name cannot be empty".to_string());
    }
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::add_custom_category(&conn, &trimmed).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_custom_category(state: tauri::State<'_, DbState>, name: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::delete_custom_category(&conn, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_content(
    state: tauri::State<'_, DbState>,
    query: String,
) -> Result<Vec<db::SearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    db::search_content(&conn, &query).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_folder_does_not_replace_existing_learning_data() {
        let dir = std::env::temp_dir().join(format!("import-test-{}", uuid::Uuid::new_v4()));
        let conn = db::init_db(&dir).unwrap();
        let mut parsed = parser::ParsedCourse { title: "Training".into(), folder_path: dir.to_string_lossy().into(), description: None, thumbnail_path: None, sections: vec![], resources: vec![], confidence: parser::Confidence::High, confidence_reasons: vec![], total_video_count: 0 };
        ensure_new_folder(&conn, &mut parsed).unwrap();
        let id = db::save_parsed_course(&conn, &parsed, &db::SaveCourseInput { title: "Training".into(), author: "Author".into(), accent_color: "#fff".into(), category: "other".into() }).unwrap();
        db::toggle_bookmark(&conn, id).unwrap();
        parsed.folder_path.push_str("/./");
        assert!(ensure_new_folder(&conn, &mut parsed).unwrap_err().contains("already in your library"));
        assert_eq!(db::get_all_courses(&conn).unwrap().len(), 1);
        assert!(db::get_course_by_id(&conn, id).unwrap().unwrap().bookmarked);
        drop(conn);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
