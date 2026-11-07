# Regular Show: El Martillo

El planeta **Un Show Más** abre El Martillo con **E/Enter** o el botón **Explorar planeta**. `config/solar-system.json` asocia el planeta `regularshow` al adaptador del mismo nombre. El código está en `src/minigames/regularshow/` y todos sus recursos en `assets/minigames/regularshow/`. Planet-Hoppers no necesita la carpeta ProyectoFinal ni lanza ejecutables externos.

## Partida y regreso

La portada es una recreativa con Mordecai, Rigby, el Martillo, selector individual/cooperativo y una pantalla CRT. Enter o clic en Empezar inicia la partida; F2 cambia el número de jugadores antes de comenzar.

Se conservan El claro, El dojo del Martillo y Fuera del juego. P1 usa WASD, J/K para atacar, F para bloquear, E para agarrar, L para esquivar y Espacio para saltar. P2 usa flechas, U/I, Y, P, O y Shift derecho. Esc pausa; Enter/Esc continúa y R reintenta desde pausa o derrota.

Retroceso desde partida o pausa abre una confirmación: Enter abandona y Esc cancela. Desde la portada devuelve Abandoned; desde derrota devuelve Defeat; desde victoria devuelve Victory. Enter también vuelve desde victoria; R inicia otra partida. Cerrar la ventana conserva la salida general de la aplicación.

El anfitrión mantiene la nave seleccionada y las órbitas durante la visita. Al regresar deja la nave detenida junto al planeta, fuera de colisiones y del alcance de interacción, con su cámara de seguimiento. Hay que soltar la interacción antes de volver a entrar.

`AttemptReporter` comunica la victoria al terminar realmente el nivel 3, antes de que reiniciar o regresar pueda perderla. Superar solo el bosque o el dojo no concede recompensa. La primera victoria entrega el token de Regular Show y desbloquea su nave; repetir, perder o abandonar después no duplica ni borra la recompensa y no cambia la nave activa.

## Música y escenas

1. Menú y niveles 1–2: `audio/musica_fondo_martillo.mp3`, continua y en bucle.
2. Al continuar desde el dojo: `video/Video_martillo_final.mp4` (27,7 s).
3. Nivel 3: `audio/musica_fondo_n3_martillo.mp3` en bucle.
4. Victoria: `video/celebration_fn_martillo.mp4` (29,1 s), seguido de la pantalla de resultado.

Los videos conservan su audio y detienen la música del combate. Enter omite; P/Esc pausa/reanuda, y Enter reanuda una escena pausada. Retroceso confirma el regreso. Reintentar el nivel 3 no repite su transición. `--mute` silencia también las escenas.

Las copias indexadas `transicion.sfv`, `celebracion.sfv` y sus audios Ogg se reproducen dentro de raylib sin FFmpeg instalado. Se conservan los originales y sus metadatos de conversión. Si cambian, `python tools/regularshow/prepare_videos.py` regenera las copias usando FFmpeg en PATH o `FFMPEG_PATH`.

## Verificación

```powershell
cargo test --locked --offline
cargo run --release -- --smoke-regularshow --ship minecraft
cargo run --release --bin regularshow -- --check-menus
cargo run --release --bin starwars -- --check-menus
cargo test --release --lib --locked --offline minigames::regularshow -- --ignored --test-threads=1
```

La prueba del anfitrión entra mediante eventos de E, completa los tres niveles y ambas escenas con el controlador de combate normal, regresa con Victory, vuelve a entrar y pierde sin atacar, y finalmente verifica pausa, cancelación, continuación y abandono confirmado. Comprueba nave y órbitas conservadas, regreso seguro, ausencia de reentrada por una tecla mantenida y conservación de la recompensa. Terminó con `REGULARSHOW_HOST_PASS` y `FRONT_END_SMOKE_PASS`. La suite del proyecto principal completó 272 pruebas sin fallos; las 26 pruebas del prototipo independiente de Star Wars y las 17 de El Martillo también pasaron.

Las capturas están en `target/regularshow/` y `target/solarsystem/`; el registro de la integración está en `target/regularshow-host-smoke.log`. Las pruebas de audio requieren dispositivos gráficos/de audio y se ejecutan por separado. La automatización verifica controles del motor; no sustituye una partida manual con teclado físico.
