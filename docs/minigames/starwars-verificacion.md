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

## Compilación final

Binarios release compilados del proyecto principal y comprobación final de arranque aprobados (`target/starwars_final_startup.log`).

- `target/release/planet-hoppers.exe`: SHA-256 `19ba6adcc4bd0158e0b030f32f3b7fe6cd0d9f4a22cc3fa9ffaeb2e03cdeefb9`.
- `target/release/solarsystem.exe`: SHA-256 `4a0ef7c8e483a57f50697f02662d9d544dbb7fe534465fa4a40e8f899361a1e8`.
- `target/release/starwars.exe`: SHA-256 `4096948665b152e6a1426d243969e1ddb559ce13f2566a77a9f61e1600dcd86f`.
