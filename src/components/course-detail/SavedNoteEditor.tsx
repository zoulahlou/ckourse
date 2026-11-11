import { useCallback, useEffect, useRef, useState } from "react";
import { NoteEditor } from "./NoteEditor";
import { addNote, updateNote } from "@/lib/store";
import { buildTimestampHtml } from "@/lib/format";
import type { Note } from "@/types";

type Props = {
  courseId: number; lessonId: number; lessonTitle: string; videoTime: number;
  note?: Note; onSaved: (note: Note) => void; onClose: () => void;
};

export function SavedNoteEditor(props: Props) {
  const key = `note-draft:${props.courseId}:${props.lessonId}:${props.note?.id ?? "new"}`;
  const [draft] = useState<{ content: string; note?: Note }>(() => {
    try { const stored = localStorage.getItem(key); if (stored) { const value = JSON.parse(stored); if (typeof value?.content === "string" && (!value.note || (Number.isInteger(value.note.id) && value.note.courseId === props.courseId && value.note.lessonId === props.lessonId))) return value; } } catch { /* Saved notes still work if draft storage is unavailable. */ }
    return { content: props.note?.content ?? `${buildTimestampHtml(props.videoTime)}&nbsp;`, note: props.note };
  });
  const note = useRef(draft.note);
  const latest = useRef(draft.content);
  const saved = useRef(props.note?.content ?? "");
  const queue = useRef(Promise.resolve());
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const mounted = useRef(true);
  const callbacks = useRef(props); callbacks.current = props;
  const [status, setStatus] = useState("Autosave · use #tags to organize notes");
  const save = useCallback((content: string) => {
    queue.current = queue.current.catch(() => {}).then(async () => {
      if (saved.current === content) return;
      // Do not create a note for an untouched timestamp or an empty editor.
      const plain = content.replace(/<[^>]*>/g, " ").replace(/&nbsp;/g, " ").trim();
      if (!plain || (!note.current && /^@?\d+:\d+(?::\d+)?$/.test(plain))) return;
      if (mounted.current) setStatus("Saving…");
      if (note.current) {
        await updateNote(note.current.id, content);
        note.current = { ...note.current, content, updatedAt: new Date().toISOString() };
      } else {
        note.current = await addNote(props.courseId, props.lessonId, props.lessonTitle, content);
      }
      saved.current = content;
      callbacks.current.onSaved(note.current);
      try {
        if (latest.current === content) localStorage.removeItem(key);
        else localStorage.setItem(key, JSON.stringify({ content: latest.current, note: note.current }));
      } catch { /* The database save succeeded. */ }
      if (mounted.current) setStatus(latest.current === content ? "Saved automatically" : "Unsaved changes…");
    });
    return queue.current.catch(error => { if (mounted.current) setStatus("Save failed — draft kept. Click Save to retry."); throw error; });
  }, [key, props.courseId, props.lessonId, props.lessonTitle]);
  const change = useCallback((content: string) => {
    latest.current = content;
    const plain = content.replace(/<[^>]*>/g, " ").replace(/&nbsp;/g, " ").trim();
    if (!note.current && (!plain || /^@?\d+:\d+(?::\d+)?$/.test(plain))) return;
    try { localStorage.setItem(key, JSON.stringify({ content, note: note.current })); } catch { /* Database autosave remains available. */ }
    setStatus("Unsaved changes…");
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => { void save(content).catch(() => {}); }, 700);
  }, [key, save]);
  useEffect(() => {
    mounted.current = true;
    if (draft.content !== (props.note?.content ?? "")) change(draft.content);
    return () => {
      mounted.current = false;
      if (timer.current) clearTimeout(timer.current);
      void save(latest.current).catch(() => {});
    };
  }, [change, save]);
  const finish = async () => {
    if (timer.current) clearTimeout(timer.current);
    await save(latest.current);
    callbacks.current.onClose();
  };
  return <div>
    <NoteEditor initialContent={draft.content} videoTime={props.videoTime} onChange={change} onSubmit={async content => { latest.current = content; await finish(); }} onCancel={() => { void finish().catch(() => {}); }} />
    <p role="status" className="mt-1 text-[11px] text-muted-foreground">{status}</p>
  </div>;
}
