# Minijuego de Smiling Friends: "Hacé reír a tres personas"

Juego de diálogo: hay que hacer reír a alguien eligiendo respuestas de texto. Dura entre 2 y 3 minutos.

## Recorrido

1. **Video de entrada** (`assets/video/smilingfriends/intro`), más chico, en un marco de papel hecho a mano sobre fondo celeste con dibujitos de crayón, y botón **SKIP** (o Enter, Espacio, Esc).
2. **El jefe** (Mr. Boss), en su oficina o en la sala de descanso, da la misión: hacer reír a tres personas. Una o dos líneas y se continúa.
3. **Tres encuentros**, cada uno con su fondo y su personaje: 3 o 4 intercambios por persona (~40 segundos cada uno).
4. **Resultado:**
   - **Victoria:** las tres personas se rieron. Pantalla de misión cumplida y regreso al sistema solar con el token.
   - **Derrota:** tres strikes en total. Pantalla de derrota y regreso al sistema solar sin recompensa; se puede volver a entrar y reintentar.

## Reglas

- Cada intercambio: el personaje dice algo y aparecen **tres respuestas** (clic o teclas 1, 2 y 3).
- Cada respuesta tiene un efecto:
  - **Buena:** llena mucho el **medidor de risa** (visible, uno por persona).
  - **Más o menos:** llena un poco.
  - **Mala:** no llena y suma un **strike**. Los strikes son globales: al tercero se pierde.
- Cuando el medidor se llena, la persona se ríe a carcajadas y pasa la siguiente.
- Si se acaban los intercambios de una persona sin llenar el medidor, esa persona suma un strike y su encuentro empieza de nuevo.

## Presentación

Estilo de recortes de cartón: los personajes son dibujos planos con un borde blanco grueso, como figuras recortadas, que entran y salen de escena con rebotes, se inclinan al hablar y se sacuden al reaccionar. El texto es protagonista: palabras que aparecen con fuerza, cajas inclinadas con colores de la serie y frases importantes que saltan a pantalla. Ganar y perder tienen su propia animación grande (carcajada con confeti y sello de "MISSION COMPLETE!", o el recorte que se cae y un sello de "NOT FUNNY").

- El fondo ocupa toda la pantalla y el PNG del personaje está parado delante.
- Caja de diálogo abajo, con el nombre del que habla y el texto que se escribe solo (un clic lo completa).
- Reacciones con un solo PNG por personaje:
  - el dibujo salta, tiembla o se encoge según la respuesta;
  - aparece un texto chico de reacción (*se ríe por la nariz*, *te mira fijo*);
  - al final, una carcajada con el dibujo rebotando.
- Strikes visibles arriba (tres íconos).

**Paleta:** la del póster de la serie (`docs/references/smilingfriends/minigame/poster_palette.png`): cielo celeste, amarillo de la carita, tinta azul oscura, naranja de Charlie, rosa de Pim y los colores de las letras del título. Los colores están arriba de `smilingfriends_game.rs`.

**Recortes:** todos se arman a la misma escala (píxeles por alto de pantalla) y antes se les saca el borde blanco que traiga el dibujo original, así el borde blanco se ve igual de grueso en todos.

## Datos y archivos

| Qué | Dónde |
| --- | --- |
| Diálogos, respuestas, puntos y reacciones (editable sin tocar código) | `config/smilingfriends-dialogue.json` |
| Fondos | `assets/textures/minigames/smilingfriends/backgrounds/` |
| Personajes (PNG con transparencia) | `assets/textures/minigames/smilingfriends/characters/` |
| Lógica (estados, medidor, strikes) con pruebas | `src/minigames/smilingfriends/` |
| Pantalla del juego (después del video) | `src/scenes/smilingfriends_game.rs` (la abre `smilingfriends_entry.rs` al terminar el video) |

El minijuego está en inglés (diálogos e interfaz). Claude escribe un primer borrador de los diálogos con el humor de la serie y Cami lo edita en el JSON.

## Escenas

| # | Escena | Fondo | Personajes | Efecto |
| --- | --- | --- | --- | --- |
| 0 | Sala de descanso | `break_room.jpg` | El jefe (dos poses) | — |
| 1 | El bar (Spaghetti Disco) | `bar.jpg` | Natasha, la cita de Alan, que la dejó plantada | Luces de boliche sobre todo. Remate: al reírse, el recorte se da vuelta como una carta y es **el casero de Alan**. |
| 2 | El bosque | `forest.png` | Mr. Frog y Glep disfrazado de rana | Hay que hacer reír a los dos. Al reírse, Glep se da vuelta y aparece con su ropa de siempre (libre). |
| 3 | La calle y el parque | `street_graffiti.png` → `rollercoaster.jpg` | Desmond (`desmond.png`, después `desmond_after_ride.png` con algodón de azúcar) | Empieza en la calle; con un cartel "LATER, AT DAVE ROLLERCOASTER..." la escena pasa al parque, donde sigue triste. Una última respuesta sincera lo hace sonreír: el recorte se da vuelta y aparece Desmond feliz (`desmond_happy.png`). En la primera pregunta solo sirve llevarlo a la montaña rusa: cualquier otra respuesta pierde la escena en el acto (`"fail": true` en esa respuesta del JSON: un strike y Desmond vuelve a empezar). Al fallar, sin bomba ni de dónde sale, una explosión de caricatura llena toda la pantalla (¡KABOOM!) y el recorte sale volando; se sigue viendo el lugar donde se perdió. |

Imágenes en `assets/textures/minigames/smilingfriends/` (convertidas a PNG: el juego no lee WebP). `street.png` (la calle del diner) queda de repuesto.

En el JSON, cada escena tiene su elenco (`cast`), quién dice cada línea (`speaker`), el efecto (`effect`: `disco` o `bomb`), el dibujo y nombre del remate (`laugh_image`, `laugh_name`) el primer plano (`laugh_card`), el lugar y tamaño de cada personaje (`x`, `size`, `floor`) los cambios de lugar dentro de una escena (en una pregunta: `background`, `image`, `scene_card`, `place`) y el **contexto** (`context`): una línea gris arriba de lo que dice el personaje, que describe lo que hace o quiere decir. En una línea del jefe, `"silence": true` corta la música de golpe mientras dura (la de "No pressure.") y la retoma desde donde quedó al pasar a la siguiente. Mr. Frog habla en oraciones completas y dice "Hello." después de cada una. Por ahora el contexto no se usa en ninguna escena.

## Pendiente de Cami

- ~~Fondo y PNG del jefe~~: listos (sala de descanso y dos poses del jefe en `assets/textures/minigames/smilingfriends/`).
- ~~Escenas y personajes~~: listos. Falta revisar y ajustar los diálogos y los tiempos de las animaciones.

## Estado

Jugable de principio a fin con las tres escenas. Prueba automática: `cargo run --bin smilingfriends -- --smoke --mute` juega con las mejores respuestas y guarda capturas en `target/smilingfriends/`. Pruebas de reglas: `cargo test --test smilingfriends_game`.

Música: después del video de la intro suena "Gameotoon" (`assets/audio/music/Smiling Friends/gameotoon.mp3`, elegida por Cami) en loop durante todo el juego; con el menú de salir abierto baja.

Controles: clic, Enter o Espacio para seguir (y completar el texto); 1, 2 y 3 o clic para elegir; Esc o el botón SALIR abren un menú (el juego queda quieto): seguir jugando, **I give up** (rendirse cuenta como ganado) o volver al sistema solar.
