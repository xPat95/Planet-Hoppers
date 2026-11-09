# Planet Hoppers

Proyecto universitario de Gráficas por Computadora: un videojuego 3D de exploración espacial en un sistema solar ficticio con seis planetas inspirados en Minecraft, South Park, Smiling Friends, Hexbound, Un Show Más (Regular Show) y Star Wars.

El ciclo de estación y hangar, los tokens, el desbloqueo de naves y las abducciones están integrados.
El progreso se guarda automáticamente y el menú permite continuar la partida.
Ver [abducciones y persistencia](docs/minigames/abduction-rewards.md) para los flujos y verificaciones.
El final descrito más abajo sigue planificado.

## Objetivo general

Desarrollar una experiencia que integre modelado 3D, navegación, órbitas, rotación, iluminación, animaciones y minijuegos en una arquitectura modular. El jugador explorará los seis planetas y completará sus misiones para reunir seis tokens y desbloquear el final.

## Flujo del juego

1. **Menú principal:** introducción espacial con el título Planet Hoppers.
2. **Selección inicial:** elección libre de una única nave entre las seis temáticas, sobre una plataforma iluminada y giratoria. Esa nave queda desbloqueada y activa; las otras cinco permanecen bloqueadas.
3. **Animación inicial:** la nave elegida abduce mediante un haz de luz a los personajes de su franquicia.
4. **Sistema solar:** aparición en un entorno 3D con un sol central, seis planetas con órbitas y rotación propia y una estación espacial interactiva.
5. **Exploración:** pilotaje libre para acercarse a cada planeta o entrar al hangar de la estación espacial. Allí se puede cambiar voluntariamente entre naves desbloqueadas.
6. **Misiones:** un botón de visita aparece por proximidad y permite iniciar el minijuego del planeta. Predominarán el raycasting y la exploración en primera persona; Star Wars incluye un recorrido de vuelo libre por el túnel de la Estrella de la Muerte, sin raycasting.
7. **Recompensas:** la primera victoria de cada minijuego entrega su token y desbloquea la nave de la franquicia si seguía bloqueada. Solo un nuevo desbloqueo activa su animación de abducción. El regreso al sistema solar conserva la nave usada antes de entrar al minijuego. Perder permite regresar y reintentar sin obtener recompensas.
8. **Final desbloqueable:** reunir los seis tokens habilita terminar el juego independientemente de la nave activa; el sol se transforma en un agujero negro que absorbe los planetas y la nave.
9. **Animación final:** devolución de los personajes abducidos a su lugar de origen.

## Progreso y estación espacial

El progreso distingue tres conceptos: **planetas o misiones completados**, que determinan los tokens obtenidos; **naves desbloqueadas**, disponibles en la estación; y **nave activa**, utilizada para explorar. Elegir una nave inicial no completa su planeta ni entrega su token.

Si se completa por primera vez el planeta de la nave inicial, se obtiene su token, pero no hay otro desbloqueo ni una segunda abducción. Los minijuegos pueden repetirse: las victorias posteriores no duplican tokens, desbloqueos ni animaciones.

Al acercarse a la estación, el jugador puede interactuar para entrar al hangar. Se muestran las seis naves con su estado de bloqueo; solo las desbloqueadas se pueden seleccionar. Al salir, la exploración continúa con la nave seleccionada. Después de la selección inicial, **los cambios de nave solo ocurren por elección voluntaria en la estación espacial**; completar un minijuego nunca cambia automáticamente la nave activa.

Por ejemplo, al empezar con South Park y ganar Hexbound, se recibe el token de Hexbound, se desbloquea su nave y se reproduce la abducción de Ghost Face. Se regresa con South Park; la estación permite elegir entre South Park y Halloween, mientras las otras cuatro siguen bloqueadas.

## Franquicias y responsables

| Franquicia | Personajes | Minijuego | Responsable |
| --- | --- | --- | --- |
| Minecraft | Steve | Explorar y encontrar un diamante | Pablo |
| South Park | Stan, Kyle, Kenny y Cartman | Recorrer South Park y resolver un puzzle relacionado con Kenny | Pablo |
| Smiling Friends | Pim y Charlie | Hacer reír a personajes eligiendo chistes | Cami |
| Hexbound | Ghost Face | [Hexbound: maldecir a las criaturas de la fiesta con hechizos dibujados](docs/minigames/halloween-plan.md) | Cami |
| Un Show Más (Regular Show) | Mordecai y Rigby | El Martillo: combate arcade en tres niveles, individual o cooperativo local | Norman |
| Star Wars | Grogu (Baby Yoda) | Vuelo libre por el túnel y ataque al reactor, sin raycasting | Norman |

La nave de Un Show Más es el carrito de golf con alas de cartón; sus personajes son Mordecai y Rigby. El planeta abre El Martillo al explorar con E/Enter; sus tres niveles, música y videos están integrados en el juego principal. Su identificador interno es `regularshow` y seguirá las mismas reglas de tokens, desbloqueo y abducción que las demás franquicias.

