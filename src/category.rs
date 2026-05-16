use serde::Serialize;
use std::fmt;

/// Whether a category is part of the always-shown core vocabulary or an
/// extended category that only surfaces in the legend / sidebar when at
/// least one entry of that category is actually present in the scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Universal categories. The color and label are reserved in the legend
    /// regardless of whether anything was matched, so the visual vocabulary
    /// stays stable across scans.
    Core,
    /// Niche categories that would be noise on a typical project tree but
    /// are valuable when present (e.g. a 5 GB `model_cache` block on an AI
    /// dev machine, or a 50 GB `vm_image` on a VM user's disk).
    Extended,
}

/// File / directory categorisation surfaced in the legend, in `--summary`,
/// and as the `--category` filter values. `Display` and `FromStr` use the
/// canonical snake_case names (`cache`, `vm_image`, …); clap awareness
/// lives in `cli/args.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    // ----- Core -----
    Cache,
    Build,
    Log,
    Media,
    Vcs,
    Ide,
    Other,
    // ----- Extended -----
    Archive,
    Installer,
    VmImage,
    ModelCache,
    Backup,
}

impl Category {
    /// Every variant in declaration order. Single source of truth for
    /// iteration (`FromStr` error message, round-trip tests, future
    /// listings). Adding a new variant to the enum requires extending
    /// this slice — the round-trip test in this module will catch
    /// drift.
    pub const ALL: &'static [Category] = &[
        Category::Cache,
        Category::Build,
        Category::Log,
        Category::Media,
        Category::Vcs,
        Category::Ide,
        Category::Other,
        Category::Archive,
        Category::Installer,
        Category::VmImage,
        Category::ModelCache,
        Category::Backup,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Category::Cache => "cache",
            Category::Build => "build",
            Category::Log => "log",
            Category::Media => "media",
            Category::Vcs => "vcs",
            Category::Ide => "ide",
            Category::Other => "other",
            Category::Archive => "archive",
            Category::Installer => "installer",
            Category::VmImage => "vm_image",
            Category::ModelCache => "model_cache",
            Category::Backup => "backup",
        }
    }

    pub fn tier(&self) -> Tier {
        match self {
            Category::Cache
            | Category::Build
            | Category::Log
            | Category::Media
            | Category::Vcs
            | Category::Ide
            | Category::Other => Tier::Core,
            Category::Archive
            | Category::Installer
            | Category::VmImage
            | Category::ModelCache
            | Category::Backup => Tier::Extended,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl std::str::FromStr for Category {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Case-insensitive to match the previous `clap::ValueEnum`
        // behaviour. Matching and the error message both come from
        // `Category::ALL` + `label()` so adding a new variant only
        // requires extending the enum and `ALL`.
        let lower = s.to_ascii_lowercase();
        Category::ALL
            .iter()
            .copied()
            .find(|c| c.label() == lower)
            .ok_or_else(|| {
                let labels: Vec<&str> = Category::ALL.iter().map(|c| c.label()).collect();
                format!(
                    "invalid category '{s}' (expected one of: {})",
                    labels.join(", ")
                )
            })
    }
}

// ============================================================================
// Rule tables — shared between `classify_*` and `explain_*` so the truth lives
// in exactly one place.
// ============================================================================

/// AI model stores (re-downloadable; large enough to deserve their own
/// category on AI dev machines). Checked before the more generic Cache rules
/// so e.g. `.ollama` wins as ModelCache instead of being dragged into Cache.
const MODEL_CACHE_DIRS: &[&str] = &[".ollama", ".lmstudio", ".huggingface"];

/// OS-level backup stores (Apple Time Machine etc.).
const BACKUP_DIRS: &[&str] = &["time machine backups", "backups.backupdb"];

const CACHE_DIRS: &[&str] = &[
    "node_modules",
    ".cache",
    "__pycache__",
    ".npm",
    ".yarn",
    ".pnpm-store",
    "caches",
    ".gradle",
    ".nuget",
    ".pub-cache",
    "pods",
    ".cocoapods",
    ".cargo",
    "bower_components",
    ".tmp",
    "tmp",
    "temp",
    ".temp",
    ".trash",
    // Language version managers + tool installs (re-downloadable)
    ".rustup",
    ".pyenv",
    ".rbenv",
    ".nvm",
    ".volta",
    ".asdf",
    "mise",
    ".pipx",
    "pipx",
    ".poetry",
    ".composer",
    ".m2",
    ".ivy2",
    ".sbt",
    ".stack",
    ".cabal",
    ".deno",
    ".bun",
    // Container / VM bundles (re-buildable / re-downloadable)
    ".docker",
    "vm_bundles",
];

const BUILD_DIRS: &[&str] = &[
    "target",
    "dist",
    "build",
    "out",
    ".next",
    ".nuxt",
    ".output",
    ".turbo",
    ".angular",
    "_build",
    "cmake-build-debug",
    "cmake-build-release",
];

const LOG_DIRS: &[&str] = &["logs", "log", ".logs"];

const VCS_DIRS: &[&str] = &[".git", ".svn", ".hg", ".jj", ".bzr", "_darcs", ".fossil"];

const IDE_DIRS: &[&str] = &[
    ".idea",
    ".vscode",
    ".vscode-insiders",
    ".vscode-server",
    ".vs",
    ".eclipse",
    ".settings",
    ".cursor",
    ".cursor-server",
    ".windsurf",
    ".zed",
    ".fleet",
];

/// Special filenames matched by suffix (not extension). Currently OrbStack's
/// raw VM disk image, which is literally `data.img.raw` — matched by name so
/// every Sony α RAW photo isn't dragged into vm_image just for its `.raw` tail.
const VM_IMAGE_FILE_SUFFIXES: &[&str] = &["data.img.raw"];

const VM_IMAGE_EXTENSIONS: &[&str] = &[
    // `.iso` is OS install media most of the time, but it can also be a game
    // ROM dump. Bucketed here for now — both are large blobs the user knows
    // they put there.
    ".vdi", ".vmdk", ".qcow2", ".vhd", ".vhdx", ".iso",
];

const INSTALLER_EXTENSIONS: &[&str] = &[
    ".dmg",      // macOS disk image
    ".pkg",      // macOS installer package
    ".msi",      // Windows installer
    ".exe",      // Windows executable / installer
    ".deb",      // Debian package
    ".rpm",      // RedHat package
    ".appimage", // Linux AppImage
    ".snap",     // Linux Snap package
    ".flatpak",  // Linux Flatpak
    ".apk",      // Android package
];

/// Match the common "single-extension" tail. Multi-part extensions like
/// `.tar.gz` are still caught because they end in `.gz`.
const ARCHIVE_EXTENSIONS: &[&str] = &[
    ".zip", ".tar", ".tgz", ".tbz2", ".txz", ".gz", ".bz2", ".xz", ".7z", ".rar", ".zst",
];

const BACKUP_EXTENSIONS: &[&str] = &[".bak", ".backup", ".old"];

// `#[rustfmt::skip]` keeps the per-section grouping (Image / Video / Audio)
// readable. Without it rustfmt collapses the trailing `// Audio.` comment
// into the preceding video line, which makes the file harder to scan.
#[rustfmt::skip]
const MEDIA_EXTENSIONS: &[&str] = &[
    // Image (including camera RAW formats — `.raw` for generic exporters,
    // `.arw` Sony α, `.cr2` Canon, `.nef` Nikon, `.dng` Adobe DNG).
    // The literal `data.img.raw` OrbStack VM image is matched earlier in
    // `explain_file` and never reaches here, so reintroducing `.raw` to
    // media doesn't bring back the VM-image-as-media confusion.
    ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".svg", ".webp", ".ico", ".tiff", ".heic", ".heif",
    ".psd", ".raw", ".arw", ".cr2", ".nef", ".dng",
    // Video.
    // `.ts` is intentionally excluded: while it's the MPEG transport-stream extension,
    // TypeScript files are vastly more common in real codebases and being miscategorized
    // as `media` is more harmful than missing the rare transport-stream file.
    ".mp4", ".avi", ".mkv", ".mov", ".wmv", ".flv", ".webm", ".m4v", ".3gp",
    // Audio.
    ".mp3", ".wav", ".flac", ".aac", ".ogg", ".wma", ".m4a", ".opus", ".aiff",
];

// ============================================================================
// Classification with explanation
// ============================================================================

/// Why an entry was assigned a category. Surfaced via `--explain-category`
/// so anyone debugging "why is this `cache`?" can see the rule that fired
/// without reading the source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClassificationReason {
    /// Matched an exact directory-name rule (case-insensitive).
    DirNameExact { needle: &'static str },
    /// Directory name contains the substring "cache" (case-insensitive).
    /// Catches things like `GPUCache` and `Code Cache` that aren't worth
    /// listing individually.
    DirNameContainsCache,
    /// Matched a special filename suffix (e.g. `data.img.raw`). Distinct
    /// from a plain extension because the rule keys off more than the dot
    /// suffix.
    FileNameSuffix { needle: &'static str },
    /// Matched a file extension (e.g. `.log`, `.gz`).
    FileExtension { needle: &'static str },
    /// No rule matched; defaulted to `other`.
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Classification {
    pub category: Category,
    pub reason: ClassificationReason,
}

impl ClassificationReason {
    /// Human-readable one-liner used in the text output of
    /// `--explain-category`. JSON output uses serde's tagged form instead.
    pub fn describe(&self) -> String {
        match self {
            ClassificationReason::DirNameExact { needle } => {
                format!("matched directory rule: {needle}")
            }
            ClassificationReason::DirNameContainsCache => {
                "directory name contains \"cache\"".to_string()
            }
            ClassificationReason::FileNameSuffix { needle } => {
                format!("matched filename suffix: {needle}")
            }
            ClassificationReason::FileExtension { needle } => {
                format!("matched file extension: {needle}")
            }
            ClassificationReason::Default => "no rule matched; defaulted to other".to_string(),
        }
    }
}

/// Same logic as [`classify_dir`] but also records *which* rule matched.
/// `classify_dir` is the thin wrapper used in the hot scanning path; this
/// is the one called by `--explain-category` for transparency.
pub fn explain_dir(name: &str) -> Classification {
    let lower = name.to_lowercase();

    // ----- Extended (checked first so they win over the more generic
    // Cache rules below; e.g. `.ollama` is more specifically a model
    // cache than just "a cache directory") -----
    if let Some(needle) = first_exact_match(&lower, MODEL_CACHE_DIRS) {
        return Classification {
            category: Category::ModelCache,
            reason: ClassificationReason::DirNameExact { needle },
        };
    }
    if let Some(needle) = first_exact_match(&lower, BACKUP_DIRS) {
        return Classification {
            category: Category::Backup,
            reason: ClassificationReason::DirNameExact { needle },
        };
    }

    // ----- Core -----
    if let Some(needle) = first_exact_match(&lower, CACHE_DIRS) {
        return Classification {
            category: Category::Cache,
            reason: ClassificationReason::DirNameExact { needle },
        };
    }
    if lower.contains("cache") {
        return Classification {
            category: Category::Cache,
            reason: ClassificationReason::DirNameContainsCache,
        };
    }
    if let Some(needle) = first_exact_match(&lower, BUILD_DIRS) {
        return Classification {
            category: Category::Build,
            reason: ClassificationReason::DirNameExact { needle },
        };
    }
    if let Some(needle) = first_exact_match(&lower, LOG_DIRS) {
        return Classification {
            category: Category::Log,
            reason: ClassificationReason::DirNameExact { needle },
        };
    }
    if let Some(needle) = first_exact_match(&lower, VCS_DIRS) {
        return Classification {
            category: Category::Vcs,
            reason: ClassificationReason::DirNameExact { needle },
        };
    }
    if let Some(needle) = first_exact_match(&lower, IDE_DIRS) {
        return Classification {
            category: Category::Ide,
            reason: ClassificationReason::DirNameExact { needle },
        };
    }

    Classification {
        category: Category::Other,
        reason: ClassificationReason::Default,
    }
}

/// Same logic as [`classify_file`] but also records *which* rule matched.
pub fn explain_file(name: &str) -> Classification {
    let lower = name.to_lowercase();

    if let Some(needle) = first_suffix_match(&lower, &[".log"]) {
        return Classification {
            category: Category::Log,
            reason: ClassificationReason::FileExtension { needle },
        };
    }

    // ----- Extended file types (checked before media so a `data.img.raw`
    // VM image isn't dragged into media just because of its `.raw` tail) -----
    if let Some(needle) = first_suffix_match(&lower, VM_IMAGE_FILE_SUFFIXES) {
        return Classification {
            category: Category::VmImage,
            reason: ClassificationReason::FileNameSuffix { needle },
        };
    }
    if let Some(needle) = first_suffix_match(&lower, VM_IMAGE_EXTENSIONS) {
        return Classification {
            category: Category::VmImage,
            reason: ClassificationReason::FileExtension { needle },
        };
    }
    if let Some(needle) = first_suffix_match(&lower, INSTALLER_EXTENSIONS) {
        return Classification {
            category: Category::Installer,
            reason: ClassificationReason::FileExtension { needle },
        };
    }
    if let Some(needle) = first_suffix_match(&lower, ARCHIVE_EXTENSIONS) {
        return Classification {
            category: Category::Archive,
            reason: ClassificationReason::FileExtension { needle },
        };
    }
    if let Some(needle) = first_suffix_match(&lower, BACKUP_EXTENSIONS) {
        return Classification {
            category: Category::Backup,
            reason: ClassificationReason::FileExtension { needle },
        };
    }
    if let Some(needle) = first_suffix_match(&lower, MEDIA_EXTENSIONS) {
        return Classification {
            category: Category::Media,
            reason: ClassificationReason::FileExtension { needle },
        };
    }

    Classification {
        category: Category::Other,
        reason: ClassificationReason::Default,
    }
}

fn first_exact_match(lower: &str, needles: &[&'static str]) -> Option<&'static str> {
    needles.iter().copied().find(|n| *n == lower)
}

fn first_suffix_match(lower: &str, needles: &[&'static str]) -> Option<&'static str> {
    needles.iter().copied().find(|n| lower.ends_with(*n))
}

/// Classify a directory by its name.
pub fn classify_dir(name: &str) -> Category {
    explain_dir(name).category
}

/// Classify a file by its name (extension or, for a few special cases,
/// full filename).
pub fn classify_file(name: &str) -> Category {
    explain_file(name).category
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_known_directories() {
        assert_eq!(classify_dir("node_modules"), Category::Cache);
        assert_eq!(classify_dir("__pycache__"), Category::Cache);
        assert_eq!(classify_dir("target"), Category::Build);
        assert_eq!(classify_dir("dist"), Category::Build);
        assert_eq!(classify_dir("logs"), Category::Log);
        assert_eq!(classify_dir(".git"), Category::Vcs);
        assert_eq!(classify_dir(".idea"), Category::Ide);
        assert_eq!(classify_dir("src"), Category::Other);
    }

    #[test]
    fn classifies_directory_names_case_insensitively() {
        assert_eq!(classify_dir("Node_Modules"), Category::Cache);
        assert_eq!(classify_dir(".GIT"), Category::Vcs);
    }

    #[test]
    fn classifies_language_toolchains_as_cache() {
        assert_eq!(classify_dir(".rustup"), Category::Cache);
        assert_eq!(classify_dir(".pyenv"), Category::Cache);
        assert_eq!(classify_dir(".nvm"), Category::Cache);
        assert_eq!(classify_dir("mise"), Category::Cache);
        assert_eq!(classify_dir("pipx"), Category::Cache);
        assert_eq!(classify_dir(".docker"), Category::Cache);
        assert_eq!(classify_dir("vm_bundles"), Category::Cache);
    }

    #[test]
    fn classifies_additional_ide_and_vcs() {
        assert_eq!(classify_dir(".vscode-insiders"), Category::Ide);
        assert_eq!(classify_dir(".cursor"), Category::Ide);
        assert_eq!(classify_dir(".zed"), Category::Ide);
        assert_eq!(classify_dir(".jj"), Category::Vcs);
    }

    #[test]
    fn partial_match_catches_cache_directories() {
        assert_eq!(classify_dir("GPUCache"), Category::Cache);
        assert_eq!(classify_dir("Code Cache"), Category::Cache);
    }

    #[test]
    fn classifies_files_by_extension() {
        assert_eq!(classify_file("debug.log"), Category::Log);
        assert_eq!(classify_file("photo.JPG"), Category::Media);
        assert_eq!(classify_file("video.mp4"), Category::Media);
        assert_eq!(classify_file("song.mp3"), Category::Media);
        assert_eq!(classify_file("main.rs"), Category::Other);
    }

    #[test]
    fn typescript_files_are_not_media() {
        // `.ts` is the MPEG transport-stream extension, but TypeScript is far
        // more common in modern codebases. Keep these classified as `other`
        // so we don't surprise users with `index.ts` showing up as `media`.
        assert_eq!(classify_file("index.ts"), Category::Other);
        assert_eq!(classify_file("App.tsx"), Category::Other);
        assert_eq!(classify_file("eleventy.config.ts"), Category::Other);
    }

    // ----- Extended categories ------------------------------------------------

    #[test]
    fn ai_model_stores_classify_as_model_cache() {
        // Beats the more generic Cache rules because ModelCache is checked first.
        assert_eq!(classify_dir(".ollama"), Category::ModelCache);
        assert_eq!(classify_dir(".lmstudio"), Category::ModelCache);
        assert_eq!(classify_dir(".huggingface"), Category::ModelCache);
    }

    #[test]
    fn time_machine_backup_directories_classify_as_backup() {
        assert_eq!(classify_dir("Time Machine Backups"), Category::Backup);
        assert_eq!(classify_dir("Backups.backupdb"), Category::Backup);
    }

    #[test]
    fn installer_files_classify_as_installer() {
        assert_eq!(classify_file("Codex.dmg"), Category::Installer);
        assert_eq!(classify_file("googlechrome.dmg"), Category::Installer);
        assert_eq!(classify_file("setup.exe"), Category::Installer);
        assert_eq!(classify_file("package.deb"), Category::Installer);
        assert_eq!(classify_file("MyApp.AppImage"), Category::Installer);
    }

    #[test]
    fn vm_images_classify_as_vm_image() {
        assert_eq!(classify_file("disk.vdi"), Category::VmImage);
        assert_eq!(classify_file("disk.vmdk"), Category::VmImage);
        assert_eq!(classify_file("disk.qcow2"), Category::VmImage);
        // OrbStack's macOS-side raw disk image — name match, not just `.raw`.
        assert_eq!(classify_file("data.img.raw"), Category::VmImage);
    }

    #[test]
    fn raw_photo_classifies_as_media() {
        // Sony α uses `.arw` natively, but third-party exporters and older
        // firmwares still emit generic `.raw` files; both belong with the
        // rest of the camera RAW formats (`.cr2`, `.nef`, `.dng`). The
        // OrbStack `data.img.raw` literal is matched earlier in
        // `explain_file` so it still wins as VmImage and never reaches the
        // media extension list.
        assert_eq!(classify_file("DSC0001.raw"), Category::Media);
        assert_eq!(classify_file("DSC0001.arw"), Category::Media);
        assert_eq!(classify_file("data.img.raw"), Category::VmImage);
    }

    #[test]
    fn archive_files_classify_as_archive() {
        assert_eq!(classify_file("snapshot.zip"), Category::Archive);
        assert_eq!(classify_file("source.tar.gz"), Category::Archive);
        assert_eq!(classify_file("source.tgz"), Category::Archive);
        assert_eq!(classify_file("blob.7z"), Category::Archive);
        assert_eq!(classify_file("data.zst"), Category::Archive);
    }

    #[test]
    fn backup_files_classify_as_backup() {
        assert_eq!(classify_file("config.bak"), Category::Backup);
        assert_eq!(classify_file("notes.old"), Category::Backup);
    }

    #[test]
    fn tier_split_matches_intent() {
        // Core categories.
        for c in [
            Category::Cache,
            Category::Build,
            Category::Log,
            Category::Media,
            Category::Vcs,
            Category::Ide,
            Category::Other,
        ] {
            assert_eq!(c.tier(), Tier::Core, "{c:?} should be Core");
        }
        // Extended categories.
        for c in [
            Category::Archive,
            Category::Installer,
            Category::VmImage,
            Category::ModelCache,
            Category::Backup,
        ] {
            assert_eq!(c.tier(), Tier::Extended, "{c:?} should be Extended");
        }
    }

    // ----- Explain ------------------------------------------------------------

    #[test]
    fn explain_dir_reports_exact_match_needle() {
        let c = explain_dir("node_modules");
        assert_eq!(c.category, Category::Cache);
        assert_eq!(
            c.reason,
            ClassificationReason::DirNameExact {
                needle: "node_modules"
            }
        );
    }

    #[test]
    fn explain_dir_reports_extended_winning_over_cache() {
        let c = explain_dir(".ollama");
        assert_eq!(c.category, Category::ModelCache);
        assert_eq!(
            c.reason,
            ClassificationReason::DirNameExact { needle: ".ollama" }
        );
    }

    #[test]
    fn explain_dir_reports_partial_cache_match() {
        let c = explain_dir("GPUCache");
        assert_eq!(c.category, Category::Cache);
        assert_eq!(c.reason, ClassificationReason::DirNameContainsCache);
    }

    #[test]
    fn explain_dir_reports_default_when_unmatched() {
        let c = explain_dir("src");
        assert_eq!(c.category, Category::Other);
        assert_eq!(c.reason, ClassificationReason::Default);
    }

    #[test]
    fn explain_file_reports_extension_needle() {
        let c = explain_file("debug.log");
        assert_eq!(c.category, Category::Log);
        assert_eq!(
            c.reason,
            ClassificationReason::FileExtension { needle: ".log" }
        );
        let c = explain_file("source.tar.gz");
        assert_eq!(c.category, Category::Archive);
        assert_eq!(
            c.reason,
            ClassificationReason::FileExtension { needle: ".gz" }
        );
    }

    #[test]
    fn explain_file_reports_filename_suffix_for_orbstack() {
        let c = explain_file("data.img.raw");
        assert_eq!(c.category, Category::VmImage);
        assert_eq!(
            c.reason,
            ClassificationReason::FileNameSuffix {
                needle: "data.img.raw"
            }
        );
    }

    #[test]
    fn explain_file_reports_default_when_unmatched() {
        let c = explain_file("main.rs");
        assert_eq!(c.category, Category::Other);
        assert_eq!(c.reason, ClassificationReason::Default);
    }

    #[test]
    fn category_display_round_trips_through_from_str() {
        for &c in Category::ALL {
            let s = c.to_string();
            let parsed: Category = s.parse().expect("label() must parse back via FromStr");
            assert_eq!(parsed, c, "round-trip mismatch for {c:?} via '{s}'");
        }
    }

    #[test]
    fn category_from_str_is_case_insensitive() {
        assert_eq!("CACHE".parse::<Category>().unwrap(), Category::Cache);
        assert_eq!("Vm_Image".parse::<Category>().unwrap(), Category::VmImage);
    }

    #[test]
    fn category_from_str_rejects_unknown_with_full_list() {
        let err = "bogus".parse::<Category>().unwrap_err();
        // Error message should mention every variant so users see the
        // full vocabulary even when they typo.
        for c in Category::ALL {
            assert!(
                err.contains(c.label()),
                "error message missing '{}': {err}",
                c.label()
            );
        }
    }
}
