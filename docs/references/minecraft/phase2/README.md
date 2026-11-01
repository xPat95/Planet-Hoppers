# Capturas de ejecución — Minecraft, fase 2

Imágenes generadas por el ejecutable Rust/raylib con OpenGL real, a la altura del jugador:

- [Aldea desde el punto de aparición](village.png).
- [Entrada de mina e indicador E](mine.png).
- [Menú DIAMOND RUSH](mine-menu.png).
- [Confirmación de implementación pendiente](challenge-pending.png).
- [Contacto con una casa al correr](house-collision.png).
- [Contacto con un tronco al correr](tree-collision.png).
- [Inspección de la mesa existente](crafting-table.png).

Abrir los PNG directamente en cualquier visor. La ventana de prueba está oculta; los casos usan input automatizado y poses de prueba. Las capturas no representan una sesión manual ni un desafío de minería.

- [runtime-validation.json](runtime-validation.json): input, estados, cursor, colisiones y auditoría de materiales Minecraft.
- [verification.json](verification.json): resultados de Cargo y pruebas automatizadas.
- [southpark-regression.json](southpark-regression.json): resultados de los modos gráficos existentes de South Park.
- [protected-resources.json](protected-resources.json): hashes de los recursos y fuentes protegidos.

Reproducir las capturas:

```powershell
cargo run --bin minecraft -- --smoke
```

F12 en una ejecución normal guarda `manual.png`. Ver [guía completa de fase 2](../../../minigames/minecraft-phase2.md) para controles y limitaciones.
