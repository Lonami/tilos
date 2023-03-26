use owo_colors::OwoColorize as _;
use std::env;
use std::fmt::{self, Write as _};
use std::mem;
use std::process;

const ERR_NO_SOLUTION: i32 = 1;
const ERR_BAD_ARGS: i32 = 2;

#[cfg(not(feature = "owo-colors"))]
mod owo_colors {
    pub trait OwoColorize {
        fn green(&self) -> String;
    }

    impl OwoColorize for usize {
        fn green(&self) -> String {
            format!("{}", self.clone())
        }
    }

    impl OwoColorize for &str {
        fn green(&self) -> String {
            self.replace("█", "▒")
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BitMatrix {
    matrix: u64,
    width: u8,
    height: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Shape(BitMatrix);

#[derive(Clone, Debug)]
struct Step {
    x: usize,
    y: usize,
    piece: Shape,
    r: usize,
}

#[derive(Debug)]
struct Settings {
    width: usize,
    height: usize,
    pieces: Vec<Shape>,
}

impl BitMatrix {
    fn new() -> Self {
        Self {
            matrix: 0,
            width: 0,
            height: 0,
        }
    }

    fn with_size(width: usize, height: usize) -> Self {
        if width < 1 || height < 1 {
            panic!("size must be at least 1x1");
        }
        let mut matrix = Self::new();
        matrix.expand_to(width - 1, height - 1, false);
        matrix
    }

    fn at(&self, x: usize, y: usize) -> bool {
        if x >= self.width as usize || y >= self.height as usize {
            panic!("index ({}, {}) out of bounds", x, y);
        }
        let i = (y as u8) * self.width + (x as u8);
        (self.matrix & (1 << i)) != 0
    }

    fn set(&mut self, x: usize, y: usize, bit: bool) {
        if x >= self.width as usize || y >= self.height as usize {
            panic!("index ({}, {}) out of bounds", x, y);
        }
        let i = (y as u8) * self.width + (x as u8);
        if bit {
            self.matrix |= 1 << i;
        } else {
            self.matrix &= !(1 << i);
        }
    }

    fn expand_to(&mut self, x: usize, y: usize, bit: bool) {
        if x * y > mem::size_of_val(&self.matrix) * 8 {
            panic!("backing storage cannot fit {}x{} elements", x, y);
        }
        self.width = self.width.max(x as u8 + 1);
        self.height = self.height.max(y as u8 + 1);
        self.set(x, y, bit);
    }
}

impl Shape {
    fn new(matrix: BitMatrix) -> Self {
        Self(matrix)
    }

    fn from_str(string: &str) -> Self {
        let mut matrix = BitMatrix::new();
        for (i, line) in string.trim().lines().enumerate() {
            for (j, c) in line.chars().filter(|c| !c.is_whitespace()).enumerate() {
                matrix.expand_to(j, i, c != '_');
            }
        }
        Self::new(matrix)
    }

    fn with_size(width: usize, height: usize) -> Self {
        Self::new(BitMatrix::with_size(width, height))
    }

    fn width(&self) -> usize {
        self.0.width as _
    }

    fn height(&self) -> usize {
        self.0.height as _
    }

    fn rot(&self) -> Self {
        let mut new = BitMatrix::with_size(self.height(), self.width());
        for i in 0..self.height() {
            for j in 0..self.width() {
                new.set(self.height() - i - 1, j, self.0.at(j, i));
            }
        }
        Self::new(new)
    }

    fn flip(&self) -> Self {
        let mut new = BitMatrix::with_size(self.width(), self.height());
        for i in 0..self.height() {
            for j in 0..self.width() {
                new.set(self.width() - j - 1, i, self.0.at(j, i));
            }
        }
        Self::new(new)
    }

    fn can_put(&self, x: usize, y: usize, other: &Self) -> bool {
        if x + other.width() > self.width() || y + other.height() > self.height() {
            return false;
        }

        for i in 0..other.height() {
            for j in 0..other.width() {
                if other.0.at(j, i) && self.0.at(x + j, y + i) {
                    return false;
                }
            }
        }

        true
    }

    fn put(&self, x: usize, y: usize, other: &Self) -> Self {
        let mut new = self.0.clone();
        for i in 0..other.height() {
            for j in 0..other.width() {
                if other.0.at(j, i) {
                    new.set(x + j, y + i, true);
                }
            }
        }
        Self::new(new)
    }

    fn len(&self) -> usize {
        (0..self.height())
            .map(|i| {
                (0..self.width())
                    .map(|j| self.0.at(j, i) as usize)
                    .sum::<usize>()
            })
            .sum()
    }

    fn is_open(&self, x: isize, y: isize) -> bool {
        0 <= x
            && (x as usize) < self.width()
            && 0 <= y
            && (y as usize) < self.height()
            && !self.0.at(x as usize, y as usize)
    }

    fn is_dead(&self, x: usize, y: usize) -> bool {
        let mut explored = [Some((x as isize, y as isize)), None, None];

        for _ in 0..4 {
            for ei in 0..explored.len() {
                if let Some((x, y)) = explored[ei] {
                    for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
                        if self.is_open(x + dx, y + dy)
                            && !explored.contains(&Some((x + dx, y + dy)))
                        {
                            let mut stored = false;
                            for iei in 0..explored.len() {
                                if explored[iei].is_none() {
                                    explored[iei] = Some((x + dx, y + dy));
                                    stored = true;
                                    break;
                                }
                            }
                            if !stored {
                                // explore list full, means there may be space for a shape
                                return false;
                            }
                        }
                    }
                }
            }
        }

        // could not fill list, probably dead
        true
    }

    fn has_dead_zones(&self) -> bool {
        (0..self.height())
            .any(|i| (0..self.width()).any(|j| !self.0.at(j, i) && self.is_dead(j, i)))
    }

    fn fit_cells_to_orig(&self) -> [Option<(usize, usize)>; 2] {
        // this method probably won't work with all shapes.
        // it assumes very simple ones, and it only tries two positions.
        if self.0.at(0, 0) {
            return [Some((0, 0)), None];
        }

        let hx = (0..self.width()).position(|j| self.0.at(j, 0)).unwrap();
        let hy = (0..self.height()).position(|i| self.0.at(0, i)).unwrap();
        [Some((hx, 0)), Some((0, hy))]
    }
}

fn solve(board: Shape, pieces: Vec<Shape>) -> Option<Vec<Step>> {
    if pieces.is_empty() {
        return Some(Vec::new());
    }

    let mut tried_pieces = Vec::new();

    'out: for y in 0..board.height() {
        for x in 0..board.width() {
            if board.0.at(x, y) {
                continue;
            }
            tried_pieces.clear();
            for (i, mut piece) in pieces.iter().cloned().enumerate() {
                for r in 0..4 {
                    if tried_pieces.contains(&piece) {
                        piece = piece.rot();
                        continue;
                    }
                    tried_pieces.push(piece.clone());

                    for (dx, dy) in piece.fit_cells_to_orig().into_iter().flatten() {
                        let (x, y) = (x.saturating_sub(dx), y.saturating_sub(dy));
                        if board.can_put(x, y, &piece) {
                            let new = board.put(x, y, &piece);
                            if !new.has_dead_zones() {
                                let mut remaining = pieces.clone();
                                remaining.swap_remove(i);

                                if let Some(mut steps) = solve(new, remaining) {
                                    steps.insert(
                                        0,
                                        Step {
                                            x,
                                            y,
                                            piece: piece.clone(),
                                            r,
                                        },
                                    );
                                    return Some(steps);
                                }
                            }
                        }
                    }
                    piece = piece.rot();
                }
            }
            break 'out;
        }
    }

    None
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.height() {
            for j in 0..self.width() {
                f.write_str(if self.0.at(j, i) { "# " } else { "_ " })?;
            }
            f.write_str("\n")?;
        }
        Ok(())
    }
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Option<Settings> {
    let shape_t = Shape::from_str(
        "
        # # #
        _ # _
        ",
    );
    let shape_i = Shape::from_str(
        "
        # # # #
        ",
    );
    let shape_l = Shape::from_str(
        "
        # # #
        # _ _
        ",
    );
    let shape_o = Shape::from_str(
        "
        # #
        # #
        ",
    );
    let shape_s = Shape::from_str(
        "
        # # _
        _ # #
        ",
    );

    let _prog = args.next()?;

    let width = match args.next()?.parse() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("could not parse board width: {e}");
            process::exit(ERR_BAD_ARGS);
        }
    };

