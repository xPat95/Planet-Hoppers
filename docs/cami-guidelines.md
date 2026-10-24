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
| Distribución procedural reproducible (semilla) | Shmaloogles y casas del pueblo, árboles de los bosques, multitud de Spamtopia. | 10 |
| Shaders propios distinguibles | Fuego del infierno, arcoíris psicodélico de Spamtopia, haz verde del OVNI, cascada del borde (ver 1.6). | hasta 24 |
| Shader animado | Fuego del infierno o arcoíris de Spamtopia que cambian con el tiempo. | 8 |
| Niveles de detalle | Personajes y edificios simplificados a distancia. | 8 |

> **Ojo:** el punto "posición y orientación correctas sobre la **superficie curva**" no aplica bien a un disco plano. Ese punto debe cubrirlo un planeta esférico de otro integrante, o se coordina con el equipo.

---

## 1. Smiling Friends

### 1.1 Concepto del planeta

El planeta es **la Tierra plana**, tal como aparece en la serie: un disco con océanos y continentes, borde de hielo y agua que cae por la orilla.

- **Forma:** disco plano, no esfera. Los continentes siguen el mapa de la Tierra plana (polo norte al centro, Antártida como muro de hielo en el borde).
- **Sin relieve realista:** en lugar de montañas, cordilleras o nubes, la superficie está ocupada por **lugares y personajes de la serie**.
- **Escala libre:** los elementos **no respetan proporciones reales**. Un edificio o un personaje puede medir lo que medio continente. La prioridad es que se reconozcan desde la nave.
- **Estilo visual:** caricatura de la serie: colores sólidos, contornos negros gruesos, sombreado mínimo. Nada fotorrealista.

### 1.2 Atracción principal: edificio de Smiling Friends Inc.

Es el elemento más grande y el primero que se debe ver al acercarse.

- Edificio con fachada de **cara sonriente amarilla gigante**: ojos blancos con pupilas negras, boca abierta negra y una puerta naranja pequeña en la base.
- Lo acompaña un tramo de calle de la serie: el edificio **ASYLUM** a la izquierda, **TOILET EMPORIUM** a la derecha, banqueta, contenedor de basura y siluetas de la ciudad en tonos lilas.
- **Ubicación:** al centro del disco (donde estaría el polo norte), como punto focal.

### 1.3 Zonas del planeta

| Zona | Episodio | Qué debe mostrarse | Ubicación (ver mapa 1.3.1) |
| --- | --- | --- | --- |
| Smiling Friends Inc. | Toda la serie | Edificio con cara sonriente y su calle (1.2). | Centro del disco, en una isla-ciudad sobre el Ártico. |
| Frowning Friends Inc. | T1 EP7 "Frowning Friends" | Su edificio, versión opuesta del de Smiling Friends: cara de ceño fruncido y colores apagados. | Asia (Siberia), frío y gris, mirando al edificio principal desde el otro lado del centro. |
| Enchanted Forest | T1 EP6 "Enchanted Forest" | Bosque de fantasía: colores saturados y árboles mágicos. | Sudamérica (Amazonas). |
| Bosque y casa de Mr. Frog | T1 EP2 "Mr. Frog" | Bosque con la casa de Mr. Frog. | Europa (Francia), guiño a "Le Voyage Incroyable de Monsieur Grenouille" (T3 EP2). |
| Bosque del demonio | T1 EP4 "A Silly Halloween Special" | Bosque oscuro y nocturno donde se pierde Pim, con el Forest Demon visible entre los árboles. | Norteamérica (bosques de Canadá). |
| Infierno | T1 EP8 "Charlie Dies and Doesn't Come Back" | Fuego, rojos y naranjas, Satanás. | **Cara inferior del disco**, justo debajo del centro, con un resplandor rojo que asoma por el borde. |
| Spamtopia | T2 EP7 "The Magical Red Jewel (aka Tyler Gets Fired)" | Desierto psicodélico: carreteras arcoíris, pirámides, formas geométricas y cactus de colores saturados. | África (Sahara y Egipto): el desierto con pirámides reales. |
| Pueblo de los Shmaloogles (Shmaloogleville) | T3 EP7 "Shmaloogles" | Colinas verdes con casas redondeadas de colores (ventanas circulares, puertas en arco, tallos morados en espiral) y caminos de tierra amarilla. | Australia: continente aislado, como un reino aparte, **lejos de Spamtopia**. |

