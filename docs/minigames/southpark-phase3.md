# SAVE KENNY! — fase 3

## Ejecutar

Desde la raíz del repositorio, con las herramientas de Rust/raylib de fase 2:

```powershell
cargo run --locked
```

Kenny aparece a unos 3.6 m del jugador, ligeramente a su izquierda, junto a North Avenue. Mirar un poco hacia abajo, acercarse y apuntarle: aparece **[E] Hablar con Kenny**. Camina a 1.3 m/s; el jugador puede alcanzarlo caminando. F3 oculta su indicador de distancia; también se puede desactivar con `kenny_hint: false` en `config/southpark-dev.json`.

Para verlo sin los tres objetos técnicos de fase 2:

```powershell
cargo run --locked -- --no-test-objects
```

E abre **SAVE KENNY! — ¿Podrás evitar que maten a Kenny?**. Los botones permiten aceptar o cancelar con mouse; Enter acepta y Escape cancela. Aceptar solo muestra una preparación, sin iniciar amenazas. Desde esa confirmación, Enter, E, Escape o el botón de regreso permiten continuar explorando. El diálogo pausa a Kenny, bloquea el movimiento y giro FPS y libera el cursor. Cerrarlo recaptura el cursor y continúa la ruta.

WASD, mouse, Shift, F1/F2, F12 y Ctrl+Q conservan sus funciones. Escape controla el cursor cuando no hay diálogo y lo cierra cuando está abierto. No se requiere numpad ni Blender.

## Inspección y PNG original

Se revisaron el Rust de fase 2, el FPS, la carga gráfica, las colisiones, el raycasting y el manifiesto. Se conserva Rust/raylib-rs 5.5.1 y serde/serde_json; Cargo.toml y Cargo.lock no cambian. El mundo usa metros, Y arriba y suelo en Y=0. Las calles se convierten una vez desde `(x,y,z)` Blender a `(x,z,-y)`.

La ruta solicitada `assets/textures/minigames/southpark/KennyMcCormick.png` no existía. El archivo comprobado y utilizado es exclusivamente **`assets/textures/minigames/South Park/KennyMcCormick.png`**. No se mueve, renombra, copia ni modifica. Es RGBA, 271 × 367 píxeles. Su SHA-256 original y final es:

```text
5735075c4aee87a2f93a3486a7226b7c1e5855ab9c944d8e1ab424dfd4b8e6d1
```

La lectura encontró 17,087 píxeles completamente transparentes y 566 completamente opacos. Los demás conservan su alfa original; no se fuerza opacidad ni se añade fondo. Las imágenes nuevas de documentación son capturas del juego, no sprites adicionales. No se modifica ningún recurso de Blender, el GLB, el manifiesto, el planeta esférico ni las naves.

## Billboard y animación

`SpriteBillboard` carga una sola textura y guarda su alfa para selección. La altura es 1.35 m y el ancho conserva la proporción `1.35 × 271 / 367 ≈ 0.997 m`, coherente con un personaje infantil y el ojo del jugador a 1.70 m.

`BillboardPose` construye el plano vertical con una normal calculada solo desde la diferencia horizontal entre cámara y personaje. Mirar desde arriba o abajo no tumba el sprite. Si la cámara está directamente encima, utiliza una orientación horizontal estable de reserva. El mismo plano se usa para renderizado y selección.

El balanceo ocurre dentro del plano, sin cambiar su normal: amplitud máxima de 10°, un ciclo por cada 0.9 m recorridos y rebote de hasta 2.5 cm. La fase depende de distancia real, y el peso de la animación converge exponencialmente usando dt. Al detenerse no avanza la fase; el balanceo y el rebote decaen gradualmente hasta quedar vertical. El soporte del giro mantiene la esquina inferior sobre el suelo más el rebote. La posición física siempre permanece sobre el suelo.

Se dibuja después de lo opaco, con prueba de profundidad y sin escrituras de profundidad durante el sprite. El fondo transparente no tapa otros objetos y las casas sí ocultan a Kenny. No tiene iluminación volumétrica ni sombras propias: es deliberadamente un personaje 2D dentro del mundo 3D.

## Waypoints y colisiones

`RoadNetwork` reutiliza las 15 rutas `roads` del manifiesto. Divide segmentos en pasos de hasta 4 m, inserta intersecciones y unifica puntos coincidentes. Cada conexión exige espacio para el cuerpo en sus extremos y un barrido válido con las colisiones existentes. La prueba gráfica sin utilería generó 638 nodos y 645 conexiones. Se calcula al cargar, no por cuadro.

La aparición y el acceso inicial se comprueban contra obstáculos y jugador. Kenny elige vecinos de forma determinista, evita regresar inmediatamente cuando hay alternativas y espera 0.8 s en algunas llegadas e intersecciones. Al completar un tramo elige otro. No usa navmesh, A*, interiores ni teletransportaciones.

Su caja física mide 0.56 × 1.35 × 0.56 m. El solucionador compartido evita los 395 obstáculos y los límites; solo excluye su propio collider. El jugador bloquea a Kenny y Kenny bloquea al jugador. Si un paso autónomo permanece bloqueado 1.5 s, elige un punto accesible y evita temporalmente el destino obstruido. Las rutas temporales solicitadas explícitamente se validan, pero no se sustituyen automáticamente si después se bloquean.

Se usa dt con el mismo límite de 0.1 s de fase 2. Las esperas consumen tiempo sin moverlo. El diálogo congela el recorrido y deja actualizar solamente el retorno visual al reposo.

## Raycasting y diálogo

El objetivo `southpark-kenny`, acción `talk-kenny`, está en la misma lista que los objetos de prueba. `select_target` mantiene su comportamiento para cajas. `select_target_with` permite refinar el impacto conservando el límite de distancia, la elección del objetivo más próximo y la oclusión compartidos.

