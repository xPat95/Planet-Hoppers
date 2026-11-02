# DIAMOND RUSH — Overworld, fase 1

Escenario independiente del planeta. Abrir **minecraft_village.blend** directamente en Blender; no requiere ejecutar scripts ni buscar texturas.

- `minecraft_village.blend`: escena editable `Minecraft_Overworld_Village`, imágenes empaquetadas, cámaras e iluminación de revisión.
- `minecraft_village.glb`: geometría visual, UV y materiales glTF 2.0, imágenes integradas, sin luces, cámaras ni proxies de colisión.
- `minecraft_village.scene.json`: contrato de escena, suelo, cajas AABB, caminos, referencias y auditoría de las texturas fuente.

Contiene dos casas pequeñas, dos medianas, una casa principal, 16 robles, pozo, huerto y una entrada corta de mina. Las casas están cerradas. No incluye lógica de juego.

Un bloque = una unidad = un metro. Blender usa Z arriba; GLB/raylib usa Y arriba: `(x,y,z) → (x,z,-y)`. Los datos del manifiesto permanecen en coordenadas Blender. Suelo transitable Z=0; colinas periféricas sólidas. Ojo de referencia 1.7 m, jugador 1.8 m y radio 0.35 m.

Colecciones `DR_Terrain`, `DR_Buildings`, `DR_Village`, `DR_Trees`, `DR_Vegetation`, `DR_Mine`, `DR_Colliders`, `DR_Markers` y `DR_Presentation`. Los proxies `COL_*` están ocultos en visor y render; se pueden mostrar desde el Outliner con Alt+H tras seleccionar los objetos de la colección. No son físicas activas. Los marcadores `*_reserved` solo reservan posiciones.

El GLB especifica filtros NEAREST y corte alfa MASK=0.5 para hojas, vidrio y puertas. El adaptador futuro de raylib deberá conservar estos modos. El cargador/shader usado actualmente por South Park no es una validación del tratamiento de alfa de Minecraft; no se ha modificado.

Ver [guía completa](../../../../docs/minigames/minecraft-phase1.md), [capturas e informe](../../../../docs/references/minecraft/phase1/README.md) y los scripts en `tools/blender/minecraft/`. No se reutilizó geometría esférica.