Los sistemas generales son trabajo compartido de los tres integrantes. La estación espacial queda como funcionalidad compartida, pendiente de asignación. Ver [distribución de responsabilidades](docs/responsibilities.md).

## Estructura del repositorio

```text
Planet-Hoppers/
├── assets/
│   ├── models/
│   │   ├── planets/
│   │   ├── ships/
│   │   ├── characters/
│   │   └── environment/
│   │       └── spaceStation/
│   ├── textures/
│   │   ├── planets/
│   │   ├── ships/
│   │   ├── characters/
│   │   ├── minigames/
│   │   ├── skybox/
│   │   └── effects/
│   ├── audio/
│   │   ├── music/
│   │   └── sfx/
│   ├── fonts/
│   └── ui/
│       ├── icons/
│       └── backgrounds/
├── src/
│   ├── core/
│   ├── scenes/
│   ├── solarSystem/
│   ├── spaceStation/
│   ├── ships/
│   ├── characters/
│   ├── graphics/
│   │   ├── shaders/
│   │   ├── lighting/
│   │   ├── materials/
│   │   └── effects/
│   ├── minigames/
│   │   ├── common/
│   │   ├── minecraft/
│   │   ├── southpark/
│   │   ├── smilingfriends/
│   │   ├── halloween/
│   │   ├── regularshow/
│   │   └── starwars/
│   ├── gameplay/
│   └── ui/
├── config/
├── docs/
│   ├── architecture.md
│   ├── responsibilities.md
│   └── development-plan.md
├── tests/
├── README.md
└── .gitignore
```

Cada una de las carpetas `assets/models/planets/`, `assets/models/ships/` y `assets/models/characters/` contiene seis subcarpetas: `minecraft/`, `southpark/`, `smilingfriends/`, `halloween/`, `regularshow/` y `starwars/`. Las carpetas vacías se conservan en Git mediante `.gitkeep`.

| Carpeta | Responsabilidad |
| --- | --- |
| `assets/` | Modelos, texturas, audio, fuentes e imágenes de interfaz. Los modelos de personajes se organizan por franquicia en `assets/models/characters/`. |
| `src/core/` | Inicialización, renderizado general, administración de escenas, entrada y estado global. |
| `src/scenes/` | Composición de menú, selección de naves, sistema solar, estación espacial, minijuegos y final, con sus transiciones. |
| `src/solarSystem/` | Sol, planetas, órbitas, rotación e interacción por proximidad. |
| `src/spaceStation/` | Interacción con la estación, entrada al hangar y coordinación de selección y cambio entre naves desbloqueadas. |
| `assets/models/environment/spaceStation/` | Modelos 3D y recursos geométricos de la estación espacial. |
| `src/ships/` | Naves disponibles, bloqueo y desbloqueo, nave activa, movimiento, controles y cámara. |
| `src/characters/` | Carga y manejo compartido de personajes; abducción inicial y por nuevo desbloqueo, y devolución final, sin modelos ni lógica duplicada por franquicia. |
| `src/graphics/` | Shaders, iluminación, materiales y efectos visuales, separados del movimiento y gameplay. |
| `src/minigames/common/` | Raycasting, controles y colisiones reutilizables. |
| `src/minigames/[franquicia]/` | Reglas y lógica exclusivas de cada minijuego. |
| `src/gameplay/` | Misiones completadas, tokens y recompensas únicas; coordinación de desbloqueos, progreso, victoria y derrota. |
| `src/ui/` | Botones, menús, indicadores, HUD e interfaces de estación con estados de bloqueo y desbloqueo. |
| `config/` | Parámetros del juego y asociaciones entre franquicias, naves, personajes y minijuegos. |
| `docs/` | Arquitectura, organización del equipo y planificación. |
| `tests/` | Futuras pruebas de los módulos. |

Consultar las reglas de separación y la asociación entre naves y personajes en [arquitectura](docs/architecture.md).

## Orden de implementación

1. Definición de estructura.
2. Creación de planetas.
3. Creación de naves.
4. Implementación del sistema solar.
5. Navegación y órbitas.
6. Personajes.
7. Minijuegos.
8. Menús, selección de naves y estación espacial.
9. Recompensas y progreso, con condiciones de victoria y derrota.
10. Agujero negro y final.

El [plan de desarrollo](docs/development-plan.md) describe las diez etapas y su estado inicial.

## Tecnologías utilizadas

El juego utiliza **Rust (edición 2021), raylib-rs 5.5.1 y serde/serde_json**. El menú principal permite elegir una nave en el hangar e iniciar el sistema solar. South Park también conserva su ejecutable independiente, con su GLB real, controlador FPS, colisiones, raycasting y Kenny como sprite con movimiento autónomo.

