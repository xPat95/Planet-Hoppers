# Minecraft / Diamond Rush desde el sistema solar

## Recursos y alcance

La rama `feature/easter-egg` contiene la implementación completa. Se comprobaron
los cuatro GLB/manifest de Minecraft y las referencias externas de sus buffers e
imágenes, sin faltantes. El flujo integrado utiliza:

- Aldea: `assets/models/minigames/minecraft/minecraft_village.{glb,scene.json}`.
- Diamond Rush ampliado: `assets/models/minigames/minecraft/rush/minecraft_mine.{glb,scene.json}`.
- Reglas existentes: `src/minigames/minecraft/{mine,mining,rush}.rs`.
- Configuración: `config/minecraft-{dev,mining,rush}.json`.
- Música: `assets/audio/music/Minecraft/Minecraft.mp3`.
- Ocho efectos: `assets/audio/sfx/Minecraft/` (`block_break`, `danger`,
  `diamond_collect`, `footsteps`, `game_over`, `pickaxe_hit`, `success`, `victory`).

No se cambiaron modelos, materiales, órbitas, colisiones del hangar, naves,
minerales, peligros, penalizaciones ni reglas de victoria/derrota. Se mantienen
150 segundos, ocho diamantes requeridos de dieciséis disponibles, penalización
de lava de diez segundos y protección de 1,5 segundos.

## Entrada y estados

El cuarto planeta conserva el identificador `minecraft`, su posición y su órbita.
En `config/solar-system.json`, su campo `minigame` ahora es `minecraft`.
`scenes::minigame_host::run` despacha ese identificador al host ya existente,
`minecraft_rush::run_visit`, sin lanzar otro proceso ni usar el host antiguo de
aldea de la fase 2. `run` sigue siendo el adaptador del ejecutable independiente.

El sistema solar sigue utilizando `SolarSystem::planet_in_reach`, el guardado de
entrada `interaction_armed`, el flanco de pulsación de E y su fundido existente.
El botón muestra `[E] Explorar planeta`; solo el planeta realmente al alcance
determina el minijuego. Mantener E durante el regreso no inicia otra transición.

La primera escena es siempre la aldea, con el spawn seguro de su manifest y
controlador FPS existente. El acceso `--mine` sigue reservado para desarrollo;
el despachador normal no pasa ese argumento.

Se conserva la secuencia:

1. Sistema solar → aldea (`SurfaceState::Exploring`).
2. Raycasting + E en la entrada → invitación (`MineMenu`).
3. Confirmar → carga de mina → `RushState::Running` una vez listos los recursos.
4. Pausa/resultados existentes → opción explícita de regresar a la aldea.
5. Aldea → Escape → pausa → Regresar al sistema solar, sin advertencia de abandono.

`VillageMenu::{Closed, Paused}` solo controla la salida de la
aldea; no es otro administrador global. Escape/Enter o Continuar cierran la
pausa; Regresar vuelve directamente al sistema solar.
El botón Regresar también admite R. Con la invitación de mina abierta, Escape
sigue cancelando esa invitación. En la mina, Escape pausa y conserva el menú
Continuar/Reiniciar/Aldea. Elegir Aldea durante una partida activa solicita
confirmación y advierte que se perderá el progreso actual. Cancelar reanuda la
misma partida; confirmar registra su abandono. Se sale al espacio desde la aldea.

Volver desde la mina conserva la posición FPS segura previa. Una sesión Running
retenida se pausa al dejarla, sin cambiar sus reglas. El HUD de mina solo se
dibuja dentro de la mina. Los resultados se registran cuando las reglas emiten
victoria/derrota, con un ID único por partida, en `MinigameVisits.minecraft`.
Salir de la aldea devuelve `MinigameExit::Returned` con el último resultado de
esa visita, sin modificarlo. Si no se inició ningún desafío devuelve
`ReturnedWithoutResult`, sin inventar un abandono ni repetir victorias anteriores.
El historial permanece durante la sesión. Ver [resultados y eventos](minecraft-results.md).

`MinecraftAudio::stop` detiene música y efectos; RAII libera audio, modelos,
texturas y contexto antes de reabrir la ventana solar. Aldea y mina comparten un
contexto gráfico y un stream musical, como antes. El viaje solar/minijuego usa
ventanas sucesivas dentro del mismo proceso, siguiendo la arquitectura existente
de South Park; no requiere cerrar y ejecutar otro programa.

## Nave y retorno seguro

La `Session` solar permanece en memoria, incluyendo nave seleccionada,
`GameState`, órbitas y visitas de South Park. Las órbitas se detienen durante
la visita. Para Minecraft se captura `PlanetReturnPose`: desplazamiento respecto
al centro del planeta y yaw/pitch/bank de la nave, sin duplicar su identidad.

Después de recargar las cotas visuales y colisiones solares, se intenta recuperar
esa aproximación respecto al centro orbital actual. Se comprueba el radio visual,
la forma de colisión de todos los planetas, el Sol y las colisiones de la estación,
con margen correspondiente a la envolvente de la nave. Si no está libre, se
busca hacia afuera del mismo planeta usando su radio y la dirección de llegada;
no hay coordenadas absolutas de retorno. Se conserva la orientación y se ponen
velocidad a cero, arrastre pendiente a None y choque a false. La cámara de vuelo
se reconstruye a partir de esa pose; las primeras entradas de mouse se descartan.
South Park y Smiling Friends conservan su retorno anterior.

