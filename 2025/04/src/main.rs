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

    let mut g = DeptFloor::parse(txt.as_str()).expect("parsing input failed");
    dbg!(g.n_accessible_rolls());
    dbg!(solve_pt2(&mut g));
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

fn surrounding_indices(
    (x, y): (usize, usize),
    radius: usize,
) -> impl Iterator<Item = (usize, usize)> {
    (x.saturating_sub(radius)..=(x + radius))
        .cartesian_product(y.saturating_sub(radius)..=(y + radius))
        .filter(move |&(ax, ay)| ax != x || ay != y)
}

struct DeptFloor {
    ground: grid::Grid<CellKind>,
    accessible: grid::Grid<bool>,
}

fn check_accessible(ground: &Grid<CellKind>, (x, y): (usize, usize)) -> bool {
    surrounding_indices((x, y), 1)
        .filter_map(|(xx, yy)| ground.get(xx, yy))
        .filter(|kind| matches!(kind, CellKind::Paper))
        .count()
        .pipe(|n| n < 4)
}

fn solve_pt2(dept: &mut DeptFloor) -> u64 {
    let mut n = 0;
    loop {
        let to_remove = dept.iter_accessible_papers().next();
        match to_remove {
            Some(to_remove) => {
                dept.remove_paper(to_remove);
                n += 1;
            }
            None => break,
        }
    }
    n
}

impl DeptFloor {
    fn parse(input: &str) -> Result<Self, ()> {
        let ground = parser::grid.parse(input).map_err(|_| ())?;
        let mut accessible =
            Grid::new_with_order(ground.rows(), ground.cols(), ground.order());
        accessible.indexed_iter_mut().for_each(|(pos, val)| {
            *val = check_accessible(&ground, pos);
        });
        Ok(Self { ground, accessible })
    }

    fn n_adjacent_papers_at(&self, x: usize, y: usize) -> usize {
        (x.saturating_sub(1)..=(x + 1))
            .cartesian_product(y.saturating_sub(1)..=(y + 1))
            .filter(|&(ax, ay)| ax != x || ay != y)
            .filter_map(|(ax, ay)| self.ground.get(ax, ay))
            .filter(|kind| matches!(kind, CellKind::Paper))
            .count()
    }

    fn is_accessible(&self, x: usize, y: usize) -> bool {
        self.n_adjacent_papers_at(x, y) < 4
    }

    fn n_accessible_rolls(&self) -> usize {
        self.ground
            .indexed_iter()
            .filter(|&((x, y), kind)| matches!(kind, CellKind::Paper) && self.is_accessible(x, y))
            .count()
    }

    fn remove_paper(&mut self, (x, y): (usize, usize)) {
        debug_assert!(matches!(self.ground.get(x, y), Some(CellKind::Paper)));
        self.ground[(x, y)] = CellKind::Empty;
        for (x, y) in surrounding_indices((x, y), 1) {
            if let Some(v) = self.accessible.get_mut(x, y) {
                *v = check_accessible(&self.ground, (x, y));
            }
        }
    }

    fn iter_accessible_papers(&self) -> impl Iterator<Item = (usize, usize)> {
        self.ground
            .indexed_iter()
            .filter(|(_, kind)| matches!(kind, CellKind::Paper))
            .filter(|(pos, _)| self.accessible.get(pos.0, pos.1).copied().unwrap())
            .map(|(pos, _)| pos)
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
