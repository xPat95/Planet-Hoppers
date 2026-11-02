# Pulido visual de controles y hangar

Trabajo en `main`, inicialmente limpio. Sin commit ni push.

## Referencia y guía visual

Se inspeccionó `src/scenes/front_end.rs` y se capturó el juego antes de editar. El menú usa Bungee, botones inclinados 0,32, borde cian de 3 px, sombra azul y degradado cromado azul/blanco/naranja. El fondo combina galaxia y estrellas. Sus botones tienen 48–66 px de altura y separación de 14 px.

Las pantallas secundarias comparten los procedimientos de botón, degradado e inclinación. Bungee identifica títulos, teclas y acciones; Titillium Web Bold, ya incluido en los recursos, se usa para las descripciones. Blanco para contenido, cian para secciones y naranja para acentos. Se cargan los glifos latinos para conservar tildes y nombres de naves.

## Cambios

- Controles: título con contorno y sombra, dos secciones, teclas destacadas sobre sus descripciones, pequeños acentos inclinados y galaxia atenuada. Sin paneles ni tabla. VOLVER usa el botón arcade compartido.
- Hangar: flechas pixeladas sin caja, cian en reposo y naranja con hover o tecla mantenida. Se conservan sus zonas de clic. Cabecera compacta, degradados transparentes en lugar de franjas sólidas de 151 y 156 px, acción arcade con ajuste para etiquetas largas, contador separado y ayuda discreta.
- Se conservan entradas, navegación, estados, condiciones de confirmación, cámara, iluminación, plataforma, modelos y lógica de vuelo. El ajuste de letra de etiquetas largas conserva las dimensiones y el aspecto de los tres botones originales del menú.

## Capturas nativas

Imágenes de raylib sin retoques, disponibles localmente en `target`:

- Controles: [antes](target/ui-before/controls.png), [después](target/front-end/controls.png).
- Hangar: [antes](target/ui-before/hangar-starwars.png), [después](target/front-end/hangar-starwars.png).
- Mínimo: [controles](target/front-end/controls-960x640.png), [hangar](target/front-end/hangar-960x640.png).
- Cuadrado: [controles](target/front-end/controls-1024x1024.png), [hangar](target/front-end/hangar-1024x1024.png).
- Amplio: [controles](target/front-end/controls-1920x1055.png), [hangar](target/front-end/hangar-1920x1055.png).
- Estación: [botón con etiqueta larga](target/front-end/docked-selected.png).

Se revisaron jerarquía, separación, ausencia de recortes, tildes, hover de flechas, integración con la escena y etiqueta larga. El escritorio limita la ventana solicitada de 1920×1080 a 1920×1055; se documenta el tamaño real. Las capturas en `target` no se versionan y pueden sobrescribirse con otro recorrido smoke.

## Verificación y límites

Pasan `cargo check --locked`, `cargo build --locked` y las 28 pruebas de `ship_selection`, `ships`, `ship_presentation`, `station_departure` y `minigame_returns`.

`--smoke --ship minecraft` pasa controles, volver, seis naves, flechas, vistas V, teclado y despegue. `--smoke-docking --ship minecraft` pasa tres ciclos: misma nave, cambio y cancelación, con despegue seguro. Logs: `target/ui-before.log`, `target/ui-after.log`, `target/ui-docking.log`.

La UI usa dibujo inmediato de raylib; se conservan las interacciones existentes sin añadir un sistema de foco. La automatización usa eventos de raylib y no sustituye una prueba manual con dispositivos físicos. El tamaño mínimo sigue siendo 960×640. La carga secundaria de fuentes pasa el número real de codepoints a raylib: el wrapper de cadenas de raylib 5.5.1 cuenta bytes UTF-8 en conjuntos personalizados. No se modifican dependencias.

Código: `src/scenes/front_end.rs` y `src/scenes/front_end_ui.rs`. No se modifican minijuegos, vuelo ni recursos 3D.
