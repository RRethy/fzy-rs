use std::mem::swap;

pub type Score = f64;

const SCORE_MIN: Score = Score::NEG_INFINITY;
const SCORE_MAX: Score = Score::INFINITY;
const SCORE_GAP_LEADING: Score = -0.005;
const SCORE_GAP_TRAILING: Score = -0.005;
const SCORE_GAP_INNER: Score = -0.01;
const SCORE_MATCH_CONSECUTIVE: Score = 1.0;
const SCORE_MATCH_SLASH: Score = 0.9;
const SCORE_MATCH_WORD: Score = 0.8;
const SCORE_MATCH_CAPITAL: Score = 0.7;
const SCORE_MATCH_DOT: Score = 0.6;

#[inline]
fn max(f1: Score, f2: Score) -> Score {
    if f1 > f2 {
        f1
    } else {
        f2
    }
}

#[inline]
fn compute_bonus(cur: u8, prev: u8) -> Score {
    match cur {
        b'A'..=b'Z' => match prev {
            b'a'..=b'z' => SCORE_MATCH_CAPITAL,
            b'/' => SCORE_MATCH_SLASH,
            b'-' | b'_' | b' ' => SCORE_MATCH_WORD,
            b'.' => SCORE_MATCH_DOT,
            _ => 0.0,
        },
        b'a'..=b'z' | b'0'..=b'9' => match prev {
            b'/' => SCORE_MATCH_SLASH,
            b'-' | b'_' | b' ' => SCORE_MATCH_WORD,
            b'.' => SCORE_MATCH_DOT,
            _ => 0.0,
        },
        _ => 0.0,
    }
}

#[inline]
fn compute_bonuses(text: &[u8]) -> Vec<Score> {
    let (_, bonuses) = text.iter().enumerate().fold(
        (b'/', vec![0.0; text.len()]),
        |(prev, mut acc), (i, cur)| {
            acc[i] = compute_bonus(*cur, prev);
            (*cur, acc)
        },
    );
    bonuses
}

pub fn has_match(pat: &[u8], text: &[u8]) -> bool {
    if pat.is_empty() {
        return true;
    }

    let mut pi = 0;
    for tc in text {
        if *tc == pat[pi] {
            pi += 1;
        }
        if pi == pat.len() {
            return true;
        }
    }
    return pi == pat.len();
}

