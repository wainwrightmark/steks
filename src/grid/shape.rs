use super::relative_coordinate::Qr;
use itertools::Itertools;
use strum::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shape<const POINTS: usize>([Qr; POINTS]);

impl<const P: usize> Shape<P> {
    pub fn deconstruct_into_rectangles(&self) -> Vec<(Qr, Qr)> {
        let mut results = vec![];

        let mut remaining_points = self.0.to_vec();

        while let Some(p1) = remaining_points.pop() {
            let mut min_x = p1.x();
            let mut max_x = p1.x();
            let mut min_y = p1.y();

            while let Some((index, &p2)) = remaining_points
                .iter()
                .find_position(|p2| p2.y() == min_y && (p2.x() == max_x + 1 || p2.x() == min_x - 1))
            {
                remaining_points.swap_remove(index);
                min_x = min_x.min(p2.x());
                max_x = max_x.max(p2.x());
            }
            let range = min_x..=max_x;

            let mut max_y = p1.y();

            'outer: loop {
                for is_max in [false, true] {
                    let y = if is_max { max_y + 1 } else { min_y - 1 };
                    let condition = |p2: &&Qr| p2.y() == y && range.contains(&p2.x());
                    if remaining_points.iter().filter(condition).count() == range.len() {
                        while let Some((position, _)) =
                            remaining_points.iter().find_position(condition)
                        {
                            remaining_points.swap_remove(position);
                        }
                        if is_max {
                            max_y += 1;
                        } else {
                            min_y -= 1;
                        }

                        continue 'outer;
                    }
                }
                break 'outer;
            }

            results.push((Qr::new(min_x, min_y), Qr::new(max_x, max_y)));
        }

        results
    }
}

impl<const P: usize> IntoIterator for Shape<P> {
    type Item = Qr;

    type IntoIter = std::array::IntoIter<Qr, P>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Shape<1> {
    pub const MONOMINO: Self = Self([Qr::ZERO]);
}

impl Shape<2> {
    pub const DOMINO: Self = Self([Qr::ZERO, Qr::NORTH]);
}

impl Shape<3> {
    pub const I_TROMINO: Self = Self([Qr::EAST, Qr::ZERO, Qr::WEST]);
    pub const V_TROMINO: Self = Self([Qr::EAST, Qr::ZERO, Qr::NORTH]);
}

impl Shape<4> {
    pub const I_TETROMINO: Self = Self([Qr::EAST, Qr::ZERO, Qr::WEST, Qr::WEST_TWO]);
    pub const O_TETROMINO: Self = Self([Qr::ZERO, Qr::EAST, Qr::NORTHEAST, Qr::NORTH]);
    pub const T_TETROMINO: Self = Self([Qr::EAST, Qr::ZERO, Qr::WEST, Qr::SOUTH]);
    pub const J_TETROMINO: Self = Self([Qr::WEST, Qr::ZERO, Qr::NORTH, Qr::NORTH_TWO]);
    pub const L_TETROMINO: Self = Self([Qr::EAST, Qr::ZERO, Qr::NORTH, Qr::NORTH_TWO]);
    pub const S_TETROMINO: Self = Self([Qr::WEST, Qr::ZERO, Qr::NORTH, Qr::NORTHEAST]);
    pub const Z_TETROMINO: Self = Self([Qr::EAST, Qr::ZERO, Qr::NORTH, Qr::NORTHWEST]);

    pub const TETROMINOS: [Self; 7] = [
        Self::I_TETROMINO,
        Self::O_TETROMINO,
        Self::T_TETROMINO,
        Self::J_TETROMINO,
        Self::L_TETROMINO,
        Self::S_TETROMINO,
        Self::Z_TETROMINO,
    ];

