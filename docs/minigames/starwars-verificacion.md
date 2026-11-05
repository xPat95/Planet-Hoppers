# Verificación de integración Star Wars — 2026-11-04

- `cargo test --locked --offline`: **231 pruebas aprobadas**, una prueba gráfica ignorada durante la suite normal. Registro: `target/starwars_all_tests_final.log`.
- La prueba gráfica OBJ se ejecutó por separado y pasó: `target/starwars_obj.log`. Comprueba el OBJ del Mandaloriano usando su exportación GLB preparada del mismo paquete y nueve cargas seguras del nivel/fuente.
- `starwars --check-ships` en release: seis recursos reales cargados, doce recorridos completos normal/turbo sin daño, cuatro encuentros de popa por recorrido, daño de una barra al chocar con un obstáculo y daño al tocar pared. Registro: `target/starwars_ships_release.log`.
- Ida y vuelta completa desde el menú principal, con selección inicial Star Wars: `target/starwars_host_roundtrip.log`. MINIGAME_RETURN Victory; STARWARS_HOST_PASS ship=starwars; FRONT_END_SMOKE_PASS active=starwars initial=starwars unlocked=1. El descriptor resuelto conserva la ruta del Mandaloriano.
- Segunda ida y vuelta, entrando al planeta Star Wars con Smiling Friends: `target/starwars_cross_ship_roundtrip.log`. STARWARS_HOST_PASS ship=smilingfriends result=Victory. Ambas cinemáticas terminaron naturalmente y la integridad final fue 3.
- Ambas pruebas integrales registraron aproximadamente 59.6 FPS de media con límite de 60, incluyendo videos y vuelo acelerado. Son comprobaciones en esta máquina, no una garantía para otros equipos.
- Formato revisado en todos los archivos nuevos de Rust. `git diff --check` aprobado. Clippy no informó diagnósticos en el código nuevo de Star Wars. El chequeo global estricto encuentra avisos previos en otros módulos (estación, Minecraft, Smiling Friends, sistema solar); no se alteraron para ampliar el alcance de esta integración. Registros: `target/starwars_clippy.log` y `target/starwars_clippy_json.log`.
- Recursos originales transferidos completos, con hashes preservados en `assets/minigames/starwars/manifest.json`. El prototipo secundario permanece independiente. Respaldo del anfitrión previo a la integración: `target/respaldo_antes_starwars/`.

## Dimensiones físicas y prueba por nave

Los tamaños incluyen la tolerancia de 0.035 por semieje. Todas las escalas son uniformes.

| Nave activa | Ancho × alto × fondo | Casco final normal / turbo | Obstáculo / pared |
| --- | --- | --- | --- |
| halloween | 0.995 × 1.326 × 2.870 | 3 / 3 | 1 barra / sí |
| starwars | 1.601 × 0.406 × 2.870 | 3 / 3 | 1 barra / sí |
| smilingfriends | 1.970 × 0.828 × 1.970 | 3 / 3 | 1 barra / sí |
| minecraft | 0.774 × 1.370 × 1.703 | 3 / 3 | 1 barra / sí |
| regularshow | 1.970 × 1.349 × 1.274 | 3 / 3 | 1 barra / sí |
| southpark | 1.675 × 1.370 × 2.018 | 3 / 3 | 1 barra / sí |

Capturas reales de los modelos dentro del túnel:

- [Calaca](../references/starwars-tunnel/halloween.png)
- [Mandaloriano](../references/starwars-tunnel/starwars.png)
- [Smiling Friends](../references/starwars-tunnel/smilingfriends.png)
- [Minecraft](../references/starwars-tunnel/minecraft.png)
- [Un Show Más](../references/starwars-tunnel/regularshow.png)
- [South Park](../references/starwars-tunnel/southpark.png)

## Compilación de la integración inicial

Binarios release compilados del proyecto principal y comprobación final de arranque aprobados (`target/starwars_final_startup.log`).

- `target/release/planet-hoppers.exe`: SHA-256 `19ba6adcc4bd0158e0b030f32f3b7fe6cd0d9f4a22cc3fa9ffaeb2e03cdeefb9`.
- `target/release/solarsystem.exe`: SHA-256 `4a0ef7c8e483a57f50697f02662d9d544dbb7fe534465fa4a40e8f899361a1e8`.
- `target/release/starwars.exe`: SHA-256 `4096948665b152e6a1426d243969e1ddb559ce13f2566a77a9f61e1600dcd86f`.


## Ampliación del recorrido: cuatro discos adicionales

Se conservan los discos de 2210 y 2340 m y se añaden cuatro conjuntos completos en 1350, 1800, 2130 y 2275 m. El recorrido contiene seis discos en total: dos repartidos tras la mitad y cuatro al final. Cada conjunto incluye pared de acero que sigue bloqueando disparos y nave, tres paneles rompibles con sus marcos y colisiones por pieza. Cada panel permite cruzar el disco al destruirlo.

La prueba de aberturas ahora cubre los seis conjuntos y sus 18 paneles, a velocidad normal y turbo. Una prueba adicional comprueba que se conserven los dos discos anteriores, que los dos intermedios estén tras la mitad y que todas las paredes mantengan sus secciones indestructibles.

Registros de esta revisión: `target/starwars_six_bulkheads_tests.log`, `target/starwars_six_bulkheads_ships.log`, `target/starwars_six_bulkheads_captures.log` y `target/starwars_six_bulkheads_smoke.log`. Las capturas 30–33 de `target/starwars/capturas/` muestran los cuatro discos nuevos.

La revisión pasó 233 pruebas automáticas (una prueba gráfica de OBJ permanece excluida de la suite habitual). Los seis modelos reales completaron el nuevo recorrido a velocidad normal y turbo con integridad 3; también se comprobó el daño por choque. Se añadió una regresión para cascos delgados, altos y anchos y se ajustó el primer disco a 1350 m para dar tiempo de disparar y reaccionar con turbo después del encuentro anterior.

Capturas de los cuatro discos completos añadidos:

- [Disco a 1350 m](../references/starwars-tunnel/30_disco_1350.png)
- [Disco a 1800 m](../references/starwars-tunnel/31_disco_1800.png)
- [Disco a 2130 m](../references/starwars-tunnel/32_disco_2130.png)
- [Disco a 2275 m](../references/starwars-tunnel/33_disco_2275.png)

La misión integral con el N-1 terminó naturalmente con integridad 3, 15 paneles destruidos y ambos videos completos. Registro: `SMOKE_OK hull=3 average_fps=59.5 intro=true outro=true`, seguido de `STARWARS_RETURN Returned(Victory)`. Se generaron y revisaron 33 capturas. Los binarios de esta ampliación quedaron compilados en release:

- `target/release/planet-hoppers.exe`: SHA-256 `feed9b435ee11188fa3919ab62743d86f71ac72ef07aa47d14ba9cef06f4d812`.
- `target/release/solarsystem.exe`: SHA-256 `24e5635a318ae5dd10b4ee84f8502098739eef7520f6e0a1ce58dcf535618286`.
- `target/release/starwars.exe`: SHA-256 `4ae7dd5db7d39766e6beddf00acfb09d4a4de51828a61a9346079dd2f771542f`.
