# Guía de diseño de Cami: planetas, naves y personajes

Este documento reúne las decisiones de diseño de los contenidos a cargo de Cami: **Smiling Friends** y **Halloween**. Es la referencia que hay que consultar antes de modelar, texturizar o programar cualquier elemento de estas franquicias. Si una idea nueva contradice algo de aquí, se actualiza este documento primero.

Las reglas generales de carpetas, identificadores y separación de módulos están en [arquitectura](architecture.md). Esta guía no las reemplaza.

| Franquicia | Identificador | Planeta | Nave | Personajes abducidos | Estado de la guía |
| --- | --- | --- | --- | --- | --- |
| Smiling Friends | `smilingfriends` | Definido (sección 1) | Definida (1.7) | Pim y Charlie | En diseño |
| Halloween | `halloween` | Pendiente | Pendiente | Ghost Face | Sin empezar |

---

## 0. Reglas del curso que condicionan el diseño

Vienen del enunciado del proyecto final (Software Renderer). Aplican a todo lo de esta guía.

- **Renderizador por software propio.** raylib solo sirve para la ventana, la entrada y mostrar el framebuffer. Nada de dibujo 3D, modelos, cámara, materiales ni shaders de GPU de raylib. Cada triángulo cuesta CPU: **modelos low-poly**. El estilo plano de la serie ayuda.
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

## 2. Halloween

**Pendiente.** Personaje abducido: Ghost Face. Minijuego: escapar de personajes de terror. El planeta y la nave se documentarán aquí con la misma estructura de la sección 1.
