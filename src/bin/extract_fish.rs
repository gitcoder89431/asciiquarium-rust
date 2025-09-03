use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

/// Extracts fish ASCII art blocks from the original Perl Asciiquarium script and
/// generates a Rust module that exposes them as &'static str constants plus a
/// helper to build Vec<FishArt>.
///
/// Usage:
///   cargo run --bin extract_fish [input_perl_path] [output_rust_path]
///
/// Defaults:
///   input_perl_path  = archive/original/asciiquarium
///   output_rust_path = src/widgets/generated_fish_assets.rs
///
/// Strategy:
/// - Locate `sub add_new_fish` and `sub add_old_fish` in the Perl script.
/// - Within each, find `my @fish_image = (` and parse quoted blocks (q{...} or q#...#).
/// - The fish arrays alternate [art, color_mask, art, color_mask, ...].
///   We extract only the art blocks at even indices (0, 2, 4, ...).
/// - Generate a Rust module with:
///     - const FISH_N: &str = "...";
///     - pub fn get_generated_fish_assets() -> Vec<FishArt> { ... }
///     - a local measure function to compute width/height
fn main() -> io::Result<()> {
    let (input_path, output_path) = parse_args();

    let input = fs::read_to_string(&input_path)?;
    let mut all_art: Vec<String> = Vec::new();

    for func in ["add_new_fish", "add_old_fish"] {
        let blocks = extract_fish_blocks_from_function(&input, func);
        all_art.extend(blocks);
    }

    if all_art.is_empty() {
        eprintln!("No fish art blocks found. Check the input file path and format.");
    }

    let generated = generate_rust_module(&all_art);

    // Ensure parent dir exists
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut file = File::create(&output_path)?;
    file.write_all(generated.as_bytes())?;

    println!(
        "Extracted {} fish art blocks into: {}",
        all_art.len(),
        output_path.display()
    );

    Ok(())
}

fn parse_args() -> (PathBuf, PathBuf) {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("archive/original/asciiquarium"));
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("src/widgets/generated_fish_assets.rs"));
    (input, output)
}

/// Extract fish art blocks from a specific Perl function:
/// - Locates `sub <func_name>`
/// - Finds `my @fish_image = (`
/// - Collects q{...} or q#...# blocks until `);`
/// - Returns only the even-indexed blocks (0, 2, 4, ...)
fn extract_fish_blocks_from_function(input: &str, func_name: &str) -> Vec<String> {
    let mut results = Vec::new();

    // Locate function start
    let func_marker = format!("sub {}", func_name);
    let Some(func_start_idx) = input.find(&func_marker) else {
        return results;
    };

    // Search from function start for the fish_image array declaration
    let array_marker = "my @fish_image";
    let array_start_rel = input[func_start_idx..].find(array_marker);
    let Some(array_start_rel) = array_start_rel else {
        return results;
    };
    let array_start = func_start_idx + array_start_rel;

    // From array start, find the opening '(' and then parse until the matching ');'
    let after_array = &input[array_start..];
    let Some(paren_idx_rel) = after_array.find('(') else {
        return results;
    };
    let cursor = array_start + paren_idx_rel + 1; // position after '('

    // We will scan line-by-line for q-blocks, until we reach a line containing ");"
    let mut in_block = false;
    let mut block_delim = BlockDelim::Brace; // default; will set per block
    let mut current_block: Vec<String> = Vec::new();

    for line in input[cursor..].lines() {
        let trimmed = line.trim_start();

        // End of array
        if !in_block && trimmed.contains(");") {
            break;
        }

        if !in_block {
            // Start of a q-block?
            // We support q{...} and q#...# patterns commonly used in the original file.
            if trimmed.starts_with("q{") {
                in_block = true;
                block_delim = BlockDelim::Brace;
                // If there is content after q{ on the same line, capture it (rare in file)
                if let Some(rest) = trimmed.strip_prefix("q{") {
                    // If same-line close occurs (unlikely in fish arrays), handle it
                    if rest.contains("},") {
                        let before_close = rest.split("},").next().unwrap_or("");
                        if !before_close.is_empty() {
                            current_block.push(before_close.to_string());
                        }
                        // End block immediately
                        results.push(current_block.join("\n"));
                        current_block.clear();
                        in_block = false;
                    } else {
                        // Typically, content starts on the next line; we still store remainder if any
                        if !rest.is_empty() {
                            current_block.push(rest.to_string());
                        }
                    }
                }
            } else if trimmed.starts_with("q#") {
                in_block = true;
                block_delim = BlockDelim::Hash;
                if let Some(rest) = trimmed.strip_prefix("q#") {
                    if rest.contains("#,") {
                        let before_close = rest.split("#,").next().unwrap_or("");
                        if !before_close.is_empty() {
                            current_block.push(before_close.to_string());
                        }
                        results.push(current_block.join("\n"));
                        current_block.clear();
                        in_block = false;
                    } else if !rest.is_empty() {
                        current_block.push(rest.to_string());
                    }
                }
            }
            // otherwise ignore non-q lines
        } else {
            // Inside a q-block: collect lines until delimiter close
            let close_hit = match block_delim {
                BlockDelim::Brace => trimmed == "}," || trimmed.ends_with("},"),
                BlockDelim::Hash => trimmed == "#," || trimmed.ends_with("#,"),
            };

            if close_hit {
                // End current block. We don't include the closing line.
                let art = current_block.join("\n");
                results.push(art);
                current_block.clear();
                in_block = false;
            } else {
                current_block.push(line.to_string());
            }
        }
    }

    // results contains [art, mask, art, mask, ...]; keep only even indices
    results
        .into_iter()
        .enumerate()
        .filter_map(|(i, s)| {
            if i % 2 == 0 {
                Some(trim_trailing_newlines(&s))
            } else {
                None
            }
        })
        .collect()
}

