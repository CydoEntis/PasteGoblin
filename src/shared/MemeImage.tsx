import { useState, useEffect } from "react";
import { getFileUrl } from "./file-url";

interface MemeImageProps {
  path: string;
  alt: string;
  className?: string;
  style?: React.CSSProperties;
}

/** Async image component that loads local files via Tauri FS plugin. */
export function MemeImage({ path, alt, className, style }: MemeImageProps) {
  const [src, setSrc] = useState<string>("");

  useEffect(() => {
    let cancelled = false;
    getFileUrl(path).then((url) => {
      if (!cancelled) setSrc(url);
    });
    return () => { cancelled = true; };
  }, [path]);

  if (!src) {
    return <div className={className} style={style} />;
  }

  return <img src={src} alt={alt} className={className} style={style} />;
}
