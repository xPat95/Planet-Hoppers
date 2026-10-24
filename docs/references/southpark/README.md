# Vistas de revisión del planeta South Park

La revisión técnica del modelo aprobado está en [docs/planets/southpark.md](../../planets/southpark.md), con [diagnóstico reproducible](southpark_technical_review.json). Las capturas siguientes se conservan como referencias visuales e historial; no son texturas del planeta.

## Iteración actual: v3

La tercera iteración conserva 160 edificios y se centra en fachadas, hitos y variedad de barrios. Ver [el informe breve](iteration03_report.md).

| Vista de v3 | Archivo |
| --- | --- |
| General en tres cuartos | [southpark_v03_three_quarter.png](southpark_v03_three_quarter.png) |
| Frontal | [southpark_v03_front.png](southpark_v03_front.png) |
| Trasera | [southpark_v03_back.png](southpark_v03_back.png) |
| Este | [southpark_v03_east.png](southpark_v03_east.png) |
| Oeste | [southpark_v03_west.png](southpark_v03_west.png) |
| Superior | [southpark_v03_top.png](southpark_v03_top.png) |
| South Park Elementary | [southpark_v03_school_detail.png](southpark_v03_school_detail.png) |
| Edificio cívico con cúpula | [southpark_v03_civic_detail.png](southpark_v03_civic_detail.png) |
| Escaparates comerciales | [southpark_v03_commercial_detail.png](southpark_v03_commercial_detail.png) |
| Pagoda roja | [southpark_v03_pagoda_detail.png](southpark_v03_pagoda_detail.png) |
| Cartel clásico | [southpark_v03_sign_detail.png](southpark_v03_sign_detail.png) |

El [inventario de v3](southpark_iteration03_inventory.json) registra modificaciones y piezas reutilizables; la [validación de v3](southpark_iteration03_validation.json) registra cantidades y pruebas. La copia anterior está en `backups/southpark_before_iteration03.blend`, excluida de Git.

Los letreros `CIVIC HALL`, `MALL`, `MARKET`, `CAFE` y `STORE` son descriptivos o genéricos: no prueban una identidad canónica de construcciones cuya rotulación en el mapa no se puede confirmar. El cartel `SOUTH PARK` y la escuela son los identificadores explícitos principales.

## Iteración histórica: v2

Se refinó el modelo existente: **160 edificios** frente a 71, conservando la esfera, el cartel y las mallas de las carreteras iniciales. Los 220 pinos y las 14 montañas se reorganizaron. Consultar la [comparación detallada](iteration02_comparison.md).

| Vista actual | Archivo |
| --- | --- |
| Tres cuartos, encuadre comparable con v1 | [southpark_v02_three_quarter.png](southpark_v02_three_quarter.png) |
| Frontal | [southpark_v02_front.png](southpark_v02_front.png) |
| Trasera | [southpark_v02_back.png](southpark_v02_back.png) |
| Lateral este | [southpark_v02_side.png](southpark_v02_side.png) |
| Lateral oeste | [southpark_v02_west.png](southpark_v02_west.png) |
| Superior | [southpark_v02_top.png](southpark_v02_top.png) |
| Escuela de cerca | [southpark_v02_school_detail.png](southpark_v02_school_detail.png) |

El [inventario de v2](southpark_iteration02_inventory.json) distingue los edificios originales del relleno interpretado de los barrios; no afirma nuevas identificaciones de lugares famosos. La [verificación de v2](southpark_iteration02_validation.json) registra geometría conservada, alineación radial, apoyos, pruebas de transformación y comprobaciones de huellas.

La copia anterior está en `backups/southpark_before_iteration02.blend`, excluida de Git. No es otro archivo final. No se generaron partículas, animaciones ni exportaciones.

## Vistas históricas: v1

Las referencias principales fueron el mapa panorámico de South Park y el cartel de madera aportados en la conversación. Las siguientes imágenes son capturas renderizadas del modelo 3D, no texturas del planeta.

| Vista | Archivo |
| --- | --- |
| Tres cuartos | [southpark_three_quarter.png](southpark_three_quarter.png) |
| Frontal | [southpark_front.png](southpark_front.png) |
| Trasera | [southpark_back.png](southpark_back.png) |
| Superior | [southpark_top.png](southpark_top.png) |
| Lateral | [southpark_side.png](southpark_side.png) |
| Cartel de cerca | [southpark_sign_detail.png](southpark_sign_detail.png) |

[southpark_inventory.json](southpark_inventory.json) enumera las 71 construcciones con coordenadas aproximadas sobre la imagen de 1600 × 408, descripciones, sectores y nombres de objetos Blender. No atribuye nombres específicos a locales ilegibles. La aproximación al objetivo de cobertura del 80 % es visual; no hay un denominador exacto de edificios verificable en la referencia.

[southpark_validation.json](southpark_validation.json) registra cantidades, alineación radial, altura de las calles respecto de la esfera y pruebas temporales de traslado, rotación y escala. No contiene animaciones ni una exportación del modelo.

La [documentación del modelo](../../../assets/models/planets/southpark/README.md) explica sectores representados, simplificaciones y límites.
