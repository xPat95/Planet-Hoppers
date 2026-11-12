# Planet Hoppers

Proyecto final de Gráficas por Computadora: un sistema solar con seis planetas (Hexbound, Star Wars, Smiling Friends, Minecraft, Un Show Más y South Park) que se exploran desde una nave. Cada planeta abre un minijuego.

## Integrantes y contribuciones

- **Pablo (xPat95):** minijuegos de Minecraft y South Park, escala del sistema solar, tokens y medallero, final con agujero negro.
- **Cami (camsandvl):** sistema solar (sol, planetas, órbitas, iluminación, shaders y bloom), naves y hangar, menús e interfaz, minijuegos de Smiling Friends y Hexbound.
- **Norman:** minijuegos de Star Wars y Un Show Más, con sus planetas y naves.

## Ejecutar

```powershell
cargo run --release
```

Requiere Rust, CMake y LLVM (ver [compilar en Windows](docs/solar-system.md#compilar-en-windows)). `cargo run --release --bin solarsystem` abre directo el sistema solar.

## Controles

W/S acelerar y frenar · mouse o A/D girar · R/F inclinar · Espacio subir · Shift turbo · Tab objetivo · B piloto automático · E explorar planeta · C vista general · H ayuda · T medallero · F4 contador de rendimiento · F5/F6 apagar descarte / nivel de detalle · F7 renderizador por software · Ctrl+X salir.

## Características que pedimos evaluar

| Característica | Dónde verla |
| --- | --- |
| Pipeline propio por software (modelo, vista, proyección, recorte, rasterizado, profundidad, shaders de fragmentos en la CPU) | F7 en el vuelo, o `cargo run --release --bin solarsystem -- --software` (`src/graphics/software.rs`) |
| Generación y subdivisión de geometría antes de rasterizar | el sol (icosaedro subdividido) y las versiones livianas de cada modelo (`SoftMesh::sphere`, `SoftMesh::decimate`) |
| 6 planetas distinguibles que orbitan y rotan | sistema solar (`src/solarSystem/`, `config/solar-system.json`) |
| Shaders propios: planetas, sol, cielo, bloom, agujero negro | `src/graphics/shaders/` |
| Shader animado | el sol (`sun.fs`) |
| Control local cerca de un planeta | la nave acompaña la órbita del planeta cercano (`src/ships/`) |
| Aceleración, frenado y giro progresivos (con `dt`) | `src/ships/mod.rs` |
| Colisión con los planetas | `Ship::resolve_collisions` |
| Descarte fuera de la vista | `in_view` (`src/scenes/solar_system.rs`); F4 y F5 |
| Niveles de detalle | Hexbound, Star Wars y Un Show Más; F4 y F6 |
| Coherencia visual y claridad | interfaz, misión en pantalla, ayuda (H) |

## Video y demostración

[Video del recorrido](docs/video/recorrido.mp4) · [Guía para la demostración en vivo](docs/demo.md) · [Detalles del proyecto](docs/detalles.md)

## Uso de IA

Usamos asistentes de IA (Claude) para escribir y revisar parte del código, los scripts de Blender y la documentación. Revisamos y entendemos el código entregado.
