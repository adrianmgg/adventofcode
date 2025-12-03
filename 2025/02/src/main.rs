#![allow(unused)]

use std::{
    cmp::{max, min},
    ops::RangeInclusive,
};

use itertools::Itertools as _;
use tap::Pipe as _;
use winnow::{
    ModalResult, Parser as _,
    ascii::{dec_uint, line_ending},
    combinator::{opt, separated, separated_pair, terminated},
};

#[derive(Debug, Clone, Copy)]
struct BadIdBlock {
    lo: u64,
    hi: u64,
    step: u64,
}

macro_rules! early_return_nones {
    ($v:expr) => {
        match $v {
            None => return None,
            Some(v) => v,
        }
    };
}

impl BadIdBlock {
    pub const fn make_nth(n: u32) -> Option<Self> {
        use early_return_nones as ern;

        // 10^n
        let ten_n = ern!(10u64.checked_pow(n));
        // 10^(n-1)
        let ten_prevn = ern!(10u64.checked_pow(n - 1));

        let min_n = ten_prevn;
        let max_n = ten_n - 1;
        let step = ten_n + 1;

        let lo = ern!(ern!(min_n.checked_mul(ten_n)).checked_add(min_n));

        // hi is the exception to us using checked ops, as we want to be able to still represent
        // the first half of a range that is cut off by the max value boundary
        let hi = max_n.saturating_mul(ten_n).saturating_add(max_n);

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
            cur: Some(first_bad_id),
            step: self.step,
            max: min(self.hi, *range.end()),
        }
    }
}

// pre-compute them. not actually super necissary since it's not expensive, but hey why not
const BLOCKS: [BadIdBlock; 9] = [
    BadIdBlock::make_nth(1).unwrap(),
    BadIdBlock::make_nth(2).unwrap(),
    BadIdBlock::make_nth(3).unwrap(),
    BadIdBlock::make_nth(4).unwrap(),
    BadIdBlock::make_nth(5).unwrap(),
    BadIdBlock::make_nth(6).unwrap(),
    BadIdBlock::make_nth(7).unwrap(),
    BadIdBlock::make_nth(8).unwrap(),
    BadIdBlock::make_nth(9).unwrap(),
];

fn parser(input: &mut &str) -> ModalResult<Vec<RangeInclusive<u64>>> {
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

fn main() {
    for inputfile in std::env::args().skip(1) {
        let txt = std::fs::read_to_string(inputfile).expect("reading input file failed");
        let product_ranges = parser.parse(&txt).expect("parsing input failed");
        let answer = product_ranges
            .iter()
            .cartesian_product(BLOCKS.iter())
            .flat_map(|(input_range, block)| block.bad_ids_in_range(input_range))
            .sum::<u64>();
        println!("{answer}");
        /*
        for range in product_ranges {
            let bad_ids: Vec<_> = BLOCKS
                .iter()
                .flat_map(|block| block.bad_ids_in_range(&range))
                .collect();
            println!(
                "range {lo}-{hi} has {len} invalid IDs: {bad_ids:?}",
                lo = range.start(),
                hi = range.end(),
                len = bad_ids.len(),
            );
        }
        */
    }
}
