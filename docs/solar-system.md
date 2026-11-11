# Sistema solar (fases 1 a 6)

El Sol en el centro, los seis planetas de Blender en sus órbitas y una nave para recorrerlos. Al acercarse a un planeta aparece **Explorar planeta**, que abre su minijuego; al terminar se vuelve al mismo sistema solar, con la nave junto al planeta.

```powershell
cargo run --locked --bin solarsystem
```

Requisitos de Windows al final. El flujo normal comienza en el [menú y hangar inicial](hangar.md) con `cargo run --locked`; el comando anterior conserva el acceso de desarrollo. La entrada a la estación y el regreso al vuelo ya funcionan. El desbloqueo por progreso, los tokens y el final siguen pendientes.

## Controles

**Nave** (la vista inicial; el mouse queda capturado):

| Tecla | Acción |
| --- | --- |
| W / S (o flechas) | Acelerar / frenar y retroceder |
| Mouse, A / D, R / F | Girar la nariz (izquierda/derecha, arriba/abajo) |
| Shift | Turbo |
| Tab | Recorrer los seis planetas, la estación y ningún objetivo; la estación usa un rombo cian |
| Espacio (mantener) | Subir en el eje vertical del mundo; Shift también acelera este ascenso |
| B (mantener) | Piloto automático: orientar hacia el destino, con predicción orbital para planetas; W y Shift siguen controlando el avance |
| E, Enter o clic en el botón | Explorar el planeta cercano |
| N | Probar otra nave (la estación las elegirá en la etapa 8) |
| Esc | Soltar o volver a capturar el mouse |

**Vista general** (C cambia entre las dos): arrastrar gira la cámara, la rueda acerca hasta 3,5 veces el sistema, Tab sigue a un planeta o la estación, P pausa y Arriba/Abajo cambian la velocidad del tiempo (×0,125 a ×16). Espacio queda reservado para ascender en vuelo.

En las dos: O muestra u oculta las órbitas, F12 guarda una captura en `target/solarsystem.png` y Ctrl+Q sale.

## Estación espacial fija

La Ciudadela permanece fija en **(0, 6200, 0)**, con **1400 unidades de largo**, por encima del plano orbital. Conserva ambos LOD originales y el descarte detrás de la cámara. No se modifica el tamaño del sistema solar, sus órbitas ni los modelos aprobados.

La partida comienza fuera de su compuerta inferior y mirando hacia el espacio libre. El punto de salida sigue la posición y rotación configuradas de la estación. Tab selecciona la estación, B orienta hacia su aproximación y limita la velocidad cerca de la entrada mientras se alinea. Al acercarse a la puerta real aparece **[E] Entrar a la estación espacial**. La esfera de seguridad conserva un único corredor de acceso alineado; no se puede entrar por otra cara del blindaje.

E abre el hangar conservando la sesión, la nave, las órbitas y el estado de los minijuegos. Confirmar otra nave aplica su modelo y vuelve a despegar; Esc/Volver cancela sin cambiarla. No se vuelve al menú ni se reinicia el sistema solar. El detalle del contrato de compuerta, configuración y pruebas está en [Hangar del Presidium](station-hangar-cycle.md).

## Fase 1: estructura

- **Centro:** el Sol está fijo en el origen; las órbitas son círculos en el plano XZ.
- **Datos:** [`config/solar-system.json`](../config/solar-system.json) conserva las unidades de diseño originales: Sol, órbita, tamaño, velocidades, inclinación, ángulo inicial, modelo y minijuego. `SolarSystemConfig::load` devuelve los valores efectivos de juego, aplicando una sola vez la escala común de [`src/solarSystem/scale.rs`](../src/solarSystem/scale.rs).
- **Validación:** al cargar se revisa que estén los seis identificadores y que cada planeta quepa entero entre su vecino de adentro y el de afuera.

