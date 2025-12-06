use std::fmt::Display;

use grid::Grid;
use itertools::Itertools;
use tap::Pipe as _;
use winnow::Parser as _;

fn main() {
    let input_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "input.txt".into());
    let txt = std::fs::read_to_string(input_file).expect("reading input file failed");

    let g = FactoryFloor::parse(txt.as_str()).expect("parsing input failed");
    dbg!(g.n_accessible_rolls());
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum CellKind {
    Paper,
    Empty,
}

impl Display for CellKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write as _;
        match self {
            CellKind::Paper => f.write_char('@'),
            CellKind::Empty => f.write_char('.'),
        }
    }
}

struct FactoryFloor {
    ground: grid::Grid<CellKind>,
    accessible: grid::Grid<bool>,
}

impl FactoryFloor {
    fn parse(input: &str) -> Result<Self, ()> {
        let ground = parser::grid.parse(input).map_err(|_| ())?;
        let mut accessible = Grid::new_with_order(ground.rows(), ground.cols(), ground.order());
        accessible.indexed_iter_mut().for_each(|((x, y), out)| {
            let &kind = ground.get(x, y).unwrap();
            if kind == CellKind::Empty {
                *out = true;
                return;
            }
            let adjacent_papers = (x.saturating_sub(1)..=(x + 1))
                .cartesian_product(y.saturating_sub(1)..=(y + 1))
                .filter(|&(ax, ay)| ax != x || ay != y)
                .filter_map(|(ax, ay)| ground.get(ax, ay))
                .filter(|kind| matches!(kind, CellKind::Paper))
                .count();
            *out = adjacent_papers < 4;
        });
        Self { ground, accessible }.pipe(Ok)
    }

    fn n_accessible_rolls(&self) -> usize {
        self.ground
            .iter()
            .zip(self.accessible.iter())
            .filter(|(kind, accessible)| matches!(kind, CellKind::Paper) && **accessible)
            .count()
    }
}

mod parser {
    use super::CellKind;
    use grid::Grid;
    use winnow::{
        ModalResult, Parser,
        ascii::line_ending,
        combinator::{alt, eof, repeat, terminated},
    };

    fn cell(input: &mut &str) -> ModalResult<CellKind> {
        alt(('@'.value(CellKind::Paper), '.'.value(CellKind::Empty))).parse_next(input)
    }

    pub fn grid(input: &mut &str) -> ModalResult<Grid<CellKind>> {
        // parse first row
        let first_row: Vec<_> =
            terminated(repeat(1.., cell), alt((line_ending, eof))).parse_next(input)?;
        let width = first_row.len();

        // extend existing vec to hold entire grid
        let mut data = first_row;
        data.reserve_exact(width * (width - 1));

        // parse the remaining rows
        for _ in 2..=width {
            for _ in 0..width {
                data.push(cell.parse_next(input)?);
            }
            alt((line_ending, eof)).void().parse_next(input)?;
        }
        let grid = Grid::from_vec_with_order(data, width, grid::Order::RowMajor);
        Ok(grid)
    }
}
