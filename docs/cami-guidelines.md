# Guía de diseño de Cami: planetas, naves y personajes

Este documento reúne las decisiones de diseño de los contenidos a cargo de Cami: **Smiling Friends** y **Halloween**. Es la referencia que hay que consultar antes de modelar, texturizar o programar cualquier elemento de estas franquicias. Si una idea nueva contradice algo de aquí, se actualiza este documento primero.

Las reglas generales de carpetas, identificadores y separación de módulos están en [arquitectura](architecture.md). Esta guía no las reemplaza.

| Franquicia | Identificador | Planeta | Nave | Personajes abducidos | Estado de la guía |
| --- | --- | --- | --- | --- | --- |
| Smiling Friends | `smilingfriends` | Definido (sección 1) | Definida (1.7) | Pim y Charlie | En diseño |
| Halloween | `halloween` | Definido (sección 2) | Definida (2.6) | Ghost Face | En diseño |

---

## 0. Reglas del curso que condicionan el diseño

Vienen del enunciado del proyecto final (Software Renderer). Aplican a todo lo de esta guía.

- **Renderizador por software propio.** raylib solo sirve para la ventana, la entrada y mostrar el framebuffer. Nada de dibujo 3D, modelos, cámara, materiales ni shaders de GPU de raylib. Cada triángulo cuesta CPU: **modelos low-poly**. El estilo plano de la serie ayuda.
- **Convenciones del equipo (tomadas del planeta Minecraft de Pablo):** los modelos se hacen en **Blender** (`.blend` en `assets/models/planets/<franquicia>/`, con todo colgado de un objeto `<Franquicia>_Planet_ROOT` en el origen), los scripts van en `tools/blender/<franquicia>/` y las capturas en `docs/references/<franquicia>/`. Ese planeta tiene ~79 000 triángulos y su revisión técnica ya advierte que es pesado: **los planetas de Cami deben quedar muy por debajo de eso**.
- **Todo se ve desde la nave.** No se aterriza ni se camina. Cada zona y cada personaje debe **reconocerse al acercarse y rodear el planeta en vuelo**, sin cambio de escena. Por eso la escala es exagerada: si algo no se lee desde la órbita cercana, no sirve.
- **El planeta orbita el sol y rota sobre su eje** en función del tiempo transcurrido, no de los cuadros dibujados.
- **Puntos que este planeta puede aportar** (menú del curso):

| Característica del menú | Cómo la cubre este planeta | Puntos |
| --- | --- | --- |
| Planeta distinguible | La Tierra plana de Smiling Friends. | 10 |
| Tipos de modelo propio sobre el planeta (máx. 4) | **Elegidos:** edificio Smiling Friends, edificio Frowning Friends, casa Shmaloogle y pirámide de Spamtopia. | 20 |
| Distribución procedural reproducible (semilla) | Shmaloogles y casas del pueblo, árboles de los bosques y edificios de relleno de Spamtopia. | 10 |
| Shaders propios distinguibles | Fuego del infierno, arcoíris psicodélico de Spamtopia, haz verde del OVNI, cascada del borde (ver 1.6). | hasta 24 |
| Shader animado | Fuego del infierno o arcoíris de Spamtopia que cambian con el tiempo. | 8 |
| Niveles de detalle | Personajes y edificios simplificados a distancia. | 8 |

> **Ojo:** el punto "posición y orientación correctas sobre la **superficie curva**" no aplica bien a un disco plano. Ese punto debe cubrirlo un planeta esférico de otro integrante, o se coordina con el equipo.

---

## 1. Smiling Friends

### 1.1 Concepto del planeta

El planeta es **la Tierra plana**, tal como aparece en la serie: un disco con océano, islas, un borde de nubes y agua que cae por la orilla.

- **Forma:** disco plano, no esfera. La tierra **no copia los continentes reales**: sigue el mapa propio de Cami (1.3.1), pensado para la serie.
- **Sin relieve realista:** en lugar de montañas, cordilleras o nubes, la superficie está ocupada por **lugares y personajes de la serie**.
- **Escala libre:** los elementos **no respetan proporciones reales**. Un edificio o un personaje puede medir lo que medio continente. La prioridad es que se reconozcan desde la nave.
- **Estilo visual:** caricatura de la serie: colores sólidos, contornos negros gruesos, sombreado mínimo. Nada fotorrealista.

