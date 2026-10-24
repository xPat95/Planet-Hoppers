# South Park: revisión técnica final

El principal es [southpark_planet.blend](../../assets/models/planets/southpark/southpark_planet.blend). Abre correctamente en Blender 5.4.0 Alpha y conserva íntegramente el diseño aprobado, incluidas las carreteras regulares. No fue necesario guardar cambios en el modelo: su SHA-256 coincide con el respaldo previo a esta revisión, `960590e3c4a2a564f35b7a49fcd499d94e35017460704815b0fecd7af93557e9`.

## Conteos y jerarquía

| Métrica | Resultado |
| --- | ---: |
| Objetos del planeta, incluida la raíz | 496 |
| Objetos de malla del planeta | 494 |
| Vértices contando cada instancia | 83 476 |
| Triángulos contando cada instancia | 98 328 |
| Mallas únicas | 226 |
| Vértices en mallas únicas | 59 620 |
| Triángulos en mallas únicas | 66 968 |
| Materiales únicos usados | 28 |
| Mallas compartidas / objetos que las utilizan | 10 / 278 |

La escena tiene además cinco objetos de presentación: 501 en total. El archivo completo contiene 504 objetos porque conserva otra escena llamada `Scene`, con el cubo, cámara y luz predeterminados. Esa escena no forma parte del planeta.

`SouthPark_Planet_ROOT` está en `(0,0,0)`, sin rotación y con escala unitaria. El ajuste de la esfera base sitúa su centro a unas 0.000054 unidades de la raíz, despreciable frente al radio 22. Los límites de la nieve son ligeramente asimétricos por el relieve; no se desplazó el origen según esos límites ni según edificios o montañas.

Todos los elementos visuales del planeta dependen de la raíz, incluido el cartel y sus letras. Las pruebas temporales de traslación, rotación y escala uniforme no dejaron elementos atrás; error máximo 0.00000382 unidades. Se restauró y comparó exactamente el estado anterior de geometría, transformaciones, propiedades, jerarquía y materiales, y se reabrió el principal sin guardar las pruebas.

## Materiales, recursos y edición

Los 28 materiales tienen asignaciones válidas, sin duplicados idénticos. Usan Principled BSDF conectado directamente a salida: colores constantes, metallic 0, roughness 0.88, alpha 1 y sin transmisión. `Glass` y `Lake` son nombres de colores opacos, no materiales transparentes. Se mantuvieron todos los valores y nodos.

No hay texturas de imagen, fuentes, bibliotecas enlazadas ni recursos externos necesarios para visualizar el planeta. Las cuatro referencias internas a scripts son relativas y sus archivos existen. Las imágenes `Render Result` y `Viewer Node` son resultados internos del editor, no dependencias externas. No hizo falta empaquetar recursos.

Los scripts se resuelven desde su ubicación en el repositorio. El generador histórico puede usar opcionalmente `C:/Windows/Fonts/ROCKEB.TTF`; tiene alternativa incorporada. Esa fuente no es necesaria para abrir el principal: todo el texto ya es malla. Regenerar en otra plataforma puede producir otra tipografía; no regenerar para integrar esta versión.

## Rendimiento y optimizaciones

No hay modificadores costosos, restricciones ni animaciones del planeta. No se detectaron caras de área nula, vértices sueltos, caras coincidentes dentro de las mallas ni objetos idénticos superpuestos. Las superficies abiertas de calles y letras tienen bordes topológicos; esto no implica por sí solo geometría rota.

La reutilización ya existente beneficia memoria, pero 494 mallas instanciadas y sus numerosos materiales por objeto pueden generar muchas pasadas. Se contabilizan **2 481 combinaciones objeto/material con caras**, como indicador previo a visibilidad y agrupación; no es una medición de draw calls ni de FPS. Medir en el dispositivo objetivo cuando exista una implementación y estudiar instanciación, agrupación espacial y LOD sobre copias para exportación, conservando este archivo editable.

Se encontraron dos pares de mallas de letras idénticas: `SOUTH` y `PARK`, cada una por delante y detrás. No son objetos superpuestos ni errores. No se enlazaron para mantener el cartel intacto y la edición independiente de ambas caras.

La prueba conservadora de triángulos completamente bajo la superficie analítica detectó dos en el pórtico cívico y dos en la base del cartel. No demuestra que toda la geometría oculta esté identificada; tampoco analiza oclusiones entre todos los tejados y fachadas. Se conservaron esos apoyos. Suprimir interiores o soldar geometría de carreteras requiere comprobación visual posterior y no se aplicó.