Para Kenny se prueba primero la caja del plano animado, después el plano real con su inclinación y rebote, y finalmente su coordenada UV: debe tener alfa de al menos 32/255. Las esquinas transparentes no activan el diálogo. La distancia máxima continúa siendo 4.5 m, configurable como antes. Un edificio u otro sólido situado delante impide hablarle. El volumen del propio objetivo se excluye como oclusor de sí mismo; de otro modo el frente de la caja física podría ocultar el plano que contiene.

Los estados `Closed`, `Invitation` y `Preparing` permiten ampliar el flujo después. En esta fase aceptar solo confirma la preparación; no activa eventos ni altera vidas o progreso.

## Interfaces para fase 4

| Necesidad | Interfaz en Kenny |
| --- | --- |
| Consultar posición | `position()` |
| Consultar marcha y pausa | `is_walking()`, `is_paused()` |
| Pausar o continuar | `pause()`, `resume()` |
| Pedir una ruta validada | `set_temporary_route(points, world)` |
| Solicitar una pose futura de peligro | `request_danger_animation(enabled)` |
| Registrar y consumir un contacto | `notify_threat_contact(source)`, `take_threat_contact()` |
| Reposicionar en un punto comprobado | `reposition_safe(position, world)` |

La solicitud de peligro solo conserva un indicador para fase 4; todavía no añade una animación de amenaza. El contacto es una notificación pendiente, sin daño ni muerte. La reposición rechaza edificios, valores inválidos y puntos fuera del terreno, y fija los pies al suelo. Es una API explícita: el recorrido autónomo nunca la llama. La escena actualiza después el collider y el objetivo visual.

## Verificación reproducible

```powershell
cargo check --locked
cargo build --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo run --locked -- --smoke-kenny
cargo run --locked -- --smoke
```

Se aprobaron **34 pruebas**: 15 de Kenny y las 19 anteriores. Dos expectativas de conteo de la suite FPS se adaptaron porque Kenny permanece al retirar la utilería técnica. Se conservan las pruebas de carreras, paredes, suelo, límites, calles y oclusión.

Las nuevas pruebas cubren el alfa original, proporciones, billboard desde varios lados y alturas, giro local y apoyo inferior, amplitud y reposo, equivalencia a 30/60/144 actualizaciones por segundo, todas las conexiones, aparición separada, diálogo, selección de píxeles y rechazo de obstáculos o posiciones inseguras. Una simulación de cuatro minutos en dos escenas compara resultados deterministas, exige más de 100 m recorridos, comprueba cada paso contra obstáculos y limita la distancia por cuadro. Otra coloca al jugador en su camino y comprueba espera y reelección sin penetración ni saltos.

`--smoke-kenny` utiliza OpenGL real, el PNG original y una ventana oculta durante 210 cuadros a dt fijo. Automatiza E, Escape, Enter y un clic sobre Aceptar con eventos de raylib. Apunta la cámara hacia Kenny para preparar los impactos; no pretende ser una prueba física de búsqueda del personaje. Envía W y movimiento de mouse durante el diálogo y confirma que el FPS quede inmóvil, el cursor libre y Kenny detenido y vertical. Verifica cancelación con reanudación, aceptación y cierre con recaptura. La captura lateral usa otra cámara solo para esa imagen; nunca mueve al NPC para tomar capturas.

Evidencia:

- [Caminando](../references/southpark/phase3/kenny-walking.png).
- [Diálogo](../references/southpark/phase3/kenny-dialogue.png).
- [Aceptación](../references/southpark/phase3/kenny-accepted.png).
- [Otra posición de cámara](../references/southpark/phase3/kenny-side.png).
- [Informe](../references/southpark/phase3/runtime-validation.json).

La regresión `--smoke` mantiene W+Shift, mouse, E sobre la caja y Escape. Ahora guarda su evidencia en `target/southpark-smoke/` para conservar las capturas históricas de fase 2. Los logs de trabajo también quedan en `target/`, ignorado por Git.

Falta probar físicamente teclado/mouse, cambios de foco durante el diálogo, F3 mediante hardware, sesiones gráficas largas y otras plataformas. Las simulaciones prolongadas son pruebas de lógica, no sesiones manuales. Conviene revisar tamaño y marcha en la laptop del jugador. Las colisiones siguen siendo cajas conservadoras y el umbral de alfa puntual puede diferir ligeramente del borde suavizado por filtrado bilineal.

## Archivos y alcance

Nuevos: `src/graphics/billboard.rs`, `src/minigames/southpark/kenny.rs`, `src/minigames/southpark/navigation.rs`, `src/scenes/kenny_dialogue.rs`, `tests/kenny_phase3.rs`, esta guía y los cinco archivos de evidencia de fase 3.

Modificados: declaraciones de módulos en `src/graphics/mod.rs` y `src/scenes/mod.rs`; `src/minigames/common/manifest.rs` para leer calles; `collision.rs` para excluir el collider propio; `interaction.rs` para refinamiento de impactos; `src/minigames/southpark/mod.rs` para componer el NPC y sus estados; `src/scenes/southpark_dev.rs` para renderizado, diálogo y pruebas; `config/southpark-dev.json` para el indicador; dos expectativas en `tests/southpark_fps.rs`; y README/arquitectura para enlazar la fase.

No cambian los recursos originales, scripts de Blender, dependencias, naves ni otros planetas. No hay autobús, OVNI, trampas, vidas, temporizador, victoria/derrota, música, Stan/Kyle ni sistema solar. No se hace commit, push, merge, rebase ni cambio de rama.