Reglas de las zonas:

1. **Los tres bosques deben distinguirse a simple vista:** Enchanted Forest luminoso y fantástico, el de Mr. Frog con tonos naturales y el del demonio oscuro y de noche.
2. **Contraste entre Smiling y Frowning:** Frowning Friends es el reflejo de Smiling Friends: misma forma general y colores invertidos o apagados.
3. **El infierno no ocupa la cara superior.** Descubrirlo al pasar por debajo del disco es la sorpresa del planeta.
4. **Spamtopia contrasta con todo lo demás:** es la zona más saturada del disco; su suelo puede llevar un shader propio (1.6).
5. Entre zonas puede quedar océano o continente simple, sin detalle.

#### 1.3.1 Mapa del disco

No existe un mapa oficial del mundo de la serie, así que la distribución usa el **mapa clásico de la Tierra plana** (proyección azimutal: polo norte al centro, continentes alrededor, Antártida como muro de hielo en el borde). Cada zona va en el continente cuyo clima o paisaje real se parece al del episodio.

```text
                      Norteamérica
                   [Bosque del demonio]
                     (bosque nocturno)

 Sudamérica                                       Asia
 [Enchanted Forest]      ( ÁRTICO )          [Frowning Friends]
 (selva saturada)   [SMILING FRIENDS INC.]     (frío y gris)
                         isla-ciudad

         Europa                              Australia
     [Mr. Frog]                          [Shmaloogleville]
     (bosque natural)                     (colinas verdes)

                       África
                    [Spamtopia]
                 (desierto + pirámides)

  ~~~~~~~~~~ Antártida: muro de hielo + cascada en todo el borde ~~~~~~~~~~

  Cara inferior, debajo del centro: [INFIERNO] con Satanás y Charlie
```

Criterios de la distribución:

1. **Sentido geográfico:** el desierto de Spamtopia va en el Sahara (con pirámides), la selva fantástica en el Amazonas, el bosque oscuro en los bosques del norte, Shmaloogleville aislada en Australia y Frowning Friends en el frío de Siberia.
2. **Sentido en la serie:** Smiling Friends es el centro de todo. Frowning Friends queda enfrente, como su reflejo, y Mr. Frog va en Europa por su episodio francés.
3. **Sentido para la cámara:** las zonas quedan repartidas **alrededor del centro en ángulos distintos**. Al rodear el disco o mientras rota, siempre aparece una zona nueva y no hay dos zonas llamativas juntas.
4. **Contraste de colores entre vecinos:** el bosque nocturno (oscuro) queda lejos de Spamtopia (neón), y los tres bosques no comparten continente.

La forma de los continentes es simplificada (siluetas planas de caricatura); solo debe reconocerse que es la Tierra.

### 1.4 Personajes en el planeta

Todos se colocan a escala exagerada en su zona y deben leerse desde la nave.

| Personaje | Zona | Notas |
| --- | --- | --- |
| Pim Pimling | Smiling Friends Inc. | Rosa/morado, frente al edificio. |
| Charlie Dompler | Smiling Friends Inc. | Amarillo con gorra y sudadera roja, frente al edificio. |
| Allan | Smiling Friends Inc. | Rojo. |
| Glep | Smiling Friends Inc. | Verde y pequeño. |
| The Boss (Mr. Boss) | Smiling Friends Inc. | |
| Grim y Gnarly | Frowning Friends Inc. | **Solo los dos Frowning Friends**, sin más personajes en esa zona. |
| Mr. Frog | Su bosque y su casa. | |
| Mip | Enchanted Forest | Enano del bosque, antagonista del episodio. |
| Forest Demon | Bosque del demonio | Solo "el demonio del bosque", sin nombre propio. |
| Satanás y Charlie | Infierno | |
| Oscar, rey de Spamtopia (Mr. Jester) | Spamtopia | **Único personaje con nombre** de la zona. |
| Multitud de Spamtopia | Spamtopia | Alrededor del rey, **sacrificando a uno de ellos**: la víctima es otra criatura genérica como las de la multitud, levantada en alto. |
| Shmaloogles | Shmaloogleville | **Muchos**, criaturas azules redondas; sin personajes con nombre. |

