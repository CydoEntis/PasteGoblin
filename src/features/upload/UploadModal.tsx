import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { X, Upload, ImagePlus } from "lucide-react";
import type { Meme } from "../../shared/types";
import { MemeImage } from "../../shared/MemeImage";
import "./upload.css";

interface UploadModalProps {
  editMeme?: Meme | null;
  onClose: () => void;
  onSaved: () => void;
}

export function UploadModal({ editMeme, onClose, onSaved }: UploadModalProps) {
  const isEdit = !!editMeme;
  const [newFilePath, setNewFilePath] = useState<string>("");
  const [previewUrl, setPreviewUrl] = useState<string>("");
  const [name, setName] = useState(editMeme?.name ?? "");
  const [dragOver, setDragOver] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const hasNewFile = !!newFilePath;
  const currentPath = hasNewFile ? newFilePath : (editMeme?.stored_path ?? "");
  const isImage = currentPath && !currentPath.match(/\.(mp4|webm)$/i);

  // Generate base64 preview for newly selected files
  useEffect(() => {
    if (!newFilePath) {
      setPreviewUrl("");
      return;
    }
    invoke<string>("cmd_read_file_base64", { path: newFilePath })
      .then((dataUrl) => setPreviewUrl(dataUrl))
      .catch(() => setPreviewUrl(""));
  }, [newFilePath]);

  const handleBrowse = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Media", extensions: ["png", "jpg", "jpeg", "gif", "webp", "mp4", "webm"] }],
    });
    if (selected) {
      setNewFilePath(selected);
      if (!name) {
        const filename = selected.split(/[\\/]/).pop() ?? "";
        setName(filename.replace(/\.[^.]+$/, ""));
      }
    }
  };

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setDragOver(false);
    const file = e.dataTransfer.files[0];
    if (file) {
      const path = (file as any).path;
      if (path) {
        setNewFilePath(path);
        if (!name) setName(file.name.replace(/\.[^.]+$/, ""));
      }
    }
  }, [name]);

  const handleSave = async () => {
    if (!newFilePath && !isEdit) {
      setError("Please select a file");
      return;
    }
    if (!name.trim()) {
      setError("Please enter a name");
      return;
    }

    const cmd = `:${name.trim().toLowerCase().replace(/\s+/g, "")}:`;

    setSaving(true);
    setError(null);

    try {
      if (isEdit && editMeme) {
        if (newFilePath) {
          await invoke("cmd_replace_meme_file", {
            id: editMeme.id,
            newPath: newFilePath,
          });
        }
        await invoke("cmd_update_meme", {
          id: editMeme.id,
          name: name.trim(),
          command: cmd,
          categoryId: null,
          tags: [],
        });
      } else {
        await invoke("cmd_import_meme", {
          path: newFilePath,
          name: name.trim(),
          command: cmd,
          categoryId: null,
          tags: [],
        });
      }
      onSaved();
      onClose();
    } catch (e: any) {
      setError(typeof e === "string" ? e : e.message ?? "Failed to save");
    } finally {
      setSaving(false);
    }
  };

  const renderPreview = () => {
    if (hasNewFile && previewUrl && isImage) {
      return (
        <div className="drop-zone-preview" onClick={handleBrowse}>
          <img src={previewUrl} alt="Preview" />
          <div className="replace-overlay">Replace image</div>
        </div>
      );
    }
    if (isEdit && editMeme && !hasNewFile && editMeme.mime.startsWith("image/")) {
      return (
        <div className="drop-zone-preview" onClick={handleBrowse}>
          <MemeImage path={editMeme.stored_path} alt={editMeme.name} />
          <div className="replace-overlay">Replace image</div>
        </div>
      );
    }
    return (
      <div
        className={`drop-zone ${dragOver ? "drag-over" : ""}`}
        onClick={handleBrowse}
        onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
        onDragLeave={() => setDragOver(false)}
        onDrop={handleDrop}
      >
        <ImagePlus size={32} />
        <p>Drag & drop or click to browse</p>
        <p style={{ fontSize: 12 }}>PNG, JPG, GIF, WebP, MP4, WebM</p>
      </div>
    );
  };

  return (
    <div className="modal-backdrop" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="modal">
        <div className="modal-header">
          <h2>{isEdit ? "Edit Meme" : "Upload Meme"}</h2>
          <button className="modal-close" onClick={onClose}>
            <X size={18} />
          </button>
        </div>

        <div className="modal-body">
          {renderPreview()}

          <div className="form-field">
            <label>Name</label>
            <input
              type="text"
              placeholder="e.g. This is Fine"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>

          {error && (
            <p style={{ color: "var(--red)", fontSize: 13 }}>{error}</p>
          )}
        </div>

        <div className="modal-footer">
          <button className="btn btn-secondary" onClick={onClose}>Cancel</button>
          <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
            {saving ? "Saving..." : isEdit ? "Save Changes" : (
              <><Upload size={14} /> Add to Library</>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