### 1.2 Atracción principal: la calle de Smiling Friends Inc. y Frowning Friends

Es el elemento más grande y el primero que se debe ver al acercarse. Reproduce la escena de T1 EP7: los dos edificios **frente a frente, separados por la calle**.

- **Smiling Friends Inc.:** una **esfera amarilla gigante** (no un edificio con fachada), con la cara pintada (ojos blancos con contorno negro, pupilas, sonrisa negra abierta) y puerta naranja pequeña en la base. Está junto al centro del disco, como en el boceto.
- **Frowning Friends Inc.:** **esfera azul más pequeña**, con ojos blancos y ceño fruncido, al otro lado de la calle y mirando a Smiling Friends. Es más pequeña para no tapar el punto central.
- **La calle:** calle recta con línea central discontinua, banquetas, árboles y **edificios a los dos lados**. Del lado de Smiling Friends: **RON'S DELI** (rosa con toldo a rayas), **ASYLUM** (gris claro con ventanas turquesa), **TOILET EMPORIUM** (rosado con ventanas en arco), **ANTIQUE STORE** y, en los extremos, edificios más pequeños. Detrás de la esfera, un muro de ladrillo que la une con sus vecinos; delante, un contenedor y una parada de autobús. Del lado de Frowning Friends: un edificio bajo a cada lado de la esfera azul (lila y ladrillo) y dos más pequeños hacia cada extremo, como en el boceto. Los letreros son texto 3D.
- **Ubicación:** centro del disco, sobre una isla de esquinas redondeadas. La calle corre de oeste a este: Smiling Friends al norte y Frowning Friends al sur. La calle es parte de la carretera que cruza el mapa (1.3.1).

### 1.3 Zonas del planeta

| Zona | Episodio | Qué debe mostrarse | Ubicación (ver mapa 1.3.1) |
| --- | --- | --- | --- |
| Calle central: Smiling Friends Inc. y Frowning Friends Inc. | Toda la serie y T1 EP7 "Frowning Friends" | Las dos esferas frente a frente y la calle con sus edificios (1.2), rodeadas de árboles. | Centro del disco. |
| Spamtopia | T2 EP7 "The Magical Red Jewel (aka Tyler Gets Fired)" | Al salir de la ciudad, el suelo pasa en degradado del pasto a la **arena del desierto** (letrero **SPAMTOPIA**, montañas naranjas y cactus) y luego al **verde brillante de Spamtopia**, donde la carretera se vuelve **arcoíris** y hay faroles rosas. Spamtopia se siente **llena de edificios psicodélicos simples** (pirámides, triángulos invertidos, cúpulas, paletas, losas, torres, picos y tubos en colores neón): importa la sensación de lugar lleno, no el detalle. Sin rey ni multitud. | Noroeste. La carretera sale de la calle central, cruza el desierto y Spamtopia en curva y se pierde por el borde del disco. |
| Casa de Mr. Frog | T1 EP2 "Mr. Frog" | Península con un sendero de tierra que sube entre montañas grises y azules nevadas, arbustos y **pirámides amarillas**, hasta la cabaña redonda de techo cónico entre pinos oscuros. | Norte: península que sale de la tierra central, detrás de Smiling Friends. |
| Shmaloogleville | T3 EP7 "Shmaloogles" | Pueblo rodeado por un **muro color ciruela**, con casas redondeadas magenta, verdes y moradas y muchos Shmaloogles azules. Se llega por la carretera, que entra en un **túnel** en una colina verde, y por un **puente** desde lo alto de la colina hasta el muro. | Este, contra el borde del disco. |
| Castillo del mago | T3 EP7 "Shmaloogles" | Castillo oscuro de torres puntiagudas sobre rocas. | Sureste, debajo de Shmaloogleville y junto a la colina. |
| Enchanted Forest | T1 EP6 "Enchanted Forest" | Isla con árboles esponjosos verdes y magenta y el **castillo amarillo** al centro. | Suroeste. Un puente de madera la une a la tierra central. |
| Bosque del Forest Demon | T1 EP4 "A Silly Halloween Special" | Isla oscura con árboles morados y el Forest Demon (negro, con astas) al centro. | Sur. Un puente gris tipo escalera la une a la tierra central, del lado de Frowning Friends. |
| Brasil | Brasil | Isla con forma de Sudamérica: el **Cristo Redentor** sobre un cerro y casitas en la ladera. | Sureste, aislada (sin puente). |
| Infierno | T1 EP8 "Charlie Dies and Doesn't Come Back" | Fuego, rojos y naranjas, Satanás. | **Cara inferior del disco**, justo debajo del centro, con un resplandor rojo que asoma por el borde. |

