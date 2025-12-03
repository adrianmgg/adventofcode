use std::{
    cmp::{max, min},
    ops::RangeInclusive,
    path::PathBuf,
};

use clap::Parser;
use itertools::Itertools as _;
#[cfg(test)]
use rstest::rstest;
use tap::Pipe as _;
use winnow::{ModalResult, Parser as _};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BadIdBlock {
    lo: u64,
    hi: u64,
    step: u64,
}

impl BadIdBlock {
    pub fn new(n_digits: u32, repeats: usize) -> Option<Self> {
        // 10^n
        let ten_n = 10u64.checked_pow(n_digits)?;
        // 10^(n-1)
        let ten_prevn = 10u64.checked_pow(n_digits - 1)?;

        // lowest possible individual (not yet repeated) number value
        let min_n = ten_prevn;
        // highest possible individual number value
        let max_n = ten_n - 1;

        // given a number, applies our repeat count for our digit count to that number
        // e.g. for 3 digits, 2 repeats, repeat(123)=123123
        let repeat = |n: u64| {
            std::iter::repeat_n((), repeats)
                .try_fold(0u64, |acc, _| acc.checked_mul(ten_n)?.checked_add(n))
        };

        let step = repeat(1)?;
        let lo = repeat(min_n)?;
        let hi = repeat(max_n).unwrap_or(u64::MAX);

        Some(BadIdBlock { lo, hi, step })
    }

    pub fn bad_ids_in_range(&self, range: &RangeInclusive<u64>) -> impl Iterator<Item = u64> {
        struct BadIdIter {
            cur: Option<u64>,
            step: u64,
            max: u64,
        }
        impl Iterator for BadIdIter {
            type Item = u64;
            fn next(&mut self) -> Option<Self::Item> {
                let ret = self.cur;
                if let Some(n) = ret {
                    self.cur = n
                        .checked_add(self.step)
                        // iterator should stop if we go past its max
                        .and_then(|n| if n > self.max { None } else { Some(n) });
                }
                ret
            }
        }

        if *range.end() < self.lo || self.hi < *range.start() {
            return BadIdIter {
                cur: None,
                step: 0,
                max: 0,
            };
        }
        let first_bad_id = (*range.start())
            // move start up to our start if input range starts before there
            .pipe(|n| max(n, self.lo))
            // move start precisely so that it lands on one of the bad ids
            .pipe(|n| {
                let rem = (n - self.lo) % self.step;
                match rem {
                    0 => n,
                    _ => n + (self.step - rem),
                }
            });
        BadIdIter {
            cur: match first_bad_id < *range.end() {
                true => Some(first_bad_id),
                false => None,
            },
            step: self.step,
            max: min(self.hi, *range.end()),
        }
    }
}

#[cfg(test)]
#[rstest]
#[case(1, 2, Some(BadIdBlock { lo: 1_1, hi: 9_9, step: 11 }))]
#[case(2, 2, Some(BadIdBlock { lo: 10_10, hi: 99_99, step: 101 }))]
#[case(3, 2, Some(BadIdBlock { lo: 100_100, hi: 999_999, step: 1001 }))]
#[case::overflow_graceful_fail(11, 2, None)]
#[case::overflow_upper_bound_truncate(10, 2, Some(BadIdBlock { lo: 1000000000_1000000000, hi: u64::MAX, step: 10000000001 }))]
#[case(1, 3, Some(BadIdBlock { lo: 1_1_1, hi: 9_9_9, step: 111 }))]
#[case(2, 3, Some(BadIdBlock { lo: 10_10_10, hi: 99_99_99, step: 10101 }))]
fn test_bad_id_block_ctor(
    #[case] n: u32,
    #[case] repeats: usize,
    #[case] expected: Option<BadIdBlock>,
) {
    assert_eq!(expected, BadIdBlock::new(n, repeats));
}

