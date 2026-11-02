# DIAMOND RUSH — Fase 5

## Ejecución

Desde la raíz del repositorio:

```powershell
cargo run --bin minecraft
cargo run --bin minecraft -- --mine
cargo run --bin minecraft -- --mute
cargo run --bin southpark
cargo run --bin solarsystem
```

El primer comando abre la aldea. Apuntar a la entrada, pulsar E y elegir
**Iniciar desafío**. `--mine` inicia una sesión independiente en la mina ampliada.
`--rush-config RUTA` permite probar otra configuración sin modificar los recursos.
El ejecutable principal y el sistema solar conservan su comportamiento previo.

WASD: caminar; mouse: mirar; Shift: correr; botón izquierdo mantenido: picar;
E: interactuar con la salida; Escape/Backspace: pausa real. Enter acepta la
primera opción del menú; también se puede usar el mouse. La pausa permite
continuar, reiniciar o regresar a la aldea. Victoria y derrota permiten repetir
o regresar. El cierre de ventana termina el modo independiente.

## Ampliación y rutas

| Medida | Fase 4 | Fase 5 |
|---|---:|---:|
| Celdas de suelo | 1100 | 2409 |
| Cámaras | 9 | 14 |
| Conexiones de túnel | 10 | 18 |
| Diamantes | 14 | 16 |
| Regiones de lava | 2 | 4 |

El suelo crece **2,19 veces**. Esta medida incluye el piso de los estanques y
el espacio ocupado por pilares; se validaron también 2371 puntos de suelo libre
mediante rayos en Blender, y la accesibilidad con el volumen real del jugador en
Rust. No se obtuvo la ampliación escalando el modelo anterior.

Los sectores A, B y C originales permanecen. Se agregan cinco cámaras:

- `D_cavern`: caverna central de 20 × 20 bloques, con dos lóbulos laterales,
  techo a nueve bloques, pilares de distintas alturas, formaciones colgantes,
  lava y tres diamantes. Varios accesos conectan los sectores viejos y nuevos.
- `D_foundry`: cámara de lava cálida, 14 × 14, dos diamantes y accesos alternativos.
- `D_quartz`: cámara de diorita, 10 × 11, un diamante y dos rutas de regreso.
- `D_gallery`: galería de madera, 11 × 12, soportes y conexión circular con la caverna.
- `E_grotto`: recompensa oculta, 13 × 12, un diamante tras el pasadizo rompible de C.

Los ocho enlaces nuevos son `C_foundry`, `C_cavern`, `C_gallery`,
`D_gallery_cavern`, `D_quartz_link`, `D_east_loop`, `D_foundry_loop` y
`E_hidden_reward`. El grafo completo tiene cinco ciclos independientes. Se
conservan callejones con recompensas y tres recompensas tras barreras selectivas.

Distribución de diamantes: A: 2; B: 3; C: 4; D: 6; E: 1. Trece son accesibles
con las barreras intactas; los dieciséis con los pasadizos abiertos. Hay sesenta
minerales comunes y doce bloques de piedra rompibles. Los minerales comunes no
incrementan el contador. Cada extraíble tiene identidad, geometría, aproximación
para minería y colisión propia, restaurables al repetir.

## Recursos y exportación

La fuente es `tools/blender/minecraft/build_rush_mine.py`, que reutiliza los
generadores anteriores. Escribe una escena nueva en
`assets/models/minigames/minecraft/rush/` y usa archivos temporales para los
intermedios. Los archivos `.blend`, `.glb` y `.scene.json` de fases previas
permanecen intactos. Abrir el nuevo `.blend` directamente en Blender.

```powershell
blender --background --python tools/blender/minecraft/build_rush_mine.py
blender --background --python tools/blender/minecraft/validate_rush_mine.py
```

