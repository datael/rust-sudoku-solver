#![deny(clippy::pedantic)]

use std::{
    fmt::{Display, Error, Formatter},
    rc::Rc,
};

#[derive(Debug)]
struct Gameboard<const X: usize, const Y: usize> {
    state: [[u8; X]; Y],
}

impl<const X: usize, const Y: usize> Gameboard<X, Y> {
    fn set_cell(&mut self, x: usize, y: usize, value: u8) {
        self.state[x][y] = value;
    }
}

impl<const X: usize, const Y: usize> Display for Gameboard<X, Y> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        for x in 0..X {
            for y in 0..Y {
                write!(
                    f,
                    "{} ",
                    match self.state[x][y] {
                        0 => ".".to_string(),
                        a => format!("{a}"),
                    },
                )?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl<const X: usize, const Y: usize> From<[[u8; X]; Y]> for Gameboard<X, Y> {
    fn from(value: [[u8; X]; Y]) -> Self {
        Self { state: value }
    }
}

trait ToCellMask {
    fn to_cell_mask(self) -> u16;
}

trait ToResult {
    fn to_result(self) -> u8;
}

#[derive(Debug)]
struct Candidates<const X: usize, const Y: usize> {
    cells: [[u16; Y]; X],
}

impl<const X: usize, const Y: usize> Default for Candidates<X, Y> {
    fn default() -> Self {
        Self {
            cells: [[511_u16; Y]; X], // 0b111_111_111
        }
    }
}

impl ToCellMask for u8 {
    fn to_cell_mask(self) -> u16 {
        1 << (self - 1)
    }
}

impl ToResult for u16 {
    #[allow(clippy::cast_possible_truncation)]
    fn to_result(self) -> u8 {
        self.ilog2() as u8 + 1
    }
}

impl<const X: usize, const Y: usize> Candidates<X, Y> {
    fn apply_uniques(&self, gameboard: &mut Gameboard<X, Y>) -> bool {
        let mut changes_made = false;

        for x in 0..X {
            for y in 0..Y {
                if self.remaining_candidates(x, y) == 1 {
                    gameboard.set_cell(x, y, self.cells[x][y].to_result());
                    changes_made = true;
                }
            }
        }

        changes_made
    }

    fn exclude_candidate(&mut self, x: usize, y: usize, candidate: u8) {
        self.cells[x][y] &= !candidate.to_cell_mask();
    }
    fn set_exclusive_candidate(&mut self, x: usize, y: usize, candidate: u8) {
        self.cells[x][y] = candidate.to_cell_mask();
    }
    fn remaining_candidates(&self, x: usize, y: usize) -> u32 {
        self.cells[x][y].count_ones()
    }
    fn mark_as_solved(&mut self, x: usize, y: usize) {
        self.cells[x][y] = 0;
    }
}

trait Rule<const X: usize, const Y: usize> {
    fn visit(&self, gameboard: &Gameboard<X, Y>, candidates: &mut Candidates<X, Y>);
}

struct ExcludeWhenSolved;

impl<const X: usize, const Y: usize> Rule<X, Y> for ExcludeWhenSolved {
    fn visit(&self, gameboard: &Gameboard<X, Y>, candidates: &mut Candidates<X, Y>) {
        for x in 0..X {
            for y in 0..Y {
                if gameboard.state[x][y] == 0 {
                    continue;
                };

                candidates.mark_as_solved(x, y);
            }
        }
    }
}

#[derive(Clone)]
struct Region {
    positions: Rc<Vec<(usize, usize)>>,
}

impl Region {
    fn new(positions: Vec<(usize, usize)>) -> Self {
        Self {
            positions: Rc::new(positions),
        }
    }
}

struct UniqueByRegion(Rc<Region>);

impl<const X: usize, const Y: usize> Rule<X, Y> for UniqueByRegion {
    fn visit(&self, gameboard: &Gameboard<X, Y>, candidates: &mut Candidates<X, Y>) {
        for (x, y) in self.0.positions.iter() {
            if gameboard.state[*x][*y] == 0 {
                continue;
            };

            for (x2, y2) in self.0.positions.iter() {
                if (x2, y2) == (x, y) {
                    continue;
                }

                candidates.exclude_candidate(*x2, *y2, gameboard.state[*x][*y]);
            }
        }
    }
}

struct FillRegionUniquely(Rc<Region>);

impl<const X: usize, const Y: usize> Rule<X, Y> for FillRegionUniquely {
    fn visit(&self, _: &Gameboard<X, Y>, candidates: &mut Candidates<X, Y>) {
        'next_n: for n in 1..=9 {
            let mut solo_position = None;

            for (x, y) in self.0.positions.iter() {
                if candidates.cells[*x][*y] & n.to_cell_mask() > 0 {
                    if solo_position.is_some() {
                        continue 'next_n;
                    }

                    solo_position = Some((x, y));
                }
            }

            if let Some((x, y)) = solo_position {
                for (x2, y2) in self.0.positions.iter() {
                    if (x2, y2) == (x, y) {
                        candidates.set_exclusive_candidate(*x2, *y2, n);
                    } else {
                        candidates.exclude_candidate(*x2, *y2, n);
                    }
                }
            }
        }
    }
}

fn main() {
    let mut gameboard: Gameboard<9, 9> = [
        [0, 0, 0, 0, 8, 0, 0, 0, 0],
        [0, 0, 5, 6, 0, 3, 9, 0, 0],
        [0, 8, 4, 0, 0, 0, 2, 7, 0],
        [0, 3, 0, 1, 0, 0, 0, 5, 0],
        [5, 0, 0, 0, 3, 0, 0, 0, 2],
        [0, 6, 0, 0, 0, 5, 0, 1, 0],
        [0, 1, 9, 0, 0, 0, 5, 6, 0],
        [0, 0, 8, 4, 0, 2, 7, 0, 0],
        [0, 0, 0, 0, 6, 0, 0, 0, 0],
    ]
    .into();

    let rules = build_9x9_rules();

    let mut candidates = Candidates::<9, 9>::default();

    loop {
        for rule in &rules {
            rule.visit(&gameboard, &mut candidates);
        }

        if !candidates.apply_uniques(&mut gameboard) {
            break;
        }
    }

    println!("{gameboard}");
    println!("{candidates:?}");
}

fn build_9x9_rules() -> Vec<Box<dyn Rule<9, 9>>> {
    let mut rules: Vec<Box<dyn Rule<9, 9>>> = vec![];

    let regions: Vec<Rc<Region>> = build_9x9_regions()
        .iter()
        .cloned()
        .map(Region::new)
        .map(Rc::new)
        .collect();

    rules.push(Box::new(ExcludeWhenSolved {}));

    for region in &regions {
        rules.push(Box::new(UniqueByRegion(region.clone())));
    }

    for region in &regions {
        rules.push(Box::new(FillRegionUniquely(region.clone())));
    }

    rules
}

fn build_9x9_regions() -> Vec<Vec<(usize, usize)>> {
    let mut regions: Vec<Vec<(usize, usize)>> = vec![];

    // rows
    for x in 0..9 {
        regions.push((0..9).map(|y| (x, y)).collect());
    }

    // columns
    for y in 0..9 {
        regions.push((0..9).map(|x| (x, y)).collect());
    }

    // 3x3 boxes
    for x_outer in 0..3 {
        for y_outer in 0..3 {
            regions.push(
                (0..3)
                    .flat_map(|x_inner| {
                        (0..3).map(move |y_inner| (x_outer * 3 + x_inner, y_outer * 3 + y_inner))
                    })
                    .collect(),
            );
        }
    }

    regions
}
