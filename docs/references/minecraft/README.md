# Referencias y revisión del planeta Minecraft

## Archivos conservados

- [minecraft_planet_concept_v01.png](minecraft_planet_concept_v01.png): captura de la primera versión sin texturas.
- [minecraft_planet_textured_v02.png](minecraft_planet_textured_v02.png): captura de la versión texturizada.
- [technical_review_v02.md](technical_review_v02.md): informe técnico para la integración futura.
- [diagnostics/technical_review_v02.json](diagnostics/technical_review_v02.json): mediciones de la revisión técnica.
- [diagnostics/texturing_v02_verification.json](diagnostics/texturing_v02_verification.json): evidencia de conservación de geometría y texturas en el texturizado.

Las capturas provienen del modelo Blender. No son texturas ni dependencias del modelo. Los JSON son evidencia histórica; sus rutas registradas corresponden al momento de la revisión y no deben interpretarse como rutas actuales de ejecución.

## Archivos locales pendientes de revisar antes del commit

Se movieron a `assets/models/planets/minecraft/backups/`, sin borrarlos. Esa carpeta está excluida de Git:

| Archivo | Recomendación |
| --- | --- |
| `minecraft_planet_textured_v02.blend1` | Respaldo automático de Blender; candidato a eliminar después de comprobar la versión principal. |
| `minecraft_planet_before_texturing_v02.blend` | Copia previa al texturizado; archivar fuera del repositorio o eliminar si ya no se necesita recuperación. |
| `minecraft_planet_textured_v02_before_portability.blend` | Copia previa al ajuste de rutas; archivar o eliminar después de verificar portabilidad. |
| `minecraft_planet_concept_v01.blend` | Versión conceptual histórica; conservar si interesa editar el blockout, preferiblemente en archivo externo. No es el modelo principal. |

Los dos JSON de `diagnostics/` son opcionales para el commit. Pueden eliminarse si no se desea conservar evidencia técnica detallada; se recomienda mantener el informe Markdown. No se ha borrado ninguno de estos archivos.

`water_still.png` se conserva en la carpeta de texturas como recurso original de la textura estática derivada. No se recomienda borrarlo solo porque el modelo utilice el fotograma derivado.

Se retiró únicamente el `.gitkeep` vacío de la carpeta del modelo. El `.gitkeep` de `assets/textures/planets/` ya estaba eliminado antes de esta reorganización y no se restauró, porque contiene texturas útiles.
