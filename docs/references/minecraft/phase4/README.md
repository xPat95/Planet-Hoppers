# Minería — capturas e informes de fase 4

Capturas del ejecutable Rust/raylib, con OpenGL real:

- [Pico de hierro en reposo](pickaxe_idle.png).
- [Golpe del pico](pickaxe_swing.png).
- [Grietas progresivas](cracks.png).
- [Diamante extraído y mineral descubierto detrás](diamond_collected.png).
- [Contador 9 / 8, sin victoria automática](diamonds_nine.png).
- [Pasadizo abierto y recorrido por el jugador](passage_open.png).
- [Menú que cancela la minería](mining_cancelled_menu.png).
- [Bloques y contador restaurados con R](reset.png).

Abrir los PNG en cualquier visor. La prueba usa eventos nativos de raylib, ventana oculta y poses de prueba; no representa una partida manual autónoma.

Informes:

- [Preparación del derivado](preparation.json).
- [Blender, colisiones y GLB reimportado](asset-validation.json).
- [Input, destrucción, eventos, paso y reinicio en ejecución](mining-runtime.json).
- [Compilación, formato y pruebas](verification.json).
- [Regresiones gráficas](regression.json).
- [Hashes originales](protected-sources.json) y [comparación final](protected-verification.json).

```powershell
cargo run --bin minecraft -- --mine --smoke-mining
```

El modo de prueba también regresa a la aldea y ejecuta su comprobación de fase 2. F12 guarda una captura manual en esta carpeta; `--capture-dir DIR` cambia el destino. Ver [guía completa](../../../minigames/minecraft-phase4.md).
