//! F3 diagnostics; rendering only, no gameplay effect.
use crate::gameplay::station::Station;
use raylib::prelude::*;

pub fn draw(d: &mut impl RaylibDraw3D, station: &Station, ship: Vector3, radius: f32) {
    // The yellow sphere is the historical envelope retained ONLY for safe launch.
    d.draw_sphere_wires(
        station.center(),
        station.collision_radius,
        16,
        24,
        Color::new(210, 170, 50, 80),
    );
    let mut count = 0;
    let local_ship = station.collision_local(ship);
    let reach = 100.0 * station.hull.reference_length / station.length;
    for (lo, hi) in station.hull.boxes() {
        if ((lo + hi) * 0.5 - local_ship).length() > reach {
            continue;
        }
        let corners: Vec<_> = (0..8)
            .map(|k| {
                station.collision_point(Vector3::new(
                    if k & 1 == 0 { lo.x } else { hi.x },
                    if k & 2 == 0 { lo.y } else { hi.y },
                    if k & 4 == 0 { lo.z } else { hi.z },
                ))
            })
            .collect();
        for k in 0..8 {
            for bit in [1, 2, 4] {
                if k & bit == 0 {
                    d.draw_line_3D(corners[k], corners[k | bit], Color::new(50, 220, 240, 150));
                }
            }
        }
        count += 1;
        if count >= 512 {
            break;
        }
    }
    let entry = station.entry_anchor(radius);
    d.draw_sphere_wires(
        entry,
        station
            .dock
            .interaction_radius
            .min(station.dock.width * 2.0),
        12,
        16,
        Color::GREEN,
    );
    d.draw_sphere_wires(ship, radius, 8, 12, Color::RED);
    d.draw_line_3D(station.exit_position(), entry, Color::GREEN);
    d.draw_line_3D(station.center(), ship, Color::new(170, 170, 210, 160));
    // Door aperture/frame guides in the exact configured global transform.
    let (w, h) = (station.dock.width * 0.5, station.dock.height * 0.5);
    for depth in [0.0, 9.0] {
        let p = |x: f32, y: f32| {
            station.exit_position()
                + station.exit_right() * x
                + station.exit_up() * y
                + station.exit_forward() * depth
        };
        for (a, b) in [
            ((-w, -h), (w, -h)),
            ((w, -h), (w, h)),
            ((w, h), (-w, h)),
            ((-w, h), (-w, -h)),
        ] {
            d.draw_line_3D(p(a.0, a.1), p(b.0, b.1), Color::ORANGE);
        }
    }
}

pub fn hud(
    d: &mut impl RaylibDraw,
    station: &Station,
    p: Vector3,
    radius: f32,
    last: &str,
    camera: &str,
) {
    d.draw_rectangle(12, 100, 770, 158, Color::new(0, 8, 16, 225));
    for (i, line) in [
        "F3 COLISIONES | cian: superficies reales (hasta 512 cajas cercanas)".to_owned(),
        "amarillo: envolvente de despegue, NO barrera | verde: entrada | rojo: nave".to_owned(),
        format!(
            "Nave ({:.1}, {:.1}, {:.1}) | radio {:.2} | distancia centro {:.1}",
            p.x,
            p.y,
            p.z,
            radius,
            (p - station.center()).length()
        ),
        format!(
            "Entrada: {} | distancia ancla {:.1} | rango {:.1}",
            station.in_reach(p, radius),
            (p - station.entry_anchor(radius)).length(),
            station
                .dock
                .interaction_radius
                .min(station.dock.width * 2.0)
        ),
        "Limite mundial de vuelo: NO EXISTE. Orbitas/zoom/clip no bloquean movimiento.".to_owned(),
        format!("Ultima correccion nave: {last}"),
        format!("Camara este frame: {camera}"),
    ]
    .iter()
    .enumerate()
    {
        d.draw_text(line, 20, 108 + i as i32 * 20, 16, Color::WHITE);
    }
}
