# DIAMOND RUSH — exploración FPS, fase 2

## Ejecutar

Desde la raíz del repositorio:

```powershell
cargo run --bin minecraft
cargo run --bin southpark
```

`cargo run` conserva el menú principal y su flujo actual. Minecraft es un binario de desarrollo independiente descubierto automáticamente por Cargo bajo `src/bin/minecraft.rs`. No se modificaron Cargo.toml, Cargo.lock ni el sistema solar. Se trabajó en `feature/minecraft-minigame`, sin commits, push ni cambios de rama.

WASD mueve al jugador, el mouse controla la mirada, Shift corre y E interactúa. Escape libera el cursor en exploración; otro Escape o un clic recupera la captura. En un menú, Escape lo cierra y restaura el FPS. Cerrar la ventana termina el ejecutable. F12 guarda `manual.png` en `docs/references/minecraft/phase2/`; `--capture-dir DIR` cambia la carpeta. `--config FILE` permite otra configuración de desarrollo.

## Inspección y reutilización

El proyecto usa Rust 2021, raylib-rs 5.5.1, serde y serde_json. El punto de entrada predeterminado `src/main.rs` llama a `scenes/front_end.rs`; South Park tiene su binario y host en `scenes/southpark_dev.rs`. Los archivos reales de fase 1 están en `assets/models/minigames/minecraft/`: `minecraft_village.blend`, `minecraft_village.glb`, `minecraft_village.scene.json` y README.

No se copiaron los controladores de South Park. Minecraft utiliza directamente:

- `common/fps.rs`: `FpsController`, `FpsConfig`, `PlayerInput` y `CursorCapture`.
- `common/collision.rs`: `CollisionWorld`, cajas AABB, barrido continuo y deslizamiento.
- `common/interaction.rs`: `select_target`, distancia máxima, selección más cercana y oclusión por sólidos.
- `common/manifest.rs`: lectura del esquema 1 y transformación de coordenadas.
- `graphics/resources.rs`: carga glTF/GLB de raylib y agrupación de triángulos por material.

La presentación y estados de South Park están vinculados a Kenny, amenazas, sesión, audio, mapas e intro. No se migraron. El audio existente utiliza recursos/configuración de South Park; Minecraft no carga esas pistas ni añade audio en esta fase. El host y menú nuevos son pequeños y específicos de la aldea. No se alteraron los módulos compartidos de FPS, cursor, raycasting o colisión.

## Carga y renderizado

Se carga directamente el GLB original de fase 1, sin modificar el `.blend`, las texturas ni el manifiesto. Un bloque equivale a un metro. Los datos de Blender Z arriba pasan una sola vez a raylib Y arriba mediante `(x,y,z) → (x,z,−y)`. No hay rotación o escala adicional. El ojo inicial queda en `(0,1.7,21)`, mirando hacia el pozo, la aldea y la mina; los pies permanecen a Y=0. El camino de acceso se recorre en pocos segundos al correr.

`StaticModel::load_overworld` reutiliza el cargador y sus buffers. Opta por iluminación diurna, conversión correcta entre texturas sRGB y luz lineal, filtrado POINT/Nearest y repetición por UV. El shader `overworld.fs` descarta alfa inferior al umbral MASK de cada material y respeta `doubleSided`, conservando el vidrio y las hojas recortados. No hay transparencia mezclada ni refracción del vidrio. Los tintes ya empaquetados de fase 1 se conservan.

El método preexistente `StaticModel::load` mantiene el shader y comportamiento de South Park. La alternativa nueva no afecta a sus llamadas. Las imágenes originales no se modifican ni se vuelven a exportar. La auditoría de ejecución verificó **383 primitivas GLB, 39,176 triángulos, 32 materiales, 27 texturas cargadas**, cuatro materiales MASK y **32 lotes de dibujo**. Límites raylib: mínimo `(-32,-2,-34)`, máximo `(32,9,26)`, coincidentes con la conversión de fase 1. El GLB exporta sus 32 materiales como doble cara; se respeta ese dato.

## Movimiento y colisiones

Velocidad normal 4.5 m/s, carrera 8 m/s, sensibilidad 0.002 rad/píxel, giro vertical limitado a ±85°, campo de visión 65°, radio 0.35 m, cuerpo 1.8 m y ojo 1.7 m. Son los mismos valores actuales de South Park. El movimiento usa delta de tiempo, normaliza diagonales y limita saltos de tiempo grandes; el mouse usa desplazamientos de píxeles sin multiplicarlos por delta.

Se cargan las **83 AABB** y el plano de suelo de fase 1. `CollisionWorld` añade cuatro barreras para los bordes. Edificios, troncos, colinas, pozo, huerto, heno, soportes y paredes de mina bloquean; caminos, plazas y espacios abiertos permiten circulación. El barrido continuo evita atravesar paredes incluso con desplazamientos grandes y proyecta el movimiento restante sobre la pared para deslizarse.

Los caminos están a altura cero. El relieve escalonado pertenece a límites sólidos; no se permite subir colinas o caminar por encima de obstáculos. Los raíles de transición son decoración baja sobre el plano de marcha. No se agregaron salto, gravedad, parkour o colisiones triangulares. Las casas permanecen cerradas como en fase 1.

## Raycasting y menú