| Planeta | Órbita | Radio | Traslación (°/s) | Rotación (°/s) | Eje | Minijuego |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Estrella de la Muerte (Star Wars) | 1320 | 264 | 0,6351 | 5 | 5° | `starwars` |
| Smiling Friends | 1896 | 264 | 0,4330 | 6 | 12° | `smilingfriends` |
| Minecraft | 2472 | 192 | 0,3175 | 10 | 8° | `minecraft` |
| Un Show Más | 3048 | 240 | 0,2454 | 9 | 18° | en construcción |
| South Park | 3624 | 216 | 0,2021 | 8 | 15° | `southpark` |
| Calaca (Halloween) | 4200 | 216 | 0,1588 | −7 (al revés) | 10° | en construcción |

La tabla muestra valores efectivos, desde el Sol hacia el exterior. Halloween es el planeta exterior; los otros cinco conservan su orden relativo. Cada órbita está a 576 unidades de la siguiente. Las posiciones iniciales se distribuyen cada 60° alrededor del Sol, comenzando con Star Wars a 300°, para que el sistema se vea circular y no como una hilera. Después, cada planeta sigue su propia velocidad orbital.

La entrada muestra la estación alrededor de la nave, con una pausa de 0,15 segundos tras el fundido, y baja la mirada hacia el sistema solar en 1,35 segundos. Los controles se habilitan durante los últimos 0,15 segundos del movimiento de cámara; la nave arranca en el eje de la estación y se orienta hacia Star Wars. Las líneas de órbita vuelven a verse y se pueden ocultar con O; las etiquetas y flechas de objetivo siguen disponibles. El objetivo (arriba a la izquierda) es el primer planeta, en el orden del sistema, cuyo minijuego todavía no se ganó: al empezar o continuar una partida, y al volver de un minijuego (si se ganó, pasa al siguiente que falta; si se perdió o se abandonó, se queda en el mismo para reintentar). Con todos ganados, la estación. Tab lo sigue cambiando a mano (`next_objective` en `src/scenes/solar_system.rs`). Arriba a la izquierda, debajo del nombre de la nave, la misión: "MISIÓN n/6" (cuántas medallas lleva, más uno; "MISIÓN COMPLETA" con las seis) y qué hacer ahora, en una línea que cambia sola (`mission_step`): "Vuela a Hexbound · 5479" (del color del logo del planeta), "Presiona E para explorar …" al llegar, "Vuela a la estación" y, con todo ganado, "¡Galaxia salvada! Vuelve a la estación"; sin destino, "Elige un destino con Tab". Debajo, la velocidad, más chica, y una sombra suave detrás para leerlo sobre un planeta claro. La distancia debajo de cada logo (y la flecha del objetivo cuando está fuera de la pantalla) va del color de ese logo (`logo_color`): la del objetivo, entera y en negrita; las demás, un poco transparentes. El cielo tiene estrellas azules, rosadas y moradas, brillo tenue y titileo. Las estrellas fuera del campo de visión se descartan antes de proyectarlas. El HUD solar usa Advent Pro. La ayuda se abre con H o el botón AYUDA de la esquina inferior derecha; un panel animado pausa la navegación y muestra los controles de nave y vista general.

