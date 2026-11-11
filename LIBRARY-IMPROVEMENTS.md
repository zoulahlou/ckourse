# Library improvements — 1.2.1-ts.3

This local Mac build extends the user's ts.2 installation, preserving its playback
optimization, database format, course/lesson IDs and original source paths.

- Dashboard cards use local preview frames from each course's first video. Only
  visible cards request extraction, which is serialized and cached. Existing
  course artwork is preferred. A card's image button accepts a JPG/PNG/WebP;
  the selected image is copied into Application Support so moving that image
  later does not break the chosen cover. Source folders are never written.
- A prominent Continue learning banner opens the most recently watched unfinished
  course. The existing lesson and saved-time resume behavior is reused.
- Notes save after 700 ms of inactivity and flush when leaving an editor. New
  notes start with a clickable video timestamp. Subsequent changes update that
  same note. Save status and errors are visible; local drafts remain available
  after failed writes. Closing an editor saves it. Tags are written inline as
  `#nutrition` and can be filtered with the existing Notes search or the new
  lesson-note filter. There is no separate tag database or tag administration.
- Global sidebar search includes courses/authors, lessons and note contents/tags.
  A note result opens its course and lesson and scrolls to that note. Results from
  outdated searches are discarded. `/` or Cmd/Ctrl+K opens search outside editors.
- Add formations via the native folder drop on the Import screen or select several
  folders with Browse. Review the detected structure and settings for each, then
  import it to proceed to the next. Invalid or duplicate folders can be skipped.
  Duplicate protection also runs in the backend while holding the database lock;
  it recognizes canonical local paths and preserves the existing course data.

Build with `npm ci && npm run build:mac-arm64`. The local app uses an ad-hoc
signature, not Developer ID signing/notarization. Installation: quit ckourse,
open the DMG and replace the app in Applications. Existing notes and progress
are kept; no reimport or manual conversion is required.

Checks: production frontend/native build, Rust library tests, frontend regression
tests for autosave/error recovery/queued imports/native drop callback/note search/
cover accessibility, native FFmpeg preview extraction from a read-only file with
cache reuse, and visual dashboard inspection using demo data in a browser.
The automatic-preview media test is explicitly run with:

    cargo test --manifest-path src-tauri/Cargo.toml creates_small_cached_preview -- --ignored

Not covered by a real Finder end-to-end test: dragging folders from Finder into
the packaged app. The native drop event and resulting import queue are tested
with fixtures. Existing TS preparation/WebKit tests remain in the repository.


## Changing the course folder — 1.2.1-ts.4

In Edit Course, choose **Change course folder**, select the new root, then click
**Apply new folder**. The original folder may already be disconnected or gone.
This reconnects the existing course; it does not move or copy any video files.

Relative filenames and subfolders must stay the same. All lesson videos must be
present, nonempty and readable in the selected folder; referenced local subtitles
and resources must exist as well. Wrong/incomplete folders or folders already
assigned to another course are rejected before changing anything.
The DB updates course, lesson, subtitle and resource paths in one transaction,
including embedded-subtitle suffixes. IDs, timestamps, notes, favorites,
bookmarks, completion and saved playback positions stay unchanged. Chosen covers
outside the original course directory remain in Application Support. Derived
video/cover caches can be regenerated for the new source path.

Background preparation of the next lesson is deliberately not included, as
requested. Tests cover reconnecting a missing root, missing-video/resource
rollback, embedded subtitles, stable learning data and the folder-selection UI.