Reglas de las zonas:

1. **Los tres bosques deben distinguirse a simple vista:** los árboles verdes redondos de la tierra central, los pinos oscuros de Mr. Frog, los verdes y magenta del Enchanted Forest y los morados del Forest Demon.
2. **Contraste entre Smiling y Frowning:** Frowning Friends es el reflejo de Smiling Friends: misma forma (esfera), pero azul, más pequeña y con ceño fruncido.
3. **El infierno no ocupa la cara superior.** Descubrirlo al pasar por debajo del disco es la sorpresa del planeta.
4. **Spamtopia contrasta con todo lo demás:** es la zona más saturada del disco; su suelo puede llevar un shader propio (1.6).
5. Las nubes de transición solo flotan sobre el océano: nunca tapan tierra ni zonas.

#### 1.3.1 Mapa del disco

El mapa es el boceto de Cami: [smilingfriends_layout_sketch.png](references/smilingfriends/smilingfriends_layout_sketch.png). El script lo usa como plano de coordenadas: el círculo del boceto es el borde del disco, y cada isla, camino y zona se ubica con las coordenadas de píxel del boceto (escalado a 2000 px de ancho). Para mover algo, se cambian sus coordenadas en el script.

```text
                       N
        [Spamtopia]         [Mr. Frog: península,
     carretera curva y        sendero y cabaña]
     figuras de colores
            |
 O   desierto y rey --- calle central: SMILING FRIENDS --- colina/túnel [SHMALOOGLEVILLE]  E
                                      FROWNING FRIENDS        [castillo del mago]
   [Enchanted Forest] =puente=  tierra central
                                     || escalera
                            [Bosque del Forest Demon]       [Brasil]
                       S
  Borde: nubes y cascada.   Cara inferior: INFIERNO.
```

Islas: tierra central (con la calle, el noroeste de Spamtopia, la península de Mr. Frog y el este de Shmaloogleville y el castillo), Enchanted Forest, Forest Demon y Brasil.

### 1.4 Personajes en el planeta

Todos se colocan a escala exagerada en su zona y deben leerse desde la nave.

| Personaje | Zona | Notas |
| --- | --- | --- |
| Pim Pimling | Smiling Friends Inc. | Rosa/morado, frente al edificio. |
| Charlie Dompler | Smiling Friends Inc. | Amarillo con gorra y sudadera roja, frente al edificio. |
| Allan | Smiling Friends Inc. | Rojo. |
| Glep | Smiling Friends Inc. | Verde y pequeño. |
| The Boss (Mr. Boss) | Smiling Friends Inc. | |
| Grim y Gnarly | Frente a Frowning Friends Inc., en la calle central. | **Solo los dos Frowning Friends**. |
| Mr. Frog | Su bosque y su casa. | |
| Mip | Enchanted Forest | Enano del bosque, antagonista del episodio. |
| Forest Demon | Bosque del demonio | Solo "el demonio del bosque", sin nombre propio. |
| Satanás y Charlie | Infierno | |
| Shmaloogles | Shmaloogleville | **Muchos**, criaturas azules redondas; sin personajes con nombre. |

**Spamtopia no lleva personajes** (decisión de Cami): la zona se enfoca en el lugar.

**Personajes recurrentes extra** (Desmond, Gwimbly, etc.): solo si sobra tiempo al final.

