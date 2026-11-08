# BLOQUE 3: medallero y presentación de recompensas

El HUD solar muestra seis círculos pequeños abajo a la izquierda: oscuros si falta
el token e iluminados en dorado si se consiguió, acompañados por el contador real.
No muestra el PNG de ningún token en el preview. **T** abre/cierra el
medallero; también se puede pulsar el contador con el cursor libre. **Esc** cierra
el panel. Durante la consulta o ceremonia se suspende la simulación, como sucede
con la ayuda, y se restaura la captura del cursor al terminar. No cambian las
teclas, reglas ni parámetros del vuelo. La ayuda H incluye la tecla T.

`src/scenes/token_collection.rs` centraliza las seis identidades, nombres, slots
y rutas originales `assets/ui/tokens/{id}.png`: minecraft, southpark,
smilingfriends, starwars, halloween y regularshow. Los seis archivos son PNG RGBA
de 256 × 256 con alfa de 0 a 255. Se cargan directamente, conservando proporción
y transparencia; no hay copias ni imágenes generadas. Los espacios sin descubrir
son únicamente círculos oscuros: no muestran PNG, silueta, nombre ni etiqueta de
estado. Los obtenidos revelan su PNG a color, nombre y halo dorado.

`token_collection_ui.rs` utiliza raylib y las tipografías Advent Pro que ya carga
el HUD, con paneles oscuros y acentos cian/dorados. El medallero tiene una grilla
de 3 × 2 con posiciones estables y geometría escalada a la ventana. El contador
y los seis slots consultan `GameState.progress()`; no existe otro contador.

El observador solar conserva `RewardOutcome` mediante una cola de presentación
propiedad de `Session`. Solo `token_obtained` agenda una ceremonia, con protección
adicional contra entregas repetidas. La cola sobrevive al reinicio de intentos y
ventanas; no se consume dentro de minijuegos. Tras regresar al sistema solar y
terminar su fundido, cada aviso dura 3,6 segundos: aparición y giro del PNG en el
centro, halo luminoso, nombre del planeta y progreso actual; luego se muestra el
medallero y la medalla se reduce y aterriza en su slot. Si `ship_unlocked` es
falso, no se muestra ninguna nave nueva. Ayuda o colección abiertas aplazan los
avisos. La cola no altera el resultado, la propiedad ni la nave activa.

El hangar conserva su lógica de selección y añade una indicación de disponibilidad
o el requisito de ganar el minijuego del planeta. La nave inicial puede estar
disponible sin poseer un token. La colección completa solo muestra una etiqueta,
sin iniciar un final.

El checkout actual ya integra Regular Show: su slot sigue el progreso real de
esa implementación. Halloween sigue pendiente mientras no exista su minijuego;
ninguna de las dos franquicias recibe tokens por abrir esta interfaz.

## Validación

```powershell
cargo test --locked --offline --test token_presentation
cargo test --locked --offline
cargo test --locked --offline --test token_presentation token_visual_smoke -- --ignored --exact
cargo run --locked --offline -- --smoke-minecraft --ship southpark
cargo run --locked --offline -- --smoke-docking --ship minecraft
```

La prueba de presentación comprueba PNG, alfa, mapeo, geometría, avisos únicos,
cola FIFO, aplazamiento durante minijuegos y ausencia de anuncio falso para la
nave inicial. La prueba gráfica ignorada requiere OpenGL y guarda las etapas de
la animación y colección a 640 × 480, 1280 × 800 y 1920 × 1080 en `target/token-ui/`.
Esos fixtures gráficos no forman parte de la lógica de progreso del juego.
