# South Park — fase 2: base jugable en Rust

## Ejecutar

Desde la raíz del repositorio:

```powershell
cargo run --locked --bin southpark
```

La primera ejecución compila las dependencias; después abre la ciudad directamente. No requiere abrir Blender, descargar modelos, usar numpad ni instalar un motor adicional. Los datos se resuelven desde la raíz usada para compilar el ejecutable. Para una distribución fuera del repositorio habrá que preparar una ruta de recursos configurable.

Se probaron Rust/Cargo 1.97.0, MSVC de Visual Studio 2022, CMake y LLVM/libclang en Windows. `raylib-sys` compila raylib nativo; en Windows necesita herramientas de C/C++, CMake y libclang para bindgen. Si no encuentra libclang, definir `LIBCLANG_PATH` apuntando a la carpeta que contiene `libclang.dll` (en esta máquina, `C:\Program Files\LLVM\bin`) antes de compilar. En Linux se requieren también las bibliotecas de desarrollo de OpenGL y del sistema de ventanas; esa plataforma no se verificó aquí. No se modifican instalaciones o variables globales.

Dependencias directas: `raylib = 5.5.1`, `serde` y `serde_json`. `Cargo.lock` fija las versiones resueltas, incluidas raylib-sys 5.5.1, serde 1.0.229 y serde_json 1.0.154. La edición de Rust es 2021. Para medir rendimiento después de una revisión manual se puede usar `cargo run --release --locked --bin southpark`; la verificación de esta entrega se realizó con el perfil de desarrollo optimizado.

## Controles y configuración

| Acción | Control |
| --- | --- |
| Avanzar / retroceder | W / S |
| Desplazarse lateralmente | A / D |
| Mirar horizontal y verticalmente | Mouse |
| Correr | Shift izquierdo o derecho |
| Interactuar con el objetivo señalado | E |
| Liberar o volver a capturar el cursor | Escape |
| Mostrar/ocultar ayuda técnica | F1 |
| Ver colisionadores | F2 |
| Guardar captura en `target/southpark.png` | F12 |
| Salir | Cerrar ventana o Ctrl+Q |

Al perder el foco se libera el cursor y se detiene el control; pulsar Escape al volver para retomarlo. El mouse se descarta brevemente al cambiar de captura para evitar saltos por recentrado. Escape no cierra la aplicación.

`config/southpark-dev.json` permite ajustar sensibilidad en radianes por píxel, velocidades en m/s, FOV, límite vertical, volumen corporal y distancia de interacción. Los valores iniciales son 4.5 m/s caminando, 8 m/s corriendo, sensibilidad 0.002, inclinación máxima ±85°, ojo de 1.70 m, cuerpo de 1.80 m y ancho de 0.70 m. El movimiento diagonal se normaliza y solo utiliza la orientación horizontal de la cámara. El tiempo se limita a 0.1 s por actualización para que una pausa o breakpoint no provoque un salto grande; por debajo de 10 FPS esa protección reduce la velocidad efectiva.

Se puede usar otra configuración sin editar la predeterminada:

```powershell
cargo run --locked --bin southpark -- --config config/southpark-dev.json
```

## Inspección y referencia del Proyecto 1

Antes de implementar se verificaron estructura, documentación y scripts de fase 1. No existían `Cargo.toml`, archivos Rust, framework ni administrador de escenas ejecutable. El manifiesto tiene 396 entradas: 395 obstáculos y un plano de suelo; no son 396 sólidos. El terreno mide 720 × 335 m. El punto inicial Blender `(-43.2, 42.9, 1.7)` se convierte a `(-43.2, 1.7, -42.9)` en raylib, mirando hacia +X sobre North Avenue. El suelo y los proxies coinciden con los límites del modelo.