    pub const TETROMINO_NAMES: [&'static str; 7] = ["I", "O", "T", "J", "L", "S", "Z"];

    pub const FREE_TETROMINOS: [Self; 5] = [
        Self::I_TETROMINO,
        Self::O_TETROMINO,
        Self::T_TETROMINO,
        Self::L_TETROMINO,
        Self::S_TETROMINO,
    ];

    pub const FREE_TETROMINO_NAMES: [&'static str; 5] = ["I", "O", "T", "L", "S"];
}

impl Shape<5> {
    pub const F_PENTOMINO: Self = Self([Qr::ZERO, Qr::NORTH, Qr::NORTHEAST, Qr::WEST, Qr::SOUTH]);
    pub const I_PENTOMINO: Self =
        Self([Qr::ZERO, Qr::NORTH, Qr::NORTH_TWO, Qr::SOUTH, Qr::SOUTH_TWO]);
    pub const L_PENTOMINO: Self =
        Self([Qr::ZERO, Qr::NORTH, Qr::NORTH_TWO, Qr::SOUTH, Qr::SOUTHEAST]);
    pub const N_PENTOMINO: Self =
        Self([Qr::ZERO, Qr::NORTH, Qr::NORTH_TWO, Qr::WEST, Qr::SOUTHWEST]);
    pub const P_PENTOMINO: Self = Self([Qr::NORTH, Qr::ZERO, Qr::NORTHEAST, Qr::EAST, Qr::SOUTH]);
    pub const T_PENTOMINO: Self =
        Self([Qr::ZERO, Qr::NORTH, Qr::NORTHEAST, Qr::NORTHWEST, Qr::SOUTH]);
    pub const U_PENTOMINO: Self =
        Self([Qr::ZERO, Qr::NORTHEAST, Qr::EAST, Qr::NORTHWEST, Qr::WEST]);
    pub const V_PENTOMINO: Self =
        Self([Qr::ZERO, Qr::NORTH, Qr::NORTH_TWO, Qr::WEST, Qr::WEST_TWO]);
    pub const W_PENTOMINO: Self =
        Self([Qr::ZERO, Qr::EAST, Qr::NORTHEAST, Qr::SOUTH, Qr::SOUTHWEST]);
    pub const X_PENTOMINO: Self = Self([Qr::ZERO, Qr::NORTH, Qr::EAST, Qr::SOUTH, Qr::WEST]);
    pub const Y_PENTOMINO: Self = Self([Qr::ZERO, Qr::NORTH, Qr::EAST, Qr::WEST, Qr::WEST_TWO]);
    pub const Z_PENTOMINO: Self =
        Self([Qr::ZERO, Qr::NORTH, Qr::NORTHWEST, Qr::SOUTH, Qr::SOUTHEAST]);

    pub const FREE_PENTOMINOS: [Self; 12] = [
        Self::F_PENTOMINO,
        Self::I_PENTOMINO,
        Self::L_PENTOMINO,
        Self::N_PENTOMINO,
        Self::P_PENTOMINO,
        Self::T_PENTOMINO,
        Self::U_PENTOMINO,
        Self::V_PENTOMINO,
        Self::W_PENTOMINO,
        Self::X_PENTOMINO,
        Self::Y_PENTOMINO,
        Self::Z_PENTOMINO,
    ];

    pub const FREE_PENTOMINO_NAMES: [&'static str; 12] =
        ["F", "I", "L", "N", "P", "T", "U", "V", "W", "X", "Y", "Z"];
}

pub trait PolyominoShape {
    type OutlineIter: Iterator<Item = Qr>;
    fn draw_outline(&self) -> Self::OutlineIter;

    fn get_centre(&self) -> (f32, f32);

    fn first_point(&self) -> Qr;
}

impl<const P: usize> PolyominoShape for Shape<P> {
    type OutlineIter = OutlineIter<P>;

    fn draw_outline(&self) -> Self::OutlineIter {
        let mut arr = self.0;
        arr.sort();
        OutlineIter {
            arr,
            next: Some((arr[0], Corner::NorthWest)),
        }
    }

    fn get_centre(&self) -> (f32, f32) {
        let mut x = 0;
        let mut y = 0;

        for point in self.0 {
            x += point.x();
            y += point.y();
        }

        (
            0.5 + ((x as f32) / (P as f32)),
            0.5 + ((y as f32) / (P as f32)),
        )
    }

    fn first_point(&self) -> Qr {
        self.0[0]
    }
}

pub struct OutlineIter<const POINTS: usize> {
    arr: [Qr; POINTS],
    next: Option<(Qr, Corner)>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
enum Corner {
    NorthWest,
    NorthEast,
    SouthEast,
    SouthWest,
}

impl Corner {
    pub fn clockwise_direction(&self) -> Qr {
        match self {
            Corner::NorthWest => Qr::NORTH,
            Corner::NorthEast => Qr::EAST,
            Corner::SouthEast => Qr::SOUTH,
            Corner::SouthWest => Qr::WEST,
        }
    }

    pub fn clockwise(&self) -> Self {
        use Corner::*;
        match self {
            Corner::NorthWest => NorthEast,
            Corner::NorthEast => SouthEast,
            Corner::SouthEast => SouthWest,
            Corner::SouthWest => NorthWest,
        }
    }

    pub fn anticlockwise(&self) -> Self {
        use Corner::*;
        match self {
            Corner::NorthWest => SouthWest,
            Corner::NorthEast => NorthWest,
            Corner::SouthEast => NorthEast,
            Corner::SouthWest => SouthEast,
        }
    }

    pub fn direction_of_northwest_corner(&self) -> Qr {
        match self {
            Corner::NorthWest => Qr::ZERO,
            Corner::NorthEast => Qr::EAST,
            Corner::SouthEast => Qr::SOUTHEAST,
            Corner::SouthWest => Qr::SOUTH,
        }
    }
}

impl<const POINTS: usize> Iterator for OutlineIter<POINTS> {
    type Item = Qr;

    fn next(&mut self) -> Option<Self::Item> {
        let mut direction_so_far: Option<Qr> = None;
        let (coordinate_to_return, corner_to_return) = self.next?;

        let mut next_coordinate = coordinate_to_return;
        let mut next_corner = corner_to_return;

        'line: loop {
            'equivalency: loop {
                let equivalent = next_coordinate + next_corner.clockwise_direction();
                if self.arr.contains(&equivalent) {
                    //perform an equivalency
                    next_coordinate = equivalent;
                    next_corner = next_corner.anticlockwise();
                    if next_coordinate == coordinate_to_return {
                        panic!("Infinite loop found in shape.")
                    }
                    if next_corner == Corner::NorthWest && next_coordinate == self.arr[0] {
                        break 'line;
                    }
                } else {
                    break 'equivalency;
                }
            }

            match direction_so_far {
                None => {
                    direction_so_far = Some(next_corner.clockwise_direction());
                    next_corner = next_corner.clockwise();
                }
                Some(d) => {
                    if d == next_corner.clockwise_direction() {
                        next_corner = next_corner.clockwise();
                    } else {
                        break 'line;
                    }
                }
            }
            if next_corner == Corner::NorthWest && next_coordinate == self.arr[0] {
                break 'line;
            }
        }

        if next_corner == Corner::NorthWest && next_coordinate == self.arr[0] {
            self.next = None;
        } else {
            self.next = Some((next_coordinate, next_corner));
        }

        //println!("{} {}", coordinate_to_return, corner_to_return);
        let r = coordinate_to_return + corner_to_return.direction_of_northwest_corner();

        Some(r)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_basic_outlines() {
        test_outline(&Shape::MONOMINO, "Square");
        test_outline(&Shape::DOMINO, "Domino");
    }

    #[test]
    fn test_tetromino_outlines() {
        for (shape, name) in Shape::TETROMINOS.iter().zip(Shape::TETROMINO_NAMES) {
            test_outline(shape, (name.to_string() + " tetromino").as_str())
        }
    }

    #[test]
    fn test_pentomino_outlines() {
        for (shape, name) in Shape::FREE_PENTOMINOS
            .iter()
            .zip(Shape::FREE_PENTOMINO_NAMES)
        {
            test_outline(shape, (name.to_string() + " pentomino").as_str())
        }
    }

    fn test_outline<P: PolyominoShape>(shape: &'static P, name: &str) {
        let outline: Vec<_> = shape.draw_outline().take(100).collect();
        assert!(outline.len() < 100);
        let max_x = outline.iter().map(|q| q.x()).max().unwrap() as f32;
        let max_y = outline.iter().map(|q| q.y()).max().unwrap() as f32;

        let min_x = outline.iter().map(|q| q.x()).min().unwrap() as f32;
        let min_y = outline.iter().map(|q| q.y()).min().unwrap() as f32;

        let (centre_x, centre_y) = shape.get_centre();

        assert!(centre_x < max_x);
        assert!(centre_y < max_y);

        assert!(centre_x > min_x);
        assert!(centre_y > min_y);

        insta::assert_debug_snapshot!(name, outline);

        // for o in outline{
        //     println!("{:?}", o);
        // }
        // println!("{},{}", center.0, center.1);
    }
}