**Abducción de Pim y Charlie.** La nave de Smiling Friends los abduce **al inicio**, si el jugador elige esa nave, o **al ganar por primera vez** el minijuego de este planeta (ver [arquitectura](architecture.md)). El planeta debe verse bien con ellos frente al edificio y también sin ellos, hasta la devolución final. **Decisión:** después de la abducción su lugar **queda vacío** (no se dibujan esos dos modelos); vuelven a aparecer con la devolución final.

### 1.5 Recursos y carpetas

| Recurso | Carpeta |
| --- | --- |
| Disco y zonas | `assets/models/planets/smilingfriends/` |
| Personajes | `assets/models/characters/smilingfriends/` |
| Nave (OVNI) | `assets/models/ships/smilingfriends/` |
| Texturas del planeta | `assets/textures/planets/` |
| Texturas de personajes | `assets/textures/characters/` |
| Capturas y referencias | `docs/references/smilingfriends/` |
| Scripts de Blender | `tools/blender/smilingfriends/` |

**Flujo de trabajo:** el planeta se genera con `tools/blender/smilingfriends/build_smilingfriends_planet.py` (Blender 5.2, semilla fija `SEED`). El script borra y recrea `smilingfriends_planet.blend` y las capturas `docs/references/smilingfriends/smilingfriends_<versión>_<vista>.png`. **No editar el `.blend` a mano:** cualquier cambio va al script, o se pierde en la siguiente ejecución.

```text
blender -b --factory-startup --python tools/blender/smilingfriends/build_smilingfriends_planet.py
```

Cada elemento grande se modela **por separado** (edificio, cada bosque, infierno, Spamtopia, pueblo), para ajustar su escala y posición sobre el disco sin rehacer el planeta. Los elementos repetidos (Shmaloogles, casas, árboles, edificios de Spamtopia) son **un solo modelo instanciado** por la distribución procedural.

### 1.6 Consideraciones técnicas

- **Rotación propia:** el disco gira sobre su eje perpendicular, como un plato. La cara superior queda hacia el plano de vuelo y el edificio central siempre se ve.
- **Recorrido de demostración:** acercarse por arriba (edificio central y zonas) → rodear el borde (cascada y hielo) → pasar por debajo (infierno) → alejarse. Sirve para el video de entrega.
- **Distribución procedural con semilla fija:** posiciones de Shmaloogles y casas dentro de Shmaloogleville, de árboles en cada bosque y de la multitud alrededor del rey. Misma semilla, mismas posiciones; nada colocado a mano una por una.
- **Shaders previstos** (cada uno con implementación distinta, no solo otro color):
  - Fuego animado del infierno.
  - Arcoíris psicodélico que se desplaza en el suelo de Spamtopia.
  - Agua que cae por el borde del disco.
  - Haz verde translúcido del OVNI.
- **Colisión:** si se implementa, debe usar la forma de disco (cilindro bajo), no una esfera.
- **Agujero negro:** en el final, el disco se absorbe como un solo objeto, igual que los planetas esféricos.

### 1.7 Nave: OVNI

Basada en el platillo volador de T2 EP6 "Charlie, Pim, and Bill vs. the Alien". Debe ser **simple**.

- Platillo plateado y metálico, con paneles marcados por líneas finas.
- Cúpula negra brillante encima.
- Haz de abducción verde translúcido que sale por debajo; es el mismo que se usa para abducir a Pim y Charlie.
- Low-poly: un disco aplanado más una media esfera bastan.

### 1.8 Decisiones abiertas

- [x] Lugar de Pim y Charlie tras la abducción: queda vacío.
- [x] Distribución de zonas en el disco (1.3.1).
- [x] Cuatro tipos de modelo propio: edificio Smiling Friends, edificio Frowning Friends, casa Shmaloogle y pirámide de Spamtopia.

No quedan decisiones abiertas para el diseño del planeta de Smiling Friends.

---

## 2. Halloween / Horror

Personaje abducido: **Ghost Face**. Minijuego: escapar de personajes de terror. Este planeta es **más simple** que el de Smiling Friends: pocas zonas, modelos repetidos y un rasgo central fuerte. **Tiene que dar terror genuino**, no ser "Halloween bonito": nada de calabazas sonrientes ni colores de dulces.

### 2.1 Concepto del planeta

