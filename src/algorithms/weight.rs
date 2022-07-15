use once_cell::sync::OnceCell;
use popoki::{Correctness, Guess, Guesser, Word, DICTIONARY};
use std::borrow::Cow;

static INITIAL: OnceCell<Vec<(&'static Word, usize)>> = OnceCell::new();

pub struct Weight {
    remaining: Cow<'static, Vec<(&'static Word, usize)>>,
}

#[allow(clippy::expect_used)]
impl Weight {
    pub fn new() -> Self {
        Self {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                DICTIONARY
                    .lines()
                    .map(|line| {
                        let (word, count) = line
                            .split_once(' ')
                            .expect("every line is word + space + frequency");
                        let count_parsed: usize = count.parse().expect("every count is a number");
                        let word_bytes = word
                            .as_bytes()
                            .try_into()
                            .expect("every dictionary word is 5 characters");
                        (word_bytes, count_parsed)
                    })
                    .collect()
            })),
        }
    }
}
impl Default for Weight {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static Word,
    goodness: f64,
}

impl Guesser for Weight {
    fn guess(&mut self, history: &[Guess]) -> Word {
        if let Some(last) = history.last() {
            if matches!(self.remaining, Cow::Owned(_)) {
                self.remaining
                    .to_mut()
                    .retain(|&(word, _)| last.matches(word));
            } else {
                self.remaining = Cow::Owned(
                    self.remaining
                        .iter()
                        .filter(|&&(word, _)| last.matches(word))
                        .copied()
                        .collect(),
                );
            }
        }

        // We use `trace` here as popoki's opening guess
        // because it has a high average score  of 10.01 (5.83 + 4.18)
        // based on a two steps approach: expected information (entropy E[i]) +
        // weighted average to guess second word.
        if history.is_empty() {
            return *b"trace";
        }

        let remaining_count: usize = self.remaining.iter().map(|&(_, c)| c).sum();

        let mut best: Option<Candidate> = None;
        for &(word, count) in &*self.remaining {
            let mut sum = 0.0_f64;
            for pattern in Correctness::patterns() {
                let mut in_pattern_total: usize = 0;
                for &(candidate, count_r) in &*self.remaining {
                    let g = Guess {
                        // Here we do not allocate a new copy of the string for a new guess
                        word: Cow::Borrowed(word),
                        mask: pattern,
                    };
                    g.matches(candidate)
                        .then(|| in_pattern_total = in_pattern_total.saturating_add(count_r));
                }
                if in_pattern_total == 0 {
                    continue;
                }
                // TODO: apply sigmoid
                let p_of_this_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += p_of_this_pattern * p_of_this_pattern.log2();
            }
            let p_word: f64 = count as f64 / remaining_count as f64;
            let goodness = p_word * -sum;
            if let Some(c) = best {
                // Is this one better?
                if goodness > c.goodness {
                    best = Some(Candidate { word, goodness });
                }
            } else {
                best = Some(Candidate { word, goodness });
            }
        }

        // Return the best guess OR a default value (it shouldn't never happen though)
        best.map_or(*b"cigar", |c| *c.word)
    }
}
