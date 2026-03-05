import logo from "../../assets/logo.png";

interface StatusBarProps {
  memeCount: number;
}

export function StatusBar({ memeCount }: StatusBarProps) {
  return (
    <div className="status-bar">
      <div className="status-bar-left">
        <img src={logo} alt="" style={{ width: 16, height: 16, objectFit: "contain" }} />
        <span style={{ color: "var(--accent)" }}>Paste Goblin</span>
        <span>&middot;</span>
        <span>{memeCount} memes in library</span>
      </div>
      <div className="status-bar-right">
        <span><kbd>&uarr;&darr;</kbd> navigate</span>
        <span><kbd>Enter</kbd> copy</span>
        <span><kbd>Esc</kbd> hide</span>
      </div>
    </div>
  );
}
