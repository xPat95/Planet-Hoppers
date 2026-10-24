# Revisión técnica del planeta Minecraft v02

## Estado del archivo

`minecraft_planet_textured_v02.blend` existe y se reabrió correctamente en Blender 5.4.0 Alpha, tanto antes como después del ajuste de portabilidad. Es un modelo editable válido para continuar el proyecto; aún requiere optimización y validación en el motor que se elija. No se realizó exportación ni se midieron FPS en un motor.

Se guardaron únicamente rutas relativas de imágenes, del texto fuente vinculado y de la salida de render. Las nueve imágenes utilizadas ya estaban empaquetadas y siguen dentro del `.blend`: no requieren las carpetas originales para visualizar el modelo. No hay bibliotecas externas vinculadas. Se conservó `minecraft_planet_textured_v02_before_portability.blend`, además de las versiones y copias anteriores; estos archivos históricos están ahora en `assets/models/planets/minecraft/backups/`, excluida de Git.

Se comprobó mediante una firma que el ajuste no cambió geometría, UV, asignaciones y valores de materiales, jerarquía ni transformaciones. Los bytes de las imágenes empaquetadas también se conservaron. Los resultados medidos están en [diagnostics/technical_review_v02.json](diagnostics/technical_review_v02.json). Los JSON conservan rutas históricas como evidencia de la revisión y no son dependencias del modelo.

## Cantidades

| Medida | Planeta |
| --- | ---: |
| Objetos de malla | 141 |
| Objetos incluyendo la raíz | 142 |
| Vértices de las mallas Blender | 52 712 |
| Caras, antes de triangular | 39 534 |
| Triángulos calculados sin modificar las mallas | 79 068 |
| Materiales utilizados y compartidos | 11 |
| Ranuras de material sumadas entre objetos | 177 |
| Imágenes utilizadas | 9, todas de 16 × 16 |

La escena del planeta tiene 147 objetos incluyendo cámara y cuatro luces. El archivo conserva otra escena con el cubo, cámara y luz predeterminados: 150 objetos en total. Esa escena y la colección `Presentation` deben excluirse de una futura exportación del planeta.

## Jerarquía y centro

Las 141 mallas del planeta dependen de `Minecraft_Planet_ROOT`; no hay mallas sueltas en la escena del planeta. Las pruebas temporales de traslado, escala uniforme y rotación conjunta funcionaron, con error máximo de matriz inferior a 0.000001. Se restauraron exactamente las transformaciones iniciales. No hay modificadores ni restricciones en esas mallas.

La raíz tiene posición y rotación cero y escala uno. Está en `(0, 0, 0)`, que coincide con el centro geométrico del cubo base y es un pivote coherente para su rotación planetaria. **No coincide con el centro de la caja envolvente de todo el relieve**, que está en `(-0.25, 0.25, 1.40)` debido a árboles y elevaciones asimétricos. No se cambió el pivote. No se calculó un centro de masa volumétrico del conjunto de bloques superpuestos.

## Rendimiento y geometría oculta

Se detectaron 12 474 grupos de caras geométricamente coincidentes, con 24 948 caras involucradas: aproximadamente el 63 % de las caras actuales. Hay ejemplos de paredes compartidas entre bloques de océano, incluso entre objetos con distintos materiales. Son candidatos claros para revisar y eliminar caras internas en una copia optimizada; la cifra no implica que todas se puedan eliminar automáticamente sin revisar transparencia y visibilidad.

También conviene revisar el núcleo de piedra cubierto por la envoltura, agua bajo continentes e intersecciones entre bloques de terreno y copas. No se eliminaron caras ni objetos. La detección de coincidencias no constituye una clasificación exhaustiva de toda la geometría oculta.

Las 141 mallas y sus divisiones de material pueden producir muchas llamadas de dibujo. Las 177 ranuras no equivalen por sí solas a una medición de draw calls. Seis planetas con la misma complejidad sumarían 474 408 triángulos antes de naves, efectos o minijuegos. La aceptabilidad depende de plataforma, distancias, sombras y presupuesto del motor; no se puede garantizar rendimiento solo con estas cifras.

## UV, materiales y futura exportación

- Todas las mallas tienen UV activas; no se encontraron caras con área UV nula. Los solapamientos y coordenadas fuera de 0–1 son intencionales para repetir texturas de bloques. No constituyen un atlas único para hornear ni un mapa de iluminación sin solapamientos.
- Las imágenes usan `Closest` y repetición. La futura importación debe mantener filtrado de píxeles y repetición; revisar mipmaps y parpadeo a distancia en el motor.
- Los materiales usan Principled BSDF, imágenes y UV explícitas. Césped, hojas y agua multiplican la imagen por un color constante con factor uno: es un patrón documentado para factores de color glTF, no requiere hornear el tinte por definición. El motor recibirá el resultado o factor PBR, no un grafo editable de nodos Blender.
- Las hojas conectan el alfa de la imagen al shader y usan el método de superficie `DITHERED`. Ese método no garantiza idéntica transparencia en un visor glTF: comprobar la interpretación de máscara o mezcla, umbral y ordenamiento cuando se autorice una prueba de exportación. No se alteró el material.
- El agua usa un fotograma 16 × 16 y alfa del shader igual a uno, sin animación. Su canal alfa de imagen no está conectado: comprobar que la futura exportación conserve la opacidad.
- Todos los materiales tienen desactivado el descarte de caras posteriores. Evaluar posteriormente qué superficies pueden usar culling sin cambiar la apariencia, especialmente las hojas.
- El sombreado plano y las discontinuidades UV pueden aumentar la cantidad de vértices al exportar. Los 52 712 vértices corresponden a las mallas Blender, no a una medición de un archivo glTF.
- Luces de área y World de Blender no forman parte del aspecto PBR portable del planeta; la iluminación final se definirá en el motor. Las versiones anteriores a Blender 5.4 deben probarse por separado porque el archivo se revisó en una compilación Alpha.

Referencia de compatibilidad: [manual oficial de Blender, glTF — materiales y factores](https://docs.blender.org/manual/en/4.2/addons/import_export/scene_gltf2.html). La evaluación aquí es estructural; no se ha generado un glTF/GLB ni validado en un motor.

## Recomendaciones antes de integrar

1. Conservar este archivo como fuente editable y optimizar una copia: eliminar caras internas comprobadas, agrupar mallas por material y evaluar reutilización de árboles y niveles de detalle.
2. Acordar escala, presupuesto de triángulos y llamadas de dibujo para el hardware objetivo. Mantener el pivote del cubo salvo decisión de diseño explícita.
3. Elegir un Blender estable y el motor objetivo; probar entonces transparencia de hojas, opacidad del agua, tintes, culling, filtrado y orientación de ejes en una exportación de revisión autorizada.
4. Exportar únicamente la jerarquía de `Minecraft_Planet_ROOT` y preparar después las colisiones necesarias. Mantener iluminación de presentación y escena predeterminada fuera del recurso del juego.

No se implementaron animaciones, órbitas ni lógica de juego. No se hicieron commits ni push.