**Estilo holográfico de cabina** (`src/scenes/holo.rs`), en cian con blanco hielo de acento (`holo::ACCENT`: Cami no quiere amarillo ni naranja en la interfaz; solo queda el dorado del logo de Star Wars en su planeta y su medalla), compartido por la ayuda del sistema solar (y su botón AYUDA, del mismo alto que el del medallero: `holo::button`, con la tecla en un chip), los avisos para explorar un planeta o entrar a la estación, los controles del menú y el medallero (con su botón y el aviso de nave nueva): paneles de vidrio azul translúcido con degradé, líneas de barrido finitas y una franja de luz que baja despacio, borde cian fino con las cuatro esquinas encendidas (que laten apenas), títulos que brillan, cada tecla en un chip cian y divisores de luz que se desvanecen en las puntas. En el medallero, cada medalla ganada brilla del color de su franquicia (el de su logo, `franchise_color` en `token_collection_ui.rs`, el mismo de la distancia de su planeta), con un aro de ese color, se mece apenas como una moneda que gira, un destello le recorre el borde cada pocos segundos y titila una estrellita; los huecos sin ganar son huecos tallados con la sombra de una moneda casi invisible (no se ve de quién es) y "???"; el botón del medallero abajo a la izquierda es más grande y muestra las monedas chiquitas de las ganadas; en el medallero, arriba a la derecha, seis segmentos de progreso que se encienden del color de cada franquicia ganada. El medallero es un panel compacto (640×480 de diseño, `PANEL_W` y `PANEL_H` en `token_collection.rs`), con las medallas juntas; la escala se calcula con un tamaño más grande (`FIT_W`, `FIT_H`, `PANEL_FILL`), así el panel queda chico sin achicar las medallas.

### Proporciones y tiempos de viaje

`WORLD_SCALE = 12` es el único factor de escala. Multiplica radios planetarios, Sol y órbitas; las mallas se ajustan con las transformaciones existentes del motor. No se editan Blender, texturas ni shaders. Los cargadores de configuraciones del espacio aplican la escala; los minijuegos no la utilizan.

| Magnitud | Antes | Ahora |
| --- | --- | --- |
| Radios planetarios | 16–22 | 192–264 |
| Diámetros planetarios / largo de nave | 6,4–11 veces | 76,8–132 veces |
| Radio del Sol | 45 | 540 |
| Radios orbitales | 150–590 | 1800–7080 |
| Separación entre centros de órbitas vecinas | 80–90 | 960–1080 |
| Largo de naves / radio de colisión | 4–5 / 1,6 | 4–5 / 1,6 |
| Velocidad normal / turbo | 40 / 130 | 138,56 / 450,33 |
| Aceleración normal | 45 | 155,88 |

`travel_scale() = sqrt(WORLD_SCALE)` deriva el balance del mismo factor: velocidades lineales y aceleración se multiplican por √12; velocidades angulares orbitales se dividen por √12. Los radios orbitales ahora están más juntos, así que hay menos distancia de viaje entre planetas sin cambiar la velocidad de las naves. La rotación propia no cambia. Como referencia geométrica, recorrer 4800 unidades toma unos 11 s con turbo o 35 s en normal, sin contar aceleración, rodeos ni aproximación.

La detección y el arrastre conservan sus unidades relativas: explorar hasta 0,9 radios desde la superficie del elipsoide de colisión (aproximadamente 173–238 unidades para una esfera), arrastre a 3 radios del centro y LOD a 9 radios. Las formas de choque se recalculan al cargar los modelos escalados; la nave conserva su radio. El Sol sigue teniendo 2,05–2,81 veces el radio de un planeta. El inicio jugable se calcula desde la compuerta de la estación. `departure_position()` conserva un punto entre el Sol y la primera órbita como referencia de las pruebas de rutas planetarias; el regreso de minijuegos mantiene sus márgenes existentes.

## Fase 2: traslación y rotación

`SolarSystem::update(dt)` suma `velocidad × dt` a dos ángulos por planeta: el de la órbita y el del eje propio. Las velocidades están en grados por segundo, así que el movimiento no depende de los FPS (hay una prueba que compara 30 y 144 FPS). La escena toma el deltaTime de raylib con un tope de 0,1 s, para que una ventana congelada no haga saltar a los planetas.

## Fase 3: los seis modelos

