# Save Kenny — Chef, fase 3

La entrada normal conserva primero la introducción original de Save Kenny, con su implementación, recursos, audio, transiciones y condiciones originales. Solo después de finalizarla (o saltarla con sus controles originales) se presenta Chef cuando corresponde, antes de explorar. El indicador de Chef es independiente y saltar Chef no modifica la introducción original. No cambia las mecánicas de los cinco puzzles.

La exploración de misiones no dibuja, permite interactuar con, ni actualiza al Kenny ambulante 3D. Retira exclusivamente su colisión para evitar un obstáculo invisible. Tampoco muestra su marcador en el minimapa o mapa ampliado. Conserva los ocho niños, sus marcadores, Ike y todos los recursos compartidos de Kenny. Los modos técnicos de peligros anteriores conservan al actor legado; las cinemáticas y los cinco puzzles 2D, incluido su recorrido por alcantarillas, permanecen intactos.

## Presentación

`src/scenes/chef_intro.rs` contiene un teatro 2D nativo de raylib: montañas, nieve, casas, bosque, recortes con sombra, entrada de Chef, cambio breve de expresión, aparición de Kenny, énfasis cómico y fades. Utiliza las mismas fuentes y paleta de los puzzles. Conserva las proporciones de ambos retratos y alinea sus pies después de recortar márgenes transparentes.

Recursos originales reutilizados, sin modificaciones:

- `assets/minigames/south park/portraits/Chef-normal.webp`
- `assets/minigames/south park/portraits/Chef-surprised.png`
- `assets/minigames/south park/audio/chef.mp3`
- `assets/textures/minigames/South Park/KennyMcCormick.png`

Las 23 intervenciones están centralizadas en una tabla EN/ES. E, Enter o clic avanzan, con detección de pulsación y antirrebote. L usa la preferencia existente `saves/save-kenny-language.txt`. Saltar permanece visible y termina mediante un fade corto; detiene inmediatamente el saludo y las animaciones narrativas. No hay voces sintéticas ni pistas prestadas de otros personajes. El saludo inglés se reproduce una sola vez por presentación, respeta el volumen de voz configurado, `--mute` y el mute de Save Kenny. Cartman no se reproduce durante esta escena.

En pausa, **H: Ver introducción / Replay intro** vuelve a mostrarla y regresa a la pausa. La presentación no recibe referencias mutables al mundo, sesión, NPC, recompensas o tokens: su bucle modal impide que esos sistemas se actualicen. Al finalizar se restablece el cursor apropiado. No deja tareas, listeners ni streams activos.

## Partidas

`GameState.southpark_chef_intro_seen` es metadato independiente de recompensas y misiones. `serde(default)` permite cargar partidas anteriores. Una nueva partida lo inicializa en falso.

El host sincroniza ese campo con `SouthParkVisits` al entrar y guarda la partida al regresar o cerrar normalmente la ventana de South Park, incluso sin completar ninguna misión. La sesión actual recuerda inmediatamente que se completó o saltó la introducción. Un cierre forzado del proceso antes de volver al host puede impedir ese guardado, como cualquier estado todavía no guardado. Repetir no modifica el resto del progreso.

El ejecutable independiente `southpark` crea una sesión nueva al arrancar; la persistencia por partida pertenece al anfitrión Planet Hoppers, no a un indicador global en otro archivo.

Cambios de integración acotados: `southpark_dev.rs`, `southpark_entry.rs`, `solar_system.rs`, `gameplay/selection.rs`, registro de módulo y acceso al decodificador WebP existente en `encounter_ui.rs`.

## Verificación

Corrección de secuencia y exploración: `--smoke-entry --mute` aprueba reproducción natural de la intro original, saltos con espacio/Enter/clic y visita posterior. `--smoke-chef --mute` confirma `ORIGINAL_BEFORE_CHEF_OK` tras 26,7 segundos, salto independiente de Chef y devolución de controles. Se ejecutaron además `--smoke-puzzle-results`, `--smoke-encounters` y `--smoke-ike`, todos silenciados: cinco victorias y derrotas reales, ocho retratos y flujo de Ike. Registros `target/correction-*.log`. La regresión de lógica suma 85 pruebas aprobadas, incluyendo ausencia de movimiento y colisión del Kenny 3D durante 300 actualizaciones de exploración, puzzles, amenazas legadas y recompensas.

- `cargo build --bins`
- `cargo test --lib chef_intro`
- `cargo test --test chef_intro_save --test southpark_puzzles --test save_kenny_encounters --test southpark_ike --test progress_rewards --test attempt_rewards`
- `southpark.exe --smoke-chef --mute`: ambas lenguas, las 23 frases, dos expresiones, Kenny, avance E nativo, salto mediante clic nativo, repetición, audio detenido y estado del mundo intacto. Comprueba la transición posterior y restauración de controles. Capturas en `target/chef-intro-smoke/`, a 1280×800 y 640×480.
- Regresión gráfica: `--smoke-puzzles --mute` y `--smoke-ike --mute`.

`tests/chef_intro_save.rs` comprueba persistencia, compatibilidad con partidas antiguas, nueva partida y separación de recompensas. La prueba de estado de la cinemática comprueba antirrebote, finalización y salto.

Las pruebas de audio se ejecutan silenciadas; no sustituyen escucha humana. No se afirma una partida manual completa desde el menú principal. Este proyecto usa Rust/raylib nativo, no un reproductor web: no hay política de autoplay del navegador que gestionar en este destino. Una futura exportación web necesitaría adaptar el arranque de audio al gesto del usuario.

Sin dependencias nuevas, cambios de mecánicas, modificación de recursos originales, commits ni push.
