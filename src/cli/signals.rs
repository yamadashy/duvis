// Restore SIGPIPE's default disposition so `duvis ... | head` exits
// silently (process killed by SIGPIPE) instead of surfacing
// `Error: Broken pipe (os error 32)`. Rust runtime ignores SIGPIPE by
// default, which is the wrong behavior for a Unix CLI that streams to
// stdout. Same approach as ripgrep, fd, etc.
pub fn reset_sigpipe() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
