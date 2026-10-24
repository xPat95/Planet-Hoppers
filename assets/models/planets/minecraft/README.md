# Planeta Minecraft

El archivo principal para revisión y futura integración es **[minecraft_planet_textured_v02.blend](minecraft_planet_textured_v02.blend)**. Contiene el modelo texturizado, con nueve imágenes empaquetadas y rutas relativas. No se ha exportado a un motor.

## Segunda iteración: texturizado v02

La versión principal aplica las [texturas locales](../../../textures/planets/minecraft/) al modelo existente. Las versiones anteriores y los respaldos se conservan localmente en `backups/`, excluida de Git. La [captura texturizada](../../../../docs/references/minecraft/minecraft_planet_textured_v02.png) muestra el resultado desde tres caras.

- Se conservan los 141 objetos de malla, sus 52 712 vértices, 39 534 caras, colecciones, transformaciones y parentesco con `Minecraft_Planet_ROOT`. No se reconstruye ni se redistribuye terreno, agua o árboles.
- La capa `MC_BlockUV` orienta las texturas según el marco local de cada cara. Los patrones se repiten aproximadamente cada unidad de bloque; las paredes de montaña repiten verticalmente en lugar de estirar una imagen sobre toda su altura. Las franjas laterales del césped usan el extremo superior de la textura lateral, acorde con su espesor existente de 0.18 unidades.
- Los materiales compartidos `MC_Grass_Top`, `MC_Grass_Side`, `MC_Dirt`, `MC_Stone`, `MC_Oak_Log`, `MC_Oak_Leaves`, `MC_Sand`, `MC_Snow`, `MC_Water`, `MC_Water_Deep` y `MC_Water_Shallow` usan nodos de imagen con interpolación **Closest** y repetición. La nieve también utiliza el recurso disponible.
- Césped superior: textura gris multiplicada por el tinte ajustable `#7CBD6B`. La textura lateral ya tiene césped verde y tierra marrón: se conserva sin teñirla entera.
- Hojas: textura `oak_leaves.png`, tinte ajustable `#58A345` y canal alfa conectado al material. No se modifica la geometría de las copas.
- Agua: `water_still.png` mide 16 × 512 y contiene 32 fotogramas cuadrados apilados. Se extrae el primer cuadro superior a `water_static_frame_00.png` (16 × 16), sin modificar el original. Es estática, con shader opaco y rugosidad 0.48; se conservan las variantes de color existentes con tintes azules.
- Todas las superficies tienen textura. No hay una imagen `oak_log_top`: los extremos de los troncos reutilizan la textura de corteza, sin inventar un recurso adicional.

Las imágenes utilizadas están empaquetadas en el `.blend` para que el archivo se pueda trasladar. El [script de texturizado](../../../../tools/blender/minecraft/texture_minecraft_planet_v02.py) documenta el procedimiento y el [diagnóstico de texturizado](../../../../docs/references/minecraft/diagnostics/texturing_v02_verification.json) registra las firmas geométricas y de las imágenes originales. El script se aplica sobre v01 con una copia de seguridad previa; no genera el planeta ni guarda automáticamente el archivo Blender. No se ha exportado un modelo final.

## Primera iteración

Primera base tridimensional editable de una Tierra cúbica voxelada. Incluye continentes en las seis caras, océanos, islas, terreno superior escalonado, una montaña con nieve, cascada y 29 árboles cúbicos. Los colores son materiales provisionales, sin imágenes aplicadas al cubo ni texturas finales.

## Organización de archivos

| Ruta desde la raíz del repositorio | Contenido |
| --- | --- |
| `assets/models/planets/minecraft/` | Archivo principal `.blend` y este README. |
| `assets/textures/planets/minecraft/` | Nueve texturas utilizadas y la tira de agua original `water_still.png`, conservada como fuente. |
| `tools/blender/minecraft/` | Scripts de generación y texturizado; instrucciones en su README. |
| `docs/references/minecraft/` | Capturas de v01 y v02, informe técnico y organización de referencias. |
| `docs/references/minecraft/diagnostics/` | Dos verificaciones JSON históricas, opcionales para el commit y sin dependencias de ejecución. |
| `assets/models/planets/minecraft/backups/` | Cuatro archivos anteriores o de respaldo conservados localmente, ignorados por Git; revisar antes de borrar. |

Consultar el [informe técnico](../../../../docs/references/minecraft/technical_review_v02.md) y las [recomendaciones de limpieza](../../../../docs/references/minecraft/README.md). No ejecutar los scripts para abrir o visualizar el `.blend` principal.

## Edición

La escena `Minecraft_Planet_Concept_v01` contiene las colecciones `Planet_Base`, `Terrain`, `Water`, `Trees`, `Mountain` y `Presentation`. Todos los componentes del planeta dependen de `Minecraft_Planet_ROOT`, situado en el origen. La cámara y las luces están separadas en `Presentation`.

Las superficies se agrupan por cara, componente y material. Son mallas de bloques reales: en modo Edición se pueden seleccionar sus componentes conectados para mover bloques. Cada árbol tiene su tronco y copa identificados por cara y número. El generador usa una semilla fija y una cuadrícula de 24 bloques por cara; la base mide 24 unidades de lado antes del relieve.

## Integración posterior

Seleccionar la jerarquía de `Minecraft_Planet_ROOT` para exportar y excluir `Presentation`. El agua es opaca en esta versión para simplificar la visualización desde cualquier ángulo. Las seis caras tienen geometría; la vista previa no sustituye el modelo.

Esta entrega es un blockout para refinamiento manual. Antes de integrarlo definitivamente se deben revisar escala, presupuesto de polígonos, caras internas entre bloques, unificación de materiales, colisiones y posibles niveles de detalle. No incluye lógica de juego ni dependencias. El formato de exportación se decidirá al elegir el motor gráfico.