**La Tierra después del apocalipsis**, convertida en una **calavera de roca quemada**. Referencias de Cami: una calavera de caricatura (cráneo redondo, cuencas grandes, dientes abultados) y una calavera low-poly con ojos y dientes que brillan.

- **Forma:** **calavera anatómicamente correcta** (proporciones de un cráneo real), gritando: bóveda ovalada, ceja de hueso que baja al centro, órbitas cuadradas redondeadas, separadas e inclinadas en V (enojo), nariz en pera con tabique, cornetes y espina nasal, pómulos, arcos cigomáticos, maxilar, mastoides, dientes de adulto en sus arcos y **mandíbula abierta**, girada sobre su articulación (cuerpo en U, rama, cóndilo y coronoides). La cara ocupa el hemisferio frontal.
- **Hueca:** el cráneo es una cáscara gruesa (~4.5 unidades); las órbitas, la nariz y el foramen magnum la atraviesan para poder poner cosas adentro. Adentro hay roca negra verdosa con **venas que brillan en verde cálido y pasan a naranja en el centro**; ese brillo se ve por las aberturas. Cada órbita tiene una sola pared curva, con el borde biselado, que se angosta hasta la abertura al interior; sus paredes van en roca oscura, sin venas. Una ceja de hueso baja, más baja al centro, le da el ceño enojado. La boca abierta es el espacio entre los dientes.
- **Superficie:** tiene que verse **rocosa, como un planeta de verdad**, no hueso liso: relieve irregular, cráteres y caras planas low-poly. Color **coral de tierra quemada** (no blanco) en **estratos horizontales de tonos muy cercanos, como el Gran Cañón**: nada de manchas que contrasten.
- **Grietas de lava:** una red de grietas con **luz naranja adentro** que cubre el cráneo, como tierra reseca. Pocas y largas; la cara queda limpia (sin arrugas: un cráneo no tiene piel).
- **Ojos, nariz y boca:** aberturas con borde oscuro por las que se ve el interior encendido.
- **Lectura desde lejos:** una calavera que brilla desde dentro. **Lectura de cerca:** las zonas de horror sobre la superficie.

Pautas para que dé miedo de verdad:

1. **Los ojos y la boca son lo más oscuro y lo más brillante a la vez:** cavidades negras con lava al fondo, como si el planeta ardiera por dentro.
2. **Iluminación lateral o desde abajo** (la lava ilumina hacia arriba), nunca una luz plana de frente.
3. **Paleta limitada y pareja:** arenisca coral y óxido en tonos vecinos, roca oscura en las cavidades, el naranja de las grietas de afuera y el verde del interior. Nada más brilla.

### 2.2 Zonas

| Zona | Qué debe mostrarse | Ubicación |
| --- | --- | --- |
| Calavera | Cráneo anatómico gritando: órbitas, nariz y boca abiertas, con lava adentro. | **Hemisferio frontal.** Se deja limpio, sin zonas encima. |
| Zombies y cementerio | Cementerio de lápidas y cruces torcidas de donde **salen los zombies**, y calles en ruinas **llenas de zombies** alrededor. | Costado izquierdo de la cabeza. |
| Payasos | Payasos **destruyendo cosas**: volcando y quemando carros y rompiendo edificios y juegos de feria. Payasos sucios y siniestros, no de fiesta infantil. | Costado derecho de la cabeza. |
| Ritual de brujas | Brujas típicas (sombrero puntiagudo, capa) en círculo alrededor de un caldero o fogata, haciendo un ritual. | Coronilla (parte de arriba). |
| Satanás destruyendo la ciudad | Satanás gigante, estilo *This Is the End*, **comiéndose y aplastando carros** entre edificios destruidos. | Nuca (parte de atrás), como sorpresa al rodear el planeta. |

```text
                    [ Ritual de brujas ]
                         coronilla

  [ Zombies +          (  CALAVERA  )           [ Payasos ]
    cementerio ]      frente, con lava          costado derecho
    costado izq.

                       nuca (atrás):
                 [ Satanás comiendo carros ]
```

Reglas:

