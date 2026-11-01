# Sistema solar (fases 1 a 6)

El Sol en el centro, los seis planetas de Blender en sus órbitas y una nave para recorrerlos. Al acercarse a un planeta aparece **Explorar planeta**, que abre su minijuego; al terminar se vuelve al mismo sistema solar, con la nave junto al planeta.

```powershell
cargo run --locked --bin solarsystem
```

Requisitos de Windows al final. La estación espacial, la selección y el desbloqueo de naves, los tokens y el final siguen pendientes (etapas 8 a 10).

## Controles

**Nave** (la vista inicial; el mouse queda capturado):

| Tecla | Acción |
| --- | --- |
| W / S (o flechas) | Acelerar / frenar y retroceder |
| Mouse, A / D, R / F | Girar la nariz (izquierda/derecha, arriba/abajo) |
| Shift | Turbo |
| Tab | Elegir el planeta objetivo (marcado en amarillo, con flecha si está fuera de la pantalla) |
| Espacio (mantener) | Piloto automático: apunta adonde va a estar el objetivo |
| E, Enter o clic en el botón | Explorar el planeta cercano |
| N | Probar otra nave (la estación las elegirá en la etapa 8) |
| Esc | Soltar o volver a capturar el mouse |

**Vista general** (C cambia entre las dos): arrastrar gira la cámara, la rueda acerca hasta 3,5 veces el sistema, Tab sigue a un planeta, Espacio pausa y Arriba/Abajo cambian la velocidad del tiempo (×0,125 a ×16).

En las dos: O muestra u oculta las órbitas, F12 guarda una captura en `target/solarsystem.png` y Ctrl+Q sale.

## Fase 1: estructura

- **Centro:** el Sol está fijo en el origen; las órbitas son círculos en el plano XZ.
- **Datos:** [`config/solar-system.json`](../config/solar-system.json) define el Sol y, por planeta, la órbita, el tamaño, las velocidades, la inclinación del eje, el ángulo inicial, el modelo y el minijuego.
- **Validación:** al cargar se revisa que estén los seis identificadores y que cada planeta quepa entero entre su vecino de adentro y el de afuera.

| Planeta | Órbita | Radio | Traslación (°/s) | Rotación (°/s) | Eje | Minijuego |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Minecraft | 150 | 16 | 2,2 | 10 | 8° | en construcción |
| South Park | 230 | 18 | 1,5 | 8 | 15° | `southpark` |
| Smiling Friends | 320 | 22 | 1,1 | 6 | 12° | en construcción |
| Halloween | 410 | 18 | 0,85 | −7 (al revés) | 10° | en construcción |
| Un Show Más | 500 | 20 | 0,7 | 9 | 18° | en construcción |
| Star Wars | 590 | 22 | 0,55 | 5 | 5° | en construcción |

El Sol mide 45 de radio. Las naves miden de 4 a 5 unidades de largo, así que cada planeta es unas diez veces la nave. Los de afuera giran más despacio alrededor del Sol, y ningún planeta pasa de ~6 unidades por segundo en su órbita: la nave va a 40 (130 con turbo).

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
- **Sol:** superficie animada con granos que hierven y borde anaranjado (`sun.fs`), más un halo de tres capas detrás de todo.
- **Fondo:** 2 600 estrellas "en el infinito" (no se acercan al moverse) con tamaños, brillos y colores distintos y un titileo suave. Son más densas en una franja de galaxia inclinada y hay nubes de gas casi invisibles.
- **Órbitas:** anillos finos que se apagan con la distancia a la cámara.
- **Materiales:** se revisaron los seis planetas desde su lado iluminado. Hay capturas en `target/solarsystem/` después de `--smoke`.

## Fase 5: navegación con naves

- **Vuelo** (`src/ships/`): la nave va hacia donde apunta su nariz. Acelera hasta la velocidad pedida, frena sola poco a poco y se ladea al girar. Todo usa deltaTime.
- **Cámara:** detrás y arriba de la nave, siguiéndola con suavidad; tiembla un poco al chocar.
- **Choques:** contra el Sol (esfera) y contra cada planeta. La forma de cada planeta es un elipsoide que sigue la caja de su modelo, con su giro e inclinación, así que el disco de Smiling Friends es plano y no una esfera. Al chocar, la nave se queda en el borde y pierde casi toda la velocidad.
- **Alcanzar planetas en movimiento:** cerca de un planeta (a menos de 3 radios) la nave lo acompaña en su órbita, así que quedarse quieta junto a él es fácil. El objetivo tiene marcador y flecha, y el piloto automático apunta adonde va a estar cuando la nave llegue.
- **Naves:** [`config/ships.json`](../config/ships.json) lista las seis que tienen modelo (Smiling Friends, el barco del End de Minecraft, Butters, el carrito de golf y los dos N-1). La activa es la de Smiling Friends. La de Halloween todavía no existe.

## Fase 6: interacción con planetas

- **Proximidad:** el botón **Explorar planeta** aparece cuando la superficie del planeta está a menos de 0,9 radios (`interaction_distance`).
- **Ida:** con minijuego, la pantalla se funde a negro, la ventana del sistema solar se cierra y se abre la del minijuego. Sin minijuego, avisa que todavía no está listo.
- **Vuelta:** al salir del minijuego (en South Park, Retroceso durante la exploración libre) vuelve a abrirse el sistema solar tal como estaba. La nave aparece junto al planeta, mirando hacia afuera. Cerrar la ventana del minijuego cierra el juego.
- **Conexión:** South Park entra por `southpark_dev::run_visit_with_args`, con sus visitas guardadas entre entradas (la intro completa se ve una sola vez). Para conectar otro minijuego hay que poner su nombre en `minigame` y agregar su caso en `scenes/solar_system.rs` (`run`).
- **Pendiente:** el resultado del minijuego (victoria o derrota) todavía no llega a `gameplay`; eso es la etapa 9.

## Archivos

| Archivo | Contenido |
| --- | --- |
| `src/solarSystem/mod.rs` | Órbitas, rotación, velocidad de cada planeta, forma de choque y proximidad. En Rust: `planet_hoppers::solar_system`. |
| `src/ships/mod.rs` | Configuración de naves y vuelo: velocidad, giro, piloto automático, arrastre orbital y choques. |
| `src/graphics/space.rs` | Carga y ajuste de los GLB, shaders del espacio y del Sol, cielo y halo. |
| `src/scenes/solar_system.rs` | La escena: vistas, cámara, interfaz, visitas a minijuegos y pruebas automáticas. |
| `config/solar-system.json`, `config/ships.json` | Planetas, modelos, minijuegos, naves y vuelo. |
| `tests/solar_system.rs`, `tests/ships.rs` | 12 pruebas: órbitas, FPS, choques, arrastre, alcance y proximidad. |

## Pruebas

```powershell
cargo test --locked --test solar_system --test ships
cargo run --locked --bin solarsystem -- --smoke         # recorrido con capturas en target/solarsystem/
cargo run --locked --bin solarsystem -- --smoke-visit   # ida y vuelta a South Park
```

`--smoke` captura la vista general, cada planeta desde su lado iluminado, la nave, el botón Explorar y cada nave de costado (para revisar hacia dónde apunta). `--smoke-visit` estaciona la nave junto a South Park, explora, deja que South Park corra su prueba de entrada (la intro dura ~27 s) y captura la vuelta.

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
