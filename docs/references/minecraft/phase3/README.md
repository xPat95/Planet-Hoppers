# DIAMOND RUSH — revisión de fase 3

Capturas del ejecutable **Rust/raylib**, con cámara FPS:

- [Entrada y soportes](runtime_entrance.png).
- [Galerías y bifurcaciones](runtime_galleries.png).
- [Profundidades, lava y minerales](runtime_depths.png).
- [Diamante independiente seleccionado, sin extracción](runtime_diamonds.png).
- [Bloqueo de un pasadizo](runtime_gate.png).
- [Menú de salida de desarrollo](runtime_exit.png).

Renders del `.blend` editable:

- [Entrada](blender_entrance.png).
- [Túneles](blender_galleries.png).
- [Cámara de lava](blender_depths.png).
- [Mineral de diamante](blender_diamonds.png).
- [Distribución completa con techos ocultos temporalmente](blender_overview.png).

Abrir los PNG directamente en cualquier visor. Los renders y las capturas Rust usan iluminación distinta. La vista general es un corte técnico; los techos están presentes en el modelo y en el modo FPS.

Informes: [recursos y GLB](asset-validation.json), [ejecución Rust](runtime-validation.json), [Cargo y regresiones](verification.json), [resultados de regresión gráfica](regression.json) y [hashes protegidos previos](protected-sources.json). La auditoría final de esos hashes está en [protected-verification.json](protected-verification.json).

```powershell
cargo run --bin minecraft -- --mine --smoke
```

La prueba usa una ventana OpenGL real oculta, eventos de entrada automatizados y poses de prueba. No sustituye una sesión manual con teclado/mouse físicos. Ver [guía completa](../../../minigames/minecraft-phase3.md).
