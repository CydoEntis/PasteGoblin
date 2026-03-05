import { Search, Upload, X } from "lucide-react";
import logo from "../../assets/logo.png";

interface HeaderProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  memeCount: number;
  onUploadClick: () => void;
}

export function Header({ searchQuery, onSearchChange, memeCount, onUploadClick }: HeaderProps) {
  return (
    <div className="header" data-tauri-drag-region>
      <img src={logo} alt="Paste Goblin" className="header-logo" />
      <div className="header-search">
        <Search size={16} className="search-icon" />
        <input
          type="text"
          placeholder="Search memes..."
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          autoFocus
        />
        {searchQuery && (
          <button className="search-clear" onClick={() => onSearchChange("")}>
            <X size={14} />
          </button>
        )}
      </div>
      <div className="header-actions">
        <span className="header-meme-count">{memeCount} memes</span>
        <button className="btn btn-primary" onClick={onUploadClick}>
          <Upload size={14} />
          Upload
        </button>
      </div>
    </div>
  );
}
