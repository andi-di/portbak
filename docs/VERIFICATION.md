# Version 0.1.0 verification

Verified on November 6, 2026, on Linux x86-64. All project-owned UI strings, error messages, defaults, test fixtures, and documentation are in English.

- `cargo test --locked --offline`: 20 tests passed.
- `cargo clippy --locked --offline --all-targets -- -D warnings`: passed.
- `cargo fmt --check`: passed.
- Linux release built with cargo-zigbuild targeting glibc 2.28. The highest required glibc symbol version was verified with readelf: 2.28. The English executable is 7,132,720 bytes.
- Linux distribution executable launched in a real X11 session with temporary source data. In Light and Dark modes: an overdue scheduled backup ran once, copied contents were correct, completion was saved, and screenshots were inspected. RSS after a small backup was approximately 90 MiB; the environment and graphics driver affect this value.
- Windows x86-64 release built with LLVM-MinGW and Rust target `x86_64-pc-windows-gnullvm`. The unwind runtime is linked statically; DLL imports include only Windows system libraries. The English executable is 3,905,536 bytes.
- 19 cross-platform Windows tests passed under isolated Wine. The additional Unix symlink test does not apply to Windows.
- The Windows distribution executable was launched under Wine without WINEPATH or extra DLLs. In Light and Dark modes: automatic backups completed correctly, completion was saved, and screenshots were inspected.
- No tests have been run on a native Windows machine. The included CI workflow runs tests and builds on Windows once activated on GitHub.
- The Linux executable has not been run on a distribution with an older glibc. Symbol inspection alone is not a complete compatibility test.
- Folder dialogs and operating system window controls have not been tested interactively. The quit confirmation is implemented; smoke tests terminate only isolated test processes after backups have completed.

Checksums for the distributed executables are in `dist/SHA256SUMS`. The reproducible UI smoke test is `tools/ui_smoke.py`; add `--wine` when testing the Windows executable.