| Planeta | Modelo de cerca | Lejos (más de 9 radios) | Triángulos de cerca |
| --- | --- | --- | ---: |
| Minecraft | `minecraft/exports/minecraft_planet.glb` | — | ~79 000 |
| South Park | `southpark/exports/southpark_planet.glb` | — | ~98 000 |
| Smiling Friends | `smilingfriends/exports/smilingfriends_planet.glb` | — | ~221 000 |
| Halloween | `halloween/exports/halloween_planet.glb` | `halloween_planet_far.glb` (solo el cráneo) | ~194 000 |
| Un Show Más | `..._near.glb` (de Norman) | `..._far.glb` | ~505 000 |
| Star Wars | `..._near.glb` (de Norman) | `..._far.glb` | ~507 000 |

- **Exportación:** [`tools/blender/solarsystem/export_solar_models.py`](../tools/blender/solarsystem/export_solar_models.py) genera los GLB que faltaban (los cuatro primeros planetas y las naves de Smiling Friends y Minecraft). Abre cada `.blend` y **nunca lo guarda**. Deja afuera `Presentation` (cámaras y luces), `LOD_Far` y el contorno `Outline`. Los tintes que glTF no soporta (pasto, hojas y agua de Minecraft) se hornean en copias de las imágenes. Las exportaciones de Norman y Pablo no se tocan.
- **Escala, pivote y orientación:** se corrigen en el juego, no en los archivos. Cada modelo se centra en su caja y se escala al radio del config (cada uno venía en su propia escala: South Park medía 22, el disco de Smiling Friends 24). `model_rotation_degrees` gira el modelo si hace falta. La versión lejana usa el mismo centro y escala que la cercana, para que no salte al cambiar.
- **Si un modelo falta** o no carga, ese planeta sigue como esfera temporal de su color y el juego avisa en la consola.
- **raylib y las mallas grandes:** raylib 5.5 pasa los índices de 32 bits a 16 y rompe las mallas de más de 65 535 vértices (dos del planeta de Star Wars). El cargador las rearma sin índices, con los índices verdaderos del GLB.

## Fase 4: iluminación y ambientación

- **Luz:** el Sol es la única luz (`graphics/shaders/space.fs`). Tiene un terminador suave entre el día y la noche, una noche azulada que no llega a negro, brillo especular leve y un borde iluminado. Se suma la emisión de cada material, así que brillan las grietas de lava de Halloween y los colores planos de Smiling Friends. Los colores de glTF se tratan como lineales y las texturas como sRGB.
- **Sol:** fotosfera emisiva blanco-amarilla con granulación animada y flujo de plasma en `sun.fs`. La emisión se comprime localmente y se convierte a sRGB para conservar detalle; el borde sigue luminoso, con tonos cálidos. El detalle se atenúa por derivadas de pantalla cuando no alcanza a resolverse, evitando ruido al verlo lejos. Un halo aditivo de cuatro capas simula el resplandor sin bloom de pantalla completa, render targets ni desenfoques. Se dibuja detrás de la geometría, de modo que planetas y naves lo ocultan. Al acercarse se reduce la intensidad y se limita el área dibujada para no lavar la navegación. La luz sobre los planetas y la escala se conservan.

  Parámetros solares centralizados junto a `SunShader` en `src/graphics/space.rs`: `SUN_EMISSION = 2.0`, `SUN_DETAIL_SCALE = 18`, `SUN_ACTIVITY_SPEED = 0.08`, `SUN_CONTRAST = 0.65`; `SUN_HALO_LAYERS` define radios de 5, 2,8, 1,65 y 1,18 veces el radio proyectado, con opacidades máximas de 8, 24, 72 y 150 sobre 255, del ámbar tenue al blanco cálido. `SUN_HALO_SCREEN_LIMIT = 1.6` limita cada capa a 1,6 veces el lado mayor de la ventana; la intensidad baja cuando el radio solar supera un tercio del lado menor. Es un glow localizado, no un pipeline HDR de bloom. El shader evalúa cuatro ruidos en lugar de los tres anteriores; no se agregan texturas ni iluminación dinámica adicional.
