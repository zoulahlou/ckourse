import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { ParsedCourse } from "@/types";

export async function selectCourseFolders(): Promise<string[] | null> {
  const selected = await open({
    directory: true,
    multiple: true,
    title: "Select Course Folder",
  });

  return selected == null ? null : Array.isArray(selected) ? selected : [selected];
}

export async function parseCourseFolder(
  folderPath: string
): Promise<ParsedCourse> {
  return invoke<ParsedCourse>("parse_course_folder", { folderPath });
}
