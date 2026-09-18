//! Presentation for the family CLI: one colour rule, one palette, one renderer.
//!
//! [`CoordError`] has always carried four facts beyond its code and reason: the
//! boundary's purpose, at least two concrete fixes, a narrow next step, and a
//! documentation anchor. `repair_metadata` fills every one of them for every
//! code. Until this module existed all four were discarded at the print site,
//! because `Display` writes `{code}: {reason}` and nothing read the rest, so an
//! operator saw a bare token and had to grep the source to learn what to do.
//!
//! `Display` keeps that exact shape. It is the machine contract, and a great
//! deal of tooling greps for `CODE: reason` on one line. This module is what a
//! human sees instead, and the first line it prints still starts with the code
//! so those greps keep working on the rendered form too.
//!
//! The colour rule lives here rather than at each print site because the family
//! already learned that lesson the expensive way: three different `NO_COLOR`
//! and tty rules in one binary meant `NO_COLOR=1` silently changed behaviour
//! that had nothing to do with colour. There is one rule, it is pure, and it is
//! tested against the terminals that actually exist rather than assumed.

use std::fmt::Write as _;

use crate::coord::CoordError;

/// What the caller asked for, before the terminal gets a say.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ColorChoice {
    /// Decide from the environment. The default, and what `--color=auto` means.
    #[default]
    Auto,
    /// Paint even when the stream is a pipe. `--color=always`.
    Always,
    /// Never paint. `--no-color`, `--plain`, or `--color=never`.
    Never,
}

impl ColorChoice {
    /// Parse the `--color=<when>` value, rejecting anything not in the set.
    pub fn parse(value: &str) -> Result<Self, CoordError> {
        match value {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err(CoordError::new(
                "USAGE",
                format!("--color expects auto, always or never, not {value}"),
            )),
        }
    }
}

/// How much colour this terminal can be trusted with.
///
/// A palette that emits truecolor escapes unconditionally is not a palette, it
/// is a bet. These four tiers are what terminals actually advertise.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ColorDepth {
    None,
    /// The eight SGR colours every terminal has had for decades.
    Basic,
    Ansi256,
    TrueColor,
}

/// Everything the colour decision reads, passed in rather than looked up, so
/// the rule can be tested without mutating the process environment.
#[derive(Clone, Copy, Debug, Default)]
pub struct TerminalFacts<'a> {
    pub is_terminal: bool,
    pub no_color: Option<&'a str>,
    pub clicolor_force: Option<&'a str>,
    pub term: Option<&'a str>,
    pub colorterm: Option<&'a str>,
}

impl TerminalFacts<'_> {
    /// The richest tier this terminal advertises, ignoring whether we may use it.
    fn advertised(&self) -> ColorDepth {
        let colorterm = self.colorterm.unwrap_or_default();
        if colorterm.eq_ignore_ascii_case("truecolor") || colorterm.eq_ignore_ascii_case("24bit") {
            return ColorDepth::TrueColor;
        }
        match self.term {
            Some(term) if term.contains("256color") => ColorDepth::Ansi256,
            Some(term) if !term.is_empty() && term != "dumb" => ColorDepth::Basic,
            _ => ColorDepth::None,
        }
    }
}

/// The single colour rule for this binary.
///
/// `NO_COLOR` wins over everything except an explicit `--color=always`, which is
/// a person overriding their own environment on purpose. A `dumb` or absent
/// `TERM` is treated as no colour even when forced, because there is no escape
/// sequence that is safe to send to a terminal that has told us it is dumb.
pub fn depth_for(choice: ColorChoice, facts: TerminalFacts<'_>) -> ColorDepth {
    match choice {
        ColorChoice::Never => ColorDepth::None,
        ColorChoice::Always => facts.advertised().max(ColorDepth::Basic),
        ColorChoice::Auto => {
            // no-color.org: honoured when present and not empty.
            if facts.no_color.is_some_and(|value| !value.is_empty()) {
                return ColorDepth::None;
            }
            let forced = facts
                .clicolor_force
                .is_some_and(|value| !value.is_empty() && value != "0");
            if !forced && !facts.is_terminal {
                return ColorDepth::None;
            }
            facts.advertised()
        }
    }
}

