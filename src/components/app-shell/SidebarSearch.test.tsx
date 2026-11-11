import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { MemoryRouter, useLocation } from "react-router-dom";
import { SidebarSearch } from "./SidebarSearch";
const { search } = vi.hoisted(() => ({ search: vi.fn() }));
vi.mock("@/lib/store", () => ({ searchContent: search }));
afterEach(cleanup);
function Location() { const l = useLocation(); return <output>{l.pathname + l.search}</output>; }
it("finds notes and opens their lesson and note directly", async () => {
  search.mockResolvedValue([{ kind: "note", courseId: 1, courseTitle: "Training", accentColor: "#fff", lessonId: 2, lessonTitle: "Lesson", noteId: 7, snippet: "<b>Important #tag</b>" }]);
  const view = render(<MemoryRouter><SidebarSearch collapsed={false} /><Location /></MemoryRouter>);
  fireEvent.click(view.getByText("Search"));
  fireEvent.change(view.getByPlaceholderText("Search courses, lessons and notes…"), { target: { value: "#tag" } });
  await waitFor(() => expect(view.getByText("Note · Lesson")).toBeTruthy());
  fireEvent.pointerDown(view.getByText("Note · Lesson"));
  expect(view.getByText("/course/1?lesson=2&note=7")).toBeTruthy();
});
