# Generador del planeta South Park

## Revisión técnica del modelo aprobado

`audit_southpark_planet.py` inspecciona el `.blend` existente con la escena `SouthPark_Planet` activa. No reconstruye, optimiza destructivamente, guarda ni exporta el modelo. Prueba temporalmente traslación, rotación y escala de la raíz, restaura su estado y comprueba que geometría, materiales y jerarquía sigan iguales. Usa únicamente Python y módulos incluidos en Blender, como `numpy` y `bmesh`.

Ejecutarlo desde disco para disponer de `__file__`. Escribe `docs/references/southpark/southpark_technical_review.json`, con conteos, dependencias, normales, duplicados y datos de conservación. Las transformaciones temporales pueden marcar la sesión como modificada; después de revisar el resultado, reabrir el archivo sin guardarlo conserva exactamente el principal. En esta revisión se hizo así. El [informe técnico](../../../docs/planets/southpark.md) documenta los pendientes y la limpieza de archivos.

## Identidad visual v3

`refine_southpark_planet_v03.py` se aplica sobre v2 existente y exige `iteration = 2` en la raíz y la copia `docs/references/southpark/backups/southpark_before_iteration03.blend`. Conserva esfera, carreteras y 160 edificios. Refina fachadas, tejados, escaparates, cúpula, pinos y materiales; genera un inventario en documentación y no guarda el `.blend` automáticamente. No ejecutarlo sobre el principal actual, que ya es v3.

Para reproducir la iteración, abrir una copia de v2 y ejecutar el archivo desde disco en Blender. Usa solo definiciones auxiliares del generador y no reconstruye la esfera. Reutiliza los 28 materiales y kits de geometría compartida. Todo el texto visible se convierte a malla; las palabras genéricas y los parámetros del cartel se conservan editables en los scripts.

`capture_southpark_v03.py` guarda las perspectivas generales y los acercamientos en `docs/references/southpark/`. Restaura cámara y luces al terminar; la iluminación de lectura de las fachadas es temporal y no se hornea en materiales o imágenes del modelo. No guarda ni exporta el archivo principal.

Las copias previas quedan fuera de la carpeta principal, en `docs/references/southpark/backups/`, excluida de Git.

## Refinamiento v2

`refine_southpark_planet_v02.py` modifica la escena v1 existente: conserva esfera, carreteras originales y los 71 objetos de edificios, adapta escalas y cimentaciones, densifica barrios, abre la cordillera y agrupa los pinos. Requiere una copia previa en `docs/references/southpark/backups/southpark_before_iteration02.blend`. Detiene la ejecución si la raíz ya indica `iteration = 2`.

Abrir una copia de v1 y ejecutar el script desde disco en Blender. No ejecutarlo sobre el principal actual, que ya es v3. El script reutiliza únicamente definiciones auxiliares del generador mediante AST: no ejecuta la construcción de la esfera ni crea otra escena. Guarda un inventario en documentación, pero no sobrescribe el `.blend` automáticamente.

Las rutas se resuelven desde la ubicación del script en el repositorio. El archivo principal se ha guardado con una copia interna del script y una referencia relativa al archivo externo. Las capturas y validaciones de esta ejecución están en `docs/references/southpark/`.

## Generación histórica de v1

`build_southpark_planet.py` construye una nueva escena con el planeta esférico y conserva las escenas existentes. No modifica Minecraft ni instala dependencias. Ejecutar desde el editor de texto de Blender abriendo el archivo desde disco para disponer de `__file__`.

El script incluye el inventario de construcciones y la conversión de coordenadas del mapa a longitud y latitud. Usa una semilla fija para vegetación y montañas. Escribe `docs/references/southpark/southpark_inventory.json`, pero no guarda automáticamente el archivo `.blend` ni exporta modelos.

El cartel utiliza una fuente slab serif local si está disponible y la convierte a malla; en otra computadora puede usar la fuente incorporada. El modelo guardado no necesita una fuente externa. La apariencia exacta del texto al regenerar puede variar si no se dispone de la misma fuente.

Archivo editable final de esta iteración: `assets/models/planets/southpark/southpark_planet.blend`. Abrirlo directamente para revisar o continuar el trabajo; regenerar crea otra escena, no actualiza la existente.
