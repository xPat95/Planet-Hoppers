//! Exact geometry and constraints of Manuel Garand's "Car Repair" case.
//! Reference: https://murdoku.com/pdf/car-repair-bw.pdf and supplied screenshots.
//! Cells are row * 6 + column, with both coordinates zero-based from top left.
pub const SIZE: usize = 6;
// Reception -> centre; Waiting Area -> bus stop; Storage -> depot; Garage -> terminal.
pub const ZONES: [u8; 36] = [
    0, 0, 0, 1, 2, 2, 0, 0, 0, 1, 1, 2, 0, 0, 0, 1, 1, 1, 0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    3, 3, 3, 3,
];
pub const BUILDINGS: [usize; 4] = [0, 2, 7, 8]; // Table and exact L-shaped desk footprint.
pub const RADIO: usize = 3; // Original TV: blocked, inside the waiting area.
pub const PLANT: usize = 10;
pub const KIOSKS: [usize; 4] = [5, 14, 21, 22]; // Original shelves.
pub const BENCHES: [usize; 3] = [1, 15, 16];
pub const MANHOLES: [usize; 2] = [27, 31];
pub const BUSES: [[usize; 2]; 2] = [[25, 26], [33, 34]];
pub const BLOCKED: [usize; 10] = [0, 2, 3, 5, 7, 8, 10, 14, 21, 22];
// Kenny, blue (A), red (B), green (C), black (D), white (E).
pub const SOLUTION_COORDS: [(usize, usize); 6] = [(1, 0), (5, 4), (4, 3), (0, 1), (2, 5), (3, 2)];
pub const SOLUTION: [usize; 6] = [6, 34, 27, 1, 17, 20];
pub const DANGEROUS: usize = 3;
pub fn name(piece: usize, es: bool) -> &'static str {
    if es {
        [
            "Kenny",
            "Chofer azul",
            "Chofer rojo",
            "Chofer verde",
            "Chofer negro",
            "Chofer blanco",
        ][piece]
    } else {
        [
            "Kenny",
            "Blue driver",
            "Red driver",
            "Green driver",
            "Black driver",
            "White driver",
        ][piece]
    }
}
pub fn zone_name(zone: u8, es: bool) -> &'static str {
    if es {
        ["Centro", "Parada", "Depósito", "Terminal"][zone as usize]
    } else {
        ["Centre", "Bus stop", "Depot", "Terminal"][zone as usize]
    }
}
pub fn clue(piece: usize, es: bool) -> &'static str {
    if es {
        [
        "Kenny está a solas, en el mismo sector, con el chofer que lo atropellará. ¡Todavía puedes salvarlo!",
        "Estoy dentro de un autobús.",
        "Estoy sobre una alcantarilla.",
        "Estoy sentado en un banco.",
        "Estoy a solas en el sector Parada.",
        "Estoy junto a un quiosco, por un lado (no en diagonal), en el mismo sector.",
    ][piece]
    } else {
        [
        "Kenny is alone in the same sector with the driver who will hit him. There is still time to save him!",
        "I am inside a bus.",
        "I am over a manhole.",
        "I am sitting on a bench.",
        "I am alone in the Bus stop sector.",
        "I am beside a kiosk, horizontally or vertically, in the same sector.",
    ][piece]
    }
}
pub fn valid_cell(cell: usize) -> bool {
    cell < 36 && !BLOCKED.contains(&cell)
}
fn adjacent(a: usize, b: usize) -> bool {
    (a / 6).abs_diff(b / 6) + (a % 6).abs_diff(b % 6) == 1 && ZONES[a] == ZONES[b]
}
pub fn local_clue(piece: usize, cell: usize) -> bool {
    if !valid_cell(cell) {
        return false;
    }
    match piece {
        0 => true,
        1 => BUSES.iter().flatten().any(|c| *c == cell),
        2 => MANHOLES.contains(&cell),
        3 => BENCHES.contains(&cell),
        4 => ZONES[cell] == 1,
        5 => KIOSKS.iter().any(|c| adjacent(cell, *c)),
        _ => false,
    }
}
/// Checks published constraints, never the stored answer coordinates.
pub fn culprit(positions: &[usize; 6]) -> Option<usize> {
    for (i, c) in positions.iter().enumerate() {
        if !local_clue(i, *c)
            || positions[..i]
                .iter()
                .any(|d| c / 6 == d / 6 || c % 6 == d % 6)
        {
            return None;
        }
    }
    if positions
        .iter()
        .filter(|c| ZONES[**c] == ZONES[positions[4]])
        .count()
        != 1
    {
        return None;
    }
    let companions: Vec<_> = (1..6)
        .filter(|i| ZONES[positions[*i]] == ZONES[positions[0]])
        .collect();
    if companions.len() == 1 {
        Some(companions[0])
    } else {
        None
    }
}
pub const HOLD_SECONDS:f32=0.45;
pub fn is_long_press(seconds:f32)->bool {seconds>=HOLD_SECONDS}
#[derive(Clone, Debug)]
pub struct Case {
    pub positions: [Option<usize>; 6],
    pub selected: Option<usize>,
    pub accused: Option<usize>,
    pub candidates: [u8;36],
    pub excluded: [bool;36],
    pub cross_mode: bool,
}
impl Default for Case {
    fn default()->Self {Self{positions:[None;6],selected:None,accused:None,candidates:[0;36],excluded:[false;36],cross_mode:false}}
}
impl Case {
    pub fn annotate(&mut self,cell:usize) {
        if cell>=36 {return;}
        if self.cross_mode {self.excluded[cell]=!self.excluded[cell];}
        else if valid_cell(cell) {if let Some(piece)=self.selected {self.candidates[cell]^=1<<piece;}}
    }
    pub fn toggle_mode(&mut self) {self.cross_mode=!self.cross_mode;}
    pub fn select(&mut self, piece: usize) {
        if piece < 6 {
            self.selected = Some(piece);
        }
    }
    pub fn place(&mut self, cell: usize) {
        let Some(piece) = self.selected else {
            return;
        };
        if valid_cell(cell)
            && !self
                .positions
                .iter()
                .enumerate()
                .any(|(i, c)| i != piece && *c == Some(cell))
        {
            self.positions[piece] = Some(cell);
        }
    }
    pub fn remove(&mut self) {
        if let Some(piece) = self.selected {
            self.positions[piece] = None;
        }
    }
    pub fn accuse(&mut self, piece: usize) {
        if (1..6).contains(&piece) {
            self.accused = Some(piece);
        }
    }
    pub fn ready(&self) -> bool {
        self.positions.iter().all(Option::is_some) && self.accused.is_some()
    }
    pub fn correct(&self) -> bool {
        self.ready() && culprit(&self.positions.map(|c| c.unwrap())) == self.accused
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
/// Development-only enumeration. Does not consult SOLUTION or DANGEROUS.
pub fn solutions() -> Vec<([usize; 6], usize)> {
    fn search(i: usize, p: &mut [usize; 6], out: &mut Vec<([usize; 6], usize)>) {
        if i == 6 {
            if let Some(driver) = culprit(p) {
                out.push((*p, driver));
            }
            return;
        }
        for cell in 0..36 {
            if local_clue(i, cell)
                && p[..i]
                    .iter()
                    .all(|c| c / 6 != cell / 6 && c % 6 != cell % 6)
            {
                p[i] = cell;
                search(i + 1, p, out);
            }
        }
    }
    let mut result = vec![];
    search(0, &mut [0; 6], &mut result);
    result
}