/// Read the real environment for the stream the caller is about to write to.
pub fn facts_for(is_terminal: bool) -> OwnedTerminalFacts {
    OwnedTerminalFacts {
        is_terminal,
        no_color: std::env::var("NO_COLOR").ok(),
        clicolor_force: std::env::var("CLICOLOR_FORCE").ok(),
        term: std::env::var("TERM").ok(),
        colorterm: std::env::var("COLORTERM").ok(),
    }
}

/// Owned form of [`TerminalFacts`], because `std::env::var` returns `String`.
#[derive(Clone, Debug, Default)]
pub struct OwnedTerminalFacts {
    pub is_terminal: bool,
    pub no_color: Option<String>,
    pub clicolor_force: Option<String>,
    pub term: Option<String>,
    pub colorterm: Option<String>,
}

impl OwnedTerminalFacts {
    pub fn borrow(&self) -> TerminalFacts<'_> {
        TerminalFacts {
            is_terminal: self.is_terminal,
            no_color: self.no_color.as_deref(),
            clicolor_force: self.clicolor_force.as_deref(),
            term: self.term.as_deref(),
            colorterm: self.colorterm.as_deref(),
        }
    }
}

/// The named roles this binary paints. Six, not forty-six: a role that cannot
/// be named is a role that will quietly acquire a second meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    /// The refusal identifier itself.
    Code,
    /// The left column of the detail block.
    Label,
    /// Ordinary prose.
    Body,
    /// A documentation anchor or a path worth the eye.
    Accent,
    /// Something the operator can act on.
    Action,
    /// Text that is present but deliberately quiet.
    Muted,
}

impl Role {
    /// One palette, three tiers, chosen so the Basic tier is never a lie.
    const fn sgr(self, depth: ColorDepth) -> &'static str {
        match depth {
            ColorDepth::None => "",
            ColorDepth::Basic => match self {
                Self::Code => "\u{1b}[1;31m",
                Self::Label => "\u{1b}[1m",
                Self::Body => "",
                Self::Accent => "\u{1b}[36m",
                Self::Action => "\u{1b}[32m",
                Self::Muted => "\u{1b}[2m",
            },
            ColorDepth::Ansi256 => match self {
                Self::Code => "\u{1b}[1;38;5;203m",
                Self::Label => "\u{1b}[1;38;5;250m",
                Self::Body => "",
                Self::Accent => "\u{1b}[38;5;117m",
                Self::Action => "\u{1b}[38;5;114m",
                Self::Muted => "\u{1b}[38;5;244m",
            },
            ColorDepth::TrueColor => match self {
                Self::Code => "\u{1b}[1;38;2;255;110;110m",
                Self::Label => "\u{1b}[1;38;2;200;206;220m",
                Self::Body => "",
                Self::Accent => "\u{1b}[38;2;169;199;255m",
                Self::Action => "\u{1b}[38;2;134;214;158m",
                Self::Muted => "\u{1b}[38;2;138;146;164m",
            },
        }
    }
}

const RESET: &str = "\u{1b}[0m";

/// Wrap `paint` around `text`, or return `text` unchanged when not painting.
fn paint(text: &str, role: Role, depth: ColorDepth) -> String {
    let sgr = role.sgr(depth);
    // Painting whitespace costs bytes and buys nothing, and the continuation
    // rows of a multi-line value are entirely whitespace in the label column.
    if sgr.is_empty() || text.trim().is_empty() {
        return text.to_string();
    }
    format!("{sgr}{text}{RESET}")
}

/// The column the detail block's values start at. Wide enough for every label
/// this module uses, narrow enough to leave room for prose at 80 columns.
const VALUE_COLUMN: usize = 14;
const INDENT: &str = "  ";
const DEFAULT_WIDTH: usize = 80;
/// Below this there is no useful wrapping left to do, so wrapping stops rather
/// than producing one word per line forever.
const MIN_PROSE_WIDTH: usize = 24;

/// Terminal width from `COLUMNS`, clamped to something a human can read.
pub fn width_from_env() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| columns(&value))
        .filter(|width| *width >= 40)
        .map_or(DEFAULT_WIDTH, |width| width.min(120))
}

