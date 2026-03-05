export interface Meme {
  id: string;
  name: string;
  command: string;
  category_id: string | null;
  category_name: string | null;
  original_filename: string;
  ext: string;
  mime: string;
  sha256: string;
  stored_path: string;
  width: number | null;
  height: number | null;
  duration_ms: number | null;
  created_at: number;
  updated_at: number;
  last_used_at: number | null;
  use_count: number;
  is_favorite: boolean;
  tags: string[];
}
