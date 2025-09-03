/*!
Asciiquarium assets: a small curated set of ASCII fish plus a helper to auto-measure width and height for any multi-line ASCII art.

Notes:
- Width is the maximum visible character count across all lines (Unicode scalar count).
- Height is the total number of lines returned by `.lines()`.
- Leading/trailing blank lines in raw strings will count toward height.
- Rendering code will clip as needed; assets need not avoid whitespace.

These assets are intentionally simple; expand as desired with additional creatures.

Example:
```text
<º)))><
```
*/

use super::asciiquarium::FishArt;

const FISH_01: &str = r#"<º)))><"#; // Facing right
const FISH_02: &str = r#"><(((º>"#; // Facing left

// A larger, multi-line fish (faces right on the first line)
const FISH_03: &str = r#"
   __
><(o )___
 ( .__> /
  `----'
"#;

// A small two-line fish, slightly stylized
const FISH_04: &str = r#"
><>
<__>
"#;

// A tiny angler-like fish
const FISH_05: &str = r#"
  __
q(==)p
  \/
"#;

// Additional fish inspired by the original Asciiquarium (simplified)

// Old fish variant A (right-facing)
const FISH_06: &str = r#"
       \
     ...\..,
\  /'       \
 >=     (  ' >
/  \      / /
    `"'"'/''
"#;

// Old fish variant B (left-facing)
const FISH_07: &str = r#"
      /
  ,../...
 /       '\  /
< '  )     =<
 \ \      /  \
  `'\'"'"'
"#;

// Old fish variant C (right-facing)
const FISH_08: &str = r#"
    \
\ /--\
>=  (o>
/ \__/
    /
"#;

// Old fish variant D (left-facing)
const FISH_09: &str = r#"
  /
 /--\ /
<o)  =<
 \__/ \
  \
"#;

// Tiny fish variant E (down-right)
const FISH_10: &str = r#"
  __
\/ o\
/\__/
"#;

// Tiny fish variant F (down-left)
const FISH_11: &str = r#"
 __
/o \/
\__/\
"#;

// Small fish variant G (right-facing)
const FISH_12: &str = r#"
  ,\
>=('>
  '/
"#;

// Small fish variant H (left-facing)
const FISH_13: &str = r#"
 /,
<')=<
 \`
"#;

/// Measure an ASCII art block's dimensions as (width, height),
/// where width is the maximum character count of any line and height
/// is the total number of lines. Guarantees minimum size of 1x1.
pub fn measure_art(art: &str) -> (usize, usize) {
    let mut max_w = 0usize;
    let mut h = 0usize;
    for line in art.lines() {
        let w = line.chars().count();
        if w > max_w {
            max_w = w;
        }
        h += 1;
    }
    (max_w.max(1), h.max(1))
}

/// Returns a vector of `FishArt` with auto-measured width/height.
/// Add more constants above and insert them in the list below to expand the set.
pub fn get_fish_assets() -> Vec<FishArt> {
    let mut out = Vec::new();
    for art in [
        FISH_01, FISH_02, FISH_03, FISH_04, FISH_05, FISH_06, FISH_07, FISH_08, FISH_09, FISH_10,
        FISH_11, FISH_12, FISH_13,
    ] {
        let (w, h) = measure_art(art);
        out.push(FishArt {
            art,
            width: w,
            height: h,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_single_line() {
        let (w, h) = measure_art(FISH_01);
        assert_eq!(h, 1);
        assert!(w >= 1);
    }

    #[test]
    fn measure_multi_line() {
        let (w, h) = measure_art(FISH_03);
        // Height is >= 1 and equals the number of lines produced by `.lines()`
        assert!(h >= 1);
        // Width should be at least the largest visible line
        assert!(w >= 4);
    }

    #[test]
    fn assets_non_empty() {
        let assets = get_fish_assets();
        assert!(!assets.is_empty());
        for a in assets {
            assert!(a.width >= 1 && a.height >= 1);
            assert!(!a.art.is_empty());
        }
    }
}
