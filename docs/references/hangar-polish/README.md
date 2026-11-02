# Estación, exhibición y vuelo: cinco correcciones

Base inspeccionada: `main`, commit `e6ff860`, árbol inicialmente limpio. Incluye el hangar Blender aprobado y el despegue seguro. No se modificaron GLB, Blender, texturas, arquitectura, iluminación del hangar, plataforma, botones, menú, minijuegos ni órbitas. No se hizo commit ni push.

## Cambios

### 1. Interacción exterior con la estación

La esfera conservadora de radio 860 contiene el exterior de la estación. La puerta se encuentra dentro de ella: con la configuración actual está en `(0,6032,707)`. El rango anterior de 80 unidades, limitado además al ancho/alto de la puerta, obligaba a entrar por el corredor perfectamente alineado; fuera de ese corredor la esfera podía bloquear la nave antes de mostrar E. No se redujo arbitrariamente la esfera.

`entry_anchor(ship_radius)` calcula la intersección de la dirección transformada de salida con la esfera ampliada por el volumen completo de la nave. El radio de la nave proviene de los semiejes ajustados del modelo cargado, o del radio físico si es mayor. Para Calaca se conserva una envolvente conservadora del modelo provisional. Colisiones de estación y proximidad usan ese mismo tamaño.

La nueva zona de interacción tiene como máximo 40 unidades de radio alrededor de ese punto exterior (`min(interaction_radius, 2*width)`). Solo admite puntos fuera de la esfera ampliada y frente a la salida. No permite entrar desde cualquier cara ni desde lejos. El acceso cercano en el corredor real también se conserva. No se alteraron `push_out`, el corredor, el cooldown ni la exigencia de una nueva pulsación.

En los recorridos nativos, E se activó a aproximadamente 902–903 unidades del centro; las envolventes de las naves eran de 2,86–4,41 unidades. Por tanto, había unas 39 unidades libres respecto a la esfera ampliada antes de entrar. El despegue continúa calculando la separación de modelo, colisionador, nave y cámara; recibe ahora también el encuadre específico de la nave para que coincida con la cámara real.

### 2. Estela discreta

Solo se modifica el efecto suplementario de tres esferas del juego. La separación entre centros se reduce un 78%, el radio principal un 77% y la opacidad principal pasa de 150 a 90. La extensión total detrás del límite del fuselaje se reduce aproximadamente un 71–74% entre ralentí y turbo. Se ancla tras el semieje longitudinal real, con una holgura que mantiene todas las esferas fuera del fuselaje, sobre el vector opuesto al avance. No se eliminan efectos de los GLB; se siguen dibujando tres esferas.

### 3. Frente y costado en el hangar

Se inspeccionaron generadores, configuración, modelos cargados y capturas. Estos son los frentes después de aplicar las rotaciones de importación existentes:

| Exhibición | Frente preparado | Tratamiento |
| --- | --- | --- |
| Calaca provisional | −Z | Misma corrección común hacia el espectador |
| N‑1 del Mandaloriano | +Z | Compensación de su eje opuesto, solo en exhibición |
| Smiling Friends | −Z | Se conserva su corrección de importación de 180° |
| Minecraft | −Z | Se conserva su orientación preparada |
| Golf | −Z | Se conserva su corrección de importación de 180° |
| Butters | −Z | Se conserva su corrección de importación de 180° |

Fuentes de orientación: `build_end_ship.py` declara frente +Y de Blender; los generadores de Butters, Golf y N‑1 declaran −Y. La conversión glTF y las rotaciones de `ships.json` explican la diferencia del N‑1. Smiling Friends se contrastó también con sus personajes y captura renderizada. No se modificaron orientaciones de vuelo.

El ángulo inicial se calcula respecto a la posición horizontal de la cámara, a 30° antes de mirar al espectador. El giro positivo existente de 10°/s alcanza el frente aproximadamente a los tres segundos y luego continúa. Se añade un desplazamiento angular **solo a la nave**: ángulo, velocidad, geometría y posición de la plataforma se conservan. El centrado y la altura de apoyo no cambian.

### 4. Presencia del N‑1 en vuelo

El problema era de encuadre: el N‑1 tiene un perfil muy delgado y ocupaba poco espacio pese a estar ajustado a cinco unidades de longitud. `ShipModel::camera_length()` aplica un factor 0,90 únicamente al encuadre de los modelos Star Wars. No aumenta el modelo ni su hitbox, y no afecta su escala dentro del hangar. Combinado con el ajuste general, su distancia horizontal de seguimiento queda un 17,2% más corta que antes; las capturas muestran una presencia mayor, sin exagerarla.

### 5. Cámara de vuelo

