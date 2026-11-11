//! Minimal, dependency-free check of the production case and confirmation rules.
//! rustc --edition 2021 -Awarnings tools/southpark/validate_murdoku.rs -o target/validate_murdoku.exe
#[path = "../../src/minigames/southpark/murdoku.rs"]
mod murdoku;
mod threats {
    #[derive(Clone, Copy, Debug)]
    pub enum ThreatKind {
        Snowman,
        Ufo,
        Bus,
        Piano,
        Sewer,
    }
}
#[path = "../../src/minigames/southpark/puzzles.rs"]
mod puzzles;
use murdoku::*;
fn run() -> puzzles::PuzzleRun {
    let mut p = puzzles::PuzzleRun::new(threats::ThreatKind::Bus, 2);
    p.tick(0.4);
    p.start();
    p
}
fn fill(p: &mut puzzles::PuzzleRun) {
    for (i, cell) in SOLUTION.into_iter().enumerate() {
        p.select_bus_piece(i);
        p.place_bus_piece(cell);
    }
}
fn main() {
    assert_eq!(SOLUTION_COORDS.map(|(r, c)| r * SIZE + c), SOLUTION);
    // Independently transcribed from the supplied board, not from the solution.
    assert_eq!(
        ZONES,
        [
            0, 0, 0, 1, 2, 2, 0, 0, 0, 1, 1, 2, 0, 0, 0, 1, 1, 1, 0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3
        ]
    );
    assert_eq!(BLOCKED, [0, 2, 3, 5, 7, 8, 10, 14, 21, 22]);
    assert_eq!(BUSES, [[25, 26], [33, 34]]);
    assert_eq!(MANHOLES, [27, 31]);
    assert_eq!(BENCHES, [1, 15, 16]);
    assert_eq!(KIOSKS, [5, 14, 21, 22]);
    let found = solutions();
    assert_eq!(found, vec![(SOLUTION, DANGEROUS)]);
    let mut p = run();
    p.confirm_bus();
    assert!(p.result.is_none());
    p.select_bus_piece(0);
    p.place_bus_piece(0);
    p.place_bus_piece(36);
    let puzzles::Board::Bus(case) = &p.board else {
        panic!()
    };
    assert!(case.positions.iter().all(Option::is_none));
    p.place_bus_piece(6);
    p.select_bus_piece(1);
    p.place_bus_piece(6);
    let puzzles::Board::Bus(case) = &p.board else {
        panic!()
    };
    assert_eq!(case.positions[1], None);
    p.select_bus_piece(0);
    p.remove_bus_piece();
    p.clear_bus();
    fill(&mut p);
    p.confirm_bus();
    assert!(p.result.is_none());
    p.accuse_bus_driver(DANGEROUS);
    p.confirm_bus();
    assert_eq!(p.result, Some(true));
    p.clear_bus();
    p.accuse_bus_driver(1);
    p.confirm_bus();
    assert_eq!(p.result, Some(true));
    for driver in [1, 2, 4, 5] {
        let mut p = run();
        fill(&mut p);
        p.accuse_bus_driver(driver);
        p.confirm_bus();
        assert_eq!(p.result, Some(false));
        p.accuse_bus_driver(DANGEROUS);
        p.confirm_bus();
        assert_eq!(p.result, Some(false));
    }
    // Correct accusation cannot rescue an incorrect full distribution.
    let mut p = run();
    fill(&mut p);
    p.select_bus_piece(0);
    p.place_bus_piece(12);
    p.accuse_bus_driver(DANGEROUS);
    p.confirm_bus();
    assert_eq!(p.result, Some(false));
    assert!(!is_long_press(0.449));assert!(is_long_press(0.450));
    let mut notes=Case::default();notes.select(1);notes.annotate(6);notes.annotate(12);
    assert_eq!(notes.candidates[6],2);assert_eq!(notes.candidates[12],2);
    notes.select(2);notes.annotate(6);assert_eq!(notes.candidates[6],6);
    assert!(notes.positions.iter().all(Option::is_none));assert!(!notes.ready());
    notes.toggle_mode();notes.annotate(6);assert!(notes.excluded[6]);assert_eq!(notes.candidates[6],6);
    notes.annotate(6);assert!(!notes.excluded[6]);notes.toggle_mode();
    notes.annotate(6);assert_eq!(notes.candidates[6],2);
    notes.positions=SOLUTION.map(Some);notes.accused=Some(DANGEROUS);
    notes.candidates=[63;36];notes.excluded=[true;36];assert!(notes.correct());
    notes.positions[0]=Some(12);assert!(!notes.correct());
    notes.clear();assert_eq!(notes.candidates,[0;36]);assert_eq!(notes.excluded,[false;36]);
    println!("MURDOKU_OK unique_solution final_positions_only independent_candidates_and_X 450ms_hold single_confirmation");
}