#[derive(Clone, Copy)]
enum BlockDelim {
    Brace,
    Hash,
}

/// Trim up to one trailing newline for a cleaner const block, but preserve interior newlines.
fn trim_trailing_newlines(s: &str) -> String {
    let mut out = s.to_string();
    while out.ends_with('\n') {
        out.pop();
    }
    out
}

/// Generate the Rust module source containing constants and a helper function.
///
/// The generated module is self-contained and does not depend on the existing
/// `asciiquarium_assets` module. It exposes:
/// - const FISH_N: &str = "...";
/// - pub fn get_generated_fish_assets() -> Vec<FishArt>
/// - a local `measure_art` function.
fn generate_rust_module(art_blocks: &[String]) -> String {
    let mut out = String::new();

    out.push_str("//! AUTO-GENERATED FILE: Do not edit by hand.\n");
    out.push_str("//!\n");
    out.push_str(
        "//! Generated by `src/bin/extract_fish.rs` from the original Perl Asciiquarium script.\n",
    );
    out.push_str(
        "//! This module provides fish ASCII assets as &'static str constants and a helper\n",
    );
    out.push_str("//! to build Vec<FishArt> for use with the Asciiquarium widget.\n\n");

    out.push_str("use super::asciiquarium::FishArt;\n\n");

    for (i, art) in art_blocks.iter().enumerate() {
        let const_name = format!("FISH_{:04}", i + 1);
        let escaped = escape_for_rust_string(art);
        out.push_str(&format!(
            "pub const {}: &str = \"{}\";\n",
            const_name, escaped
        ));
    }

    out.push_str("\nfn measure_art(art: &str) -> (usize, usize) {\n");
    out.push_str("    let mut max_w = 0usize;\n");
    out.push_str("    let mut h = 0usize;\n");
    out.push_str("    for line in art.lines() {\n");
    out.push_str("        let w = line.chars().count();\n");
    out.push_str("        if w > max_w { max_w = w; }\n");
    out.push_str("        h += 1;\n");
    out.push_str("    }\n");
    out.push_str("    (max_w.max(1), h.max(1))\n");
    out.push_str("}\n\n");

    out.push_str("pub fn get_generated_fish_assets() -> Vec<FishArt> {\n");
    out.push_str("    let mut out = Vec::new();\n");
    if !art_blocks.is_empty() {
        out.push_str("    let arts: &[&str] = &[\n");
        for i in 0..art_blocks.len() {
            out.push_str(&format!("        FISH_{:04},\n", i + 1));
        }
        out.push_str("    ];\n");
        out.push_str("    for art in arts {\n");
        out.push_str("        let (w, h) = measure_art(art);\n");
        out.push_str("        out.push(FishArt { art, width: w, height: h });\n");
        out.push_str("    }\n");
    }
    out.push_str("    out\n");
    out.push_str("}\n");

    out
}

/// Escape a string for inclusion in a standard Rust string literal.
/// - Escapes backslashes and double quotes
/// - Converts CRLF to LF
/// - Preserves newlines as \\n
fn escape_for_rust_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for ch in s.replace("\r\n", "\n").chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_trailing_newlines() {
        assert_eq!(trim_trailing_newlines("abc\n"), "abc");
        assert_eq!(trim_trailing_newlines("abc\n\n"), "abc");
        assert_eq!(trim_trailing_newlines("abc"), "abc");
    }

    #[test]
    fn test_escape_for_rust_string() {
        let s = "a\\b\"c\nd";
        let e = escape_for_rust_string(s);
        assert_eq!(e, "a\\\\b\\\"c\\nd");
    }

    #[test]
    fn test_generate_empty_module() {
        let m = generate_rust_module(&[]);
        assert!(m.contains("get_generated_fish_assets"));
        // Should not include arts array if empty
        assert!(!m.contains("let arts: &[&str] = &["));
    }
}