- **Fondo:** campo continuo azul y violeta calculado por `sky_nebula.fs` a partir de la dirección de la cámara. Las bandas suaves atraviesan la esfera celeste sin parches circulares ni saltos al girar. Sobre el campo se dibujan 9 000 estrellas procedurales azules, blancas, violetas y rosadas, con titileo y halos puntuales. Las estrellas fuera del campo de visión se descartan antes de proyectarlas. El shader se carga una vez y dibuja una sola pasada de pantalla; los materiales del Sol y planetas no cambian.
- **Órbitas:** anillos finos que se apagan con la distancia a la cámara.
- **Contorno de caricatura:** los modelos que lo traen como geometría (material `*Outline`, por ahora la nave de Smiling Friends) lo dibujan en negro liso descartando las caras de adelante, así solo asoma el borde.
- **Materiales:** se revisaron los seis planetas desde su lado iluminado. Hay capturas en `target/solarsystem/` después de `--smoke`.

## Fase 5: navegación con naves

- **Vuelo** (`src/ships/`): la nave va hacia donde apunta su nariz. Acelera hasta la velocidad pedida, frena sola poco a poco y se ladea al girar. Todo usa deltaTime.
- **Cámara:** detrás y arriba de la nave, a 3 largos de distancia y 0,9 de altura, con FOV de 60°. Acompaña su desplazamiento para evitar que el turbo la deje atrás; suaviza los giros y se mantiene fuera de las colisiones de los cuerpos. La vista general conserva FOV de 50°, arranca a 2,7 veces la extensión del sistema (incluye los radios planetarios), y compensa ventanas verticales. El plano cercano sigue en 0,5; el lejano parte de 65 664 y crece si la nave se aleja, en función de la posición de cámara y el tamaño del sistema.
- **Choques:** contra el Sol (esfera) y contra cada planeta. La forma de cada planeta es un elipsoide que sigue la caja de su modelo, con su giro e inclinación, así que el disco de Smiling Friends es plano y no una esfera. Al chocar, la nave se queda en el borde y pierde casi toda la velocidad.
- **Alcanzar planetas en movimiento:** cerca de un planeta (a menos de 3 radios) la nave lo acompaña en su órbita. El piloto automático predice la posición sobre la órbita circular y rodea el Sol cuando bloquea la ruta. Los controles se conservan: el jugador acelera y elige normal o turbo.
- **Naves:** [`config/ships.json`](../config/ships.json) lista las seis que tienen modelo (Smiling Friends, el barco del End de Minecraft, Butters, el carrito de golf y los dos N-1). El acceso directo usa Smiling Friends como activa de desarrollo. La aventura recibe la elección del hangar mediante `run_with_game`; Calaca utiliza un placeholder geométrico de 4 unidades hasta integrar su modelo. N recorre únicamente las desbloqueadas en una aventura y mantiene el catálogo completo en desarrollo. Ver [selección y propiedad](hangar.md).

## Fase 6: interacción con planetas

- **Proximidad:** el botón **Explorar planeta** aparece cuando la superficie del planeta está a menos de 0,9 radios (`interaction_distance`).
- **Ida:** con minijuego, la pantalla se funde a negro, la ventana del sistema solar se cierra y se abre la del minijuego. Sin minijuego, avisa que todavía no está listo.
- **Vuelta:** `MinigameExit::Returned` comunica `Victory`, `Defeat` o `Abandoned`. El anfitrión conserva todo su estado en memoria y reabre su ventana con la misma nave, detenida a 2,2 radios del planeta y mirando hacia afuera. Es necesario soltar E, Enter y clic antes de volver a explorar. Cerrar la ventana del minijuego conserva el cierre explícito del juego, separado del abandono confirmado.
- **Conexión:** `scenes::minigame_host::run` despacha por el identificador configurado en el planeta. South Park entra por `southpark_dev::run_visit_with_args`; solo conserva si ya se vio la intro, y cada visita crea un intento nuevo. Smiling Friends abre primero el video de `assets/video/smilingfriends/` y después muestra una pantalla temporal de construcción. Ambos adaptadores devuelven el contrato de `minigames::common::host`. Ver [resultados, pausa y regreso](minigames/host-return.md).
- **Pendiente:** el anfitrión retiene `MinigameCompletion` con planeta, minijuego y resultado para un futuro consumidor de progreso. No otorga tokens, recompensas ni desbloqueos.

