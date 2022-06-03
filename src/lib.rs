pub mod algorithms;

pub fn play<G: Guesser>(answer: &'static str, mut guesser: G) -> Option<usize> {
    let mut history = Vec::new();

    // Wordle only allows six guesses.
    // Popoki allows more to avoid chopping off the score distribution for stats
    // purposes.
    for i in 1..=32 {
        let guess = guesser.guess(&history);
        if guess == answer {
            return Some(i);
        }
        let correctness = Correctness::compute(answer, &guess);
        history.push(Guess {
            word: guess,
            mask: correctness,
        });
    }
    None
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Correctness {
    // Green
    Correct,
    /// Yellow
    Misplaced,
    /// Gray
    Wrong,
}

impl Correctness {
    /// Given an answer and a guess, return an array of 5 elements, each of which is
    /// a `Result` indicating whether the guess is correct, incorrect, or not present
    pub fn compute(answer: &str, guess: &str) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);
        let mut c = [Correctness::Wrong; 5];

        // Mark things green
        for (i, item) in c.iter_mut().enumerate() {
            if answer.chars().nth(i) == guess.chars().nth(i) {
                *item = Correctness::Correct;
            }
        }

        let mut used = [false; 5];
        for (i, c) in c.iter().enumerate() {
            if *c == Correctness::Correct {
                used[i] = true;
            }
        }

        // Mark things yellow
        for (i, g) in guess.chars().enumerate() {
            if c[i] == Correctness::Correct {
                // Already marked as green
                continue;
            }
            if answer.chars().enumerate().any(|(i, a)| {
                if a == g && !used[i] {
                    used[i] = true;
                    return true;
                }
                false
            }) {
                c[i] = Correctness::Misplaced;
            }
        }
        c
    }
}

pub struct Guess {
    pub word: String,
    pub mask: [Correctness; 5],
}

pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> String;
}

#[cfg(test)]
mod tests {
    mod compute {
        use crate::Correctness;

        macro_rules! mask {
            (C) => {Correctness::Correct};
            (M) => {Correctness::Misplaced};
            (W) => {Correctness::Wrong};
            ($($c:tt)+) => {[
                $(mask!($c)),+
            ]}
        }

        #[test]
        fn all_green() {
            assert_eq!(Correctness::compute("abcde", "abcde"), mask![C C C C C]);
        }

        #[test]
        fn all_gray() {
            assert_eq!(Correctness::compute("abcde", "fghij"), mask![W W W W W]);
        }

        #[test]
        fn all_yellow() {
            assert_eq!(Correctness::compute("abcde", "eabcd"), mask![M M M M M]);
        }

        #[test]
        fn repeat_green() {
            assert_eq!(Correctness::compute("aabbb", "aaccc"), mask![C C W W W]);
        }

        #[test]
        fn repeat_yellow() {
            assert_eq!(Correctness::compute("aabbb", "ccaac"), mask![W W M M W]);
        }

        #[test]
        fn repeat_some_green() {
            assert_eq!(Correctness::compute("aabbb", "caacc"), mask![W C M W W]);
        }

        #[test]
        fn only_one_yellow() {
            assert_eq!(Correctness::compute("azzaz", "aaabb"), mask![C M W W W]);
        }

        #[test]
        fn only_one_green() {
            assert_eq!(Correctness::compute("baccc", "aaddd"), mask![W C W W W]);
        }

        #[test]
        fn only_one_gray() {
            assert_eq!(Correctness::compute("abcde", "aacde"), mask![C W C C C]);
        }
    }
}