El rayo parte del centro de la cámara, sigue su dirección y alcanza como máximo **4.5 m**. Se reutiliza la comprobación común de obstáculos; una pared impide seleccionar la entrada aunque el objetivo esté en la dirección del rayo. La entrada obtiene su posición del marcador `Mine_interaction_reserved`, con un volumen de selección dentro de la abertura y separado de los colisionadores. No se añade una pared invisible al acceso.

Al apuntar aparece **[E] Explorar mina**. E utiliza el flanco `is_key_pressed`, por lo que mantener la tecla no vuelve a ejecutar la acción. El menú muestra el título, la descripción solicitada y los botones **Iniciar desafío** / **Cancelar**. Enter también inicia y Escape cancela.

- `Exploring`: FPS y selección activos con cursor capturado y ventana enfocada.
- `MineMenu`: cursor libre, movimiento y mirada congelados.
- `ChallengePending`: confirmación «El desafío está pendiente de implementación», sin temporizador, carga de escena o recolección. Volver, Cerrar o Escape retoman la aldea.

El cambio de estado sincroniza el cursor nativo y descarta los primeros deltas de mouse después de capturarlo. Al perder el foco se detiene el movimiento y se libera el cursor; los menús no aceptan acciones sin foco.

La mesa de crafteo existente, junto a la mina, también se puede inspeccionar con E. Usa su collider original como objetivo y muestra una confirmación breve en pantalla, sin inventario ni crafting. El selector ignora únicamente el collider del propio objeto al comprobar oclusión.

`ChallengeRequest` conserva la pose de retorno —pies, yaw y pitch— y la referencia de la entrada en coordenadas del motor. Fase 3 podrá consumirla al disponer de una escena minera real. En esta entrega se expone mientras está la confirmación pendiente y se descarta al regresar; no hay transición automática.

## Verificación

```powershell
cargo check --offline --locked
cargo build --offline --locked
cargo test --offline --locked
cargo run --offline --locked --bin minecraft -- --smoke
```

Los flags offline/locked usan las dependencias locales fijadas, sin cambiar el lockfile. Las ocho pruebas nuevas en `tests/minecraft_phase2.rs` comprueban spawn, todas las bandas de los caminos con barridos reales, bloqueo de las cinco casas y tronco, límites, independencia de FPS, mouse, alcance/dirección/oclusión, pausa modal, retorno y mesa. Se ejecutó también la suite existente: **124 pruebas aprobadas**, incluidas 116 preexistentes.

La prueba `--smoke` abre un contexto OpenGL real en ventana oculta y ejecuta **136 cuadros** mediante eventos de raylib, usando el mismo bucle de input, render y UI que la ejecución normal. Prueba W+Shift, mouse, E, clics de cancelar/iniciar, Escape, captura/liberación nativa del cursor, congelación modal y colisiones con una casa y un roble reales. Cambia de posición entre casos como fixture de prueba; no constituye un recorrido autónomo completo del mapa. Solo omite el requisito de foco de la ventana oculta y utiliza delta fijo 1/60 s.

[Capturas e informes de ejecución](../references/minecraft/phase2/README.md). Son imágenes de Rust/raylib, no renders de Blender. La prueba visual y la inspección de esas imágenes verificaron escala, materiales, UV, texturas pixeladas, vidrio y hojas recortadas. No hubo prueba manual prolongada con teclado/mouse físicos.

Se ejecutaron además los modos gráficos existentes de South Park: `--smoke`, `--smoke-kenny`, `--smoke-threats`, `--smoke-world --mute` y `--smoke-entry --mute`. Cubren FPS/cursor, Kenny, las cinco amenazas, partidas de victoria/derrota, vidas/salvadas, mapa ampliado y su pausa, recuperación de controles e intro. El audio se cargó sin advertencias; los modos extendidos usaron silencio y no sustituyen una revisión auditiva. Sus resultados se resumen en `southpark-regression.json`.

La comparación SHA-256 de **210 archivos protegidos** confirma que no cambiaron modelos, texturas, audio ni fuentes específicos de South Park. Se incluyen el planeta, la nave y el escenario Minecraft de fase 1. El informe `protected-resources.json` registra los hashes iniciales y cualquier diferencia. El estado de Git conserva únicamente los cambios de código/documentación de esta fase.

## Archivos y límites

Nuevos: `src/bin/minecraft.rs`, `src/minigames/minecraft/mod.rs`, `src/scenes/minecraft_dev.rs`, `src/scenes/minecraft_smoke.rs`, `src/graphics/shaders/overworld.fs`, `config/minecraft-dev.json`, `tests/minecraft_phase2.rs`, esta guía y capturas/informes bajo `docs/references/minecraft/phase2/`.

Modificados: `src/graphics/resources.rs` para la opción de materiales pixelados; `src/minigames/mod.rs` y `src/scenes/mod.rs` para registrar los módulos. No se modificaron archivos de configuración o implementación de South Park.

La iluminación diurna es direccional/ambiental, sin sombras dinámicas ni equivalencia exacta con Cycles. El vidrio es recortado, el agua estática y opaca, los edificios no tienen interiores transitables y las colinas no son escalables. Sigue pendiente una partida manual con hardware físico y revisión auditiva de South Park. La cobertura automatizada y gráfica no implica haber probado una mina jugable.

Fase 3 queda preparada mediante el objetivo de entrada, estados del menú y pose de retorno. Falta construir/cargar la mina, definir su spawn real y conectar el desafío cuando se autorice. No se implementaron túneles adicionales, minerales, minería, pico, diamantes, temporizador, salud, lava, victoria/derrota de DIAMOND RUSH ni integración con el sistema solar.