    let height = match args.next()?.parse() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("could not parse board height: {e}");
            process::exit(ERR_BAD_ARGS);
        }
    };

    let mut pieces = Vec::new();
    for c in args.next()?.chars() {
        pieces.push(match c {
            't' => shape_t.clone(),
            'i' => shape_i.clone(),
            'l' => shape_l.clone(),
            'o' => shape_o.clone(),
            's' => shape_s.clone(),
            'T' => shape_t.clone(),
            'I' => shape_i.clone(),
            'L' => shape_l.flip().rot().rot(),
            'O' => shape_o.clone(),
            'S' => shape_s.flip(),
            _ => {
                eprintln!("unknown shape: {c}");
                process::exit(ERR_BAD_ARGS);
            }
        });
    }

    Some(Settings {
        width,
        height,
        pieces,
    })
}

fn render_step(board: &Shape, step: &Step) -> String {
    let mut result = String::new();

    result.push_str("╲x│");
    for j in 0..board.width() {
        if j % 2 == 1 {
            if j == step.x {
                write!(result, "{:>2}", j.green()).unwrap();
            } else {
                write!(result, "{:>2}", j).unwrap();
            }
        } else {
            result.push_str("  ");
        }
    }
    write!(result, "│ place at x={}, y={}\n", step.x, step.y).unwrap();

    result.push_str("y╲│");
    for j in 0..board.width() {
        if j % 2 == 0 {
            if j == step.x {
                write!(result, "{:>2}", j.green()).unwrap();
            } else {
                write!(result, "{:>2}", j).unwrap();
            }
        } else {
            result.push_str("  ");
        }
    }
    write!(
        result,
        "│ rotate {} time{}\n",
        step.r,
        if step.r == 1 { "" } else { "s" }
    )
    .unwrap();

    write!(result, "──┼{:─>w$}┤\n", "", w = board.width() * 2).unwrap();

    let orig_piece = {
        let mut piece = step.piece.clone();
        for _ in 0..(4 - step.r) {
            piece = piece.rot();
        }
        piece
    };

    let board_after = board.put(step.x, step.y, &step.piece);

    for i in 0..board.height() {
        if i == step.y {
            write!(result, "{:>2}│", i.green()).unwrap();
        } else {
            write!(result, "{:>2}│", i).unwrap();
        }

        for j in 0..board.width() {
            if board.0.at(j, i) {
                result.push_str("██");
            } else if board_after.0.at(j, i) {
                write!(result, "{}", "██".green()).unwrap();
            } else {
                result.push_str("  ");
            }
        }

        result.push_str("│");

        if i < orig_piece.height() {
            result.push_str(" ");
            for j in 0..orig_piece.width() {
                result.push_str(if orig_piece.0.at(j, i) {
                    "██"
                } else {
                    "  "
                });
            }
        }

        result.push_str("\n");
    }
    write!(result, "  └{:─>w$}┘\n", "", w = board.width() * 2).unwrap();

    result
}

