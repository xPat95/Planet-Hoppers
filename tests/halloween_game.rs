use planet_hoppers::minigames::halloween::{recognize, Balance, Cast, Glyph, Kind, Phase, Session, State};
use std::f32::consts::PI;

/// Trazo por los vértices, con puntos cada `step` píxeles y un temblor determinista de `wobble` píxeles.
fn stroke(vertices: &[(f32, f32)], step: f32, wobble: f32) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    let mut k = 0u32;
    for w in vertices.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let n = (len / step).max(1.0) as u32;
        for i in 0..n {
            let t = i as f32 / n as f32;
            k += 1;
            let j = wobble * ((k as f32 * 1.7).sin() + 0.5 * (k as f32 * 4.3).cos());
            out.push((a.0 + (b.0 - a.0) * t + j, a.1 + (b.1 - a.1) * t - j));
        }
    }
    out.push(*vertices.last().unwrap());
    out
}

fn circle(cx: f32, cy: f32, r: f32, start: f32, dir: f32, closing: f32) -> Vec<(f32, f32)> {
    (0..=60)
        .map(|i| {
            let a = start + dir * (2.0 * PI * closing) * i as f32 / 60.0;
            (cx + r * a.cos() + (i as f32 * 2.1).sin(), cy + r * a.sin())
        })
        .collect()
}

#[test]
fn reconoce_lineas_en_cualquier_direccion_y_tamano() {
    for (a, b) in [((100.0, 300.0), (500.0, 310.0)), ((600.0, 200.0), (420.0, 180.0)), ((10.0, 10.0), (90.0, 30.0))] {
        assert_eq!(recognize(&stroke(&[a, b], 6.0, 2.0)), Some(Glyph::Line), "{a:?}->{b:?}");
    }
}

#[test]
fn una_linea_vertical_no_es_ningun_glifo() {
    assert_eq!(recognize(&stroke(&[(300.0, 100.0), (305.0, 400.0)], 6.0, 2.0)), None);
}

#[test]
fn reconoce_v_en_las_dos_direcciones() {
    let v = [(100.0, 100.0), (200.0, 320.0), (300.0, 110.0)];
    assert_eq!(recognize(&stroke(&v, 5.0, 2.0)), Some(Glyph::Vee));
    let back: Vec<_> = v.iter().rev().copied().collect();
    assert_eq!(recognize(&stroke(&back, 5.0, 2.0)), Some(Glyph::Vee));
    // Chica y un poco torcida
    assert_eq!(recognize(&stroke(&[(50.0, 50.0), (75.0, 120.0), (115.0, 60.0)], 3.0, 1.0)), Some(Glyph::Vee));
}

#[test]
fn una_v_al_reves_no_es_v() {
    assert_ne!(recognize(&stroke(&[(100.0, 320.0), (200.0, 100.0), (300.0, 310.0)], 5.0, 2.0)), Some(Glyph::Vee));
}

#[test]
fn reconoce_circulos_en_los_dos_sentidos_aunque_no_cierren_del_todo() {
    assert_eq!(recognize(&circle(300.0, 300.0, 120.0, 0.3, 1.0, 1.0)), Some(Glyph::Circle));
    assert_eq!(recognize(&circle(300.0, 300.0, 60.0, 2.0, -1.0, 0.92)), Some(Glyph::Circle));
}

#[test]
fn reconoce_el_rayo() {
    let bolt = [(260.0, 60.0), (200.0, 220.0), (300.0, 200.0), (230.0, 380.0)];
    assert_eq!(recognize(&stroke(&bolt, 5.0, 2.0)), Some(Glyph::Bolt));
    let back: Vec<_> = bolt.iter().rev().copied().collect();
    assert_eq!(recognize(&stroke(&back, 5.0, 2.0)), Some(Glyph::Bolt));
}

#[test]
fn rechaza_clics_y_garabatos() {
    assert_eq!(recognize(&[(10.0, 10.0), (12.0, 11.0), (13.0, 12.0)]), None);
    let scribble = [(100.0, 100.0), (300.0, 120.0), (110.0, 160.0), (290.0, 200.0), (120.0, 240.0), (300.0, 280.0)];
    assert_eq!(recognize(&stroke(&scribble, 5.0, 1.0)), None);
}

