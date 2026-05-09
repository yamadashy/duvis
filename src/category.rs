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

/// Classify a directory by its name.
pub fn classify_dir(name: &str) -> Category {
    let lower = name.to_lowercase();

    // ----- Extended (checked first so they win over the more generic
    // `cache` rules below; e.g. `.ollama` is more specifically a model
    // cache than just "a cache directory") -----

    // AI model stores (re-downloadable; large enough to deserve their own
    // category on AI dev machines).
    if matches!(lower.as_str(), ".ollama" | ".lmstudio" | ".huggingface") {
        return Category::ModelCache;
    }

    // OS-level backup stores (Apple Time Machine etc.).
    if matches!(lower.as_str(), "time machine backups" | "backups.backupdb") {
        return Category::Backup;
    }

    // ----- Core -----

    // Exact match: Cache directories
    if matches!(
        lower.as_str(),
        "node_modules"
            | ".cache"
            | "__pycache__"
            | ".npm"
            | ".yarn"
            | ".pnpm-store"
            | "caches"
            | ".gradle"
            | ".nuget"
            | ".pub-cache"
            | "pods"
            | ".cocoapods"
            | ".cargo"
            | "bower_components"
            | ".tmp"
            | "tmp"
            | "temp"
            | ".temp"
            | ".trash"
            // Language version managers + tool installs (re-downloadable)
            | ".rustup"
            | ".pyenv"
            | ".rbenv"
            | ".nvm"
            | ".volta"
            | ".asdf"
            | "mise"
            | ".pipx"
            | "pipx"
            | ".poetry"
            | ".composer"
            | ".m2"
            | ".ivy2"
            | ".sbt"
            | ".stack"
            | ".cabal"
            | ".deno"
            | ".bun"
            // Container / VM bundles (re-buildable / re-downloadable)
            | ".docker"
            | "vm_bundles"
    ) {
        return Category::Cache;
    }

    // Partial match: directories containing "cache" (catches GPUCache, Code Cache, etc.)
    if lower.contains("cache") {
        return Category::Cache;
    }

    // Exact match: Build artifact directories
    if matches!(
        lower.as_str(),
        "target"
            | "dist"
            | "build"
            | "out"
            | ".next"
            | ".nuxt"
            | ".output"
            | ".turbo"
            | ".angular"
            | "_build"
            | "cmake-build-debug"
            | "cmake-build-release"
    ) {
        return Category::Build;
    }

    // Exact match: Log directories
    if matches!(lower.as_str(), "logs" | "log" | ".logs") {
        return Category::Log;
    }

    // Exact match: VCS directories
    if matches!(
        lower.as_str(),
        ".git" | ".svn" | ".hg" | ".jj" | ".bzr" | "_darcs" | ".fossil"
    ) {
        return Category::Vcs;
    }

    // Exact match: IDE directories
    if matches!(
        lower.as_str(),
        ".idea"
            | ".vscode"
            | ".vscode-insiders"
            | ".vscode-server"
            | ".vs"
            | ".eclipse"
            | ".settings"
            | ".cursor"
            | ".cursor-server"
            | ".windsurf"
            | ".zed"
            | ".fleet"
    ) {
        return Category::Ide;
    }

    Category::Other
}

/// Classify a file by its name (extension or, for a few special cases,
/// full filename).
pub fn classify_file(name: &str) -> Category {
    let lower = name.to_lowercase();

    // Log files
    if lower.ends_with(".log") {
        return Category::Log;
    }

    // ----- Extended file types (checked before media so a `data.img.raw`
    // VM image isn't dragged into media just because of its `.raw` tail) -----

    // OrbStack-style raw VM disk image: file is literally named
    // `data.img.raw`. We match on the name so we don't pull every Sony α
    // RAW photo into the vm_image bucket.
    if lower.ends_with("data.img.raw") {
        return Category::VmImage;
    }

    if is_vm_image_extension(&lower) {
        return Category::VmImage;
    }

    if is_installer_extension(&lower) {
        return Category::Installer;
    }

    if is_archive_extension(&lower) {
        return Category::Archive;
    }

    if is_backup_extension(&lower) {
        return Category::Backup;
    }

    // Media files
    if is_media_extension(&lower) {
        return Category::Media;
    }

    Category::Other
}

fn is_media_extension(lower_name: &str) -> bool {
    const MEDIA_EXTENSIONS: &[&str] = &[
        // Image (including camera RAW formats — `.raw` for generic exporters,
        // `.arw` Sony α, `.cr2` Canon, `.nef` Nikon, `.dng` Adobe DNG).
        // The literal `data.img.raw` OrbStack VM image is matched earlier in
        // `classify_file` and never reaches here, so reintroducing `.raw` to
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
    MEDIA_EXTENSIONS.iter().any(|ext| lower_name.ends_with(ext))
}

fn is_vm_image_extension(lower_name: &str) -> bool {
    // `.iso` is OS install media most of the time, but it can also be
    // a game ROM dump. Bucketed here for now — both are large blobs the
    // user knows they put there.
    const VM_IMAGE_EXTENSIONS: &[&str] = &[".vdi", ".vmdk", ".qcow2", ".vhd", ".vhdx", ".iso"];
    VM_IMAGE_EXTENSIONS
        .iter()
        .any(|ext| lower_name.ends_with(ext))
}

fn is_installer_extension(lower_name: &str) -> bool {
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
    INSTALLER_EXTENSIONS
        .iter()
        .any(|ext| lower_name.ends_with(ext))
}

fn is_archive_extension(lower_name: &str) -> bool {
    // Match the common "single-extension" tail. Multi-part extensions like
    // `.tar.gz` are still caught because they end in `.gz`.
    const ARCHIVE_EXTENSIONS: &[&str] = &[
        ".zip", ".tar", ".tgz", ".tbz2", ".txz", ".gz", ".bz2", ".xz", ".7z", ".rar", ".zst",
    ];
    ARCHIVE_EXTENSIONS
        .iter()
        .any(|ext| lower_name.ends_with(ext))
}

fn is_backup_extension(lower_name: &str) -> bool {
    const BACKUP_EXTENSIONS: &[&str] = &[".bak", ".backup", ".old"];
    BACKUP_EXTENSIONS
        .iter()
        .any(|ext| lower_name.ends_with(ext))
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
        // `classify_file` so it still wins as VmImage and never reaches the
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
}
