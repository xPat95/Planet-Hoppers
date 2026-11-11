# Save Kenny — rediseño, fase 1

La partida normal ofrece cinco encuentros desde el inicio. Se explora sin reloj global, se conversa con E o clic y se completa cada misión una vez, en cualquier orden. Esc cierra la conversación; L alterna English/Español y guarda la preferencia en `saves/save-kenny-language.txt`. M conserva el mapa existente.

## Implementación

- `src/minigames/southpark/encounters.rs`: ocho personajes, guiones bilingües, cinco grupos, colores y colocación sobre rutas comprobadas contra las colisiones del escenario. Los NPC no añaden obstáculos al movimiento. Las parejas comparten encuentro.
- `src/minigames/southpark/session.rs`: adaptador `begin_encounter`, progreso individual y regreso a exploración tras éxito o muerte. Conserva tres vidas, cinco rescates para victoria, reacción Stan/Kyle, señal de audio Danger/Cartman y entrega de resultados al host.
- `src/scenes/encounter_ui.rs`: retratos, sprites verticales orientados a cámara, indicadores, diálogo modal y una sola voz por intervención. Solo Tweek, Jimmy y Timmy tienen voz; los MP3 originales permanecen en inglés.
- `src/scenes/world_map.rs`: encuentros proyectados mediante `circle_marker` y `MapView::screen`, los mismos métodos usados por el jugador y los peligros. Los completados muestran un check verde.
- `src/scenes/session_ui.rs`, `southpark_dev.rs`, `southpark_entry.rs`: integración, textos bilingües del HUD y pausa, controles, reinicio y prueba gráfica.
- `src/graphics/billboard.rs`: entrada adicional desde una imagen decodificada para reutilizar el render de sprites existente.
- `Cargo.toml` / `Cargo.lock`: `image-webp` y sus dos dependencias transitivas. Necesario porque el raylib de este proyecto rechaza WebP. Se decodifican los ocho archivos originales en memoria; no se convierten ni modifican los recursos.

La arquitectura antigua se conserva para las pruebas y herramientas de desarrollo anteriores. La partida normal usa los encuentros. Hablar con Kenny orienta hacia los marcadores, sin reiniciar silenciosamente el progreso. El código y recursos de Ike no se modificaron: conserva sus reglas existentes de disponibilidad, reproducción y limpieza al entrar a un desafío.

## Adaptación de los desafíos existentes

Después del diálogo se prepara el peligro en una zona compatible ya validada. El trayecto hasta Kenny no consume tiempo ni activa el peligro. Al acercarse comienza la preparación local existente y luego el desafío original:

| Encuentro | Desafío actual |
| --- | --- |
| Tolkien | Apagar las tres luces con E |
| Tweek y Craig | Encender tres calefactores con E |
| Clyde | Activar el freno en el panel rojo |
| Jimmy y Timmy | Activar el seguro amarillo del piano |
| Wendy y Bebe | Recoger la tapa y cubrir el agujero |

Los guiones conservan las referencias solicitadas a cables, interruptores, secuencias y 30 segundos. El HUD explica las acciones que realmente funcionan en esta fase. No se añadió un reloj de 30 segundos ni los puzzles de fase 2. El punto de sustitución es `begin_encounter`: NPC, diálogo y marcadores no dependen del mecanismo de resolución, solo del resultado del desafío.

## Verificación

- `cargo build --bins`: compilación de los ejecutables.
- `cargo test --test save_kenny_encounters`: cinco encuentros accesibles, ocho personajes, guiones de ambos idiomas, exploración sin plazo, bloqueo de inicios duplicados, muerte y disponibilidad para reintentar, completado único en orden no secuencial y victoria con los desafíos originales. Las acciones del desafío se invocan mediante su API real; no es una partida humana.
- Regresión: `kenny_phase3`, `save_kenny_session`, `southpark_fps`, `southpark_ike`, `southpark_phase6b`, `southpark_threats`, `southpark_world`, `progress_rewards`, `attempt_rewards`.
- `southpark --smoke-encounters --mute`: OpenGL real, carga de ocho WebP y tres MP3, diálogos de ambos idiomas y alternancia de hablantes, mapa, E para iniciar sin saltar la primera frase, E/clic para avanzar, bloqueo de WASD y cierre con Esc. Capturas: `target/encounter-{0,3,7,9,10}.png`; registro: `target/encounters-smoke.log`. Se comprueba que nunca hay dos voces de encuentros reproduciéndose a la vez.
- `southpark --smoke-ike --mute`: interacción por raycast, reproducción, movimiento durante el video, restauración, repetición y limpieza al iniciar desafío; registro `target/ike-regression.log`.
- `southpark --smoke-session --mute`: flujo legado de victoria/derrota y audios originales; registro `target/session-regression.log`.

Las pruebas gráficas son automatizadas y silenciadas: no sustituyen la escucha humana de los MP3 ni una revisión con teclado físico de las cinco rutas completas. No se afirma haber realizado esas revisiones manuales. Las nuevas mecánicas quedan pendientes exclusivamente para fase 2.

Sin commit, push, merge ni cambio de rama.
