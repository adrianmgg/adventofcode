#![allow(unused)]

use std::fmt::{Display, Write as _};

use tap::Pipe as _;
use winnow::Parser as _;

fn main() {
    let input_files = std::env::args().skip(1).collect::<Vec<_>>().pipe(|files| {
        if files.is_empty() {
            vec!["sample.txt".into(), "input.txt".into()]
        } else {
            files
        }
    });

    for input_file in input_files {
        println!("{input_file}");
        let txt = std::fs::read_to_string(input_file).expect("reading input file failed");
        dbg!(part1(&txt));
    }
}

fn part1(txt: &str) -> u64 {
    let mut manifold = parse::tachyon_manifold.parse(txt).unwrap();

    let mut n_splits = 0u64;

    for row_idx in 0..manifold.rows.len() {
        if row_idx == 0 {
            continue;
        }
        let [prevrow, mut row] = manifold
            .rows
            .get_disjoint_mut([row_idx - 1, row_idx])
            .unwrap();
        for (cur, prev) in row.iter_mut().zip(prevrow.iter()) {
            if matches!(prev, Kind::Beam | Kind::EntryPoint) {
                match cur {
                    cur @ Kind::Air => *cur = Kind::Beam,
                    Kind::Splitter { hit } => {
                        *hit = true;
                        n_splits += 1;
                    },
                    _ => {}
                }
            }
        }
        for idx in 0..row.len() {
            if row[idx] != Kind::Air {
                continue;
            }
            let left = (idx > 0).then(|| prevrow[idx - 1]);
            let right = prevrow.get(idx + 1);
            if matches!(left, Some(Kind::Splitter { hit: true }))
                || matches!(right, Some(Kind::Splitter { hit: true }))
            {
                row[idx] = Kind::Beam;
            }
        }
    }
    // println!("{manifold}");
    n_splits
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Air,
    Splitter { hit: bool },
    EntryPoint,
    Beam,
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Air => f.write_char('.'),
            Kind::Splitter { .. } => f.write_char('^'),
            Kind::EntryPoint => f.write_char('S'),
            Kind::Beam => f.write_char('|'),
        }
    }
}

struct TachyonManifold {
    rows: Vec<Vec<Kind>>,
}

impl Display for TachyonManifold {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, row) in self.rows.iter().enumerate() {
            if idx > 0 {
                f.write_char('\n');
            }
            for cell in row {
                write!(f, "{cell}")?;
            }
        }
        Ok(())
    }
}

mod parse {
    use super::{Kind, TachyonManifold};

    use winnow::{ascii::*, combinator::*, prelude::*, token::*};

    fn manifold_cell(input: &mut &str) -> ModalResult<Kind> {
        alt((
            '.'.value(Kind::Air),
            '^'.value(Kind::Splitter { hit: false }),
            'S'.value(Kind::EntryPoint),
            '|'.value(Kind::Beam),
        ))
        .parse_next(input)
    }

    pub fn tachyon_manifold(input: &mut &str) -> ModalResult<TachyonManifold> {
        repeat(
            1..,
            terminated(repeat(1.., manifold_cell).map(|v: Vec<_>| v), line_ending),
        )
        .map(|rows| TachyonManifold { rows })
        .parse_next(input)
    }
}
