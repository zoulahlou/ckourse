import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { CourseCard } from "./CourseCard";
import type { Course } from "@/types";
vi.mock("@tauri-apps/api/core", () => ({ invoke: async () => "/preview.jpg", convertFileSrc: (p: string) => p }));
vi.mock("@/lib/store", () => ({ toggleBookmark: vi.fn() }));
class Observer {
  constructor(private cb: (entries: { isIntersecting: boolean }[]) => void) {}
  observe() { this.cb([{ isIntersecting: true }]); }
  disconnect() {}
}
vi.stubGlobal("IntersectionObserver", Observer);
afterEach(cleanup);
it("loads a preview with a separate accessible image chooser and keeps course navigation", async () => {
  const course: Course = { id: 1, title: "Training", author: "Author", folderPath: "/course", completedLessons: 0, totalLessons: 0, status: "not-started", accentColor: "#fff", lastWatched: null, category: "other", description: null, thumbnailPath: null, bookmarked: false };
  const view = render(<MemoryRouter><CourseCard course={course} /></MemoryRouter>);
  await waitFor(() => expect(view.container.querySelector("img")?.getAttribute("src")).toBe("/preview.jpg"));
  expect(view.getByRole("button", { name: "Choose cover for Training" }).closest("a")).toBeNull();
  expect(view.getByRole("link", { name: "Open Training" }).getAttribute("href")).toContain("/course/1");
  expect(view.getByText("0%")).toBeTruthy();
});
