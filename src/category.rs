use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Cache,
    Build,
    Log,
    Media,
    Vcs,
    Ide,
    Other,
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
        }
    }

    pub fn is_deletable(&self) -> bool {
        matches!(self, Category::Cache | Category::Build | Category::Log)
    }

    pub fn deletable_hint(&self) -> &'static str {
        match self {
            Category::Cache => "safely deletable",
            Category::Build => "rebuildable",
            Category::Log => "usually deletable",
            _ => "",
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
            // Container / VM images (re-buildable / re-downloadable)
            | ".docker"
            | "vm_bundles"
            // AI model stores (re-downloadable)
            | ".ollama"
            | ".lmstudio"
            | ".huggingface"
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

/// Classify a file by its extension.
pub fn classify_file(name: &str) -> Category {
    let lower = name.to_lowercase();

    // Log files
    if lower.ends_with(".log") {
        return Category::Log;
    }

    // Media files
    if is_media_extension(&lower) {
        return Category::Media;
    }

    Category::Other
}

fn is_media_extension(lower_name: &str) -> bool {
    const MEDIA_EXTENSIONS: &[&str] = &[
        // Image
        ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".svg", ".webp", ".ico", ".tiff", ".raw", ".heic",
        ".heif", ".psd", ".cr2", ".nef", ".dng", // Video
        ".mp4", ".avi", ".mkv", ".mov", ".wmv", ".flv", ".webm", ".m4v", ".ts", ".3gp",
        // Audio
        ".mp3", ".wav", ".flac", ".aac", ".ogg", ".wma", ".m4a", ".opus", ".aiff",
    ];
    MEDIA_EXTENSIONS.iter().any(|ext| lower_name.ends_with(ext))
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
        assert_eq!(classify_dir(".ollama"), Category::Cache);
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
    fn deletable_categories() {
        assert!(Category::Cache.is_deletable());
        assert!(Category::Build.is_deletable());
        assert!(Category::Log.is_deletable());
        assert!(!Category::Media.is_deletable());
        assert!(!Category::Vcs.is_deletable());
        assert!(!Category::Ide.is_deletable());
        assert!(!Category::Other.is_deletable());
    }
}
