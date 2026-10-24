# Planeta South Park

Archivo principal: **[southpark_planet.blend](southpark_planet.blend)**.

Revisión técnica final: [estado, rendimiento, portabilidad y pendientes de exportación](../../../../docs/planets/southpark.md). El archivo aprobado se conservó sin cambios, incluidas las carreteras regulares. Antes de integrar, revisar las normales invertidas de carreteras y tablas del cartel en una copia de trabajo; no activar culling de caras sin esa revisión.

Tercera iteración editable del planeta esférico nevado, centrada en identidad visual y refinada sobre el modelo existente. Se conservaron exactamente la geometría de la esfera y las carreteras, así como los 160 edificios de v2. Los edificios, calles, pinos y montañas siguen la curvatura y están distribuidos alrededor del planeta. No hay un terreno plano superpuesto ni una imagen aplicada como ciudad.

## Contenido

- **160 construcciones:** 118 casas, 34 construcciones comerciales y 8 edificios públicos o hitos. Se añadieron 89 construcciones de relleno de barrio, sin atribuirles nuevas identidades famosas.
- **220 pinos** que comparten tres variantes de malla y **14 montañas** tridimensionales con picos nevados.
- South Park Elementary con entrada doble, fascia azul y bandas rojas; edificio cívico con pórtico, reloj, ventanas y cúpula; galería comercial del edificio con estacionamiento; detalles de pagoda y granero. No se afirma que el edificio cívico sea un ayuntamiento confirmado ni se inventa una comisaría.
- 34 escaparates con tres kits compartidos y rótulos genéricos; ocho familias de tejados residenciales y cuatro kits reutilizados en 24 casas de esquina.
- Red vial curva, estacionamiento del edificio comercial grande y lago occidental. Se añadieron seis tramos secundarios sin sustituir las carreteras originales.
- Cartel independiente con poste, tablas y letras tridimensionales `SOUTH PARK`, legibles por ambos lados.
- Materiales de colores planos, sin texturas externas. El texto se convirtió a malla y no depende de una fuente instalada.
- Colección `SouthPark_SnowFX` vacía, reservada para un futuro efecto independiente. Sin partículas, órbitas ni animación.

## Organización y edición

La escena `SouthPark_Planet` contiene la raíz `SouthPark_Planet_ROOT`, centrada en `(0, 0, 0)`. La esfera tiene radio base de 22 unidades y elevaciones sutiles. Todas las mallas del planeta dependen de la raíz, incluido el cartel y sus letras; cámara y luces de revisión están en `Presentation`.

```text
Planet_Surface/
City/
  Residential/
  Commercial/
  Public_Buildings/
  Landmarks/
Roads/
Mountains/
Trees/
SouthPark_Sign/
SouthPark_SnowFX/     # Vacía
Presentation/        # Excluir de una futura exportación del planeta
```

Las viviendas se derivan de ocho variantes, con apoyo recalculado individualmente sobre la esfera. Sus mallas se independizaron en v2 para adaptar las cimentaciones con precisión; v3 añade familias de tejado y pequeños kits de detalles compartidos. Los pinos, escaparates y detalles de esquina usan mallas enlazadas: para editar solo una instancia, crear primero una copia independiente de sus datos. Cada construcción incluye identificador, descripción, coordenadas aproximadas sobre el mapa y factores de escala de v2; las viviendas incluyen su familia de v3.

Las casas originales tienen aproximadamente 1.9 veces su altura nominal anterior; las huellas se adaptaron según espacio y categoría. Los edificios públicos y comerciales se ampliaron de manera diferenciada. Se dejó visible más tejado de color y se redujo la nieve a placas en las viviendas. La escuela tiene el rótulo `SOUTH PARK ELEMENTARY` y se conservaron los demás hitos y el cartel clásico.

Las montañas forman ahora una cordillera abierta y ramificada, con latitudes, anchuras y alturas variadas. Los pinos, más grandes, se agrupan en bosques y parques con separación respecto de calles y edificios.