Se reutilizan stone, cobblestone, andesite, granite, diorite, gravel,
coal_ore, iron_ore, gold_ore, redstone_ore, lapis_ore, diamond_ore,
oak_planks, oak_log, oak_log_top y lava. Los nombres concretos se resuelven con
los mismos materiales del generador anterior. Las 16 imágenes están empaquetadas,
con UV por bloque e interpolación Closest; Rust mantiene filtrado nearest.
No se modificaron PNG ni audio originales.

Escala: bloque = una unidad; Blender Z arriba; conversión Rust `(x,z,-y)`.
Entrada y aparición original preservadas, ojos a 1,7 unidades. Límites visuales
Blender: X −45…52, Y −3…84, Z −0,08…9. Túneles principales de al menos tres
bloques de altura. Suelo plano, sin nueva física vertical.

El GLB tiene 40184 triángulos, 20 materiales y 116 mallas al reimportarlo. El
cargador produce 102 lotes de dibujo y comparte materiales y texturas; conserva
separados los 88 grupos removibles. Hay 465 AABB de escena, más los límites que
construye el cargador. Las formaciones estáticas evitan caras internas. Las 39
luces usan el shader de mina existente, ampliado de 24 a 48 luces; no se añade un
motor de iluminación ni se cambia el shader de South Park.

## Sesión, HUD y peligros

`config/minecraft-rush.json` centraliza 150 segundos iniciales, ocho diamantes
requeridos, dieciséis disponibles, penalización de lava de diez segundos,
protección de 1,5 segundos y alerta a treinta segundos.

La máquina de estados distingue Loading, Running, Paused, Victory y Defeat.
El reloj empieza después de cargar modelo, materiales y colisiones, cuando el
jugador puede actuar. Usa tiempo real monotónico; pausa y carga quedan excluidas.
El HUD muestra MM:SS, contador, objetivo, avisos y una indicación discreta de
dirección/distancia a la salida después del octavo diamante. Esta guía marca la
dirección general, no calcula una ruta por los pasillos.

Ocho diamantes activan el mensaje de regreso y `success`, **sin ganar**. La victoria
requiere pulsar E, mediante el raycasting compartido con rango y oclusión, en la
salida real, con ocho diamantes y tiempo positivo. Si faltan diamantes, aparece un
aviso sin detener el desafío. Al llegar a cero, el reloj se fija en cero, se
detienen movimiento y minería y aparece derrota. Los menús liberan el cursor.

Las cuatro regiones de lava comprueban el volumen corporal. El contacto resta
diez segundos, cancela el golpe en curso, teletransporta a suelo firme y activa
1,5 segundos de protección. Cada región tiene un punto seguro comprobado contra
lava y colisionadores. Si la penalización agota el reloj, la derrota es inmediata.
No hay corazones ni daño progresivo.

Reiniciar reutiliza `MiningSession::reset_mining_session`: restaura los 88 bloques,
sus colisiones y visibilidad, contador, grietas, objetivos y animación. También
restaura reloj, alertas, lava, posición y orientación y detiene efectos previos.
La mina queda cargada para repetir sin recargar modelos. Regresar a la aldea
restaura la posición segura guardada antes de entrar, dentro del mismo contexto
gráfico y de audio.

## Audio

Una sola instancia en bucle de `assets/audio/music/Minecraft/Minecraft.mp3`
permanece entre aldea, mina y reintentos. Volumen normal 0,22; alerta, pausa y
resultados 0,12. El adaptador Minecraft reutiliza `AudioAudit`; el gestor South
Park, cuyos recursos y pistas están acoplados a ese juego, permanece intacto.

Los ocho efectos existentes se cargan una vez desde `assets/audio/sfx/Minecraft/`:

| Archivo | Evento |
|---|---|
| `pickaxe_hit.mp3` | Impacto real del pico, limitado por su animación |
| `block_break.wav` | Bloque destruido |
| `diamond_collect.wav` | Diamante añadido |
| `footsteps.mp3` | Desplazamiento efectivo al caminar/correr; se detiene al quedar quieto |
| `danger.mp3` | Cruce del umbral de treinta segundos, una vez por intento |
| `success.mp3` | Objetivo de ocho alcanzado, una vez |
| `victory.mp3` | Victoria en la salida, una vez |
| `game_over.mp3` | Derrota, una vez |