fn main() {
    let settings = match parse_args(env::args()) {
        Some(s) => s,
        None => {
            eprintln!("usage: {} WIDTH HEIGHT PIECES", env::args().next().unwrap());
            eprintln!("available pieces (uppercase to flip it): tilos");
            process::exit(ERR_BAD_ARGS);
        }
    };

    if settings
        .pieces
        .iter()
        .map(|piece| piece.len())
        .sum::<usize>()
        != settings.width * settings.height
    {
        eprintln!("available pieces (given pieces are not enough or are too much)");
        process::exit(ERR_NO_SOLUTION);
    }

    if let Some(solution) = solve(
        Shape::with_size(settings.width, settings.height),
        settings.pieces,
    ) {
        let mut board = Shape::with_size(settings.width, settings.height);
        for step in solution {
            println!("{}", render_step(&board, &step));
            board = board.put(step.x, step.y, &step.piece);
        }
    } else {
        println!("no solution found!");
        process::exit(ERR_NO_SOLUTION);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_deadzones() {
        assert!(Shape::from_str(
            "
            # # # _ _ _ _ _
            _ # _ _ _ _ _ _
            # # # _ _ _ _ _
            _ # _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            ",
        )
        .has_dead_zones());
        assert!(Shape::from_str(
            "
            # # # _ _ _ _ _
            _ _ # _ _ _ _ _
            # # # _ _ _ _ _
            _ # _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            ",
        )
        .has_dead_zones());
        assert!(Shape::from_str(
            "
            # # # _ _ _ _ _
            _ _ # _ _ _ _ _
            # _ # # # _ _ _
            # # # _ _ _ _ _
            # _ _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            ",
        )
        .has_dead_zones());
        assert!(!Shape::from_str(
            "
            # # # # # _ _ _
            _ _ _ _ # _ _ _
            # # # # # # _ _
            _ _ _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            _ _ _ _ _ _ _ _
            ",
        )
        .has_dead_zones());
    }

    #[test]
    fn check_renders_itself() {
        let settings =
            parse_args(vec!["", "0", "0", "tilosLS"].into_iter().map(String::from)).unwrap();

        settings
            .pieces
            .into_iter()
            .for_each(|piece| assert_eq!(Shape::from_str(&format!("{}", piece)), piece));
    }

    #[test]
    fn check_flips_back_itself() {
        let settings =
            parse_args(vec!["", "0", "0", "tilos"].into_iter().map(String::from)).unwrap();

        settings
            .pieces
            .into_iter()
            .for_each(|piece| assert_eq!(piece.flip().flip(), piece));
    }

    #[test]
    fn test_puts_itself() {
        let settings = parse_args(vec!["", "0", "0", "t"].into_iter().map(String::from)).unwrap();

        let piece = settings.pieces.into_iter().next().unwrap();
        assert_eq!(
            Shape::with_size(piece.width(), piece.height()).put(0, 0, &piece),
            piece
        );
    }

    #[test]
    fn test_put_respects_filled() {
        let a = Shape::from_str(
            "
            # _ _ _
            # # # _
            ",
        );
        let b = Shape::from_str(
            "
            _ # # #
            _ _ _ #
            ",
        );
        let c = Shape::from_str(
            "
            # # # #
            # # # #
            ",
        );
        assert_eq!(a.put(0, 0, &b), c);
    }

    #[test]
    fn check_all_rotations() {
        let settings = parse_args(vec!["", "4", "3", "LLs"].into_iter().map(String::from)).unwrap();
        assert!(solve(
            Shape::with_size(settings.width, settings.height),
            settings.pieces,
        )
        .is_some());
    }
}