**Estilo de la multitud de Spamtopia:** criaturas muy distintas entre sí (alargadas, triangulares, cuadradas, con muchos ojos), colores neón saturados (verde menta, rosa, morado, amarillo, naranja), ojos grandes, bocas abiertas gritando con dientes y brazos levantados. Para el renderizador se resuelve con **pocos modelos base simples** (3 o 4 siluetas) que la distribución procedural repite con distinto color, escala y rotación.

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

Cada elemento grande se modela **por separado** (edificio, cada bosque, infierno, Spamtopia, pueblo), para ajustar su escala y posición sobre el disco sin rehacer el planeta. Los elementos repetidos (Shmaloogles, casas, árboles, multitud) son **un solo modelo instanciado** por la distribución procedural.

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

**La Tierra después del apocalipsis**, convertida en una **cara humana sufriendo**.

- **Forma:** esfera **deformada para formar una cara humana** (no una calavera): frente con el ceño contraído, cuencas de los ojos hundidas, nariz en relieve, pómulos y una **boca muy abierta, gritando de dolor**. La cara ocupa un hemisferio entero.
- **Superficie:** continentes reconocibles pero **quemados y destruidos**: negros, grises y café ceniza. Los océanos están secos o son oscuros.
- **Grietas de lava:** grietas con **luz naranja y roja adentro**. Siguen las líneas de la cara (contorno de ojos y boca, arrugas de dolor en la frente, surcos de las mejillas) y la hacen **legible desde lejos, brillando en la oscuridad**.
- **Lectura desde lejos:** una cara humana gritando que brilla desde dentro. **Lectura de cerca:** las zonas de horror sobre la superficie.

Pautas para que dé miedo de verdad:

1. **Expresión de dolor, no de enojo ni caricatura:** cejas levantadas hacia el centro, ojos hundidos y vacíos, boca estirada hacia abajo. Las proporciones son de rostro real.
2. **Los ojos y la boca son lo más oscuro y lo más brillante a la vez:** cavidades negras con lava al fondo, como si el planeta ardiera por dentro.
3. **Iluminación lateral o desde abajo** (la lava ilumina hacia arriba), nunca una luz plana de frente.
4. **Paleta limitada:** negro, gris, café ceniza y el naranja/rojo de la lava. Nada más brilla.

### 2.2 Zonas

| Zona | Qué debe mostrarse | Ubicación |
| --- | --- | --- |
| Cara humana | Ojos, nariz y boca formados por la deformación, con las grietas de lava. | **Hemisferio frontal.** Se deja limpio, sin zonas encima. |
| Zombies y cementerio | Cementerio de lápidas y cruces torcidas de donde **salen los zombies**, y calles en ruinas **llenas de zombies** alrededor. | Costado izquierdo de la cabeza. |
| Payasos | Payasos **destruyendo cosas**: volcando y quemando carros y rompiendo edificios y juegos de feria. Payasos sucios y siniestros, no de fiesta infantil. | Costado derecho de la cabeza. |
| Ritual de brujas | Brujas típicas (sombrero puntiagudo, capa) en círculo alrededor de un caldero o fogata, haciendo un ritual. | Coronilla (parte de arriba). |
| Satanás destruyendo la ciudad | Satanás gigante, estilo *This Is the End*, **comiéndose y aplastando carros** entre edificios destruidos. | Nuca (parte de atrás), como sorpresa al rodear el planeta. |

```text
                    [ Ritual de brujas ]
                         coronilla

  [ Zombies +          ( CARA HUMANA )          [ Payasos ]
    cementerio ]      frente, con lava          costado derecho
    costado izq.

                       nuca (atrás):
                 [ Satanás comiendo carros ]
```

Reglas:

1. **La cara manda.** Ninguna zona tapa los ojos, la nariz ni la boca.
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
| Planeta distinguible | Tierra apocalíptica con cara humana. | 10 |
| **Deformación de geometría** | La esfera se deforma con una función: desplazamientos suaves hacia dentro (ojos, boca) y hacia fuera (frente, nariz, pómulos), más ruido para terreno irregular y destruido. | 10 |
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
