# South Park: intro de video con marco de cartón

## Fuente y preparación

Archivo real encontrado: `assets/video/south park/South_park_intro.mp4`.
El MP4 original se conserva sin cambios: 1920×1080, 30 fps, 26,7 s, video y audio
con inicio en cero. SHA-256 registrado en `assets/video/south park/intro.json`.

Raylib 5.5.1 no decodifica MP4 en esta aplicación. Se usa el mismo formato de
reproducción que ya utiliza Smiling Friends, con un reproductor y layout propios:

- `intro.sfv`: 801 fotogramas JPEG a 1280×720 y 30 fps; 35.141.735 bytes.
- `intro.ogg`: una sola pista extraída del audio del MP4; 391.160 bytes.
- `intro.json`: fuente, hash, dimensiones, frecuencia y duración preparados.

Para regenerarlos, con FFmpeg en PATH o `FFMPEG_PATH` configurado:

```powershell
python tools/video/prepare_southpark_intro.py
```

El script comprueba la existencia de la ruta real, conserva el MP4 y publica solo
archivos completos. FFmpeg se necesita únicamente para preparar los recursos;
el jugador no necesita instalarlo. No se agregan dependencias de Cargo.

## Flujo y audio

El sistema solar entra por `minigame_host` → `southpark_dev::run_visit_with_args`.
La ejecución directa utiliza ese mismo adaptador:

```powershell
cargo run --locked --bin southpark
```

En la primera visita, el video reemplaza el recorrido de cámara completo. Se
eliminan los puntos de la antigua cinemática y su título/letterbox. Durante el
video se omite el render de ciudad y HUD, y permanecen bloqueadas las
actualizaciones de jugador, Kenny y sesión. No hay otra intro detrás del video.

`AudioController` carga `assets/video/south park/intro.ogg` en lugar de la antigua
`assets/audio/music/South Park/intro.mp3`. Su reloj de reproducción selecciona el
fotograma mostrado y determina el final natural. El Ogg dura unos 26,709 s por
el relleno del codificador; se mantiene el último fotograma hasta terminar. Se
rechaza una diferencia superior a 0,2 s entre los recursos de audio y video.

Solo se decodifica y sube a la misma textura el JPEG correspondiente al tiempo
actual. No se decodifican todos los fotogramas en cada actualización, no hay
subprocesos de FFmpeg durante el juego ni texturas acumuladas. El paquete
comprimido ocupa unos 33,5 MiB en memoria mientras la intro está activa. El
lector valida cabecera, dimensiones, frecuencia, cantidad y límites de frames;
rechaza paquetes truncados o con datos sobrantes y limita archivos a 256 MiB.

Al terminar o saltar, se detiene el audio de intro, se liberan el paquete, la
textura y la fuente, se captura el cursor y comienza la música normal de la
sesión. Los estados de Save Kenny, mapa, HUD y resultados permanecen intactos.
Se conserva la política existente de visitas: la intro no vuelve a reproducirse
al reiniciar una partida; visitas posteriores usan únicamente el breve fundido
de regreso de 0,7 s en la posición FPS.

## Diseño e interacción

Video centrado con ajuste proporcional, cielo azul suave, banco de nieve de
papel, una nube, borde irregular de cartón marrón y dos cintas pequeñas. El
video es el protagonista. No se reutiliza el layout de Smiling Friends ni se
agregan imágenes generadas o recursos 3D.

`SKIP >>` es un recorte de cartón en la esquina inferior derecha, fuera del video,
con sombra y cambio de color al hover. Se puede saltar con clic, Enter, Espacio
y Esc. La ayuda inferior muestra discretamente Enter/Espacio. El cursor queda
libre durante la intro y se recaptura al entrar a la ciudad.

## Evidencia y pruebas

Capturas reales de raylib, sin retoques, disponibles localmente:

- [Video a los 3 s](../../target/southpark-video/intro-30.png).
- [Video a los 12 s](../../target/southpark-video/intro-120.png).
- [Video a los 22 s](../../target/southpark-video/intro-220.png).
- [Ventana mínima 640×480](../../target/southpark-video/intro-small.png).
- [Ventana amplia](../../target/southpark-video/intro-wide.png).
- [Ventana vertical y hover](../../target/southpark-video/intro-portrait-hover.png).
- [Gameplay después del clic](../../target/southpark-video/gameplay-case-3.png).
- [Informe de reproducción y transiciones](../../target/southpark-video/intro-validation.json).

Reproducción de la comprobación gráfica:

```powershell
cargo run --locked --bin southpark -- --smoke-entry --mute
```

El recorrido usa el reloj real del audio con volumen cero. Verifica final
natural, Espacio, Enter, clic y visita posterior; comprueba mundo congelado,
cursor recuperado, video liberado y audio de intro detenido en cada transición.
Las capturas permiten comprobar avance del video, marco artesanal, proporción,
legibilidad y que el botón no tapa el contenido. El escritorio limita la altura
útil de la ventana solicitada de 1920×1080; la captura registra el tamaño real.

También se ejecutan `cargo check --locked`, `cargo build --locked`, pruebas de
video, sesión, amenazas, FPS, fase 6b y retornos, además del recorrido nativo
`--smoke-session --mute`. Logs locales: `target/sp-tests-final.log`,
`target/sp-video-smoke.log` y `target/sp-session-regression.log`.

Resultado: `ENTRY_SMOKE_PASS` en los cinco casos y `SESSION_SMOKE_PASS` con las
cinco amenazas completadas, victoria, derrota, pausa/reanudación y reinicio.
Pasan 72 pruebas:

```powershell
cargo test --locked --test southpark_video --test southpark_phase6b --test save_kenny_session --test minigame_returns --test southpark_threats --test southpark_fps
```

## Límites

- Las barras negras y la resolución del contenido original dentro del MP4 se
  conservan: no se recorta ni se deforma la imagen para llenar el marco.
- Se usa una copia preparada a 720p; el MP4 original de 1080p permanece incluido.
  Si se reemplaza el MP4, es necesario regenerar ambos recursos.
- Sin dispositivo de audio, la reproducción usa el reloj temporal de reserva y
  permanece saltable; no se reproduce ninguna pista alternativa.
- La automatización verifica eventos de raylib y estado de las pistas. Queda
  la escucha subjetiva con altavoces/auriculares y la prueba con teclado y mouse
  físicos. No se afirma que esas comprobaciones manuales se hayan realizado.
- Las capturas y logs de `target` no se versionan y otro smoke puede sobrescribirlos.

No se modifican ciudad, mapa, modelos, mecánicas de Save Kenny ni Smiling Friends.
Trabajo en `feature/sp-intro`, sin commit, push ni cambio de rama.
