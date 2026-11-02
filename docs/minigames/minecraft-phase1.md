# DIAMOND RUSH — escenario Overworld, fase 1

## Inspección y alcance

Se trabajó en `feature/minecraft-minigame`, con autorización explícita del usuario tras comprobar que el nombre difiere de `minecraft-minigame`. No se cambió de rama, no se realizaron commits, push, merges ni rebases.

Recursos inspeccionados:

- Planeta original: `assets/models/planets/minecraft/minecraft_planet_textured_v02.blend`, exportado a `assets/models/planets/minecraft/exports/minecraft_planet.glb`.
- Nave: `assets/models/ships/minecraft/end_ship.blend` y `exports/end_ship.glb`. Su estética The End no se trasladó a la aldea.
- Texturas históricas del planeta: `assets/textures/planets/minecraft/`.
- Las texturas preparadas para este minijuego: `assets/textures/minigames/Minecraft/`, respetando mayúsculas y nombres originales.
- South Park fase 1: `tools/blender/southpark/build_explorable_town.py`, `validate_explorable_town.py`, `docs/minigames/southpark-phase1.md` y sus tres recursos de escena en `assets/models/minigames/southpark/`.
- Motor actual: Rust, raylib-rs **5.5.1**, carga GLB en `src/graphics/resources.rs`, manifiesto y controlador compartidos en `src/minigames/common/`.

Blender MCP no estaba conectado. Se utilizó Blender **5.4.0 Alpha**, hash `16a7f36b710d`, compilado el 8 de octubre de 2026, extraído del ZIP local existente a una carpeta temporal. No se alteró una sesión interactiva. El exportador `io_scene_gltf2` está incluido; esta distribución omite un módulo de animación y necesita el mismo stub local para exportación estática que South Park. El stub solo existe en memoria del proceso; no se modificó la instalación y las animaciones se desactivan.

## Construcción

Mapa de **64 × 60 bloques**, X entre −32 y 32, Y entre −26 y 34. Terreno sólido de bloques con caras internas eliminadas y agrupado en secciones de 8 × 8. El núcleo mantiene el suelo Z=0 para reutilizar en el futuro el controlador existente, cuyo mundo de colisión solo admite un plano horizontal. Los bordes forman bancales y colinas sólidas; el extremo norte contiene el macizo de la mina. No es un plano vacío: incluye relieve, estratos de tierra/piedra, árboles, vegetación, caminos, cultivos y centro de aldea.

Cinco edificios cerrados: **dos pequeños** de 5 × 5, **dos medianos** de 7 × 6 y **uno principal** de 9 × 7. Se construyeron paredes cúbicas, cimientos de cobblestone, esquinas de tronco, tablones, ventanas y puertas de dos bloques. Los techos a dos aguas utilizan escalones de medio bloque. La escala de las puertas es visual; los edificios completos se consideran sólidos y sus interiores no son transitables.

**16 robles** con troncos de cuatro bloques y copas escalonadas; solo los troncos bloquean el recorrido. El pozo techado ocupa el centro, rodeado por dos pasos laterales. Caminos de tres bloques conectan la entrada sur, los cinco accesos, el centro, el huerto y la mina. El huerto tiene bancales húmedos, canal central de agua, borde de troncos, cultivos geométricos y cuatro fardos de heno. Flores y tallos sencillos completan los espacios abiertos.

La mina tiene boca oscura de **5 bloques de ancho × 4 de alto**, soportes de roble, cartel de malla `DIAMOND RUSH`, revestimiento de piedra/andesita, grava, fragmentos decorativos de granito/diorita, mesa de crafteo, horno y raíles estilizados. Solo hay una transición corta de seis bloques, con pared final estática. La colina cubre el túnel; no hay red subterránea, minerales interactivos ni geometría destructible.

## Texturas y materiales

Se inspeccionaron **38 PNG**: 36 de 16 × 16, `water_still.png` de 16 × 512 y `lava_still.png` de 16 × 320. Todas las texturas mencionadas por el usuario estaban disponibles. No se descargaron, renombraron ni sobrescribieron originales.

El generador prepara 28 materiales texturados de terreno, roble, aldea y roca, más cinco colores sencillos para cultivos, flores, metal y letras. Blender descarta los materiales que no tienen caras visibles al guardar/exportar; el informe registra el número realmente utilizado. Los minerales, nieve, arena y lava se reservan para fases posteriores y no se colocan en esta aldea.

- Césped: `grass_block_top` arriba, `grass_block_side` en lados, `dirt` abajo.
- Camino: `dirt_path_top` arriba y `dirt_path_side` en caras laterales expuestas; `dirt` abajo. En el suelo llano, los lados están ocultos por bloques adyacentes y se eliminan.
- Troncos: `oak_log` en lados, `oak_log_top` en extremos. Heno: variantes lateral y superior.
- Mesa y horno: frente orientado hacia −Y y variantes propias de lados/techo; abajo se utiliza tablón para la mesa y `furnace_top` para el horno porque no existe una textura inferior específica en el conjunto.
- Puertas: mitad inferior y superior separadas. Hojas, vidrio y puertas conservan alfa recortado; GLB usa MASK, umbral 0.5 y doble cara. No se simula refracción del vidrio.
- Agua: `water_static_frame_00.png` cuadrada, estática y opaca. No se estira la tira de animación ni se usa lava.