En v3 se hicieron 29 ajustes leves de implantación residencial, sin eliminar edificios ni sustituir barrios. Las pendientes de tejado y los acabados varían dentro de una familia pequeña. Las montañas conservan su distribución de v2 y reciben picos algo menos simétricos y laderas verdes oscuras o grisáceas. La nieve tiene variaciones tonales amplias y sutiles; dos pinos se apartaron de los bordes de calzada. El cartel clásico se conserva geométricamente y su texto de malla sigue siendo legible por ambos lados.

## Referencia, cobertura y límites

Se conservaron las relaciones entre las avenidas superior, comercial y residencial; los barrios central y oriental; el sector cívico occidental; los comercios al noreste y la cadena montañosa al fondo. La longitud da una vuelta completa al recorrer los 1600 píxeles de ancho del mapa: hay construcciones en la parte frontal, trasera y los laterales, a ambos lados del ecuador.

La referencia permite reconocer la escuela amarilla y roja, la pagoda roja, el edificio con cúpula, el gran edificio amarillo con estacionamiento y el granero occidental. Los nombres de locales cuya rotulación no se puede leer se mantienen genéricos. Son interpretaciones simplificadas, no reconstrucciones arquitectónicas exactas.

Se priorizó una cobertura cercana al objetivo visual del 80 % de construcciones, pero **no se certifica un porcentaje exacto**: la imagen no permite contar con fiabilidad todos los anexos y volúmenes. El [inventario actual](../../../../docs/references/southpark/southpark_iteration02_inventory.json) distingue las 71 construcciones conservadas de las 89 adiciones reinterpretadas para densificar barrios. El [inventario de v1](../../../../docs/references/southpark/southpark_inventory.json) se conserva como referencia histórica. No se cuentan árboles, ventanas, carreteras o el cartel como edificios adicionales.

Se simplificaron fachadas, parcelas, bordes de calles y distancias. Se omitieron volúmenes muy pequeños o ambiguos de los extremos, interiores, vehículos, rótulos ilegibles y el símbolo de Canadá de la esquina superior derecha. El polo norte queda como región nevada abierta detrás de las montañas, y el extremo sur tiene más espacio natural que urbano.

## Verificación y futura integración

La versión actual tiene **98 328 triángulos** contando todas las instancias, 496 objetos del planeta (494 de malla) y 226 mallas únicas utilizadas. Utiliza los mismos 28 materiales compartidos. Las pruebas de transformación de la raíz y alineación radial de los 160 edificios fueron satisfactorias. Las cimentaciones se asientan aproximadamente 0.01 unidades en la superficie para evitar huecos, sin edificios flotando. No se detectaron solapamientos de huellas entre edificios ni contactos importantes entre pinos, edificios y carreteras.

El [informe de v3](../../../../docs/references/southpark/iteration03_report.md) recoge los cambios actuales. La comparación histórica con v1 está en [iteration02_comparison.md](../../../../docs/references/southpark/iteration02_comparison.md). Las copias previas a v2 y v3 están en `docs/references/southpark/backups/`, conservadas localmente y excluidas de Git. El principal sigue siendo `southpark_planet.blend`.

Esto no garantiza un presupuesto de rendimiento para un motor todavía no elegido. Antes de integrar: evaluar agrupación por material, instancias, colisiones, niveles de detalle, escala y comportamiento de normales. No se ha exportado el modelo.

Ver [las vistas actuales y las históricas](../../../../docs/references/southpark/README.md), el [generador de v1](../../../../tools/blender/southpark/build_southpark_planet.py), el [refinamiento de v2](../../../../tools/blender/southpark/refine_southpark_planet_v02.py) y el [refinamiento de v3](../../../../tools/blender/southpark/refine_southpark_planet_v03.py). No es necesario ejecutar scripts para abrir el `.blend`.
