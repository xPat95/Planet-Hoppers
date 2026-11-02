# DIAMOND RUSH — recurso derivado para minería, fase 4

Este derivado conserva la mina de fase 3 en su carpeta original. Abrir `minecraft_mine.blend` directamente en Blender: escena `Minecraft_Mining_Ready`, escala 1 bloque = 1 metro, imágenes empaquetadas.

- `minecraft_mine.glb`: mismos túneles, cámaras, materiales y UV; 86 grupos extraíbles independientes.
- `minecraft_mine.scene.json`: IDs y colliders individuales, puntos de aproximación transitables y referencias de obstrucción `occluded_by`.

Se separaron únicamente los 60 cubos de mineral común que estaban agrupados en las paredes. Los 14 diamantes y 12 bloques de paso ya eran independientes. Se añadieron caras de fondo a los nichos y se restaron exactamente esos volúmenes de los colliders estructurales. Suelos, techos, soportes, lava y distribución de cámaras conservan sus posiciones.

No se crearon ni sobrescribieron PNG. Las grietas y el pico se dibujan mediante geometría/pixel art en Rust. Los archivos de este directorio son nuevos; no reemplazan los de `../mine/` ni los de la aldea.

```powershell
cargo run --bin minecraft -- --mine
```

Mantener clic izquierdo para picar; R restaura bloques y contador y devuelve al spawn. Ver [guía de fase 4](../../../../../docs/minigames/minecraft-phase4.md) y [capturas](../../../../../docs/references/minecraft/phase4/README.md).
