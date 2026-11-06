//! A throwaway data directory for tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

static NEXT: AtomicU32 = AtomicU32::new(0);

/// Sources, vocabularies and a one-page text, as `tuvi/` tests need them.
const SOURCES: &str = r#"[
  { "id": "book", "title": "Sách", "rights": "open", "packet": "pages.txt" },
  { "id": "closed", "title": "Sách đóng", "rights": "unverified" },
  { "id": "absent", "title": "Sách vắng", "rights": "open", "packet": "absent.txt" }
]"#;

const PAGES: &str = "=== tr. 158 ===\n- Sát đồng cung: có uy quyền, nên chuyên về quân sự\n- một hai ba bốn năm sáu bảy tám chín mười mười một mười hai\n=== tr. 159 ===\nĐơn thủ tại Tý: bình thường.\n";

/// A repository root under the system temporary directory, removed on drop.
pub struct TestRoot {
    path: PathBuf,
}

impl TestRoot {
    /// A root with the base files.
    pub fn new() -> Self {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("astro-data-{}-{id}", std::process::id()));
        let root = TestRoot { path };
        root.write("pages.txt", PAGES);
        root.write("data/interpretations/sources.json", SOURCES);
        root.write(
            "data/interpretations/vocab/activities.json",
            r#"[{ "id": "CuoiHoi", "label": "Cưới hỏi" }]"#,
        );
        root.write(
            "data/interpretations/vocab/star_groups.json",
            r#"[{ "id": "KinhDa", "stars": ["KinhDuong", "DaLa"], "source": "book tr. 158" }]"#,
        );
        root
    }

    /// The root directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Writes `text` at `relative`.
    pub fn write(&self, relative: &str, text: &str) {
        let path = self.path.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
