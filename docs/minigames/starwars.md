# Star Wars — Asalto al núcleo integrado

La versión jugable está en `src/minigames/starwars/`, dentro de Planet Hoppers. La copia independiente de ProyectoFinal se conserva como prototipo anterior; el juego principal no depende de ella.

## Jugar

Ejecutar `cargo run --release`, elegir una nave y acercarse al planeta Star Wars. Explorar con E/Enter abre el nivel. Primero se reproduce el video inicial; al terminarlo u omitirlo aparece el menú del nivel. Enter inicia el vuelo.

- WASD/flechas: desplazamiento por el interior del túnel.
- Espacio: láseres manuales y seis torpedos al llegar al núcleo.
- Shift: turbo. P: pausa y reanudación. Enter continúa desde pausa.
- R: reiniciar desde el video inicial y volver al menú.
- Esc: regresar al sistema solar (abandono; si ya hubo derrota, resultado Defeat).
- F11: pantalla completa. F12: captura en `target/starwars/capturas/`.
- Cerrar la ventana con X mantiene el contrato general de salida del juego principal.

El video final devuelve directamente al sistema solar con Victory, sin pantalla adicional. La canción se limita al vuelo/explosión y respeta la pausa. Se conservan tres barras de integridad, disparos manuales, cuatro ataques desde popa, obstáculos industriales y tres paneles independientes en cada compuerta final.

## Nave, tamaño y colisión

`solar_system::Exit::Visit` toma una copia del `ShipModel` ya resuelto de `session.ships.ships[session.ship_index]` y la pasa a `minigame_host::run_with_ship`. No se deduce la nave por el planeta ni se vuelve a consultar el active_ship inicial del JSON. Esto conserva también el alias starwars -> N-1 del Mandaloriano.

Se pasan metadatos y rutas, nunca recursos GPU entre ventanas. `TunnelShip` carga el modelo después de abrir el contexto del minijuego y lo libera antes de cerrar esa ventana. El anfitrión conserva su sesión y vuelve a cargar sus recursos al regresar.

El catálogo actual usa GLB exportados de Blender. Se reutiliza exactamente la ruta activa y el cargador `SpaceModel` del sistema solar, con transparencia, emisiones y materiales. Si una ruta apunta a un OBJ del paquete Blender y existe su `<nombre>_near.glb` preparado al lado, se utiliza esa exportación de juego del mismo recurso. Los OBJ pequeños sin exportación preparada pueden usar el cargador de Raylib con su MTL y texturas; los OBJ grandes deben prepararse para uso en tiempo real. No se sustituyen silenciosamente errores de carga por el Halcón.

El ajuste centra y orienta el modelo, conserva sus proporciones y aplica UN único factor para caber en un semivolumen máximo de [0.95, 0.65, 1.40] unidades. Añade 0.035 de tolerancia física por eje; no modifica el tamaño solar ni deforma las naves por eje. Al vuelo solar normalizado hacia -Z se aplica una base de 180° en Y para avanzar por el túnel hacia +Z.

`Game.ship_half` se deriva del modelo ajustado. `hull_half()` proyecta ese volumen con cabeceo y ladeo y lo usa para paredes, obstáculos, disparos de popa y alerta visual. Se conserva durante reinicio y reproducción del video. El disparo nace delante del volumen del casco; el refuerzo de estela queda detrás de sus límites.

## Código y recursos

- `mod.rs`: ventana, flujo de videos/menú y contrato de retorno al anfitrión.
- `simulation.rs`: movimiento, dimensiones del casco, colisiones, integridad y pruebas.
- `ship.rs`: carga y ajuste uniforme del modelo elegido.
- `render.rs`, `finish.fs`: escenario y acabado de la imagen. Mantienen la carga segura de fuente Unicode.
- `video.rs`, `music.rs`: cinemáticas indexadas, audio sincronizado y canción del vuelo.
- `assets/minigames/starwars/`: originales MP4, SFV/OGG preparados, canción y fuente con licencia. `manifest.json` conserva tamaños y hashes de los archivos transferidos.
- `tools/starwars/prepare_videos.py`: preparación de los videos si se reemplazan sus fuentes; requiere FFmpeg solo durante esa preparación.

## Comprobar

```powershell
cargo test --locked --offline
cargo run --release --bin starwars -- --check-ships
cargo run --release --bin starwars -- --ship starwars --check-startup
cargo run --release -- --smoke-starwars --ship starwars
```

`--check-ships` carga las seis elecciones reales, completa el recorrido normal y con turbo con cada una, comprueba daño por pared/obstáculo y guarda seis vistas en `target/starwars/ships/`. `--smoke-starwars` verifica selección inicial -> sistema solar -> nivel completo con ambos videos -> regreso con nave, órbitas y selección conservadas.

La prueba gráfica del OBJ del Mandaloriano se ejecuta explícitamente con `cargo test --test starwars_host -- --ignored --exact obj_ship_uses_the_same_loader_and_safe_startup --nocapture`. La suite normal permanece libre de ventanas.
