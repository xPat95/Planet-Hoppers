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

**Renderizador por software (F7):** el sistema solar entero dibujado en la CPU (`src/graphics/software.rs`): matrices de modelo, vista y proyección, recorte contra el plano cercano, rasterizado con baricéntricas y buffer de profundidad propio, y los "shaders de fragmentos" en Rust (luz del sol en los planetas; el sol animado con ruido), con los colores calculados lineales y pasados a la pantalla con gamma 2,2. Al cargar, la textura de cada material se lee en la coordenada de cada vértice y queda horneada en su color, y las versiones livianas solo juntan vértices del mismo material. Se reparte en franjas, una por núcleo. Arriba muestra la resolución (`SOFT_HEIGHT`, 420 de alto), los triángulos rasterizados y los milisegundos por cuadro (unos 6 a 22 ms). Para cambiar en vivo: `SOFT_HEIGHT` y `SOFT_NEAR_TRIS` / `SOFT_FAR_TRIS` (detalle de cada planeta) en `src/scenes/solar_system.rs`, o la luz ambiente y el borde frío en `fragment` (`software.rs`).

**Video:** `docs/video/recorrido.mp4` se generó con `cargo run --release --bin solarsystem -- --demo-video` (vuelo automático hasta Smiling Friends, vuelta completa y alejamiento; cuadros en `target/solarsystem/demo-video/`) y ffmpeg a 30 cuadros por segundo.
