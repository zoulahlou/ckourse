import {
  NotePencilIcon as NotePencil,
  PencilSimpleIcon as PencilSimple,
  TrashIcon as Trash,
} from "@phosphor-icons/react";
import { SavedNoteEditor } from "./SavedNoteEditor";
import { useState, useEffect } from "react";
import { SNAPPY } from "@/lib/constants";
import type { Note } from "@/types";

interface NotesPanelProps {
  notes: Note[];
  videoTime: number;
  editingNoteId: number | null;
  showEditor: boolean;
  courseId: number;
  lessonId: number;
  lessonTitle: string;
  onSaved: (note: Note) => void;
  onDelete: (noteId: number) => void;
  onSetEditing: (id: number | null) => void;
  onSetShowEditor: (show: boolean) => void;
  onTimestampClick?: (seconds: number, lessonId: number) => void;
}

export function NotesPanel({
  notes,
  videoTime,
  editingNoteId,
  showEditor,
  courseId, lessonId, lessonTitle, onSaved,
  onDelete,
  onSetEditing,
  onSetShowEditor,
  onTimestampClick,
}: NotesPanelProps) {
  const [query, setQuery] = useState("");
  const [draftId, setDraftId] = useState<number | null>(null);
  useEffect(() => { setDraftId(null); }, [showEditor, lessonId]);
  return (
    <div className="flex flex-col gap-3">
      {showEditor ? (
        <SavedNoteEditor key={lessonId} courseId={courseId} lessonId={lessonId} lessonTitle={lessonTitle}
          videoTime={videoTime} onSaved={note => { setDraftId(note.id); onSaved(note); }} onClose={() => onSetShowEditor(false)} />
      ) : (
        <button
          onClick={() => onSetShowEditor(true)}
          className="flex items-center gap-2 rounded-lg border border-dashed border-border px-3 py-2.5 font-sans text-xs text-muted-foreground transition-colors hover:border-border hover:bg-secondary hover:text-foreground"
          style={{ transitionTimingFunction: SNAPPY }}
        >
          <NotePencil className="size-3.5" />
          Add a note...
        </button>
      )}

      {notes.length === 0 && !showEditor && (
        <p className="py-4 text-center font-sans text-xs text-muted-foreground/60">
          No notes yet. Start capturing your thoughts.
        </p>
      )}

      <input aria-label="Filter notes or tags" placeholder="Filter notes or #tags…" value={query} onChange={e => setQuery(e.target.value)} className="rounded-lg border border-border bg-transparent p-2 text-xs" />
      {notes.filter(n => !(showEditor && n.id === draftId)).filter(n => n.content.replace(/<[^>]*>/g, " ").toLowerCase().includes(query.toLowerCase())).map((note) => (
        <NoteCard
          key={note.id}
          note={note}
          videoTime={videoTime}
          isEditing={editingNoteId === note.id}
          onSaved={onSaved}
          onDelete={onDelete}
          onStartEdit={() => onSetEditing(note.id)}
          onCancelEdit={() => onSetEditing(null)}
          onTimestampClick={onTimestampClick}
        />
      ))}
    </div>
  );
}

interface NoteCardProps {
  note: Note;
  videoTime: number;
  isEditing: boolean;
  onSaved: (note: Note) => void;
  onDelete: (noteId: number) => void;
  onStartEdit: () => void;
  onCancelEdit: () => void;
  onTimestampClick?: (seconds: number, lessonId: number) => void;
}

function NoteCard({
  note,
  videoTime,
  isEditing,
  onSaved,
  onDelete,
  onStartEdit,
  onCancelEdit,
  onTimestampClick,
}: NoteCardProps) {
  if (isEditing) {
    return (
      <SavedNoteEditor note={note} courseId={note.courseId} lessonId={note.lessonId} lessonTitle={note.lessonTitle}
        videoTime={videoTime} onSaved={onSaved} onClose={onCancelEdit} />
    );
  }

  const date = new Date(note.updatedAt);
  const formatted = date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
  });

  return (
    <div
      id={`note-${note.id}`} className="group rounded-lg border border-border bg-card px-3 py-2.5 transition-colors hover:bg-secondary/50"
      style={{ transitionTimingFunction: SNAPPY }}
    >
      <div className="flex items-start gap-2">
        <div
          className="note-content flex-1 font-sans text-xs leading-relaxed text-foreground/90"
          dangerouslySetInnerHTML={{ __html: note.content }}
          onClick={(e) => {
            const target = e.target as HTMLElement;
            if (target.classList.contains("note-timestamp")) {
              const seconds = Number(target.dataset.timestamp);
              if (!isNaN(seconds) && onTimestampClick) {
                onTimestampClick(seconds, note.lessonId);
              }
            }
          }}
        />
        <div className="flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
          <button
            onClick={onStartEdit}
            className="rounded-md p-1 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          >
            <PencilSimple className="size-3" />
          </button>
          <button
            onClick={() => onDelete(note.id)}
            className="rounded-md p-1 text-muted-foreground transition-colors hover:bg-destructive/15 hover:text-destructive"
          >
            <Trash className="size-3" />
          </button>
        </div>
      </div>

      <div className="mt-1.5 flex items-center gap-2">
        <span className="font-mono text-[10px] text-muted-foreground/50">
          {note.lessonTitle}
        </span>
        <span className="font-mono text-[10px] text-muted-foreground/50">
          {formatted}
        </span>
      </div>
    </div>
  );
}