## Archivos

| Archivo | Contenido |
| --- | --- |
| `src/solarSystem/mod.rs` | Órbitas, rotación, velocidad de cada planeta, forma de choque y proximidad. En Rust: `planet_hoppers::solar_system`. |
| `src/solarSystem/scale.rs` | Escala común del espacio y factor de viaje derivado. |
| `src/ships/mod.rs` | Configuración de naves y vuelo: velocidad, giro, piloto automático, arrastre orbital y choques. |
| `src/graphics/space.rs` | Carga y ajuste de los GLB, shaders del espacio y del Sol, cielo y halo. |
| `src/scenes/solar_system.rs` | La escena: vistas, cámara, interfaz, visitas a minijuegos y pruebas automáticas. |
| `config/solar-system.json`, `config/ships.json` | Planetas, modelos, minijuegos, naves y vuelo. |
| `tests/solar_system.rs`, `tests/ships.rs` | Pruebas de órbitas, FPS, choques, arrastre, alcance y proximidad. |

## Pruebas

```powershell
cargo test --locked --test solar_system --test ships
cargo run --locked --bin solarsystem -- --smoke         # recorrido con capturas en target/solarsystem/
cargo run --locked --bin solarsystem -- --smoke-visit   # prueba anterior de entrada
cargo run --locked --bin solarsystem -- --smoke-returns # cuatro visitas y los tres resultados
```

`--smoke` captura la vista general, cada planeta desde su lado iluminado, la nave, el botón Explorar, la aproximación a South Park (`target/solarsystem/scale-approach.png`) y cada nave de costado. Además genera `sun-far.png`, `sun-medium.png` y `sun-near.png` a 14, 4 y 1,8 radios solares del centro para revisar emisión, detalle y control del halo. Las pruebas de vuelo usan la misma predicción que la escena y verifican 144 rutas: los seis destinos desde el inicio y desde los otros cinco planetas, en normal y turbo, a 60 FPS y con dt de 0,1 s. Cada ruta debe entrar al alcance de exploración en menos de 180 s simulados. También se comprueban partida segura, proporciones, colisiones y proximidad. `--smoke-visit` estaciona la nave junto a South Park, explora, deja que South Park corra su prueba de entrada (la intro dura ~27 s) y captura la vuelta.

La compilación, las 106 pruebas y el recorrido gráfico se validaron con esta escala. El recorrido gráfico termina correctamente; persiste el aviso conocido de raylib sobre índices de 32 bits de Star Wars, que el cargador existente reconstruye. La prueba de visita escribe evidencias del minijuego en `docs/references/southpark/phase6b`; en entornos que deniegan esas escrituras no se puede completar ese recorrido sin cambiar sus permisos. No se modificó el minijuego para este ajuste.

No se midieron los FPS en una ventana visible: Star Wars y Un Show Más tienen ~500 000 triángulos de cerca cada uno. Si va lento, el siguiente paso es dar versiones lejanas a South Park, Smiling Friends y Minecraft.

## Compilar en Windows

Igual que South Park: `raylib-sys` necesita CMake y libclang. Para no configurarlos en cada terminal, se pueden dejar en `%USERPROFILE%\.cargo\config.toml` (solo afecta a ese usuario):

```toml
[env]
CMAKE = "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\Common7\\IDE\\CommonExtensions\\Microsoft\\CMake\\CMake\\bin\\cmake.exe"
LIBCLANG_PATH = "C:\\Program Files\\LLVM\\bin"
```

O, solo para la terminal actual, en PowerShell:

```powershell
$env:PATH = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin;$env:PATH"
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
```
