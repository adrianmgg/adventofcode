#![allow(unused)]

use std::cmp::{max, min};

use winnow::Parser as _;

fn main() {
    let input_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sample.txt".into());
    let txt = std::fs::read_to_string(input_file).expect("reading input file failed");
    let (fresh_ranges, active_ids) = parse::full_input
        .parse(txt.as_str())
        .expect("failed to parse input");

    let pt1_answer = active_ids
        .iter()
        .filter(|id| fresh_ranges.iter().any(|range| range.contains(id)))
        .count();
    dbg!(pt1_answer);

    // part 2
    let mut fresh_ranges = fresh_ranges;
    // merge the ranges
    fresh_ranges.sort_by_key(|range| *range.start());
    let fresh_ranges = {
        let mut merged = Vec::new();
        for range in fresh_ranges {
            match merged.last_mut() {
                None => merged.push(range),
                Some(prevrange) => {
                    if prevrange.end() >= range.start() {
                        *prevrange = min(*prevrange.start(), *range.start())
                            ..=max(*prevrange.end(), *range.end());
                    } else {
                        merged.push(range);
                    }
                }
            }
        }
        merged
    };
    let pt2_answer: u64 = fresh_ranges
        .iter()
        .map(|range| range.end() - range.start() + 1)
        .sum();
    dbg!(pt2_answer);
}

mod parse {
    use std::ops::RangeInclusive;

    use winnow::{
        ModalResult, Parser,
        ascii::{dec_uint, line_ending},
        combinator::{alt, eof, repeat, separated_pair, terminated},
    };

    fn id(i: &mut &str) -> ModalResult<u64> {
        dec_uint.parse_next(i)
    }

    fn range(i: &mut &str) -> ModalResult<RangeInclusive<u64>> {
        separated_pair(id, '-', id)
            .map(|(lo, hi)| lo..=hi)
            .parse_next(i)
    }

    pub fn full_input(i: &mut &str) -> ModalResult<(Vec<RangeInclusive<u64>>, Vec<u64>)> {
        separated_pair(
            repeat(0.., terminated(range, line_ending)),
            line_ending,
            repeat(0.., terminated(id, alt((line_ending, eof)))),
        )
        .parse_next(i)
    }
}