Se consultaron en modo lectura el [Cargo del Proyecto 1](https://github.com/xPat95/raycasting-maze/blob/main/Cargo.toml), su [jugador](https://github.com/xPat95/raycasting-maze/blob/main/src/player.rs), [ciclo principal](https://github.com/xPat95/raycasting-maze/blob/main/src/main.rs), [caster](https://github.com/xPat95/raycasting-maze/blob/main/src/caster.rs) y [recursos](https://github.com/xPat95/raycasting-maze/blob/main/src/textures.rs). Su configuración declara raylib 6.0.0; aquí se eligió 5.5.1, disponible localmente y comprobada con el GLB real. Se retoman la lectura de deltas de mouse, controles relativos, normalización diagonal, tiempo de cuadro y separación de módulos. No se copió el renderizador por columnas, framebuffer, mapas de celdas, DDA, audio ni lógica del laberinto. Los dos proyectos permanecen independientes.

La prueba inicial abrió un contexto OpenGL y cargó el GLB completo antes de implementar el controlador. La distribución probada carga las transformaciones de nodos y los materiales correctamente. El cargador final verifica nuevamente cada primitiva y cada color contra el JSON interno del GLB; detecta una carga parcial en lugar de aceptar silenciosamente un modelo de sustitución. Las APIs se comprobaron en el código instalado de raylib-rs/raylib 5.5.1; documentación de referencia: [carga de modelos](https://docs.rs/raylib/5.5.1/raylib/core/models/index.html).

## Renderizado y recursos

Se carga directamente `assets/models/minigames/southpark/southpark_town.glb`, sin exportaciones adicionales ni modelos duplicados en disco. Las 777 mallas visuales del archivo contienen 2,734 primitivas; raylib carga cada primitiva como una malla. Conservan 76,212 triángulos y 27 materiales, más el material predeterminado de raylib. La geometría está en metros con Y arriba; los proxies del manifiesto se convierten desde Z arriba una sola vez, reordenando mínimos y máximos al invertir un eje.

`StaticModel` expande y agrupa los triángulos estáticos por material en memoria, con lotes de hasta 60,000 vértices. El resultado probado son **28 llamadas de dibujo** para la ciudad. Las tres formas de prueba y las líneas opcionales de depuración se dibujan aparte. Las mallas originales cargadas se conservan hasta cerrar la escena para mantener la propiedad de sus materiales; los buffers adicionales tienen un coste de memoria, sin duplicación de archivos. Las asignaciones nativas usan el asignador de raylib y se liberan mediante RAII antes de cerrar el contexto gráfico. El shader tiene un único propietario, compartido como handle no propietario por los materiales.

Los shaders GLSL 330 aplican luz direccional suave, ambiente, conversión de color lineal a pantalla y una neblina distante. No implementan PBR completo ni sombras proyectadas. El suelo plano se dibuja primero sin escribir profundidad para evitar que las calles, separadas apenas milímetros de la nieve, parpadeen en vistas lejanas. No se alteran vértices ni materiales del GLB. La selección de ese material de fondo se hace explícitamente desde la escena; otros recursos deben definir su propio fondo plano o desactivar esa opción.

Este cargador está orientado al recurso estático de colores planos entregado. Antes de usarlo con personajes animados, modelos con instancias o nuevas texturas, revisar la política de agrupación y el alcance de la auditoría. El controlador y los rayos no dependen de esa agrupación.

## Colisiones

Se reutilizan los 395 proxies sin editar el manifiesto: edificios completos, troncos de pino, montañas y cartel. El jugador es una caja vertical de 0.70 × 1.80 × 0.70 m. El solucionador expande los obstáculos por ese volumen y calcula el primer contacto a lo largo del desplazamiento completo. Proyecta el movimiento restante sobre la pared para deslizarse y resuelve hasta cuatro contactos por actualización. No depende de que el punto final del movimiento quede dentro de una caja, por lo que una pared delgada bloquea incluso desplazamientos muy grandes.

Cuatro barreras adicionales protegen los extremos del terreno teniendo en cuenta el ancho del jugador. El suelo mantiene los pies en Y=0; no hay salto, gravedad, escalones, rampas ni caídas. Las capas visuales de calles y estacionamiento no se apilan como colisionadores. El punto inicial se valida contra todos los obstáculos antes de abrir la escena.

Las cajas son deliberadamente conservadoras: pueden bloquear aire alrededor de tejados, porches o laderas. Las copas de árboles siguen siendo visuales; el tronco tiene su proxy. F2 permite revisar estos volúmenes. Un recorrido manual amplio sigue siendo necesario para decidir qué cajas conviene subdividir en fases posteriores.

## Raycasting e interacción

El rayo parte del ojo hacia la dirección tridimensional de mirada. Se selecciona el volumen interactivo más cercano dentro de 4.5 m y se compara su distancia con el primer obstáculo sólido. Un edificio, un tronco, una montaña u otro objeto de prueba situado delante bloquea la selección. Las pruebas incluyen casos con inclinación vertical, rayos paralelos, distancias máximas y objetivos ocultos.

Hay tres formas de colores alrededor del punto inicial: Caja, Cartel e Interruptor. Son volúmenes técnicos, no modelos definitivos de utilería. Al apuntar, se resaltan y aparece `[E] Interactuar`; E muestra una confirmación durante 2.5 s y la escribe en consola. Cada objetivo tiene identificador, bounds y acción; la escena interpreta la acción, sin reglas de Kenny dentro del sistema de rayos.

Para retirarlos sin tocar recursos:

```powershell
cargo run --locked --bin southpark -- --no-test-objects
```

También se pueden desactivar con `test_objects: false` en una configuración propia. Sus geometrías y colisiones solo existen en memoria, en `SouthParkScene::load`.

## Comprobaciones reproducibles

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo run --locked --bin southpark -- --smoke
```

Se aprobaron **19 pruebas automáticas**: importación de los 395 proxies, ejes y escala, aparición válida, retirada de utilería, movimiento equivalente a 30/60/144 FPS, diagonales, orientación, pitch, paredes delgadas, esquinas, deslizamiento, cuatro límites, suelo, rayos 3D, oclusión, distancia, selección cercana y confirmaciones. Incluyen más de 7,000 posiciones de marcha en las calles reales y barridos contra los cuatro tipos de obstáculos del manifiesto.

`--smoke` requiere la configuración predeterminada con objetos de prueba. Abre un contexto gráfico real, oculta la ventana y ejecuta 90 cuadros usando eventos de automatización de raylib. Las consultas de teclado y mouse son las mismas que en la ejecución normal. Comprueba W+Shift, giro del mouse, E y cambio del cursor con Escape. La carrera se detuvo a aproximadamente 2.50 m del origen, al contactar con la caja, sin penetración. La prueba usa dt fijo; ignora únicamente el foco real de la ventana oculta y vuelve al origen al terminar las comprobaciones para tomar las capturas. Una vista general se usa exclusivamente en el cuadro de captura. No es un controlador de vuelo disponible durante el juego.

Evidencia: [informe de ejecución](../references/southpark/phase2/runtime-validation.json), [calle](../references/southpark/phase2/street.png), [interacción](../references/southpark/phase2/interaction.png), [cursor liberado](../references/southpark/phase2/cursor-released.png) y [vista general](../references/southpark/phase2/overview.png). Se verificó OpenGL 3.3 sobre Intel UHD Graphics. El tiempo del informe incluye capturas, sincronización de lectura y una ventana oculta; **no es un benchmark de FPS** ni garantiza el rendimiento en otras máquinas.

No se probaron físicamente el teclado y mouse, sesiones largas, el enfoque al alternar aplicaciones, F1/F2/F12 mediante hardware ni otras plataformas. No existe un sistema solar ejecutable para hacer regresión o transiciones. Para comprobarlo manualmente: caminar por las avenidas, correr directamente contra casas y árboles, avanzar en diagonal contra una fachada, recorrer los bordes, apuntar a cada forma, presionar E, ocultarla detrás de una casa, y alternar Escape dos veces. Revisar que mirar hacia arriba/abajo no cambie la altura al caminar. Si la ventana no toma el foco inicialmente, hacer clic en ella y pulsar Escape para capturar el cursor.

## Archivos y próxima fase

Nuevos: `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, `src/bin/southpark.rs`, declaraciones de módulos bajo `src/graphics/`, `src/minigames/` y `src/scenes/`, los cuatro módulos de `src/minigames/common/`, `src/minigames/southpark/mod.rs`, `src/scenes/southpark_dev.rs`, `src/graphics/resources.rs`, `src/graphics/lighting/mod.rs`, los shaders `winter.vs` / `winter.fs`, `config/southpark-dev.json`, `tests/southpark_fps.rs`, esta guía y los cinco archivos de evidencia de fase 2.

Modificados: `.gitignore` para excluir `target/`; `README.md`, `docs/architecture.md` y el README de `src/minigames/southpark/` para enlazar la implementación actual. No se modifican los recursos de fase 1, el planeta esférico, las naves, otros planetas ni `KennyMcCormick.png`. No se cambia de rama, ni se hace commit o push.

Para fase 3: decidir la representación visual de Kenny, definir sus puntos de aparición y rutas sobre calles transitables, mover su volumen con delta de tiempo y comprobación de obstáculos, mantener actualizado su objetivo de interacción y separar su estado/comportamiento de los controles compartidos. El manifiesto ya conserva las polilíneas viales, pero todavía no hay navegación autónoma ni NPC. Kenny, amenazas, música, diálogos, vidas, temporizador, victoria/derrota y viajes entre planetas no forman parte de esta fase.
