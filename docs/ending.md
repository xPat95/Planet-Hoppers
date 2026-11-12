# Final cinematográfico

Al terminar las seis misiones, el sistema solar espera a que se consuman todas las recompensas pendientes, incluidos el token y el aviso de nave del sexto mundo. Después de cerrar cualquier panel abierto, empieza automáticamente el final. Los controles de vuelo y las interacciones quedan bloqueados durante la secuencia.

La animación dura 36 segundos: carga solar (0–4), compresión y formación del agujero negro (4–7), absorción escalonada de los seis planetas (7–18,25), pérdida de control y caída de la nave activa (19–25), fundido a negro (25–27), las seis postales (27–36) y FIN. Enter o el botón de la pantalla final regresan al menú principal; cerrar la ventana termina la aplicación.

`scenes/ending.rs` anima transformaciones de presentación sin modificar órbitas, progreso ni vuelo. Reutiliza Sol, modelos y LOD, iluminación, cielo, bloom y recursos de las abducciones. El disco de acreción usa un shader analítico sin texturas nuevas, ray marching ni simulación física; las estelas tienen un número fijo de segmentos. Después del fundido a negro no hay otra abducción (para ganar ya se vieron las seis): un **collage tipo bento** de seis tarjetas de distintos tamaños, en una grilla de 4×3 (`bento_cell` en `ending.rs`, según la forma de cada foto: Smiling Friends, la grande de 2×2; Hexbound, la alta; South Park y Star Wars, las anchas; Regular Show y Minecraft, las chicas). Cada tarjeta tiene su imagen (`assets/images/ending/<franquicia>.png`): las que eligió Cami (los originales, en `assets/images/ending/source/`: Steve, Mordecai y Rigby, el elenco de Smiling Friends, el pueblo de South Park entero y Grogu) y, para Hexbound, el render de frente del planeta calavera (`docs/references/halloween/halloween_v01_front.png`). Minecraft se recorta hacia arriba (`photo_focus`), así no se le corta la cabeza a Steve. y es una postal: papel crema con sombra, apenas torcida, la foto con margen, su logo sobre una franja oscura al pie de la foto, y una estampilla dentada del color de la franquicia en la esquina, con un brillo suave de ese color alrededor. Caen de a una y se asientan con un rebote; con todas puestas aparece "¡Ganaste! Estás a salvo" y "Sobreviviste al agujero negro y salvaste los seis mundos". El estilo es el de la interfaz (cian con blanco, Advent Pro, títulos que brillan), y es una sola pantalla con el FIN: al terminar, debajo del collage aparecen los créditos ("PLANET HOPPERS · Gracias por jugar · Pablo / Cami / Norman") y el botón de volver al menú en el estilo holográfico, sin otra pantalla encima.

El tiempo y la finalización se guardan en `TokenPresentation.ending`, con valores por defecto para partidas anteriores. Un cierre normal conserva la secuencia; al continuar se reanuda desde el último segundo guardado. Una aventura terminada abre directamente FIN y no vuelve a reproducir el colapso. Una partida nueva crea otro estado independiente. Ctrl+X conserva su cierre inmediato global y puede perder el último segundo del final.

## Verificación

Desde WSL en la raíz del repositorio (este equipo tiene Rust para Windows accesible como `cargo.exe`):

```sh
cargo.exe check --locked
cargo.exe test --locked --no-fail-fast
cargo.exe test --locked --test ending
cargo.exe fmt --all -- --check
cargo.exe run --locked --bin solarsystem -- --smoke-ending minecraft
```

`--smoke-ending` acepta también `halloween`, `starwars`, `smilingfriends`, `regularshow` y `southpark`. Usa una partida de prueba con seis tokens y una nave diferente al origen, ejecuta todo el final en una ventana oculta y guarda siete capturas en `target/ending/`. No lee ni escribe la partida del usuario. Termina con `ENDING_SMOKE_PASS` y la identidad inicial y activa. Las capturas y los logs son locales, excluidos de Git.

Las tres pruebas de `tests/ending.rs` comprueban los seis posibles últimos mundos y todas sus recompensas, las 36 combinaciones de origen/nave, compatibilidad del guardado y contracción acelerada hasta desaparecer.

Validación realizada: `cargo check` correcto, 30 pruebas dirigidas correctas y las seis variantes gráficas con `ENDING_SMOKE_PASS`. La suite heredada dejó 319 pruebas correctas, tres fallidas y diez ignoradas. Se revisaron capturas del agujero negro, la caída de la nave y los regresos; están disponibles localmente en `target/ending/`.

La suite heredada presenta tres fallos reproducidos también en una copia aislada de `main` (`0e35656`): `exploration_encounters_all_2d_puzzles_and_victory` en `save_kenny_encounters`, y `manholes_are_fair_and_reachable_with_fixed_time_steps` / `ufo_reflexes_win_and_idle_or_wrong_clicks_lose` en `southpark_puzzles`. El formato global también tiene diferencias previas; los archivos Rust nuevos se formatearon individualmente para evitar cambios masivos ajenos al final.