## Instrucciones de ejecución

Desde la raíz del repositorio, con Rust/Cargo y las herramientas nativas de raylib instaladas:

```powershell
cargo run --locked
```

El flujo normal es **Menú principal → Nueva partida → Hangar → Elegir nave → Sistema solar**. Las seis franquicias están disponibles al inicio; Calaca utiliza un placeholder hasta integrar su nave. La elegida es la única desbloqueada y permanece activa durante la sesión, también al regresar de South Park. No hay guardado ni opción Continuar. Ver [menú, selección y preparación del hangar definitivo](docs/hangar.md).

Para ejecutar la prueba independiente de South Park:

```powershell
cargo run --locked --bin southpark
```

No requiere Blender ni teclado numérico para jugar. WASD mueve al jugador, mouse mira, Shift corre, E interactúa y Escape abre la pausa (o cierra primero el mapa o el diálogo abierto); P conserva el acceso a pausa. Cerrar la ventana o Ctrl+Q termina la prueba. Ver [requisitos, configuración, pruebas y limitaciones de la fase 2](docs/minigames/southpark-phase2.md).

Al entrar se reproduce una presentación 3D sincronizada con el audio completo de `intro.mp3` (**27,376 segundos**); **Espacio o Enter** la saltan. Kenny aparece visible cerca del jugador inicial. Acercarse, apuntarle y pulsar **E** abre **SAVE KENNY!**. **Comenzar desafío** inicia tres vidas y cinco amenazas automáticas en distintas zonas. La búsqueda tiene un plazo calculado por las calles: acercarse adelanta la preparación, pero el peligro comienza también al agotarse el plazo aunque el jugador siga lejos. **M** abre el mapa y pausa el mundo, incluido ese plazo; rueda acerca, arrastrar desplaza, R recentra y M/Escape cierra. **Esc/P** abre la pausa con Continuar, Reiniciar y Regresar al sistema solar; regresar requiere confirmación. **Retroceso** abre esa confirmación durante exploración libre. Victoria y derrota permiten regresar con el resultado correspondiente; derrota también permite reintentar. Ver [contrato de resultados, regreso y pausa](docs/minigames/host-return.md), [intro y balance de fase 6B](docs/minigames/southpark-phase6b.md), [zonas y mapas de fase 6A](docs/minigames/southpark-phase6a.md) y [HUD, audio y diálogos de fase 5](docs/minigames/southpark-phase5.md).


### Sistema solar

```powershell
cargo run --locked --bin solarsystem
```

Fases 1 a 6: Sol central y los seis planetas de Blender en sus órbitas, con traslación y rotación propia (deltaTime), luz del Sol, fondo de estrellas y una nave para recorrerlos (W/S, mouse, Shift turbo, C vista general). Cerca de un planeta aparece **Explorar planeta**: South Park, Smiling Friends, Minecraft, Star Wars y Regular Show abren sus minijuegos y permiten regresar al sistema solar. Ver [sistema solar](docs/solar-system.md).

## Planetas disponibles para integración

Modo de desarrollo de South Park: `cargo run --locked --bin southpark -- --dev-threats` habilita **F5** autobús, **F6** OVNI, **F7** alcantarilla, **F8** reiniciar y **F9** volver al paseo. También: `cargo run --locked --bin southpark -- --threat bus` (o `ufo`, `sewer`). Estas pruebas no descuentan vidas ni suman salvadas. Ver [fase 4A: controles, recursos y validación](docs/minigames/southpark-phase4a.md).

La fase 4B agrega **F10** piano y **F11** muñeco de nieve: `cargo run --locked --bin southpark -- --threat piano` o `cargo run --locked --bin southpark -- --threat snowman`. No requiere numpad. `--smoke-threats` comprueba ahora los diez resultados de las cinco amenazas. Ver [fase 4B: mecánicas, modelos, pruebas y capturas](docs/minigames/southpark-phase4b.md).

Los siguientes recursos son **planetas del sistema solar**. Sus archivos Blender son las fuentes editables; las exportaciones de juego y sus límites están documentados por franquicia.

| Planeta | Fuente editable | Revisión técnica |
| --- | --- | --- |
| Minecraft | [Minecraft](assets/models/planets/minecraft/README.md) | [Revisión](docs/references/minecraft/technical_review_v02.md) |
| South Park | [South Park](assets/models/planets/southpark/README.md) | [Revisión](docs/planets/southpark.md) |
| Smiling Friends | [Smiling Friends](assets/models/planets/smilingfriends/README.md) | [Organización](assets/models/planets/smilingfriends/README.md) |
| Estrella de la Muerte | [Planeta de Star Wars](assets/models/planets/starwars/README.md) | [Revisión](docs/planets/starwars.md) |
| Domo Espacial | [Planeta de Regular Show](assets/models/planets/regularshow/README.md) | [Revisión](docs/planets/regularshow.md) |