/// ASCII digits to a width, folded by hand rather than through `str::parse`.
///
/// `crates/bullet-wire/tests/canonical_hostile/metadata.rs` inventories every
/// `.parse(` call in production Rust and admits exactly one, in bullet-wire's
/// canonical decoder, so that no other decode path can grow quietly. A terminal
/// width is not a reason to widen that inventory, and the explicit fold is the
/// better code anyway: it cannot panic and it cannot overflow.
fn columns(value: &str) -> Option<usize> {
    let digits = value.trim().as_bytes();
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let mut width: usize = 0;
    for digit in digits {
        width = width
            .checked_mul(10)?
            .checked_add(usize::from(digit - b'0'))?;
    }
    Some(width)
}

/// Greedy word wrap. Words longer than the width are left whole rather than
/// broken, because the long words here are paths, digests and codes, and a
/// broken digest is worse than a ragged line.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(MIN_PROSE_WIDTH);
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.chars().count() + 1 + word.chars().count() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Write one `label   value` row, wrapping the value under itself.
fn row(out: &mut String, label: &str, value: &str, depth: ColorDepth, width: usize) {
    let prose_width = width.saturating_sub(INDENT.len() + VALUE_COLUMN);
    let mut lines = wrap(value, prose_width).into_iter();
    let Some(first) = lines.next() else {
        return;
    };
    let padded = format!("{label:<VALUE_COLUMN$}");
    let _ = writeln!(
        out,
        "{INDENT}{}{}",
        paint(&padded, Role::Label, depth),
        paint(&first, Role::Body, depth)
    );
    for line in lines {
        let _ = writeln!(
            out,
            "{INDENT}{:<VALUE_COLUMN$}{}",
            "",
            paint(&line, Role::Body, depth)
        );
    }
}

