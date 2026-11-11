import { Link } from "react-router-dom";
import { useEffect, useRef, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { ImageIcon } from "@phosphor-icons/react";
import { toast } from "sonner";
import type { Course } from "@/types";

export function CourseCover({ course, href }: { course: Course; href: string }) {
  const ref = useRef<HTMLDivElement>(null);
  const [src, setSrc] = useState<string | null>(null);
  const [revision, setRevision] = useState(0);
  useEffect(() => {
    let cancelled = false;
    const load = () => { void invoke<string | null>("get_course_cover", { courseId: course.id })
      .then(path => { if (!cancelled) setSrc(path ? convertFileSrc(path) : null); }).catch(() => {}); };
    const observer = new IntersectionObserver(entries => {
      if (entries.some(e => e.isIntersecting)) { observer.disconnect(); load(); }
    }, { rootMargin: "100px" });
    if (ref.current) observer.observe(ref.current);
    return () => { cancelled = true; observer.disconnect(); };
  }, [course.id, course.thumbnailPath, revision]);
  return <div ref={ref} className="pointer-events-auto relative h-32 overflow-hidden" style={{ backgroundColor: `${course.accentColor}18` }}>
    <Link to={href} aria-label={`Open ${course.title}`} className="block h-full">
    {src ? <img src={src} alt="" className="h-full w-full object-cover" loading="lazy" onError={() => setSrc(null)} />
      : <span className="flex h-full items-center justify-center font-heading text-2xl opacity-40" style={{ color: course.accentColor }}>{course.title.split(/\s/)[0]}</span>}
    </Link>
    <button aria-label={`Choose cover for ${course.title}`} title="Choose cover image" className="absolute bottom-2 right-2 rounded-md bg-black/60 p-2 text-white hover:bg-black/80 focus-visible:outline-2"
      onClick={async e => {
        e.preventDefault(); e.stopPropagation();
        try {
          const path = await open({ multiple: false, filters: [{ name: "Images", extensions: ["jpg", "jpeg", "png", "webp"] }] });
          if (typeof path === "string") { await invoke("set_course_cover", { courseId: course.id, path }); setRevision(r => r + 1); }
        } catch (error) { toast.error(String(error)); }
      }}><ImageIcon className="size-4" /></button>
  </div>;
}
