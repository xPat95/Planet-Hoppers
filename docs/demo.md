# Demostración en vivo

Cambios que el profesor puede pedir, dónde se hacen y qué se ve. Después de cambiar un valor: `cargo run --release --bin solarsystem` (abre directo el sistema solar).

| Parámetro | Dónde | Qué pasa |
| --- | --- | --- |
| Radio de un planeta | `config/solar-system.json` → `radius` | El planeta (y su colisión y su alcance de exploración) cambia de tamaño. Todo se multiplica por `WORLD_SCALE = 12` (`src/solarSystem/scale.rs`). |
| Distancia de la órbita | `config/solar-system.json` → `orbit_radius` | El planeta orbita más cerca o más lejos del sol. |
| Velocidad orbital | `config/solar-system.json` → `orbit_speed_degrees` | Grados por segundo alrededor del sol (multiplicado por el tiempo, no por los cuadros). |
| Rotación propia | `config/solar-system.json` → `rotation_speed_degrees` | Grados por segundo sobre su eje; negativo gira al revés. |
| Semilla | `SKY_SEED` en `src/graphics/space.rs` | Las 9000 estrellas del cielo cambian de lugar; con la misma semilla salen siempre igual. |
| Intensidad del sol | `SUN_EMISSION` en `src/graphics/space.rs` (shader `sun.fs`) | El sol brilla más o menos. |
| Intensidad del bloom | `INTENSITY` y `THRESHOLD` en `src/graphics/bloom.rs` | Más o menos brillo derramado alrededor de lo que brilla. |
| Distancia del nivel de detalle | `FAR_LOD` en `src/scenes/solar_system.rs` | A cuántos radios un planeta pasa a su versión liviana. |

**Descarte y nivel de detalle:** durante el vuelo, F4 muestra los triángulos dibujados, los ahorrados por descarte y por nivel de detalle, y qué versión de cada planeta se dibuja. F5 apaga el descarte y F6 el nivel de detalle: el contador muestra cuánto trabajo extra hace el renderizador.

### Renderizador por software

Ejecutar `cargo run --release --locked --bin solarsystem -- --software` para abrir el vuelo de evaluación. F7 alterna el renderizador durante el vuelo. Los minijuegos, el hangar y las cinemáticas usan su presentación original.

- **Modelos.** `src/graphics/software_asset.rs` lee las escenas GLB, sus transformaciones, normales, coordenadas UV y materiales en memoria de la CPU. Las naves configuradas con OBJ usan su exportación GLB incluida en los recursos.
- **Rasterización.** `src/graphics/software.rs` transforma los vértices compartidos una vez por objeto y cuadro. Recorta los triángulos que cruzan el plano cercano y descarta los que quedan fuera de los centros de píxel. Reparte los demás entre las franjas de pantalla que ocupan.
- **Superficies.** Las coordenadas UV se interpolan con corrección de perspectiva. La textura se consulta por píxel. La prueba de profundidad resuelve los objetos sólidos y la mezcla ordenada permite ver el interior de la cúpula de Un Show Más. El sol tiene su propio shader animado en Rust.
- **Rendimiento.** El rótulo de F7 muestra resolución interna, triángulos y tiempo del renderizador. F4 detalla el descarte y el nivel de detalle por planeta. Se pueden ajustar `SOFT_HEIGHT`, `SOFT_NEAR_TRIS` y `SOFT_FAR_TRIS` en `src/scenes/solar_system.rs` y recompilar.

### Recorrido de entrega

[Ver el vuelo con el renderizador por software](video/recorrido-software.webm). Incluye vuelo espacial, acercamiento a Smiling Friends, vuelta y alejamiento. Para reproducir la trayectoria y guardar sus cuadros en `target/solarsystem/demo-video/`, ejecutar `cargo run --release --locked --bin solarsystem -- --software --demo-video`. La simulación avanza a 60 pasos por segundo y guarda un cuadro cada dos pasos para el video de 30 FPS.

[Ver el recorrido con la presentación original](video/recorrido.mp4).