1. **La calavera manda.** Ninguna zona tapa los ojos, la nariz ni la boca.
2. **Las zonas se reparten alrededor de la cabeza** (costados, coronilla y nuca): al rodear el planeta siempre aparece algo nuevo.
3. **Los carros son un solo modelo** que se reutiliza en la zona de payasos y en la de Satanás.
4. Este Satanás es de **otra película y otro diseño** que el de Smiling Friends; no se reutiliza ese modelo.
5. **No se agregan más zonas** al planeta.

### 2.3 Personajes y modelos

| Modelo | Dónde | Cantidad |
| --- | --- | --- |
| Zombie | Cementerio y calles | **Muchos**, distribución procedural con semilla. |
| Lápida / cruz | Cementerio | Muchas, distribución procedural. |
| Payaso | Zona de payasos | Varios. |
| Bruja | Ritual | Un círculo de 5 a 7. |
| Caldero / fogata | Ritual | 1. |
| Satanás | Nuca | 1, gigante. |
| Carro | Payasos y Satanás | Varios, volteados y destrozados. |
| Edificio en ruinas | Calles, payasos y Satanás | Varios. |
| Ghost Face | *Por decidir* (2.7). | 1. |

### 2.4 Lo técnico y los puntos

Este planeta cubre justo lo que el disco de Smiling Friends no puede:

| Característica del menú | Cómo la cubre este planeta | Puntos |
| --- | --- | --- |
| Planeta distinguible | Tierra apocalíptica con forma de calavera. | 10 |
| **Deformación de geometría** | La esfera se deforma con una función: desplazamientos hacia dentro (ojos, nariz, boca, mejillas) y hacia fuera (maxilar, mandíbula, dientes), más ruido para terreno irregular y destruido. | 10 |
| **Posición y orientación sobre superficie curva** | Zombies, lápidas, payasos y brujas parados correctamente sobre la esfera mientras rota (el disco de Smiling Friends no cubre este punto). | 5 |
| Distribución procedural con semilla | Zombies y lápidas. | 10 (una sola vez en el proyecto) |
| Shader animado | Lava de las grietas que **pulsa y fluye** con el tiempo. | 8 |
| Shaders propios | Lava; superficie quemada (posible mapeo de normales para grietas y relieve). | 6 c/u |

**Nota:** los máximos del menú son **para todo el proyecto**, no por planeta. Los 4 tipos de modelo (20 pts) ya los cubre Smiling Friends, así que los modelos de este planeta se hacen por diseño, no por puntos. Lo mismo con los shaders: máximo 4 en todo el proyecto. Hay que coordinarlo con el equipo para no duplicar esfuerzo.

### 2.5 Recursos y carpetas

| Recurso | Carpeta |
| --- | --- |
| Planeta y zonas | `assets/models/planets/halloween/` |
| Personajes | `assets/models/characters/halloween/` |
| Nave de zombies | `assets/models/ships/halloween/` |
| Texturas | `assets/textures/planets/halloween/`, `assets/textures/characters/` |
| Capturas y referencias | `docs/references/halloween/` |
| Scripts de Blender | `tools/blender/halloween/` |

### 2.6 Nave: nave de cuerpos

Una nave **formada por zombies**: literalmente cuerpos sobre cuerpos, amontonados y entrelazados, que dan la forma de una nave. Tiene que dar miedo.

- **Silueta clara de nave** desde lejos (forma de flecha o de nave de combate), para que se lea como nave y no como un montón informe.
- **De cerca se notan los cuerpos:** brazos colgando y estirándose, cabezas, torsos retorcidos y manos aferradas a otras.
- **Colores de carne podrida:** gris verdoso, morado, café y rojo oscuro. Sin brillo metálico.
- **Propulsión:** brillo rojo/naranja de lava saliendo por detrás, conectado visualmente con el planeta.
- **Low-poly:** se arma con **un solo modelo de zombie** (el mismo del planeta, o una versión aún más simple) **repetido y rotado** muchas veces sobre una forma base de nave. No se esculpe cada cuerpo.
- La abducción de Ghost Face también sale de esta nave.

### 2.7 Decisiones abiertas

- [ ] Si Ghost Face se ve en el planeta antes de la abducción, y dónde.