fn parser(input: &mut &str) -> ModalResult<Vec<RangeInclusive<u64>>> {
    use winnow::{
        ascii::{dec_uint, line_ending},
        combinator::{opt, separated, separated_pair, terminated},
    };
    terminated(
        separated(
            1..,
            separated_pair(dec_uint, '-', dec_uint).map(|(lo, hi)| lo..=hi),
            ',',
        ),
        opt(line_ending),
    )
    .parse_next(input)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
enum Ruleset {
    Part1,
    Part2,
}

#[derive(clap::Parser)]
struct Args {
    ruleset: Ruleset,
    input_file: PathBuf,
    #[arg(long)]
    explain: bool,
}

fn build_bad_id_blocks(ruleset: Ruleset) -> Vec<BadIdBlock> {
    let repeat_counts = match ruleset {
        Ruleset::Part1 => 2..=2,
        Ruleset::Part2 => 2..=usize::MAX,
    };

    repeat_counts
        .map_while(|n_repeats| {
            // make all possible blocks at this repeat count
            (1..)
                .map_while(|n_digits| BadIdBlock::new(n_digits, n_repeats))
                .collect::<Vec<_>>()
                // return None instead of empty vec if we got no blocks,
                //  to stop iteration.
                .pipe(|blocks| (!blocks.is_empty()).then_some(blocks))
        })
        .flatten()
        .collect()
}

fn get_bad_ids_in_range(
    range: &RangeInclusive<u64>,
    blocks: &[BadIdBlock],
) -> impl Iterator<Item = u64> {
    blocks
        .iter()
        .flat_map(|block| block.bad_ids_in_range(range))
        // some might be double-counted by multiple blocks
        // e.g. '222222' can be (2)(2)(2)(2)(2)(2) or (22)(22)(22) or (222)(222)
        .unique()
}

fn main() {
    let args = Args::parse();

    let bad_id_blocks = build_bad_id_blocks(args.ruleset);

    let txt = std::fs::read_to_string(args.input_file).expect("reading input file failed");
    let product_ranges = parser.parse(&txt).expect("parsing input failed");

    let answer = product_ranges
        .iter()
        .flat_map(|input_range| get_bad_ids_in_range(input_range, &bad_id_blocks))
        .sum::<u64>();
    println!("{answer}");

    if args.explain {
        for range in product_ranges {
            let bad_ids: Vec<_> = get_bad_ids_in_range(&range, &bad_id_blocks).collect();
            println!(
                "range {lo}-{hi} has {len} invalid IDs: {bad_ids:?}",
                lo = range.start(),
                hi = range.end(),
                len = bad_ids.len(),
            );
        }
    }
}

#[cfg(test)]
#[rstest]
#[case::part1(Ruleset::Part1, 11..=22, vec![11, 22])]
#[case::part1(Ruleset::Part1, 95..=115, vec![99])]
#[case::part1(Ruleset::Part1, 998..=1012, vec![1010])]
#[case::part1(Ruleset::Part1, 1188511880..=1188511890, vec![1188511885])]
#[case::part2(Ruleset::Part2, 95..=115, vec![99, 111])]
#[case::part2(Ruleset::Part2, 998..=1012, vec![999, 1010])]
#[case::part2(Ruleset::Part2, 1188511880..=1188511890, vec![1188511885])]
#[case::part2(Ruleset::Part2, 222220..=222224, vec![222222])]
#[case::part2(Ruleset::Part2, 1698522..=1698528, vec![])]
#[case::part2(Ruleset::Part2, 446443..=446449, vec![446446])]
#[case::part2(Ruleset::Part2, 38593856..=38593862, vec![38593859])]
#[case::part2(Ruleset::Part2, 565653..=565659, vec![565656])]
#[case::part2(Ruleset::Part2, 824824821..=824824827, vec![824824824])]
#[case::part2(Ruleset::Part2, 2121212118..=2121212124, vec![2121212121])]
fn test_bad_ids_in_range(
    #[case] ruleset: Ruleset,
    #[case] range: RangeInclusive<u64>,
    #[case] expected: Vec<u64>,
) {
    use std::collections::HashSet;
    assert_eq!(
        expected.pipe(HashSet::from_iter),
        get_bad_ids_in_range(&range, &build_bad_id_blocks(ruleset)).collect::<HashSet<_>>()
    );
}