**Optimizaciones aplicadas al modelo: ninguna.** No se encontraron redundancias eliminables con suficiente beneficio que respetasen todas las restricciones. Se conservó la reutilización existente y se organizó el respaldo automático.

## Pendientes para glTF/GLB

1. **Normales:** 27 de los 28 objetos de `Roads` tienen sus superficies orientadas hacia dentro, incluido estacionamiento y marcas; `Road_Town_Entrance` está hacia fuera. Las tres tablas del cartel tienen orientación invertida y el poste está correcto. Los materiales actuales permiten doble cara, por eso se ven. Antes de activar culling, revisar normales en una copia y comparar iluminación. No se corrigieron porque podría afectar el aspecto aprobado.
2. **Texto y curvas:** el cartel y los rótulos ya son mallas; no necesitan otra conversión. La geometría sigue editable y los scripts históricos conservan parámetros para regenerar texto. No hay objetos FONT/CURVE del planeta.
3. **UV:** solo 7 de 226 mallas tienen UV. Los colores constantes no requieren desplegar UV; futuros atlas, texturas o lightmaps sí necesitarán preparación.
4. **Transformaciones:** hay escalas locales no unitarias intencionadas, sin determinantes negativos. Mantener la jerarquía y decidir unidades, ejes y aplicación de transformaciones en una copia cuando se elija la tecnología.
5. **Alcance de exportación:** incluir únicamente la raíz y sus descendientes; excluir `Presentation` y la escena predeterminada. No se realizó exportación ni se eligió motor.

El exportador oficial reconoce colores constantes de Principled. La triangulación y separación de vértices en bordes planos pueden cambiar los conteos exportados. Las instancias GPU requieren soporte de la extensión correspondiente y cumplir sus restricciones. El doble lado se controla mediante culling, y luces Area/World no se trasladan como la iluminación de Blender. Por eso habrá que comprobar el resultado en el destino. [Documentación oficial del exportador mantenido por Khronos](https://github.com/KhronosGroup/glTF-Blender-IO/blob/main/docs/blender_docs/scene_gltf2.rst).

## Organización y limpieza

```text
assets/models/planets/southpark/
  southpark_planet.blend
  README.md
tools/blender/southpark/
  build_southpark_planet.py
  refine_southpark_planet_v02.py
  refine_southpark_planet_v03.py
  capture_southpark_v03.py
  audit_southpark_planet.py
  README.md
docs/planets/southpark.md
docs/references/southpark/
  README.md
  iteration02_comparison.md, iteration03_report.md
  southpark*_inventory.json                  # Inventarios históricos
  southpark*validation.json                  # Verificaciones históricas
  southpark_technical_review.json            # Diagnóstico actual
  southpark*.png                             # Capturas v1, v2, v3
  backups/                                   # Local, ignorado por Git
```

| Clasificación | Decisión |
| --- | --- |
| Modelo principal | Conservado en su ruta original; sin sobrescribir |
| Scripts útiles | Conservados en `tools/blender/southpark/`; añadido auditor no destructivo |
| Capturas de referencia | Conservadas; v1/v2 son historial y v3 es la apariencia aprobada |
| Inventarios y verificaciones | Conservados en referencias; no necesarios para cargar el modelo |
| Recursos externos | Ninguno necesario para el modelo guardado |
| Respaldos | Conservados todos en `docs/references/southpark/backups/` |

**Archivo movido:** `assets/models/planets/southpark/southpark_planet.blend1` → `docs/references/southpark/backups/southpark_automatic_before_technical_review.blend1`. Se añadió una regla limitada a South Park para ignorar futuros `.blend1`, `.blend2`, etc.

**Respaldo nuevo:** `backups/southpark_before_technical_review.blend`, copia exacta del principal antes de la auditoría. Los respaldos de v1/v2 y el previo al experimento de carreteras se mantienen.

**Candidato a eliminar después de revisión:** `backups/southpark_automatic_before_technical_review.blend1`, idéntico por SHA-256 a `backups/southpark_before_roads_refinement.blend`. No se eliminó. Las capturas históricas podrían reducirse si el equipo decide prescindir del historial visual; no se consideran descartables automáticamente. También puede revisarse la escena predeterminada en una copia futura. No se eliminaron archivos dudosos ni datos del cartel.

La revisión visual del render actual fue satisfactoria. El principal siguió abriendo con sus materiales, las pruebas se descartaron y los archivos de otras franquicias permanecieron intactos. No se instalaron complementos ni dependencias y no hubo commit ni push.

Evidencia: [diagnóstico JSON](../references/southpark/southpark_technical_review.json), generado por [audit_southpark_planet.py](../../tools/blender/southpark/audit_southpark_planet.py).