## Archivos

Modificados:

- `config/solar-system.json`: registro de Minecraft.
- `src/scenes/minigame_host.rs`: despacho al adaptador existente.
- `src/scenes/minecraft_rush.rs`: contrato de visita, salida de aldea y prueba gráfica del host.
- `src/minigames/minecraft/sound.rs`: detención explícita al abandonar.
- `src/minigames/minecraft/mod.rs` y `src/gameplay/mod.rs`: módulos nuevos.
- `src/scenes/solar_system.rs`: pose de retorno Minecraft, etiqueta y prueba integrada.
- `src/scenes/front_end.rs`: únicamente flag de automatización `--smoke-minecraft`.
- `tests/solar_system.rs`: Minecraft entre las identidades conectadas.
- `docs/minigames/host-return.md`: referencia al nuevo adaptador.

Creados:

- `src/minigames/minecraft/visit.rs`: estados del menú de salida de aldea.
- `src/gameplay/planet_visit.rs`: captura y recuperación segura de pose.
- `tests/minecraft_host.rs`: registro, salida directa y retorno para todas las naves.
- Este documento.

## Verificación

```powershell
cargo fmt --check
cargo check
cargo build
cargo test
cargo run --bin planet-hoppers -- --smoke-minecraft --ship minecraft
cargo run --bin planet-hoppers -- --smoke-docking --ship minecraft
cargo run --bin solarsystem -- --smoke-returns
git diff --check
git diff
git status
```

La prueba integrada Minecraft pasó con dos visitas en un mismo proceso y con
la nave inicial conservada (`FRONT_END_SMOKE_PASS`, `VISIT_SMOKE_PASS visitas=2`).
También se probó el acceso solar directo con distintas naves entre visitas.
`RUSH_SMOKE_PASS` comprueba el menú de mina, carga, minería real de ocho bloques,
victoria en la salida, derrota, reintentos, lava, pausa y regreso a la aldea.
`MINECRAFT_HOST_PASS` comprueba victoria → aldea → sistema solar sin advertencia,
congelación de posición/cámara y reloj retenido, registro único y música detenida.
`--smoke-exploration-exit` verifica la salida libre sin resultado de desafío.

Los cuatro tests nuevos comprueban identidad del cuarto planeta/despacho,
salida directa y recuperación de pose para todas las naves, incluso
después de movimiento orbital y con una aproximación bloqueada.
`cargo test` completo pasó, incluyendo Minecraft, South Park/Ike, naves y estación.
La regresión gráfica de South Park completó victoria, derrota y dos abandonos
(`VISIT_SMOKE_PASS visitas=4`). La del hangar completó tres ciclos de entrada,
misma nave, cambio de Minecraft a Star Wars, cancelación y despegue seguro
(`DOCKING_SMOKE_PASS`, `FRONT_END_SMOKE_PASS`).
El easter egg de Ike también pasó su prueba gráfica (`IKE_SMOKE_OK`).

`cargo fmt --check` global no pasa: existen diferencias de formato en el repositorio,
incluidos archivos ajenos a esta integración como `src/audio.rs` y escenas de
South Park. No se aplicó un reformateo general. Los módulos nuevos y el host de
Minecraft se formatearon de manera localizada.

Capturas e informes: `target/minecraft-host-{0,1}/` y `target/solarsystem/`.
Se inspeccionaron pausa, advertencia de abandono activo y nave al regresar. La automatización usa
OpenGL, dispositivo de audio con volumen cero y eventos nativos; emplea posiciones
y tiempos de prueba para evitar recorrer manualmente cada trayecto. No demuestra
una partida física completa ni una escucha de audio.

## Recorrido manual pendiente

1. `cargo run`: iniciar partida, seleccionar nave y despegar desde el hangar.
2. Seleccionar Minecraft con Tab y pilotar hasta el cuarto planeta. Verificar
   `[E] Explorar planeta`; mantener E debe producir una sola visita a la aldea.
3. Caminar por la aldea, localizar la mina y pulsar E apuntando a la entrada.
   Confirmar el desafío; comprobar 02:30 y cero diamantes al comenzar.
4. Picar, recoger ocho diamantes y volver a la salida con E, o perder por tiempo.
   Volver a la aldea desde el menú del resultado. Probar también Escape en la mina:
   debe pausar, sin abandonar inmediatamente.
5. Escape en la aldea → Regresar al sistema solar, sin advertencia. Comprobar
   la misma nave y orientación, fuera de Minecraft; acelerar
   y continuar pilotando. Escuchar que la música y los efectos de Minecraft cesan.
6. Volver a entrar, iniciar otro intento y repetir la salida.
7. Entrar/salir de South Park, regresar a la estación, cambiar de nave y despegar.
   Comprobar intro, Kenny, amenazas, mapa e Ike durante su exploración habitual.

No se hizo commit ni push ni se cambió de rama.