/// Render a refusal the way an operator needs to read it.
///
/// The first line is `bullet-family: CODE`, so a grep for the code still finds
/// it. Everything below is the metadata the type has always carried.
pub fn render(error: &CoordError, depth: ColorDepth, width: usize) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{} {}",
        paint("bullet-family:", Role::Muted, depth),
        paint(error.code(), Role::Code, depth)
    );

    // `USAGE` carries a command listing as its reason. It is already laid out,
    // and rewrapping a layout destroys it, so it is printed as written.
    if error.code() == "USAGE" && error.reason().starts_with("usage:") {
        for line in error.reason().lines() {
            if line.is_empty() {
                out.push('\n');
            } else {
                let _ = writeln!(out, "{INDENT}{}", paint(line, Role::Body, depth));
            }
        }
        return out;
    }

    // The reason is never hard-wrapped. `Display` is `{code}: {reason}` and a
    // great deal of tooling greps stderr for a phrase inside it; inserting a
    // newline mid-phrase silently breaks every one of those greps. A terminal
    // soft-wraps a long line on its own, which costs nothing and keeps the text
    // contiguous. Only the metadata below, which nothing greps because until
    // now it was never printed, is laid out to a width.
    for line in error.reason().lines() {
        let _ = writeln!(out, "{INDENT}{}", paint(line, Role::Body, depth));
    }
    if error.reason().is_empty() {
        out.push('\n');
    }

    let has_detail = !error.purpose().is_empty()
        || !error.common_fixes().is_empty()
        || !error.repair_hint().is_empty()
        || !error.docs_url().is_empty();
    if !has_detail {
        return out;
    }
    out.push('\n');

    if !error.purpose().is_empty() {
        row(&mut out, "protecting", error.purpose(), depth, width);
    }
    let mut fixes = error.common_fixes().iter();
    if let Some(first) = fixes.next() {
        row(&mut out, "try", &format!("- {first}"), depth, width);
        for fix in fixes {
            row(&mut out, "", &format!("- {fix}"), depth, width);
        }
    }
    if !error.repair_hint().is_empty() {
        row(&mut out, "next", error.repair_hint(), depth, width);
    }
    if !error.docs_url().is_empty() {
        let padded = format!("{:<VALUE_COLUMN$}", "docs");
        let _ = writeln!(
            out,
            "{INDENT}{}{}",
            paint(&padded, Role::Label, depth),
            paint(error.docs_url(), Role::Accent, depth)
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(
        term: &'static str,
        colorterm: &'static str,
        is_terminal: bool,
    ) -> TerminalFacts<'static> {
        TerminalFacts {
            is_terminal,
            term: Some(term),
            colorterm: if colorterm.is_empty() {
                None
            } else {
                Some(colorterm)
            },
            ..TerminalFacts::default()
        }
    }

    #[test]
    fn a_pipe_is_never_painted_without_being_asked() {
        assert_eq!(
            depth_for(ColorChoice::Auto, facts("xterm-256color", "", false)),
            ColorDepth::None
        );
    }

    #[test]
    fn a_terminal_gets_the_depth_it_advertises() {
        assert_eq!(
            depth_for(ColorChoice::Auto, facts("xterm-256color", "", true)),
            ColorDepth::Ansi256
        );
        assert_eq!(
            depth_for(
                ColorChoice::Auto,
                facts("xterm-256color", "truecolor", true)
            ),
            ColorDepth::TrueColor
        );
        assert_eq!(
            depth_for(ColorChoice::Auto, facts("xterm", "", true)),
            ColorDepth::Basic
        );
    }

    #[test]
    fn a_dumb_terminal_is_never_painted_even_when_forced() {
        assert_eq!(
            depth_for(ColorChoice::Auto, facts("dumb", "", true)),
            ColorDepth::None
        );
        // Always still has to send something a dumb terminal can survive, and
        // the safest thing to send it is nothing beyond the basic set.
        assert_eq!(
            depth_for(ColorChoice::Always, facts("dumb", "", true)),
            ColorDepth::Basic
        );
    }

    #[test]
    fn no_color_wins_over_a_capable_terminal_but_not_over_an_explicit_ask() {
        let mut set = facts("xterm-256color", "truecolor", true);
        set.no_color = Some("1");
        assert_eq!(depth_for(ColorChoice::Auto, set), ColorDepth::None);
        assert_eq!(depth_for(ColorChoice::Always, set), ColorDepth::TrueColor);
    }

    #[test]
    fn an_empty_no_color_is_not_a_no_color() {
        let mut set = facts("xterm-256color", "", true);
        set.no_color = Some("");
        assert_eq!(depth_for(ColorChoice::Auto, set), ColorDepth::Ansi256);
    }

    #[test]
    fn clicolor_force_paints_a_pipe_but_zero_does_not() {
        let mut set = facts("xterm-256color", "", false);
        set.clicolor_force = Some("1");
        assert_eq!(depth_for(ColorChoice::Auto, set), ColorDepth::Ansi256);
        set.clicolor_force = Some("0");
        assert_eq!(depth_for(ColorChoice::Auto, set), ColorDepth::None);
    }

    #[test]
    fn color_choice_parses_exactly_three_values() {
        assert_eq!(ColorChoice::parse("auto").unwrap(), ColorChoice::Auto);
        assert_eq!(ColorChoice::parse("always").unwrap(), ColorChoice::Always);
        assert_eq!(ColorChoice::parse("never").unwrap(), ColorChoice::Never);
        let refusal = ColorChoice::parse("sometimes").unwrap_err();
        assert_eq!(refusal.code(), "USAGE");
    }

    /// Collapse runs of whitespace so a content assertion is not also a layout
    /// assertion. Wrapping is deliberate and is asserted separately.
    fn flat(text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn an_unpainted_render_leads_with_the_code_and_carries_the_metadata() {
        let error = CoordError::new("CLAIM_OVERLAP", "src/coord/mod.rs is already claimed");
        let rendered = render(&error, ColorDepth::None, DEFAULT_WIDTH);
        let first = rendered.lines().next().unwrap();
        assert!(
            first.contains("CLAIM_OVERLAP"),
            "the code must survive on line one for greps: {first}"
        );
        let flattened = flat(&rendered);
        assert!(flattened.contains("src/coord/mod.rs is already claimed"));
        assert!(flattened.contains(&flat(error.purpose())));
        assert!(flattened.contains(&flat(error.repair_hint())));
        assert!(flattened.contains(error.docs_url()));
        for fix in error.common_fixes() {
            assert!(flattened.contains(&flat(fix)), "missing fix: {fix}");
        }
        assert!(
            !rendered.contains('\u{1b}'),
            "an unpainted render must contain no escape"
        );
    }

    #[test]
    fn every_code_the_family_can_raise_renders_a_complete_repair_block() {
        // repair_metadata answers for every code, so no refusal should reach an
        // operator as a bare token. A code with no metadata would render as a
        // reason and nothing else, which is the failure this whole module exists
        // to end, and it must fail here rather than in front of a person.
        for code in [
            "CLAIM_OVERLAP",
            "CORRUPT_COORD_LOG",
            "UNSUPPORTED_SCHEMA",
            "PARTIAL_COORD_WRITE",
            "COORD_IO_FAILED",
            "COORD_JSON_FAILED",
            "USAGE",
            "INVALID_ARGUMENT",
        ] {
            let error = CoordError::new(code, "reason");
            let rendered = flat(&render(&error, ColorDepth::None, DEFAULT_WIDTH));
            assert!(
                rendered.contains("protecting"),
                "{code} rendered without a purpose"
            );
            assert!(rendered.contains("docs/errors.md#"), "{code} has no anchor");
            assert!(
                error.common_fixes().len() >= 2,
                "{code} offers fewer than two fixes"
            );
        }
    }

    #[test]
    fn every_repair_anchor_reaches_the_operator_documentation() {
        // The anchors are only useful if they resolve. This asserts the shape
        // that docs/errors.md is indexed by; docs_links.rs asserts they exist.
        let error = CoordError::new("CLAIM_OVERLAP", "reason");
        assert!(
            error.docs_url().starts_with("docs/errors.md#"),
            "unexpected anchor {}",
            error.docs_url()
        );
        assert!(
            error.common_fixes().len() >= 2,
            "a repair block with fewer than two choices is not a repair block"
        );
    }

    #[test]
    fn a_painted_render_resets_every_sequence_it_opens() {
        let error = CoordError::new("CLAIM_OVERLAP", "src/coord/mod.rs is already claimed");
        let rendered = render(&error, ColorDepth::TrueColor, DEFAULT_WIDTH);
        let opens = rendered.matches('\u{1b}').count();
        let resets = rendered.matches(RESET).count();
        assert_eq!(
            opens,
            resets * 2,
            "every opened sequence must be closed exactly once"
        );
    }

    #[test]
    fn display_is_still_the_machine_contract() {
        let error = CoordError::new("CLAIM_OVERLAP", "reason");
        assert_eq!(error.to_string(), "CLAIM_OVERLAP: reason");
    }

    #[test]
    fn a_usage_listing_is_printed_as_laid_out_not_rewrapped() {
        let listing = "usage: bullet-family <command>\n\n  doctor        report on the hub\n";
        let error = CoordError::new("USAGE", listing);
        let rendered = render(&error, ColorDepth::None, 40);
        assert!(rendered.contains("  doctor        report on the hub"));
    }

    #[test]
    fn a_reason_is_never_hard_wrapped_because_tooling_greps_it() {
        let reason = "legacy-v1-26 is diagnostic only and is not a release profile, so \
                      an ordinary release must name one of the other nineteen";
        let error = CoordError::new("PROFILE_REQUIRED", reason);
        let rendered = render(&error, ColorDepth::None, 40);
        assert!(
            rendered.contains(reason),
            "a 40-column render must still carry the reason contiguously"
        );
    }

    #[test]
    fn a_width_is_read_only_from_digits_and_never_overflows() {
        assert_eq!(columns("100"), Some(100));
        assert_eq!(columns("  80  "), Some(80));
        assert_eq!(columns(""), None);
        assert_eq!(columns("80x"), None);
        assert_eq!(columns("-80"), None);
        assert_eq!(columns("1".repeat(40).as_str()), None);
    }

    #[test]
    fn wrapping_never_breaks_a_digest_and_never_loses_a_word() {
        let digest = "a".repeat(64);
        let text = format!("subject {digest} rejected");
        let lines = wrap(&text, 30);
        assert!(lines.iter().any(|line| line.contains(&digest)));
        let rejoined = lines.join(" ");
        assert_eq!(rejoined.split_whitespace().count(), 3);
    }

    #[test]
    fn a_narrow_terminal_still_produces_readable_rows() {
        let error = CoordError::new("CLAIM_OVERLAP", "src/coord/mod.rs is already claimed");
        let rendered = render(&error, ColorDepth::None, 40);
        assert!(rendered.contains("protecting"));
        assert!(rendered.contains("docs/errors.md#"));
    }
}