La distancia horizontal general pasa de `3*longitud` a `2,76*longitud`: un 8% más cerca. Conserva FOV 60°, seguimiento suave, objetivo y protecciones de colisión. La cámara del hangar no cambia. El cálculo compartido con el despegue mantiene a la cámara inicial fuera de la estación.

## Evidencia visual

Comparación en la misma aplicación y resolución:

- N‑1 en vuelo: [antes](before/launch-starwars.png) / [después](launch-starwars.png).
- Smiling Friends en el hangar: [antes](before/hangar-smilingfriends.png) / [después](hangar-smilingfriends.png).
- N‑1 en el hangar: [antes](before/hangar-starwars.png) / [después](hangar-starwars.png).
- Las otras presentaciones: [Calaca](hangar-halloween.png), [Minecraft](hangar-minecraft.png), [Golf](hangar-regularshow.png), [Butters](hangar-southpark.png).
- Vuelo con otras siluetas: [Smiling Friends](launch-smilingfriends.png), [Minecraft](launch-minecraft.png), [Golf](launch-regularshow.png), [Butters](launch-southpark.png), [Calaca](launch-halloween.png).
- Interacción exterior y regreso: [Smiling Friends](smilingfriends-dock-approach-0.png), [Star Wars](starwars-dock-approach-0.png), [Minecraft](minecraft-dock-approach-0.png).
- Nuevos despegues tras regresar y cambiar de nave: [ciclo Smiling Friends](smilingfriends-dock-launch-2.png), [ciclo Star Wars](starwars-dock-launch-2.png), [ciclo Minecraft](minecraft-dock-launch-2.png).

Las imágenes son capturas de raylib, no renders de Blender. Las capturas de salida muestran espacio libre delante; la estación está detrás de nave y cámara. Calaca sigue siendo el recurso provisional que ya existía.

## Verificaciones y límites

`cargo check --locked` y `cargo build --locked` pasan. Pasan 33 pruebas:

```powershell
cargo test --locked --test ship_presentation --test station --test station_departure --test ship_selection --test ships --test minigame_returns
```

Se comprueban las seis envolventes reales, acceso exterior con aproximación lateral y distintas rotaciones, rechazo de puntos lejanos o detrás, colisiones activas, despegue seguro, cámara, orientación inicial, dirección del giro y extensión de estela hasta turbo. Se conservan las pruebas de control, ascenso, selección y regresos de minijuegos.

`cargo fmt --check` falla tanto en la base limpia como después, por formato preexistente en `audio.rs`, `front_end.rs`, `solar_system.rs` y otros módulos. No se hizo un reformateo masivo. Los módulos y tests nuevos, `station.rs` y sus tests pasan `rustfmt --check` individualmente. Los logs completos de formato anterior/posterior quedan en `target/hangar-polish-format-before.log` y `target/hangar-polish-format-after.log`.

El script [tools/verify_hangar_polish.py](../../../tools/verify_hangar_polish.py) ejecuta seis selecciones/despegues nativos y tres recorridos de tres regresos cada uno (misma nave, cambio y cancelación), con Smiling Friends, N‑1 y Minecraft. Verifica entrada desde fuera del colisionador, avance inicial sin golpe y estación/cámara detrás. [Resultados](runtime-summary.json); los `.log` de esta carpeta contienen los hitos, y los logs nativos completos quedan en `target/hangar-polish-runtime/`. Los recorridos de regreso comienzan en una colocación de prueba próxima a la aproximación; después usan navegación e interacción reales.

Se revisaron visualmente las seis presentaciones y las capturas de vuelo. Queda la revisión subjetiva manual con ratón, giros rápidos junto a superficies y el aspecto de turbo sostenido: la envolvente de la estela a máxima intensidad sí tiene prueba geométrica. La colisión de estación sigue siendo una esfera conservadora con corredor, no colisión por triángulos.

Raylib avisa de conversión de índices `u32` a `u16` en recursos exteriores de la estación y de la Estrella de la Muerte. El mismo aviso aparece siete veces en la ejecución base anterior a los cambios; no se corrigieron ni reexportaron esos recursos durante esta tarea. Las capturas y los logs permiten revisar esa limitación existente.

Para revisar: `cargo run --locked`, recorrer las seis naves, observar que en unos tres segundos pasan por la vista frontal, despegar con N‑1 y Smiling Friends, avanzar con W, ascender con Espacio y regresar con Tab/B/W hasta mostrar E. Entrar, cambiar y repetir. No se requiere commit para probar.

Archivos de implementación: `src/gameplay/station.rs`, `src/scenes/solar_system.rs`, `src/graphics/hangar.rs`, `src/graphics/ship_presentation.rs`, `src/graphics/mod.rs` y `src/ships/mod.rs`. Tests: `tests/station.rs`, `tests/station_departure.rs` y `tests/ship_presentation.rs`. La configuración y todos los recursos visuales se conservan.
