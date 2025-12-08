mod spiral;

use std::fmt::Display;

use itertools::Itertools;
use tap::Pipe as _;
use winnow::Parser as _;

use crate::grid::SquareGrid;

fn main() {
    let input_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sample.txt".into());
    let txt = std::fs::read_to_string(input_file).expect("reading input file failed");

    let g = DeptFloor::parse(txt.as_str()).expect("parsing input failed");

    println!("{g}");
    dbg!(g.n_accessible_rolls());
    dbg!(solve_pt2(&g));
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

#[derive(Clone)]
struct DeptFloor {
    ground: grid::SquareGrid<CellKind>,
    accessible: grid::SquareGrid<bool>,
    n_papers: usize,
}

impl Display for DeptFloor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write as _;
        for x in 0..self.ground.size() {
            if x > 0 {
                f.write_char('\n')?;
            }
            for y in 0..self.ground.size() {
                f.write_char(
                    match (
                        self.ground.get((x, y)).unwrap(),
                        self.accessible.get((x, y)).unwrap(),
                    ) {
                        // (CellKind::Paper, false) => '@',
                        // (CellKind::Paper, true) => 'x',
                        (CellKind::Paper, _) => '@',
                        (CellKind::Empty, _) => '.',
                    },
                )?;
            }
        }
        Ok(())
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

fn check_accessible(ground: &SquareGrid<CellKind>, (x, y): (usize, usize)) -> bool {
    surrounding_indices((x, y), 1)
        .filter_map(|pos| ground.get(pos))
        .filter(|kind| matches!(kind, CellKind::Paper))
        .count()
        .pipe(|n| n < 4)
}

impl DeptFloor {
    fn parse(input: &str) -> Result<Self, ()> {
        let ground = parser::grid.parse(input).map_err(|_| ())?;
        let accessible =
            SquareGrid::new_from_compute(ground.size(), |(x, y)| check_accessible(&ground, (x, y)));
        let n_papers = ground
            .indexed_iter()
            .filter(|(_, a)| matches!(a, CellKind::Paper))
            .count();
        Self {
            ground,
            accessible,
            n_papers,
        }
        .pipe(Ok)
    }

    fn n_accessible_rolls(&self) -> usize {
        self.iter_papers()
            .filter(|&pos| *self.accessible.get(pos).unwrap())
            .count()
    }

    fn with_paper_removed(&self, pos: (usize, usize)) -> Self {
        debug_assert!(matches!(self.ground.get(pos), Some(CellKind::Paper)));
        let ground = self.ground.with_set(pos, CellKind::Empty);
        let accessible =
            surrounding_indices(pos, 1).fold(self.accessible.clone(), |acc, accpos| {
                if acc.contains_index(accpos) {
                    acc.with_set(accpos, check_accessible(&ground, accpos))
                } else {
                    acc
                }
            });
        Self {
            ground,
            accessible,
            n_papers: self.n_papers - 1,
        }
    }

    fn iter_papers(&self) -> impl Iterator<Item = (usize, usize)> {
        self.ground
            .indexed_iter()
            .filter(|&(_, &kind)| matches!(kind, CellKind::Paper))
            .map(|((x, y), _)| (x, y))
    }

    fn is_accessible(&self, pos: (usize, usize)) -> bool {
        self.accessible.get(pos).copied().unwrap_or(false)
    }
}

fn solve_pt2(dept: &DeptFloor) -> u64 {
    let cur_accessible = dept.n_accessible_rolls();

    if dept.n_papers.is_multiple_of(10) {
        dbg!(dept.n_papers);
    }

    let mut foo = None;
    for roll in dept.iter_papers() {
        if !dept.is_accessible(roll) {
            continue;
        }
        let subdept = dept.with_paper_removed(roll);
        // if we're net neutral on leaving things accessible, can just pick it right away
        if subdept.n_accessible_rolls() >= cur_accessible {
            // println!("->\n{subdept}");
            return 1 + solve_pt2(&subdept);
        } else {
            foo = Some(subdept);
        }
    }
    if let Some(subdept) = foo {
        return 1 + solve_pt2(&subdept);
    }
    0
}

mod grid {
    use im_rc::Vector;
    use itertools::Itertools as _;

    use crate::spiral::SpiralIndexIterator;

    #[derive(Clone)]
    pub struct SquareGrid<T> {
        size: usize,
        data: Vector<T>,
    }

    impl<T: Clone> SquareGrid<T> {
        pub fn new_from_slice(size: usize, data: &[T]) -> Self {
            assert_eq!(data.len(), size * size);
            Self {
                size,
                data: data.into(),
            }
        }

        pub fn new_from_compute<F: FnMut((usize, usize)) -> T>(
            size: usize,
            mut compute: F,
        ) -> Self {
            let mut data = Vector::new();
            for x in 0..size {
                for y in 0..size {
                    data.push_back(compute((x, y)));
                }
            }
            Self { size, data }
        }

        pub fn size(&self) -> usize {
            self.size
        }

        pub fn indices(&self) -> impl Iterator<Item = (usize, usize)> {
            // (0..self.size).cartesian_product(0..self.size)
            SpiralIndexIterator::new(self.size as u32)
        }

        pub fn get(&self, (x, y): (usize, usize)) -> Option<&T> {
            if x >= self.size || y >= self.size {
                None
            } else {
                self.data.get(Self::to_flat_index(self.size, (x, y)))
            }
        }

        fn to_flat_index(size: usize, (x, y): (usize, usize)) -> usize {
            y + (x * size)
        }

        pub fn indexed_iter(&self) -> impl Iterator<Item = ((usize, usize), &T)> {
            self.indices().map(|pos| (pos, self.get(pos).unwrap()))
        }

        pub fn contains_index(&self, (x, y): (usize, usize)) -> bool {
            x < self.size && y < self.size
        }

        pub fn with_set(&self, (x, y): (usize, usize), v: T) -> SquareGrid<T> {
            debug_assert!(self.contains_index((x, y)));
            Self {
                size: self.size,
                data: self.data.update(Self::to_flat_index(self.size, (x, y)), v),
            }
        }
    }
}

mod parser {
    use crate::grid::SquareGrid;

    use super::CellKind;
    use winnow::{
        ModalResult, Parser,
        ascii::line_ending,
        combinator::{alt, eof, repeat, terminated},
    };

    fn cell(input: &mut &str) -> ModalResult<CellKind> {
        alt(('@'.value(CellKind::Paper), '.'.value(CellKind::Empty))).parse_next(input)
    }

    pub fn grid(input: &mut &str) -> ModalResult<SquareGrid<CellKind>> {
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
        let grid = SquareGrid::new_from_slice(width, &data);
        Ok(grid)
    }
}
