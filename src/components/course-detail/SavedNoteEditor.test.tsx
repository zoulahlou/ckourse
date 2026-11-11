import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { SavedNoteEditor } from "./SavedNoteEditor";
const { add, update } = vi.hoisted(() => ({ add: vi.fn(), update: vi.fn() }));
vi.mock("@/lib/store", () => ({ addNote: add, updateNote: update }));
const note = { id: 7, courseId: 1, lessonId: 2, lessonTitle: "Lesson", content: "", createdAt: "now", updatedAt: "now" };
beforeEach(() => { localStorage.clear(); add.mockReset(); update.mockReset(); add.mockImplementation(async (_c, _l, _t, content) => ({ ...note, content })); update.mockResolvedValue(undefined); document.queryCommandState = () => false; });
afterEach(cleanup);
function editor() {
  const onSaved = vi.fn(), onClose = vi.fn();
  return { ...render(<SavedNoteEditor courseId={1} lessonId={2} lessonTitle="Lesson" videoTime={42} onSaved={onSaved} onClose={onClose} />), onSaved, onClose };
}
async function type(el: HTMLElement, html: string) { await act(async () => { el.innerHTML = html; fireEvent.input(el); }); }
it("autosaves one timestamped note then updates it, without duplicating it", async () => {
  const view = editor(); const el = view.getByRole("textbox");
  expect(el.innerHTML).toContain('data-timestamp="42"');
  await type(el, el.innerHTML + "Important #nutrition");
  await waitFor(() => expect(add).toHaveBeenCalledTimes(1), { timeout: 2000 });
  await type(el, el.innerHTML + " updated");
  await waitFor(() => expect(update).toHaveBeenCalledTimes(1), { timeout: 2000 });
  fireEvent.mouseDown(view.getByText("Save"));
  await waitFor(() => expect(view.onClose).toHaveBeenCalled());
  expect(add).toHaveBeenCalledTimes(1);
});
it("keeps failed content recoverable and retries the save", async () => {
  add.mockRejectedValueOnce(new Error("disk full"));
  const view = editor(); await type(view.getByRole("textbox"), "Never lose this #tag");
  await waitFor(() => expect(view.getByRole("status").textContent).toContain("Save failed"), { timeout: 2000 });
  expect(localStorage.getItem("note-draft:1:2:new")).toContain("Never lose this");
  fireEvent.mouseDown(view.getByText("Save"));
  await waitFor(() => expect(view.onClose).toHaveBeenCalled());
  expect(localStorage.getItem("note-draft:1:2:new")).toBeNull();
});
it("flushes pending changes when changing lessons", async () => {
  const view = editor(); await type(view.getByRole("textbox"), "Pending note");
  view.unmount();
  await waitFor(() => expect(add).toHaveBeenCalledWith(1, 2, "Lesson", "Pending note"));
});
