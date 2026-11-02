# DIAMOND RUSH — mina subterránea, fase 3

Abrir **minecraft_mine.blend** directamente en Blender. Escena `Minecraft_Underground_Mine`, un bloque = un metro, Z arriba, imágenes empaquetadas. La cámara inicial mira desde la entrada hacia los soportes de la mina superior. Numpad 0 activa la cámara; Z → Rendered o F12 muestra sus luces.

- `minecraft_mine.glb`: geometría/UV/materiales glTF 2.0, Y arriba, 16 imágenes integradas y filtros Nearest. Sin luces de Blender ni proxies de colisión.
- `minecraft_mine.scene.json`: esquema compartido 1, suelo, AABB, grafo de nueve cámaras y diez túneles, 14 diamantes, 12 bloques rompibles, dos regiones de lava, 22 luces y referencias de entrada/salida.

La geometría estática se agrupa en `DRM_Static`; los 26 bloques `ORE_*` / `BREAK_*` permanecen como mallas independientes en `DRM_Interactive`. `DRM_Lava` contiene las dos piscinas. `DRM_Colliders` conserva proxies ocultos, y `DRM_Markers` / `DRM_Presentation` contienen referencias y cámaras/luces de revisión.

Las caras estáticas no visibles se eliminan. Detrás de cada mineral queda un revestimiento para cuando se retire posteriormente. Sus colliders no están cubiertos por otras cajas estáticas. Los objetos completos, su material y sus seis caras se conservan; el manifiesto incluye ID, posición, collider, estado inicial y datos de restauración.

Hay diez diamantes accesibles evitando lava con los bloqueos intactos; cuatro se reservan tras dos puertas de piedra de seis bloques cada una. En fase 3 no se puede romper ninguna. La lava usa el primer cuadro cuadrado de la tira original, empaquetado en memoria sin PNG derivados; tiene suelo sólido de desarrollo y sensores sin daño.

Ejecutar desde la raíz:

```powershell
cargo run --bin minecraft -- --mine
```

La aldea conserva `cargo run --bin minecraft`. Ver [guía de sectores, contrato, controles y validación](../../../../../docs/minigames/minecraft-phase3.md) y [capturas](../../../../../docs/references/minecraft/phase3/README.md).
