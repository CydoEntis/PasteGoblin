import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Fuse from "fuse.js";
import type { Meme } from "../../shared/types";
import { Header } from "./Header";
import { StatusBar } from "./StatusBar";
import { MemeList } from "../meme-list/MemeList";
import { MemeDetail } from "../meme-detail/MemeDetail";
import { UploadModal } from "../upload/UploadModal";
import { getFileUrl } from "../../shared/file-url";
import "./layout.css";

export function MainWindow() {
  const [memes, setMemes] = useState<Meme[]>([]);
  const [recentlyUsed, setRecentlyUsed] = useState<Meme[]>([]);
  const [selectedMeme, setSelectedMeme] = useState<Meme | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [showUpload, setShowUpload] = useState(false);
  const [editMeme, setEditMeme] = useState<Meme | null>(null);
  const [copyFeedback, setCopyFeedback] = useState(false);
  const fuseRef = useRef<Fuse<Meme> | null>(null);

  const loadData = useCallback(async () => {
    try {
      const [memeData, recentData] = await Promise.all([
        invoke<Meme[]>("cmd_get_all_memes"),
        invoke<Meme[]>("cmd_get_recently_used", { limit: 2 }),
      ]);
      setMemes(memeData);
      setRecentlyUsed(recentData);
      fuseRef.current = new Fuse(memeData, {
        keys: ["name"],
        threshold: 0.4,
      });
    } catch (e) {
      console.error("Failed to load data:", e);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  let filteredMemes = memes;
  if (searchQuery.trim()) {
    if (fuseRef.current) {
      const results = fuseRef.current.search(searchQuery);
      const resultIds = new Set(results.map((r) => r.item.id));
      filteredMemes = filteredMemes.filter((m) => resultIds.has(m.id));
    }
  }

  const handleCopied = () => {
    setCopyFeedback(true);
    setTimeout(() => setCopyFeedback(false), 1500);
    loadData();
  };

  const handleCopy = async (meme: Meme) => {
    try {
      await invoke("cmd_copy_to_clipboard", { id: meme.id });
      handleCopied();
    } catch (e) {
      console.error("Copy failed:", e);
    }
  };

  const handleDownload = async (meme: Meme) => {
    try {
      const url = await getFileUrl(meme.stored_path);
      const link = document.createElement("a");
      link.href = url;
      link.download = meme.original_filename;
      link.click();
    } catch (e) {
      console.error("Download failed:", e);
    }
  };

  const handleDelete = async (meme: Meme) => {
    try {
      await invoke("cmd_delete_meme", { id: meme.id });
      if (selectedMeme?.id === meme.id) setSelectedMeme(null);
      loadData();
    } catch (e) {
      console.error("Delete failed:", e);
    }
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (showUpload || editMeme) {
          setShowUpload(false);
          setEditMeme(null);
        } else {
          getCurrentWindow().hide();
        }
        return;
      }

      if (showUpload || editMeme) return;
      const tag = (e.target as HTMLElement).tagName;
      if (tag === "INPUT" || tag === "TEXTAREA") return;

      if (e.ctrlKey && e.key === "c" && selectedMeme) {
        e.preventDefault();
        handleCopy(selectedMeme);
        return;
      }

      if (e.ctrlKey && e.key === "s" && selectedMeme) {
        e.preventDefault();
        handleDownload(selectedMeme);
        return;
      }

      if (e.ctrlKey && (e.key === "Delete" || e.key === "Backspace") && selectedMeme) {
        e.preventDefault();
        handleDelete(selectedMeme);
        return;
      }

      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        const currentIdx = selectedMeme
          ? filteredMemes.findIndex((m) => m.id === selectedMeme.id)
          : -1;
        let nextIdx: number;
        if (e.key === "ArrowDown") {
          nextIdx = currentIdx < filteredMemes.length - 1 ? currentIdx + 1 : 0;
        } else {
          nextIdx = currentIdx > 0 ? currentIdx - 1 : filteredMemes.length - 1;
        }
        if (filteredMemes[nextIdx]) {
          setSelectedMeme(filteredMemes[nextIdx]);
        }
      }

      if (e.key === "Enter" && selectedMeme) {
        e.preventDefault();
        handleCopy(selectedMeme);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [filteredMemes, selectedMeme, showUpload, editMeme]);

  const showRecent = !searchQuery.trim() && recentlyUsed.length > 0;

  return (
    <div className="main-window">
      <Header
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        memeCount={memes.length}
        onUploadClick={() => setShowUpload(true)}
      />
      <div className="split-pane">
        <MemeList
          memes={filteredMemes}
          recentlyUsed={showRecent ? recentlyUsed : []}
          selectedId={selectedMeme?.id ?? null}
          onSelect={setSelectedMeme}
        />
        <MemeDetail
          meme={selectedMeme}
          onEdit={(m) => setEditMeme(m)}
          onDelete={handleDelete}
          onCopy={handleCopy}
          onDownload={handleDownload}
        />
      </div>
      <StatusBar memeCount={memes.length} />

      {copyFeedback && (
        <div style={{
          position: "fixed", bottom: 48, left: "50%", transform: "translateX(-50%)",
          background: "var(--green)", color: "var(--bg-base)", padding: "8px 20px",
          borderRadius: "var(--radius)", fontSize: 13, fontWeight: 500, zIndex: 200,
        }}>
          Copied to clipboard!
        </div>
      )}

      {(showUpload || editMeme) && (
        <UploadModal
          editMeme={editMeme}
          onClose={() => { setShowUpload(false); setEditMeme(null); }}
          onSaved={loadData}
        />
      )}
    </div>
  );
}
