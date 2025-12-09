#![allow(unused)]

use winnow::Parser as _;

fn main() {
    let input_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sample.txt".into());
    let txt = std::fs::read_to_string(input_file).expect("reading input file failed");
    let (fresh_ranges, ids) = parse::full_input
        .parse(txt.as_str())
        .expect("failed to parse input");

    let pt1_answer = ids
        .iter()
        .filter(|id| fresh_ranges.iter().any(|range| range.contains(id)))
        .count();
    dbg!(pt1_answer);
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
