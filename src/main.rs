use std::fs;
// use std::io::Write; // not needed currently
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use std::time::Instant;

use anyhow::{Context, Result};
use clap::{ArgAction, Parser, ValueEnum};
use console::{measure_text_width, style};
use ignore::{overrides::OverrideBuilder, WalkBuilder, WalkState};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use num_format::{Locale, ToFormattedString};
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[command(version = env!("PKG_VERSION_WITH_BUILD"), about = "Fast symlink finder (Rust)")]
struct Opts {
    /// Absolute path to target to match against
    target: String,
    /// Scan hidden files and folders (on by default, matches `find`)
    #[arg(long, action = ArgAction::SetFalse, default_value_t = true)]
    hidden: bool,
    /// Maximum depth to recurse
    #[arg(long, value_name = "N")] 
    max_depth: Option<usize>,
    /// Disable TUI progress output
    #[arg(long, action = ArgAction::SetTrue)]
    no_tui: bool,
    /// Emit JSON array of matches
    #[arg(long, action = ArgAction::SetTrue)]
    json: bool,
    /// Respect .gitignore during scan (off by default)
    #[arg(long, action = ArgAction::SetTrue)]
    respect_gitignore: bool,
    /// Do not cross filesystem boundaries
    #[arg(long, action = ArgAction::SetTrue)]
    one_filesystem: bool,
    /// Thread count for traversal (default: auto)
    #[arg(long, value_name = "N")]
    threads: Option<usize>,
    /// Additional ignore glob(s) (gitignore-style). Repeatable.
    #[arg(long = "ignore", value_name = "GLOB")]
    ignores: Vec<String>,
    /// Additional ignore file(s) to load patterns from. Repeatable.
    #[arg(long = "ignore-file", value_name = "PATH")]
    ignore_files: Vec<PathBuf>,
    /// Include heavy directories like node_modules, .cache, target (off by default)
    #[arg(long, action = ArgAction::SetTrue)]
    include_heavy: bool,
    /// Color output: auto, always, or never
    #[arg(long, value_enum, default_value_t = ColorChoice::Auto)]
    color: ColorChoice,
    /// Disable streaming matches; only show final boxed summary
    #[arg(long, action = ArgAction::SetTrue)]
    no_stream: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ColorChoice { Auto, Always, Never }

fn realpath(path: &Path) -> Result<PathBuf> {
    // Resolve symlinks and normalize
    let rp = fs::canonicalize(path).with_context(|| format!("realpath of {}", path.display()))?;
    Ok(rp)
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let overall_start = Instant::now();

    // Configure ANSI color usage
    let enable_colors = match opts.color {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto => console::colors_enabled(),
    };
    console::set_colors_enabled(enable_colors);

    // Resolve target
    let target_resolved = realpath(Path::new(&opts.target))
        .with_context(|| "Failed to resolve target")?;

    // No immediate header; will render results in a bordered box

    // Build fast walker
    let mut wb = WalkBuilder::new(".");
    wb.follow_links(false)
        .hidden(opts.hidden) // include hidden by default
        .git_ignore(opts.respect_gitignore)
        .git_exclude(opts.respect_gitignore)
        .require_git(false)
        .same_file_system(opts.one_filesystem);
    if let Some(n) = opts.threads { wb.threads(n); }
    for f in &opts.ignore_files { let _ = wb.add_ignore(f); }

    // Default heavy directory skip list (can be re-enabled with --include-heavy)
    const HEAVY_DIRS: &[&str] = &[
        "node_modules",
        ".cache",
        "target",
        "build",
        "dist",
        "out",
        ".git",
        ".venv",
        "venv",
    ];

    if !opts.include_heavy {
        wb.filter_entry(|e| {
            if let Some(ft) = e.file_type() {
                if ft.is_dir() {
                    let name = e.file_name().to_string_lossy();
                    return !HEAVY_DIRS.contains(&name.as_ref());
                }
            }
            true
        });
    }

    // User-specified ignore globs
    if !opts.ignores.is_empty() {
        let mut ob = OverrideBuilder::new(".");
        for g in &opts.ignores {
            // In override matcher, a pattern starting with '!' is an ignore glob
            // (whitelist otherwise). We want ignores here.
            let pat = if g.starts_with('!') { g.clone() } else { format!("!{}", g) };
            let _ = ob.add(&pat);
        }
        if let Ok(ov) = ob.build() { wb.overrides(ov); }
    }
    if let Some(d) = opts.max_depth { wb.max_depth(Some(d)); }

    // TUI: spinner while walking, determinate bar while resolving
    let mp = if opts.no_tui { None } else { Some(MultiProgress::new()) };
    let walk_pb = mp.as_ref().map(|mp| {
        let pb = mp.add(ProgressBar::new_spinner());
        pb.set_style(ProgressStyle::with_template("{spinner:.green} {msg}").unwrap());
        pb.set_message("Walking filesystem…");
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        pb
    });

    // Collect symlink entries and count files/dirs traversed (parallel walk)
    let file_count = Arc::new(AtomicUsize::new(0));
    let dir_count = Arc::new(AtomicUsize::new(0));
    let entries: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));

    wb.build_parallel().run(|| {
        let file_count = Arc::clone(&file_count);
        let dir_count = Arc::clone(&dir_count);
        let entries = Arc::clone(&entries);
        Box::new(move |res| {
            if let Ok(e) = res {
                if let Some(ft) = e.file_type() {
                    if ft.is_dir() { dir_count.fetch_add(1, Ordering::Relaxed); }
                    else if ft.is_file() { file_count.fetch_add(1, Ordering::Relaxed); }
                    if ft.is_symlink() {
                        if let Ok(mut v) = entries.lock() { v.push(e.into_path()); }
                    }
                }
            }
            WalkState::Continue
        })
    });

    if let Some(pb) = &walk_pb { pb.finish_and_clear(); }

    let entries = entries.lock().unwrap().clone();
    let total = entries.len();
    let target = Arc::new(target_resolved);
    let target_meta = fs::metadata(&*target).ok();
    let matches_out = Arc::new(Mutex::new(Vec::<PathBuf>::new()));
    let streamed_count = Arc::new(AtomicUsize::new(0));

    // Determinate progress bar for resolving symlinks
    let resolve_pb = if !opts.no_tui {
        mp.as_ref().map(|mp| {
            let pb = mp.add(ProgressBar::new(total as u64));
            pb.set_style(
                ProgressStyle::with_template("{bar:40.cyan/blue} {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("##-"),
            );
            pb.set_message("Checking symlinks");
            pb
        })
    } else { None };

    // Parallel resolve and stream matches
    let streaming_allowed = !opts.json && !opts.no_stream;
    entries.par_iter().for_each(|p| {
        let is_match = match &target_meta {
            #[cfg(unix)]
            Some(tm) => {
                // Fast path on Unix: compare device+inode without allocating full realpath
                use std::os::unix::fs::MetadataExt;
                let ok = fs::metadata(p).map(|m| m.dev() == tm.dev() && m.ino() == tm.ino()).unwrap_or(false);
                if ok { true } else { realpath(p).map_or(false, |resolved| resolved == *target) }
            }
            _ => realpath(p).map_or(false, |resolved| resolved == *target),
        };
        if is_match {
            if let Ok(mut v) = matches_out.lock() { v.push(p.clone()); }
            if streaming_allowed {
                // On first streamed line, print a leading blank line to frame the results.
                let prev = streamed_count.fetch_add(1, Ordering::Relaxed);
                if prev == 0 {
                    if let Some(pb) = &resolve_pb { pb.println(String::from("")); } else { println!(""); }
                }
                let styled = style(p.display()).white().bold();
                if let Some(pb) = &resolve_pb { pb.println(format!("{}", styled)); } else { println!("{}", styled); }
            }
        }
        if let Some(pb) = &resolve_pb { pb.inc(1); }
    });

    if let Some(pb) = &resolve_pb { pb.finish_and_clear(); }

    let mut matches = matches_out.lock().unwrap().clone();
    matches.sort();
    if opts.json {
        println!("{}", serde_json::to_string_pretty(&matches)?);
    } else {
        let streamed_any = streamed_count.load(Ordering::Relaxed) > 0;
        if !streaming_allowed || !streamed_any {
            let lines: Vec<String> = if matches.is_empty() {
                vec![style("No matches found.").yellow().to_string()]
            } else {
                matches.iter().map(|p| style(p.display()).white().bold().to_string()).collect()
            };
            print_box(&lines);
        }
        // If we streamed any results, add a blank line after them before stats
        if streaming_allowed && streamed_any {
            println!("");
        }

        // Stats below results
        let elapsed = overall_start.elapsed();
        let secs = elapsed.as_secs_f64();
        let rate = if secs > 0.0 { (total as f64 / secs).round() as usize } else { total };
        if !(streaming_allowed && streamed_any) { println!(""); }

        let folders_s = dir_count.load(Ordering::Relaxed).to_formatted_string(&Locale::en);
        let files_s = file_count.load(Ordering::Relaxed).to_formatted_string(&Locale::en);
        let syms_s = total.to_formatted_string(&Locale::en);
        let matches_s = (matches.len()).to_formatted_string(&Locale::en);
        let rate_s = rate.to_formatted_string(&Locale::en);

        println!("{} {}", style("Folders traversed:").dim(), style(folders_s).bold().cyan());
        println!("{} {}", style("Files traversed:").dim(), style(files_s).bold().cyan());
        println!("{} {}", style("Symlinks scanned:").dim(), style(syms_s).bold().cyan());
        println!("{} {}", style("Matches:").dim(), style(matches_s).bold().green());
        println!("{} {:.2}s", style("Elapsed:").dim(), secs);
        println!("{} {} {}", style("Rate:").dim(), style(rate_s).bold().magenta(), style("symlinks/s").dim());
    }

    fn print_box(lines: &[String]) {
        let pad = 1usize;
        let content_width = lines.iter().map(|s| measure_text_width(s)).max().unwrap_or(0);
        let width = content_width + pad * 2;
        println!("{}", style(format!("┌{}┐", "─".repeat(width))).cyan());
        for line in lines {
            let w = measure_text_width(line);
            let right = width.saturating_sub(w + pad);
            print!("{}{}{}", style("│").cyan(), " ".repeat(pad), line);
            println!("{}{}", " ".repeat(right), style("│").cyan());
        }
        println!("{}", style(format!("└{}┘", "─".repeat(width))).cyan());
    }

    Ok(())
}
