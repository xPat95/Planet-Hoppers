# Mina ampliada — DIAMOND RUSH, fase 5

Abrir `minecraft_mine.blend` en Blender. El modelo que carga Rust es
`minecraft_mine.glb`; `minecraft_mine.scene.json` contiene navegación, colisiones,
minerales individuales, iluminación, lava y puntos de retorno.

La fuente de verdad es `tools/blender/minecraft/build_rush_mine.py`:

```powershell
blender --background --python tools/blender/minecraft/build_rush_mine.py
blender --background --python tools/blender/minecraft/validate_rush_mine.py
cargo run --bin minecraft -- --mine
```

El generador reutiliza las herramientas de las fases 3 y 4 mediante un directorio
intermedio temporal. Escribe exclusivamente estos derivados y las evidencias de
fase 5; no sustituye las escenas anteriores. Las imágenes originales no cambian.
Las imágenes necesarias están empaquetadas en el blend y el GLB.

Escala: un bloque = una unidad. Blender usa Z arriba; Rust recibe `(x,z,-y)`.
Se conserva la entrada y la cámara de aparición original. Suelo a altura cero,
ojos a 1,7 unidades. La caverna nueva llega a nueve unidades de altura.

Hay 14 cámaras y 18 conexiones, 16 diamantes, 60 minerales comunes y 12 bloques
de pasadizos. Los 88 bloques extraíbles conservan nodos y colisionadores
independientes. Las cuatro regiones de lava incluyen retorno seguro. No editar
manualmente estos derivados: regenerar desde el script.

Ver diseño, controles y resultados en
[minecraft-phase5.md](../../../../../docs/minigames/minecraft-phase5.md).