Las UV repiten una imagen cuadrada por bloque y usan caras planas. `Closest` en Blender y filtros glTF `9728`/NEAREST conservan los píxeles. Las superficies fraccionarias muestran solo la fracción correspondiente de la imagen. Los tintes verdes de césped/hojas y azul del agua se aplican a copias en memoria y se empaquetan: el GLB no depende de nodos Multiply incompatibles. Las imágenes originales conservan sus hashes. El texto y los cultivos son geometría sencilla porque no se proporcionaron atlas para estos elementos.

## Coordenadas y contrato para Rust

Un bloque = una unidad Blender = un metro. Blender: +Z arriba, norte +Y; glTF/raylib: +Y arriba, norte −Z. Conversión `(x,y,z) → (x,z,−y)`. Al convertir una caja hay que recalcular mínimos y máximos tras cambiar ejes y signo. El origen está en el suelo del centro del mapa. Posición ocular inicial `(0,−21,1.7)`, mirada +Y, jugador de referencia de 1.8 m y radio 0.35 m.

`minecraft_village.scene.json` sigue el esquema 1 de South Park: `id`, `visual_asset`, `coordinates`, `spawn`, `roads` y `colliders`. Añade categorías, referencias y procedencia de texturas. Se preparan:

- Un plano `walkable` en Z=0 y límites rectangulares.
- AABB de edificios cerrados, troncos, colinas/bordes, pozo, heno, huerto, decoraciones mineras, postes, paredes, techo y fondo de transición.
- Proxies ocultos `COL_*` en el `.blend`; nunca se exportan como geometría visual.
- Marcadores reservados para interacción frente a la mina, transición al desafío y regreso al sistema solar. Son referencias sin eventos ni scripts de juego.

Los colisionadores del techo empiezan por encima del jugador, y la boca no tiene una caja envolvente que la cierre. El huerto se considera decoración sólida para evitar atravesar agua/cultivos. Las colinas son límites sólidos, no superficies para escalar en el controlador actual.

La entrega validada contiene **98 mallas visuales**, **39,176 triángulos**, **32 materiales** y **27 imágenes integradas** en el GLB. Hay **83 AABB** más el plano de suelo: 5 edificios, 16 troncos, 46 cajas de relieve agrupadas, 10 decoraciones y 6 componentes de mina (postes, paredes, techo y fondo). Los 2,136 puntos muestreados de caminos están libres y los cinco edificios son visibles en la cámara general.

En fase 2 habrá que conectar el manifiesto a una escena Rust y revisar el material/shader de Minecraft: imponer filtrado POINT/NEAREST, conservar tintes y realizar el descarte alfa. El shader compartido de South Park no implementa explícitamente ese descarte, y su cargador está orientado a materiales de color plano. La compatibilidad entregada es GLB 2.0, el formato que ya carga raylib; no se afirma que el adaptador de South Park pueda mostrarlo sin ajustes. No se modificó Rust para esta entrega.

## Abrir, capturar y regenerar

Abrir `assets/models/minigames/minecraft/minecraft_village.blend` en Blender. Todas las imágenes están empaquetadas. Numpad 0 muestra la cámara guardada a nivel del jugador. En el selector de cámaras están `DR_Camera_overview`, `player`, `mine`, `farm` y `center` (todas con prefijo `DR_Camera_`). Para mirar el entorno se puede usar F3 → Walk Navigation; es una herramienta de Blender y no ejecuta las colisiones ni el FPS de Rust.

[Capturas e informe](../references/minecraft/phase1/README.md): vista general, nivel del jugador, mina, huerto y centro. Los PNG se abren directamente en cualquier visor.

Desde la raíz, reemplazando `blender` por la ruta al ejecutable local:

```powershell
blender --background --factory-startup --python-exit-code 1 --python tools/blender/minecraft/build_explorable_village.py
blender --background --factory-startup --python-exit-code 1 --python tools/blender/minecraft/validate_explorable_village.py
```

El generador es determinista y sustituye exclusivamente los derivados de esta fase. Guardar antes una copia si se hicieron ediciones manuales del escenario. El registro `protected_sources.json` conserva los hashes previos de esta entrega; no se regenera automáticamente para encubrir cambios en originales.

El validador reabre el `.blend`, comprueba materiales, imágenes empaquetadas, interpolación, UV, asignaciones por cara, escala, acceso y visibilidad de las cinco casas. Muestrea tres bandas de cada camino con el volumen del jugador, verifica conectividad mediante flood fill independiente, contrasta suelo visual mediante raycasts, reimporta el GLB y compara mallas, triángulos, límites, orientación y UV. También revisa imágenes integradas, samplers, máscaras alfa y conservación SHA-256 de 170 archivos protegidos. El resultado concreto está en `validation.json`; no se presenta como una prueba de gameplay.

## Limitaciones y fase 2

Arquitectura estilizada inspirada en Minecraft, sin reproducir estructuras oficiales bloque a bloque. Casas cerradas, vegetación/cultivos sencillos, agua estática, sin animación de puertas ni interiores amueblados. Los colisionadores son conservadores; se deben probar con el controlador real. No hay validación de renderizado dentro de raylib en esta fase.

Los raíles bajos son decoración visual de hasta 0.1 m sobre el plano de marcha y no agregan obstáculos. La comprobación de suelo permite ese detalle únicamente en el destino de transición de la mina.

Pendiente: escena FPS de Rust usando los componentes comunes, iluminación/materiales apropiados para Minecraft, carga de colisiones y comprobación jugable de escala/accesos. Después corresponderán la activación por E/raycasting, escena minera, diamantes y retorno, según el alcance que se apruebe para las siguientes fases. No se implementaron temporizador, salud, inventario, minería, peligros, victoria/derrota, integración solar ni cambios de nave o South Park.