fn play_until(s: &mut Session, phase: Phase, max: f32) {
    let mut t = 0.0;
    while s.phase != phase && t < max {
        s.update(0.05);
        t += 0.05;
    }
}

#[test]
fn empieza_en_la_primera_oleada_con_salud_llena() {
    // Los glifos se aprenden en la intro: la partida arranca directo en la oleada
    let s = Session::new(Balance::default(), 7);
    assert_eq!(s.phase, Phase::Wave(1));
    assert_eq!(s.health, 3);
}

#[test]
fn un_hechizo_calma_a_la_mas_cercana_con_ese_glifo() {
    let mut s = Session::new(Balance::default(), 3);
    play_until(&mut s, Phase::Wave(1), 60.0);
    // Sin lanzar nada hasta que alguna sale de lo oscuro (todavía no alcanzó a la bruja)
    for _ in 0..200 {
        if s.creatures.iter().any(|c| c.state == State::Approaching && s.seen(c)) {
            break;
        }
        s.update(0.05);
    }
    let first = s.creatures.iter().find(|c| c.state == State::Approaching && s.seen(c)).expect("hay criaturas").clone();
    let glyph = first.glyphs[0];
    let nearest = s
        .creatures
        .iter()
        .filter(|c| c.state == State::Approaching && s.seen(c) && c.glyphs[0] == glyph)
        .min_by(|a, b| a.distance().total_cmp(&b.distance()))
        .unwrap()
        .id;
    match s.cast(glyph) {
        Cast::Hit { id, .. } => assert_eq!(id, nearest),
        Cast::Miss => panic!("no le pegó a nadie"),
    }
}

#[test]
fn equivocarse_de_glifo_no_cuesta_nada() {
    let mut s = Session::new(Balance::default(), 3);
    s.creatures.clear();
    assert_eq!(s.cast(Glyph::Circle), Cast::Miss);
    assert_eq!(s.health, 3);
}

#[test]
fn sin_hechizos_se_pierde_y_la_proteccion_evita_golpes_seguidos() {
    let mut s = Session::new(Balance::default(), 11);
    let mut last = s.health;
    let mut t = 0.0;
    let mut last_hit = -10.0;
    while !s.finished() && t < 400.0 {
        s.update(0.02);
        t += 0.02;
        if s.health < last {
            assert!(t - last_hit >= Balance::default().protection - 0.03, "dos golpes juntos");
            last_hit = t;
            last = s.health;
        }
    }
    assert_eq!(s.phase, Phase::Defeat);
}

#[test]
fn calmando_todo_se_gana_en_menos_de_tres_minutos_y_la_fiesta_se_queda() {
    let mut s = Session::new(Balance::default(), 5);
    let mut t = 0.0;
    let mut demons = 0;
    while !s.finished() && t < 400.0 {
        s.update(0.05);
        t += 0.05;
        demons += s.creatures.iter().filter(|c| c.kind == Kind::Demon).count();
        let targets: Vec<Glyph> =
            s.creatures.iter().filter(|c| c.state == State::Approaching && c.distance() < 4.0).map(|c| c.glyphs[0]).collect();
        for g in targets {
            s.cast(g);
        }
    }
    assert_eq!(s.phase, Phase::Victory);
    assert_eq!(s.health, 3);
    assert!(s.total_time <= 180.0, "duró {}", s.total_time);
    assert!(demons > 0, "los demonios salen en la segunda oleada");
    assert!(s.creatures.iter().all(|c| matches!(c.state, State::Calmed { .. })));
    assert!(s.calmed_count() > 20);
}

#[test]
fn los_demonios_necesitan_dos_glifos() {
    let mut s = Session::new(Balance::default(), 9);
    let mut t = 0.0;
    // Defendiéndose con los glifos de las que están cerca, hasta que aparece un demonio
    while !s.creatures.iter().any(|c| c.kind == Kind::Demon) && t < 400.0 {
        s.update(0.05);
        t += 0.05;
        let targets: Vec<Glyph> =
            s.creatures.iter().filter(|c| c.state == State::Approaching && c.distance() < 2.7).map(|c| c.glyphs[0]).collect();
        for g in targets {
            s.cast(g);
        }
    }
    assert_eq!(s.phase, Phase::Wave(2));
    let d = s.creatures.iter().find(|c| c.kind == Kind::Demon).unwrap().clone();
    assert_eq!(d.glyphs.len(), 2);
}
