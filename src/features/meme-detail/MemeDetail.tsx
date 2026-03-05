import { Copy, Download, Edit2, Trash2 } from "lucide-react";
import type { Meme } from "../../shared/types";
import { MemeImage } from "../../shared/MemeImage";
import logo from "../../assets/logo.png";
import "./meme-detail.css";

interface MemeDetailProps {
  meme: Meme | null;
  onEdit: (meme: Meme) => void;
  onDelete: (meme: Meme) => void;
  onCopy: (meme: Meme) => void;
  onDownload: (meme: Meme) => void;
}

export function MemeDetail({ meme, onEdit, onDelete, onCopy, onDownload }: MemeDetailProps) {
  if (!meme) {
    return (
      <div className="meme-detail">
        <div className="meme-detail-empty">
          <img src={logo} alt="" style={{ width: 120, height: 120, objectFit: "contain", opacity: 0.4 }} />
          <p>Select a meme to view details</p>
        </div>
      </div>
    );
  }

  const isImage = meme.mime.startsWith("image/");
  const isVideo = meme.mime.startsWith("video/");

  return (
    <div className="meme-detail">
      <div className="meme-detail-header">
        <div>
          <div className="meme-detail-title">{meme.name}</div>
          {meme.use_count > 0 && (
            <div className="meme-detail-meta">Used {meme.use_count} time{meme.use_count !== 1 ? "s" : ""}</div>
          )}
        </div>
        <button className="btn btn-ghost" onClick={() => onEdit(meme)} title="Edit">
          <Edit2 size={16} />
        </button>
      </div>

      <div className="meme-detail-preview">
        {(isImage || isVideo) && (
          <MemeImage path={meme.stored_path} alt={meme.name} />
        )}
      </div>

      <div className="meme-detail-actions">
        <button className="meme-detail-action primary" onClick={() => onCopy(meme)}>
          <span className="action-label"><Copy size={16} /> Copy to Clipboard</span>
          <span className="action-shortcut">Ctrl+C</span>
        </button>
        <button className="meme-detail-action" onClick={() => onDownload(meme)}>
          <span className="action-label"><Download size={16} /> Download Image</span>
          <span className="action-shortcut">Ctrl+S</span>
        </button>
        <button className="meme-detail-action danger" onClick={() => onDelete(meme)}>
          <span className="action-label"><Trash2 size={16} /> Delete Meme</span>
          <span className="action-shortcut">Ctrl+Del</span>
        </button>
      </div>
    </div>
  );
}