pub fn score(pat: &[u8], text: &[u8]) -> Score {
    if pat.len() == 0 || pat.len() > text.len() {
        return SCORE_MIN;
    }
    if pat.len() == text.len() {
        return SCORE_MAX;
    }

    let bonuses = compute_bonuses(text);

    let mut prev_d = vec![0.0; text.len()];
    let mut cur_d = vec![0.0; text.len()];
    let mut prev_m = vec![0.0; text.len()];
    let mut cur_m = vec![0.0; text.len()];

    for (pi, pc) in pat.to_ascii_lowercase().iter().enumerate() {
        let mut prev_score = SCORE_MIN;
        let gap_score = if pi == pat.len() - 1 {
            SCORE_GAP_TRAILING
        } else {
            SCORE_GAP_INNER
        };

        for (ti, tc) in text.to_ascii_lowercase().iter().enumerate() {
            if pc == tc {
                let score = if pi == 0 {
                    (ti as Score) * SCORE_GAP_LEADING + bonuses[ti]
                } else if ti > 0 {
                    max(
                        prev_m[ti - 1] + bonuses[ti],
                        prev_d[ti - 1] + SCORE_MATCH_CONSECUTIVE,
                    )
                } else {
                    SCORE_MIN
                };
                cur_d[ti] = score;
                prev_score = max(score, prev_score + gap_score);
                cur_m[ti] = prev_score;
            } else {
                cur_d[ti] = SCORE_MIN;
                prev_score += gap_score;
                cur_m[ti] = prev_score;
            }
        }

        swap(&mut cur_d, &mut prev_d);
        swap(&mut cur_m, &mut prev_m);
    }
    *prev_m.last().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_prefer_starts_of_words() {
        // App/Models/Order is better than App/MOdels/zRder
        assert!(score(b"amor", b"app/models/order") > score(b"amor", b"app/models/zrder"));
    }

    #[test]
    fn should_prefer_consecutive_letters() {
        // App/MOdels/foo is better than App/M/fOo
        assert!(score(b"amo", b"app/m/foo") < score(b"amo", b"app/models/foo"));
    }

    #[test]
    fn should_prefer_contiguous_over_letter_following_period() {
        // GEMFIle.Lock < GEMFILe
        assert!(score(b"gemfil", b"Gemfile.lock") < score(b"gemfil", b"Gemfile"));
    }

    #[test]
    fn should_prefer_shorter_matches() {
        assert!(score(b"abce", b"abcdef") > score(b"abce", b"abc de"));
        assert!(score(b"abc", b"    a b c ") > score(b"abc", b" a  b  c "));
        assert!(score(b"abc", b" a b c    ") > score(b"abc", b" a  b  c "));
    }

    #[test]
    fn should_prefer_shorter_candidates() {
        assert!(score(b"test", b"tests") > score(b"test", b"testing"));
    }

    #[test]
    fn should_prefer_start_of_candidate() {
        // Scores first letter highly
        assert!(score(b"test", b"testing") > score(b"test", b"/testing"));
    }

    #[test]
    fn score_exact_match() {
        // Exact fzy_score is SCORE_MAX
        assert_eq!(SCORE_MAX, score(b"abc", b"abc"));
        assert_eq!(SCORE_MAX, score(b"aBc", b"abC"));
    }

    #[test]
    fn score_empty_query() {
        // Empty query always results in SCORE_MIN
        assert_eq!(SCORE_MIN, score(b"", b""));
        assert_eq!(SCORE_MIN, score(b"", b"a"));
        assert_eq!(SCORE_MIN, score(b"", b"bb"));
    }

    #[test]
    fn score_gaps() {
        assert_eq!(SCORE_GAP_LEADING, score(b"a", b"*a"));
        assert_eq!(SCORE_GAP_LEADING * 2.0, score(b"a", b"*ba"));
        assert_eq!(
            SCORE_GAP_LEADING * 2.0 + SCORE_GAP_TRAILING,
            score(b"a", b"**a*")
        );
        assert_eq!(
            SCORE_GAP_LEADING * 2.0 + SCORE_GAP_TRAILING * 2.0,
            score(b"a", b"**a**")
        );
        assert_eq!(
            SCORE_GAP_LEADING * 2.0 + SCORE_MATCH_CONSECUTIVE + SCORE_GAP_TRAILING * 2.0,
            score(b"aa", b"**aa**")
        );
        assert_eq!(
            SCORE_GAP_LEADING
                + SCORE_GAP_LEADING
                + SCORE_GAP_INNER
                + SCORE_GAP_TRAILING
                + SCORE_GAP_TRAILING,
            score(b"aa", b"**a*a**")
        );
    }

    #[test]
    fn score_consecutive() {
        assert_eq!(
            SCORE_GAP_LEADING + SCORE_MATCH_CONSECUTIVE,
            score(b"aa", b"*aa")
        );
        assert_eq!(
            SCORE_GAP_LEADING + SCORE_MATCH_CONSECUTIVE * 2.0,
            score(b"aaa", b"*aaa")
        );
        assert_eq!(
            SCORE_GAP_LEADING + SCORE_GAP_INNER + SCORE_MATCH_CONSECUTIVE,
            score(b"aaa", b"*a*aa")
        );
    }

    #[test]
    fn score_slash() {
        assert_eq!(SCORE_GAP_LEADING + SCORE_MATCH_SLASH, score(b"a", b"/a"));
        assert_eq!(
            SCORE_GAP_LEADING * 2.0 + SCORE_MATCH_SLASH,
            score(b"a", b"*/a")
        );
        assert_eq!(
            SCORE_GAP_LEADING * 2.0 + SCORE_MATCH_SLASH + SCORE_MATCH_CONSECUTIVE,
            score(b"aa", b"a/aa")
        );
    }

    #[test]
    fn score_capital() {
        assert_eq!(SCORE_GAP_LEADING + SCORE_MATCH_CAPITAL, score(b"a", b"bA"));
        assert_eq!(
            SCORE_GAP_LEADING * 2.0 + SCORE_MATCH_CAPITAL,
            score(b"a", b"baA")
        );
        assert_eq!(
            SCORE_GAP_LEADING * 2.0 + SCORE_MATCH_CAPITAL + SCORE_MATCH_CONSECUTIVE,
            score(b"aa", b"baAa")
        );
    }

    #[test]
    fn score_dot() {
        assert_eq!(SCORE_GAP_LEADING + SCORE_MATCH_DOT, score(b"a", b".a"));
        assert_eq!(
            SCORE_GAP_LEADING * 3.0 + SCORE_MATCH_DOT,
            score(b"a", b"*a.a")
        );
        assert_eq!(
            SCORE_GAP_LEADING + SCORE_GAP_INNER + SCORE_MATCH_DOT,
            score(b"a", b"*a.a")
        );
    }

    #[test]
    fn score_long_string() {
        let string: [u8; 4096] = [b'a'; 4096];
        assert_eq!(SCORE_MIN, score(&string, b"aa"));
        assert_eq!(SCORE_MAX, score(&string, &string));
    }

    #[test]
    fn is_match_matches() {
        assert!(has_match(b"abcd", b"/aqq/bqq/cdef"));
        assert!(has_match(b"abcd", b"abcde"));
        assert!(has_match(b"abcd", b"xabcde"));
        assert!(has_match(b"a", b"a"));
        assert!(has_match(b"a", b"ab"));
        assert!(has_match(b"a", b"ba"));
        assert!(has_match(b"abc", b"a|b|c"));
        assert!(has_match(b"", b""));
        assert!(has_match(b"", b"a"));
    }

    #[test]
    fn is_match_doesnt_match() {
        assert!(!has_match(b"abcd", b"/aqq/cqq/bdef"));
        assert!(!has_match(b"abcd", b"/aqq/bqq/cef"));
        assert!(!has_match(b"abcd", b"ab"));
        assert!(!has_match(b"a", b""));
        assert!(!has_match(b"a", b"b"));
        assert!(!has_match(b"ass", b"tags"));
    }
}
