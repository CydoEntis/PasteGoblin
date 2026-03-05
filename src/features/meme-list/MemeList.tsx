import { useState, useEffect } from "react";
import type { Meme } from "../../shared/types";
import { MemeImage } from "../../shared/MemeImage";
import { ChevronLeft, ChevronRight } from "lucide-react";
import logo from "../../assets/logo.png";
import "./meme-list.css";

const PAGE_SIZE = 20;

interface MemeListProps {
  memes: Meme[];
  recentlyUsed: Meme[];
  selectedId: string | null;
  onSelect: (meme: Meme) => void;
}

function MemeListItem({ meme, selected, onSelect }: { meme: Meme; selected: boolean; onSelect: () => void }) {
  return (
    <div
      className={`meme-list-item ${selected ? "selected" : ""}`}
      onClick={onSelect}
    >
      {meme.mime.startsWith("image/") ? (
        <MemeImage
          className="meme-list-item-thumb"
          path={meme.stored_path}
          alt={meme.name}
        />
      ) : (
        <div className="meme-list-item-thumb" style={{
          display: "flex", alignItems: "center", justifyContent: "center",
          fontSize: 20, color: "var(--text-muted)"
        }}>
          &#x1F3AC;
        </div>
      )}
      <div className="meme-list-item-info">
        <div className="meme-list-item-name">{meme.name}</div>
      </div>
    </div>
  );
}

export function MemeList({ memes, recentlyUsed, selectedId, onSelect }: MemeListProps) {
  const [page, setPage] = useState(0);

  // Reset to first page when meme list changes (e.g. new upload, search)
  useEffect(() => {
    setPage(0);
  }, [memes.length]);

  if (memes.length === 0) {
    return (
      <div className="meme-list">
        <div className="meme-list-empty">
          <img src={logo} alt="Paste Goblin" className="meme-list-empty-icon" style={{ width: 120, height: 120, objectFit: "contain" }} />
          <p>No memes yet</p>
          <p style={{ fontSize: 12 }}>Click Upload to add your first meme</p>
        </div>
      </div>
    );
  }

  const recentIds = new Set(recentlyUsed.map((m) => m.id));
  const allMemes = memes.filter((m) => !recentIds.has(m.id) || recentlyUsed.length === 0);
  const totalPages = Math.ceil(allMemes.length / PAGE_SIZE);
  const safePage = Math.min(page, totalPages - 1);
  const pagedMemes = allMemes.slice(safePage * PAGE_SIZE, (safePage + 1) * PAGE_SIZE);

  return (
    <div className="meme-list">
      {recentlyUsed.length > 0 && (
        <>
          <div className="meme-list-section-header">Recently Used</div>
          {recentlyUsed.map((meme) => (
            <MemeListItem
              key={`recent-${meme.id}`}
              meme={meme}
              selected={selectedId === meme.id}
              onSelect={() => onSelect(meme)}
            />
          ))}
          <div className="meme-list-section-header">All Memes</div>
        </>
      )}
      {pagedMemes.map((meme) => (
        <MemeListItem
          key={meme.id}
          meme={meme}
          selected={selectedId === meme.id}
          onSelect={() => onSelect(meme)}
        />
      ))}
      {totalPages > 1 && (
        <div className="meme-list-pagination">
          <button
            className="pagination-btn"
            disabled={safePage === 0}
            onClick={() => setPage(safePage - 1)}
          >
            <ChevronLeft size={14} />
          </button>
          <span className="pagination-info">
            {safePage + 1} / {totalPages}
          </span>
          <button
            className="pagination-btn"
            disabled={safePage >= totalPages - 1}
            onClick={() => setPage(safePage + 1)}
          >
            <ChevronRight size={14} />
          </button>
        </div>
      )}
    </div>
  );
}