Impactos, destrucción y recolección reutilizan una voz por efecto sin instancias
superpuestas. Los recursos se liberan antes de cerrar el dispositivo. No hay
nueva música ni sonidos de lava. El smoke carga el dispositivo real con volumen
cero: comprueba decodificación y eventos, no una evaluación auditiva de mezcla.

## Cambios de código

- Nuevos `src/minigames/minecraft/rush.rs` y `sound.rs`: reglas y audio.
- Nuevo `src/scenes/minecraft_rush.rs`: host de aldea y mina, menús y HUD;
  `minecraft_rush_smoke.rs`: automatización gráfica.
- `src/bin/minecraft.rs` abre ese host; los flags antiguos `--smoke` y
  `--smoke-mining` conservan las pruebas anteriores. Registros mínimos en `mod.rs`.
- `mine.rs` añade retorno seguro y validación de lava activa. `resources.rs` y
  `shaders/mine.fs` aumentan la capacidad de luces de mina.
- Los generadores anteriores admiten límites dinámicos, distribución de minerales
  y conteo de bloques variable; sus parámetros por defecto conservan lo anterior.
- Nuevo `tests/minecraft_rush.rs`: diez pruebas de sesión, geometría y recorridos.

Se reutilizan sin reescribir controlador FPS, colisión barrida con deslizamiento,
raycasting con oclusión, carga de modelos, minería, grietas y pico de la fase 4.

## Verificación y balance

`cargo check`, `cargo build` y `cargo test` pasaron: **157 pruebas**, incluidas diez
nuevas. Se comprobó la minería de los 88 bloques, actualización de colisiones y
reinicio. Las pruebas de navegación usan suelo y AABB reales: todos los sectores
y los dieciséis diamantes resultan alcanzables, con regreso a la salida.

Dos estrategias simuladas con controlador FPS real a 20 Hz y minería real,
caminando sin sprint, recorren 282,7 y 278,7 unidades y terminan en 85,1 y 84,1
segundos, incluidos ocho diamantes y regreso. Quedan unos 65 segundos, por lo que
se mantiene el valor inicial de 150. Es una referencia con orientación instantánea
y rutas conocidas; **no es una prueba manual de balance ni de primer intento**.

El `.blend` se abrió en Blender 5.4 Alpha y el GLB se reimportó: UV, imágenes,
materiales, nodos independientes, escala y ejes verificados. Se revisaron capturas
del render Rust y de Blender. El smoke gráfico comprueba carga, menús, pausa,
lava, extracción real de ocho minerales, victoria, derrota, reintentos y regreso.
Los presets de posición y tiempo se usan únicamente en esa automatización.

```powershell
cargo run --bin minecraft -- --smoke-rush
cargo test --test minecraft_rush
```

Los resultados y capturas están en [las evidencias de fase 5](../references/minecraft/phase5/README.md).
Las pruebas gráficas anteriores de aldea, minería, South Park, Kenny y sus cinco
amenazas también pasaron. Se conserva un inventario SHA-256 de 241 archivos
originales y se comprueba que no cambian.

`cargo fmt --check` global se ejecutó y detecta diferencias preexistentes en
archivos ajenos, entre ellos `src/audio.rs` y escenas South Park. Se dejaron
intactos para preservar el alcance. El chequeo de rustfmt sobre todos los archivos
Rust nuevos/modificados y `git diff --check` sí pasa.

Queda para fase 6: prueba manual completa de recorrido y balance, evaluación
auditiva, pulido visual y la integración definitiva con el sistema solar. Las
luces actuales no proyectan sombras dinámicas; la lava usa el fotograma existente.
No se afirmó una prueba manual con mouse/teclado ni escucha de audio.
